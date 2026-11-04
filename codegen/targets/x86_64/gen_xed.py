#!/usr/bin/env python3
"""Writes x86-64's instructions as description entries, from the instruction
tables of Intel's XED (https://github.com/intelxed/xed, Apache-2.0): its
datafiles give each instruction's encodings (prefixes, opcode map, ModRM
fields, immediates) and operands. Each XED form becomes an entry in the
"enc" language with the fragments of 00-base.ll (rex, rr, rm, vex, evex).

XED's operands are in Intel's order and its names are Intel's; the
assembly templates are AT&T's, found by assembling each entry with a
placeholder template and reading it back with llvm-objdump
(discover_att.py). They are kept in att.txt; with them, this writes the
entries with their templates, entries written alike merged into one with
the encodings as alternatives in the order llvm-mc prefers (order.txt).

Usage:
    gen_xed.py XED_DIR raw   > raw.ll       entries with placeholder templates
    gen_xed.py XED_DIR final > 82-xed.ll    with att.txt and order.txt
"""

import glob
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))


# ---------------------------------------------------------------------------
# Reading XED's datafiles

def lines(path):
    """Lines, with those ending in a backslash joined to the next."""
    held = ''
    for line in open(path):
        s = line.split('#')[0].rstrip()
        if s.endswith('\\'):
            held += s[:-1] + ' '
            continue
        yield held + s
        held = ''


def records(path):
    rec = None
    for line in lines(path):
        s = line.strip()
        if s == '{':
            rec = {'forms': [], 'file': path}
            continue
        if s == '}':
            if rec and 'ICLASS' in rec:
                yield rec
            rec = None
            continue
        if rec is None or ':' not in s:
            continue
        k, v = s.split(':', 1)
        k, v = k.strip(), v.strip()
        if k == 'PATTERN':
            rec['forms'].append({'PATTERN': v.split(), 'OPERANDS': '', 'IFORM': ''})
        elif k in ('OPERANDS', 'IFORM') and rec['forms']:
            rec['forms'][-1][k] = v
        else:
            rec[k] = v


def instruction_files(xed):
    """The files of instructions, as each directory's files.cfg lists them
    (dec-instructions)."""
    out = set()
    for cfg in glob.glob(os.path.join(xed, 'datafiles', '**', 'files.cfg'), recursive=True):
        for line in open(cfg):
            m = re.match(r'\s*dec-instructions\s*:\s*(\S+)', line)
            if m:
                out.add(os.path.join(os.path.dirname(cfg), m.group(1)))
    return sorted(out)


def all_forms(xed):
    """Every form, with a number for it unique among them; not those a
    file deletes (UDELETE, by IFORM: where another file replaces them)."""
    files = instruction_files(xed)
    deleted = set()
    for path in files:
        for line in lines(path):
            m = re.match(r'\s*UDELETE\s*:\s*(\S+)', line)
            if m:
                deleted.add(m.group(1))
    n = 0
    for path in files:
        for r in records(path):
            for f in r['forms']:
                if f['IFORM'] not in deleted:
                    yield r, n, f
                n += 1


class Op:
    """An operand: its kind (reg, mem, imm, relbr, agen), XED nonterminal
    or register, visibility (explicit, IMPL, SUPP) and width code."""

    def __init__(self, tok):
        head, *attrs = tok.split(':')
        self.tok = tok
        self.attrs = attrs
        self.supp = 'SUPP' in attrs
        self.impl = 'IMPL' in attrs
        name, _, val = head.partition('=')
        self.slot = name
        self.val = val
        self.kind = re.sub(r'\d+$', '', name)
        widths = [a for a in attrs if a not in ('r', 'w', 'rw', 'cr', 'cw', 'rcw', 'crw', 'SUPP', 'IMPL', 'EXPL')
                  and not a.startswith('TXT=')]
        self.width = widths[0] if widths else ''


# ---------------------------------------------------------------------------
# Encodings

SKIP_TOKENS = {'not64', 'mode16', 'mode32', 'MPXMODE=1', 'VEXVALID=2', 'MAP4', 'EVAPX', 'EVAPX_SCC',
               'REX2=1', 'rex2_refining_prefix', 'eamode16', 'eamode32', 'MODEP5=1', 'REP=2', 'REP=3',
               'MOD=0', 'MOD=1', 'MOD=2', 'no67_prefix', 'rexb4_prefix'}
# Decoder states for CPUs without a feature: the form for those with it is
# kept.
OFF_FEATURES = re.compile(r'(LZCNT|TZCNT|P4|WBNOINVD|PREFETCHIT|PREFETCHRST|CLDEMOTE|IBHF|CET|MODE_SHORT_UD0)=0$')

GPR = {'GPR8': 'r8', 'GPR16': 'r16', 'GPR32': 'r32', 'GPR64': 'r64'}
CLASS_OF = {'MMX': 'mm', 'XMM': 'x', 'SEG': 'seg', 'SEG_MOV': 'seg', 'CR': 'cr', 'DR': 'dr', 'X87': 'st'}


def reg_class(nt, eosz):
    """The class of a register operand's nonterminal, and its field: R
    (ModRM.reg), B (ModRM.rm), SB (the opcode's low bits)."""
    if not nt.endswith('()'):
        return None, None
    base, _, field = nt[:-2].rpartition('_')
    if field not in ('R', 'B', 'SB', 'N', 'SE', 'R3', 'B3', 'N3'):
        base, field = nt[:-2], None
    if nt in ('SEG()', 'SEG_MOV()'):
        return 'seg', 'R'
    if nt == 'X87()':
        return 'st', 'B'
    if base in GPR:
        return GPR[base], field
    if base in ('A_GPR', 'GPRa'):
        return 'r64', field
    if base == 'GPRv':
        return {16: 'r16', 32: 'r32', 64: 'r64'}[eosz], field
    if base == 'GPRy':
        return ('r64' if eosz == 64 else 'r32'), field
    if base == 'GPRz':
        return ('r16' if eosz == 16 else 'r32'), field
    if base in CLASS_OF:
        return CLASS_OF[base], field
    return None, None


def bits8(v):
    return format(v, '08b')


class Unsupported(Exception):
    pass


def sizes(pattern, ops):
    """The operand sizes a form comes in, and whether REX.W says 64."""
    p = set(pattern)
    shown = [o for o in ops if not o.supp]
    variable = any(re.match(r'(GPR[vyz]|O[re]|A[re])', o.val or '') for o in shown) or \
        any(o.kind in ('MEM', 'IMM', 'RELBR') and o.width in ('v', 'z', 'y') for o in shown) or \
        any(t in ('SIMMz()', 'UIMMv()', 'BRDISPz()') for t in pattern)
    mandatory66 = bool(p & {'osz_refining_prefix', 'REFINING66()', '66_prefix'})
    if 'rexw_prefix' in p or 'W1' in p:
        return [(64, True)]
    # (A push or pop's width is the stack's, even of an immediate.)
    if not variable and not ('DF64()' in p and not p & {'IMMUNE66()', 'IMMUNE66_LOOP64()'}):
        return [(0, False)]
    if 'FORCE64()' in p or 'CR_WIDTH()' in p:
        return [(64, False)]
    if 'DF64()' in p and ('IMMUNE66()' in p or 'IMMUNE66_LOOP64()' in p):
        return [(64, False)]
    if 'DF64()' in p:
        return [(16, False), (64, False)]
    out = [16, 32, 64]
    if 'norexw_prefix' in p or 'W0' in p or 'IMMUNE_REXW()' in p:
        out = [16, 32]
    if mandatory66 or 'IMMUNE66()' in p or 'IMMUNE66_LOOP64()' in p:
        out = [s for s in out if s != 16]
    return [(s, s == 64) for s in out]


MEMS = [
    # suffix, operands, assembly, rex's index and base, rm's arguments
    ('m', ['i64 "reg"="r64" %m.base', 'i64 "imm" %m.disp'], '{m.disp}({m.base})', ('4', 'm.base'),
     'm.base, 4, 1, m.disp, 0, 1'),
    ('x', ['i64 "reg"="r64" %m.base', 'i64 "reg"="r64" %m.index', 'i64 "imm" %m.scale', 'i64 "imm" %m.disp'],
     '{m.disp}({m.base},{m.index},{m.scale})', ('m.index', 'm.base'), 'm.base, m.index, m.scale, m.disp, 0, 1'),
    ('p', ['ptr "sym" %m.sym'], '{m.sym}(%rip)', ('0', '0'), '16, 4, 1, 0, m.sym, 1'),
]


def legacy(rec, form, eosz, rexw, mem):
    """A legacy (not VEX or EVEX) form as (params, enc, holes), for operand
    size `eosz` and memory form `mem` (or None)."""
    pat = form['PATTERN']
    p = set(pat)
    ops = [Op(t) for t in form['OPERANDS'].split()]
    params, holes = [], []
    reg_f = rm_f = None      # the ModRM.reg and ModRM.rm (or opcode) operands
    byte_regs = []
    imms = []
    label = None
    memop = None
    for o in ops:
        if o.supp:
            continue
        if o.kind == 'REG':
            if o.val.startswith('XED_REG_') or re.match(r'(O[re]|A[re]|rIP|FINAL)', o.val):
                continue  # a fixed register: written in the template as is
            cls, field = reg_class(o.val, eosz or 32)
            if cls is None:
                raise Unsupported(o.val)
            name = {'R': 'r', 'B': 'b', 'SB': 'b'}.get(field)
            if name is None:
                raise Unsupported(o.val)
            if field == 'R':
                reg_f = name
            else:
                rm_f = name
            if cls == 'r8':
                byte_regs.append(name)
            params.append(f'i64 "reg"="{cls}" %{name}')
            holes.append('{' + name + '}')
        elif o.kind in ('MEM', 'AGEN'):
            memop = o
            if mem is None:
                raise Unsupported('memory')
            params += mem[1]
            holes.append(mem[2])
        elif o.kind == 'IMM' and 'ONE()' in pat:
            continue  # by one: no immediate written
        elif o.kind == 'IMM':
            name = 'c' if not imms else 'c2'
            imms.append(name)
            params.append(f'i64 "imm" %{name}')
            holes.append('${' + name + '}')
        elif o.kind == 'RELBR':
            label = 'dest'
            params.append('ptr "label" %dest')
            holes.append('{dest}')
        else:
            raise Unsupported(o.kind)
    if (memop is None) != (mem is None):
        raise Unsupported('memory form')
    if mem and rm_f and 'SRM[rrr]' not in pat:
        raise Unsupported('ModRM.rm both a register and memory')

    # Prefixes in llvm-mc's order: rep, 66, lock, then the F2 or F3 an
    # instruction is told apart by.
    pieces = []
    if 'repne' in p:
        pieces.append('11110010')
    if 'repe' in p:
        pieces.append('11110011')
    if eosz == 16 or p & {'osz_refining_prefix', 'REFINING66()', '66_prefix'}:
        pieces.append('01100110')
    if 'lock_prefix' in p:
        pieces.append('11110000')
    if 'f2_refining_prefix' in p:
        pieces.append('11110010')
    if p & {'f3_refining_prefix', 'refining_f3'}:
        pieces.append('11110011')
    # ModRM.reg: an operand, or fixed bits.
    reg_bits = None
    for t in pat:
        m = re.match(r'REG\[0b([01]{3})\]$', t)
        if m:
            reg_bits = int(m.group(1), 2)
    rm_bits = None
    for t in pat:
        m = re.match(r'RM\[0b([01]{3})\]$', t)
        if m:
            rm_bits = int(m.group(1), 2)
    r = reg_f if reg_f else '0'
    w = 1 if rexw else 0
    if mem:
        x, b = mem[3]
    else:
        x, b = '0', (rm_f if rm_f else '0')
    if byte_regs:
        conds = ' '.join(f'if({v}<4)' for v in byte_regs)
        pieces.append(f'{{ {conds} rex({w}, {r}, {x}, {b}) ; 0100 W:1={w} R:1=({r})>>3 X:1=({x})>>3 B:1=({b})>>3 }}')
    else:
        pieces.append(f'rex({w}, {r}, {x}, {b})')
    # The opcode: whole bytes, or five bits and a register.
    opbits = []
    for t in pat:
        if re.match(r'0x[0-9A-Fa-f]{2}$', t):
            pieces.append(bits8(int(t, 16)))
        elif re.match(r'0b[01_]+$', t):
            opbits.append(t[2:].replace('_', ''))
    if opbits:
        if 'SRM[rrr]' not in pat or not rm_f:
            raise Unsupported('opcode bits')
        pieces.append(''.join(opbits) + f' o:3={rm_f}&7')
    modrm = any(t.startswith('MOD[') for t in pat)
    if modrm:
        rf = f'{reg_f}&7' if reg_f else None
        if mem:
            rr = reg_f if reg_f else str(reg_bits if reg_bits is not None else 0)
            pieces.append(f'rm({rr}, {mem[4]})')
        else:
            # (ModRM.reg of no operand is ignored: 0, as llvm-mc writes it.)
            regpart = f'reg:3={rf}' if rf else bits8(reg_bits or 0)[5:]
            if rm_f:
                rmpart = f'rm:3={rm_f}&7'
            elif rm_bits is not None:
                rmpart = bits8(rm_bits)[5:]
            else:
                # (Nor is ModRM.rm of none: mfence is 0F AE F0.)
                rmpart = '000'
            pieces.append(f'11 {regpart} {rmpart}')
    # Immediates, in order.
    k = 0
    for t in pat:
        if t in ('SIMM8()', 'UIMM8()', 'UIMM8_1()', 'UIMM16()', 'SIMMz()', 'UIMMv()', 'UIMM32()'):
            name = imms[k] if k < len(imms) else None
            if name is None:
                raise Unsupported('immediate')
            k += 1
            # (XED has some arithmetic's byte immediates unsigned, some
            # signed: signed, as llvm-mc writes them.)
            if t == 'UIMM8()' and rec.get('CATEGORY') in ('LOGICAL', 'BINARY', 'DATAXFER'):
                t = 'SIMM8()'
            f = {'SIMM8()': 's8', 'UIMM8()': '8', 'UIMM8_1()': '8', 'UIMM16()': '16', 'UIMM32()': '32',
                 'SIMMz()': 's16' if eosz == 16 else 's32',
                 'UIMMv()': {16: '16', 32: '32', 64: 's64'}.get(eosz, '32')}[t]
            # An immediate as wide as the operation is read either way, as
            # llvm-mc reads it ($-1 and $0xffff are one 16-bit value):
            # signed, or else unsigned, of the same bits.
            full = (t in ('UIMMv()', 'SIMMz()') and eosz in (16, 32)) or \
                (t == 'SIMM8()' and 'BYTEOP' in rec.get('ATTRIBUTES', ''))
            if full:
                w = f.lstrip('s')
                pieces.append(f'{{ i{k}:s{w}={name} ; i{k}:{w}={name} }}')
            elif t == 'SIMM8()' and eosz in (16, 32):
                # A byte sign-extended: of the value written unsigned too
                # ($0xffff is -1 to a 16-bit operation).
                pieces.append(f'{{ i{k}:s8={name} ; i{k}:s8={name}-{1 << eosz} }}')
            else:
                pieces.append(f'i{k}:{f}={name}')
        elif t in ('BRDISP8()', 'BRDISP32()', 'BRDISPz()'):
            f = 's8' if t == 'BRDISP8()' else 's32'
            pieces.append(f'rel:{f}=dest-end')
        elif t in ('MEMDISPv()', 'BRDISP64()', 'UIMM8_1()'):
            raise Unsupported(t)
    enc = ' | '.join(pieces)
    guards = []
    if 'SRM!=0' in pat:
        guards.append(f'if({rm_f}!=0)')  # (90 with rax is nop)
    if 'SRM=0' in pat and rm_f:
        guards.append(f'if(({rm_f}&7)==0)')
    if 'rexb_prefix' in pat:
        guards.append(f'if({rm_f}>=8)')
    if 'norexb_prefix' in pat:
        guards.append(f'if({rm_f or 0}<8)')
    if guards:
        enc = f'{{ {" ".join(guards)} {enc} }}'
    return params, enc, holes


VCLASS = {'XMM': 'xe', 'YMM': 'ye', 'ZMM': 'z', 'MASK': 'k', 'VGPR32': 'r32', 'VGPR64': 'r64', 'TMM': 'tmm', 'GPR32': 'r32',
          'GPR64': 'r64'}


def vex(rec, form, mem):
    """A VEX form as (params, enc, holes)."""
    pat = form['PATTERN']
    p = set(pat)
    ops = [Op(t) for t in form['OPERANDS'].split()]
    params, holes = [], []
    fields = {}
    imms = []
    memop = None
    for o in ops:
        if o.supp or o.kind.startswith('EMX_BROADCAST'):
            continue
        if o.kind == 'REG':
            if o.val.startswith('XED_REG_'):
                continue
            base, _, field = o.val[:-2].rpartition('_')
            cls = VCLASS.get(base)
            if cls is None or field not in ('R', 'B', 'N', 'SE'):
                raise Unsupported(o.val)
            name = {'R': 'r', 'B': 'b', 'N': 'n', 'SE': 'e'}[field]
            fields[field] = name
            params.append(f'i64 "reg"="{cls}" %{name}')
            holes.append('{' + name + '}')
        elif o.kind == 'MEM':
            memop = o
            if mem is None:
                raise Unsupported('memory')
            params += mem[1]
            holes.append(mem[2])
        elif o.kind == 'IMM':
            name = 'c' if not imms else 'c2'
            imms.append(name)
            params.append(f'i64 "imm" %{name}')
            holes.append('${' + name + '}')
        else:
            raise Unsupported(o.kind)
    if (memop is None) != (mem is None):
        raise Unsupported('memory form')
    if any(t in p for t in ('VMODRM_XMM()', 'VMODRM_YMM()', 'SIB', 'MAP7')):
        raise Unsupported('vector index, sibmem or map 7')
    pp = 1 if 'V66' in p else 2 if 'VF3' in p else 3 if 'VF2' in p else 0
    mp = 1 if 'V0F' in p else 2 if 'V0F38' in p else 3 if 'V0F3A' in p else None
    if mp is None:
        raise Unsupported('map')
    w = 1 if p & {'W1', 'rexw_prefix'} else 0
    l = 1 if 'VL256' in p else 0
    r = fields.get('R')
    reg_bits = None
    for t in pat:
        m = re.match(r'REG\[0b([01]{3})\]$', t)
        if m:
            reg_bits = int(m.group(1), 2)
    modrm_byte = any(t.startswith('MOD[') for t in pat)
    rr = r if r else str(reg_bits or 0)
    v = fields.get('N', '0')
    opc = [bits8(int(t, 16)) for t in pat if re.match(r'0x[0-9A-Fa-f]{2}$', t)]
    if len(opc) != 1:
        raise Unsupported('opcode')
    if not modrm_byte:
        # No ModRM (vzeroupper): the prefix and the opcode.
        return params, f'vex(0, 0, 0, {mp}, {w}, {v}, {l}, {pp}) | {opc[0]}', holes
    if mem:
        base, index = mem[4].split(', ')[:2]
        prefix = f'vexm({rr}, {base}, {index}, {mp}, {w}, {v}, {l}, {pp})'
        modrm = f'rm({rr}, {mem[4]})'
    else:
        b = fields.get('B')
        rm_bits = None
        for t in pat:
            m = re.match(r'RM\[0b([01]{3})\]$', t)
            if m:
                rm_bits = int(m.group(1), 2)
        if b is None and rm_bits is None:
            raise Unsupported('rm')
        prefix = f'vex({rr}, 0, {b or 0}, {mp}, {w}, {v}, {l}, {pp})'
        regpart = f'reg:3={r}&7' if r else bits8(reg_bits or 0)[5:]
        rmpart = f'rm:3={b}&7' if b else bits8(rm_bits)[5:]
        modrm = f'11 {regpart} {rmpart}'
    pieces = [prefix, opc[0], modrm]
    for t in pat:
        if t == 'UIMM8()':
            pieces.append(f'i1:8={imms[0]}')
        elif t == 'SE_IMM8()':
            # is4: a register in the top four bits, an immediate below.
            low = f'i1:4={imms[0]}' if imms else '0000'
            pieces.append(f'is4:4={fields["SE"]} {low}')
    return params, ' | '.join(pieces), holes


ESIZE = {'ESIZE_8_BITS()': 1, 'ESIZE_16_BITS()': 2, 'ESIZE_32_BITS()': 4, 'ESIZE_64_BITS()': 8,
         'ESIZE_128_BITS()': 16}
ROUNDING = ['rn-sae', 'rd-sae', 'ru-sae', 'rz-sae']


def evex_variants(rec, form):
    """The entries an EVEX form makes: (mask, broadcast, rounding), mask
    None, 'merge' or 'zero', rounding None, 0-3 or 'sae'."""
    pat, ops = form['PATTERN'], form['OPERANDS']
    p = set(pat)
    masks = [None]
    if 'MASK1()' in ops and 'MASK=0' not in p:
        masks = [None, 'merge'] + ([] if 'ZEROING=0' in p else ['zero'])
    if 'MASKNOT0()' in ops:
        masks = ['merge']
    mem = 'MOD!=3' in p
    out = []
    for mk in masks:
        if mem:
            bcs = [0] if 'BCRC=0' in p else [1] if 'BCRC=1' in p else [0] + ([1] if 'BCASTSTR' in ops else [])
            out += [(mk, bc, None) for bc in bcs]
        elif 'AVX512_ROUND()' in p:
            out += [(mk, 1, rc) for rc in range(4)]
        elif 'SAE()' in p:
            out += [(mk, 1, 'sae')]
        else:
            out += [(mk, 0, None)]
    return out


def evex(rec, form, mem, variant):
    """An EVEX form as (params, enc, holes), for `variant` (evex_variants)."""
    mk, bc, rc = variant
    pat = form['PATTERN']
    p = set(pat)
    ops = [Op(t) for t in form['OPERANDS'].split()]
    params, holes = [], []
    fields = {}
    imms = []
    memop = None
    for o in ops:
        if o.supp or o.kind.startswith('EMX_BROADCAST'):
            continue
        if o.kind == 'REG':
            if o.val.startswith('XED_REG_'):
                continue
            if o.val in ('MASK1()', 'MASKNOT0()'):
                if mk:
                    params.append('i64 "reg"="k" %k')
                    holes.append('{lb}{k}{rb}' + (' {lb}z{rb}' if mk == 'zero' else ''))
                continue
            base, _, field = o.val[:-2].rpartition('_')
            field = field.rstrip('3')
            cls = VCLASS.get(base)
            if cls is None or field not in ('R', 'B', 'N'):
                raise Unsupported(o.val)
            name = {'R': 'r', 'B': 'b', 'N': 'n'}[field]
            fields[field] = name
            params.append(f'i64 "reg"="{cls}" %{name}')
            holes.append('{' + name + '}')
        elif o.kind == 'MEM':
            memop = o
            if mem is None:
                raise Unsupported('memory')
            params += mem[1]
            holes.append(mem[2] + (' {lb}bcst{rb}' if bc else ''))
        elif o.kind == 'IMM':
            name = 'c' if not imms else 'c2'
            imms.append(name)
            params.append(f'i64 "imm" %{name}')
            holes.append('${' + name + '}')
        else:
            raise Unsupported(o.kind)
    if (memop is None) != (mem is None):
        raise Unsupported('memory form')
    if any(t.startswith('UISA_VMODRM') for t in p):
        raise Unsupported('vector index')
    pp = 1 if 'V66' in p else 2 if 'VF3' in p else 3 if 'VF2' in p else 0
    mp = {'V0F': 1, 'V0F38': 2, 'V0F3A': 3, 'MAP5': 5, 'MAP6': 6}
    mp = next((v for k, v in mp.items() if k in p), None)
    if mp is None:
        raise Unsupported('map')
    w = 1 if 'W1' in p else 0
    vl = 2 if 'VL512' in p or 'FIX_ROUND_LEN512()' in p else 1 if 'VL256' in p else 0
    # With {sae}, L'L is 00 (as llvm-mc writes it; it is ignored).
    ll = rc if isinstance(rc, int) else 0 if rc == 'sae' else vl
    r = fields.get('R')
    reg_bits = None
    for t in pat:
        m = re.match(r'REG\[0b([01]{3})\]$', t)
        if m:
            reg_bits = int(m.group(1), 2)
    if r is None and reg_bits is None:
        raise Unsupported('reg')
    rr = r if r else str(reg_bits)
    v = fields.get('N', '0')
    aaa = 'k' if mk else '0'
    z = 1 if mk == 'zero' else 0
    opc = [bits8(int(t, 16)) for t in pat if re.match(r'0x[0-9A-Fa-f]{2}$', t)]
    if len(opc) != 1:
        raise Unsupported('opcode')
    guard = 'if(k!=0) ' if mk else ''
    if mem:
        # disp8*N: the tuple type's N.
        es = next((v for k, v in ESIZE.items() if k in p), 4)
        vbytes = 16 << vl
        tuple_n = {'NELEM_FULL()': es if bc else vbytes, 'NELEM_FULLMEM()': vbytes,
                   'NELEM_HALF()': es if bc else vbytes // 2, 'NELEM_HALFMEM()': vbytes // 2,
                   'NELEM_QUARTER()': es if bc else vbytes // 4, 'NELEM_QUARTERMEM()': vbytes // 4, 'NELEM_EIGHTHMEM()': vbytes // 8, 'NELEM_ONE()': es,
                   'NELEM_TUPLE1_4X()': 16, 'NELEM_TUPLE2()': 2 * es, 'NELEM_TUPLE4()': 4 * es,
                   'NELEM_TUPLE8()': 8 * es, 'NELEM_MEM128()': 16,
                   'NELEM_MOVDDUP()': 8 if vl == 0 else vbytes}
        n = next((v for k, v in tuple_n.items() if k in p), None)
        if n is None:
            raise Unsupported('tuple')
        base, index = mem[4].split(', ')[:2]
        rma = mem[4].rsplit(', ', 1)[0] + f', {n}'
        prefix = f'evexm({rr}, {base}, {index}, {mp}, {w}, {v}, {ll}, {pp}, {bc}, {aaa}, {z})'
        modrm = f'rm({rr}, {rma})'
    else:
        b = fields.get('B')
        if b is None:
            raise Unsupported('rm')
        prefix = f'evexr({rr}, {b}, {mp}, {w}, {v}, {ll}, {pp}, {bc}, {aaa}, {z})'
        regpart = f'reg:3={r}&7' if r else bits8(reg_bits or 0)[5:]
        modrm = f'11 {regpart} rm:3={b}&7'
    pieces = [prefix, opc[0], modrm]
    for t in pat:
        if t == 'UIMM8()':
            pieces.append(f'i1:8={imms[0]}')
    if rc is not None:
        holes.insert(0, '{lb}' + (ROUNDING[rc] if isinstance(rc, int) else 'sae') + '{rb}')
    enc = ' | '.join(pieces)
    if guard:
        enc = f'{{ {guard}{enc} }}'
    return params, enc, holes


def forms(xed):
    """Every form, as (name, params, enc, holes, extension)."""
    seen = set()
    for rec, k, form in all_forms(xed):
        pat = form['PATTERN']
        if any(t in SKIP_TOKENS or OFF_FEATURES.match(t) for t in pat):
            continue
        kind = 'vex' if 'VV1' in pat else 'evex' if 'EVV' in pat else 'legacy'
        if any(t == 'XOPV' or t.startswith('VEXVALID') for t in pat):
            continue  # XOP: later
        if 'apx' in rec['file']:
            continue
        ops = [Op(t) for t in form['OPERANDS'].split()]
        base = rec['ICLASS']
        has_mem = any(o.kind in ('MEM', 'AGEN') and not o.supp for o in ops)
        variants = evex_variants(rec, form) if kind == 'evex' else [None]
        for eosz, rexw, variant in ((e, w, v) for e, w in (sizes(pat, ops) if kind == 'legacy' else [(0, False)])
                                    for v in variants):
            for mem in (MEMS if has_mem else [None]):
                name = f"{base}_{k}" + (f"_{eosz}" if eosz else '') + (f"_{mem[0]}" if mem else '')
                if variant:
                    mk, bc, rc = variant
                    name += f"_{mk or 'u'}{'_b' if bc and mem else ''}{'_r' + str(rc) if rc is not None else ''}"
                name = re.sub(r'[^A-Za-z0-9_]', '_', name)
                try:
                    if kind == 'evex':
                        if mem is None and variant[1] and variant[2] is None:
                            continue
                        params, enc, holes = evex(rec, form, mem, variant)
                    elif kind == 'vex':
                        params, enc, holes = vex(rec, form, mem)
                    else:
                        params, enc, holes = legacy(rec, form, eosz, rexw, mem)
                except Unsupported as e:
                    sys.stderr.write(f'skipped {name}: {e}\n')
                    continue
                if name in seen:
                    continue
                seen.add(name)
                yield (name, params, enc, holes, rec.get('EXTENSION', ''), rec['ICLASS'],
                       ' "reloc"="pc32x"' if mem and mem[0] == 'p' else '')


# ---------------------------------------------------------------------------
# Templates, and merging entries written alike

REG_NAMES = None


def register_names():
    """Each class's register names, from 00-base.ll."""
    out = {}
    text = open(os.path.join(HERE, '00-base.ll')).read()
    for m in re.finditer(r'^!\d+ = !\{!"(\w+)", ((?:!"[^"]*"(?:, )?)+)\}', text, re.M):
        cls, rest = m.group(1), m.group(2)
        names = re.findall(r'!"([^"]*)"', rest)
        if not names or not names[0].startswith('%'):
            continue
        full = []
        for n in names:
            r = re.match(r'(.*)\{(\d+)-(\d+)\}(.*)', n)
            if r:
                full += [f'{r.group(1)}{k}{r.group(4)}' for k in range(int(r.group(2)), int(r.group(3)) + 1)]
            else:
                full.append(n)
        out[cls] = full
    return out


def canonical(params, enc, tpl):
    """Renames the register and immediate operands by where the template
    writes them: registers o0, o1, ..., immediates c, c2."""
    order = re.findall(r'\{([\w.]+)\}', tpl)
    kinds = {re.search(r'%([\w.]+)$', p).group(1): p for p in params}
    ren, nreg, nimm = {}, 0, 0
    for op in order:
        if op in ren or op.startswith('m.') or op == 'dest' or op not in kinds:
            continue
        if '"reg"' in kinds[op]:
            ren[op] = f'o{nreg}'
            nreg += 1
        elif '"imm"' in kinds[op]:
            ren[op] = 'c' if nimm == 0 else 'c2'
            nimm += 1
    sub = lambda t: re.sub(r'(?<![\w.%])([a-z][\w.]*)(?![\w.(])',
                           lambda m: ren.get(m.group(1), m.group(1)), t)
    tpl2 = re.sub(r'\{([\w.]+)\}', lambda m: '{' + ren.get(m.group(1), m.group(1)) + '}', tpl)
    params2 = [re.sub(r'%([\w.]+)$', lambda m: '%' + ren.get(m.group(1), m.group(1)), p) for p in params]
    # Parameters in the order the template writes them.
    names2 = [ren.get(o, o) for o in order if o in kinds]
    params2.sort(key=lambda p: names2.index(re.search(r'%([\w.]+)$', p).group(1))
                 if re.search(r'%([\w.]+)$', p).group(1) in names2 else 99)
    return params2, sub(enc), tpl2


def class_of(params, op):
    for p in params:
        if p.endswith('%' + op):
            m = re.search(r'"reg"="(\w+)"', p)
            return m.group(1) if m else None
    return None


def two_byte_vex(alt):
    """The operand in ModRM.rm of a VEX register form that may have the
    two-byte prefix (map 0F, W0), or None."""
    m = re.match(r'(?:if\(\S+\) )*vex\((\w+), 0, (\w+), (\w+), (\w+),', alt)
    if m and m.group(3) == '1' and m.group(4) == '0' and not m.group(2).isdigit():
        return m.group(2)
    return None


def opcode(alt):
    """The bytes of an alternative's pieces that are literal bits only (as
    order_llvm.py reads them)."""
    out = []
    for piece in re.split(r'\s*\|\s*', alt):
        p = re.sub(r'^\S+:0=\S+\s*', '', piece.strip())
        p = re.sub(r'^(if\(\S+\)\s*)+', '', p)
        if re.fullmatch(r'[01]{8}', p):
            out.append(int(p, 2))
    # And a VEX or EVEX prefix's map, W and pp, as 0x100 + map, 0x200 + W,
    # 0x300 + pp.
    m = re.search(r'(e?vex)[mr]?\(([^()]*)\)', alt)
    if m:
        args = [a.strip() for a in m.group(2).split(',')]
        # vex(r, x, b, map, w, v, l, pp); vexm(r, base, index, map, w, v, l,
        # pp); evexr(r, b, map, w, v, ll, pp, ...); evexm(r, base, index,
        # map, w, v, ll, pp, ...).
        k = 2 if m.group(0).startswith('evexr') else 3
        if args[k].isdigit() and args[k + 1].isdigit() and args[k + 4].isdigit():
            out += [0x100 + int(args[k]), 0x200 + int(args[k + 1]), 0x300 + int(args[k + 4])]
    return out


LLVM_CHOICES = [
    # xchg %ax, %ax is 66 90 (with eax, 90 would be nop, which does not
    # clear the top half).
    ('xchgw {o0}, {o1}', 'acc:0=o0 acc2:0=o1 01100110 | 10010000'),
    # and xchg %rax, %rax is 90.
    ('xchgq {o0}, {o1}', 'acc:0=o0 acc2:0=o1 10010000'),
] + [
    # An x87 operation of st(0) with st(0) is D8's form, st(0) op= st(i).
    (f'{mn} %st, {{o0}}', f'acc:0=o0 11011000 | 11 {k:03b} 000')
    for mn, k in [('fadd', 0), ('fmul', 1), ('fsub', 4), ('fsubr', 5), ('fdiv', 6), ('fdivr', 7)]
]


def merged(xed):
    """The entries with templates (att.txt), those written alike merged."""
    att = {}
    for line in open(os.path.join(HERE, 'att.txt')):
        name, tpl, length = line.rstrip('\n').split('\t')
        att[name] = (tpl, int(length))
    groups = {}
    for name, params, enc, holes, ext, iclass, rip in forms(xed):
        if 'X_' + name not in att:
            continue
        tpl, length = att['X_' + name]
        params, enc, tpl = canonical(params, enc, tpl)
        key = (tpl, tuple(params))
        g = groups.setdefault(key, {'name': name, 'alts': [], 'rip': rip, 'ext': ext})
        if enc not in [e for _, e in g['alts']]:
            g['alts'].append((length, enc))
    # An entry with a fixed register (the accumulator's short forms) joins
    # the one with an operand there, its register constrained by a
    # zero-width field.
    regs = register_names()
    for key in list(groups):
        tpl, params = key
        lits = re.findall(r'%[a-z0-9()]+', tpl)
        for lit in lits:
            for cls, names in regs.items():
                if lit not in names or cls in ('xe', 'ye'):
                    continue
                k = names.index(lit)
                n = len(re.findall(r'\{o\d+\}', tpl))
                # The template with the register an operand: its name is
                # where it falls among the others.
                pos = tpl.index(lit)
                before = len(re.findall(r'\{o\d+\}', tpl[:pos]))
                shift = lambda m: f'{{o{int(m.group(1)) + 1}}}' if int(m.group(1)) >= before else m.group(0)
                t2 = re.sub(r'\{o(\d+)\}', shift, tpl)
                t2 = t2[:t2.index(lit)] + f'{{o{before}}}' + t2[t2.index(lit) + len(lit):]
                for gkey in groups:
                    if gkey[0] != t2 or gkey == key or class_of(gkey[1], f'o{before}') != cls:
                        continue
                    g = groups[gkey]
                    for length, enc in groups[key]['alts']:
                        e2 = re.sub(r'(?<![\w.%])o(\d+)(?![\w.(])',
                                    lambda m: f'o{int(m.group(1)) + 1}' if int(m.group(1)) >= before else m.group(0), enc)
                        g['alts'].append((length, f'acc:0=o{before}-{k} {e2}'))
                    groups[key]['folded'] = True
                    break
    # A shift or rotate by one (D0/D1 /r) joins the one by a count, as its
    # form for a count of 1; written without the count, it is still read,
    # but the disassembler reads its bytes as the counted form.
    for key in list(groups):
        tpl, params = key
        if any(p.endswith('%c') for p in params):
            continue
        mn, _, rest = tpl.partition(' ')
        if not re.match(r'(rol|ror|rcl|rcr|shl|shr|sar|sal)[bwlq]?$', mn):
            continue
        t2 = f'{mn} ${{c}}, {rest}' if rest else f'{mn} ${{c}}'
        p2 = ('i64 "imm" %c',) + params
        if (t2, p2) in groups:
            for length, enc in groups[key]['alts']:
                groups[(t2, p2)]['alts'].append((length, f'one:0=c-1 {enc}'))
            groups[key]['nodecode'] = True

    # llvm-mc's own choices, where it writes an instruction otherwise than
    # XED's forms would: (template, the encoding it uses, first).
    for tpl, enc in LLVM_CHOICES:
        for (t, params), g in groups.items():
            if t == tpl:
                g['alts'].insert(0, (0, enc))

    # Those llvm-mc 18 does not assemble (rejects.txt): newer than it, or
    # forms only newer disassemblers write.
    rejects = set(open(os.path.join(HERE, 'rejects.txt')).read().split())
    commute = set(open(os.path.join(HERE, 'commute.txt')).read().split())
    order = {}
    for line in open(os.path.join(HERE, 'order.txt')):
        name, ops = line.rstrip('\n').split('\t')
        order.setdefault(name, []).append([int(b, 16) for b in ops.split()])
    for (tpl, params), g in groups.items():
        if g.get('folded') or 'X_' + g['name'] in rejects:
            continue
        # Shortest first; of those as short, the one llvm-mc writes.
        prefs = order.get('X_' + g['name'], [])
        alts = [e for _, e in sorted(g['alts'], key=lambda a: (a[0], opcode(a[1]) not in prefs))]
        # A commutative VEX operation llvm-mc swaps the sources of, for the
        # two-byte prefix: the swapped form as an alternative.
        swapped = lambda e: re.sub(r'\bo([01])\b', lambda m: 'o' + str(1 - int(m.group(1))), e)
        if 'X_' + g['name'] in commute:
            alts.append(swapped(alts[0]))
        # A packed compare is commuted when its predicate is symmetric (equal,
        # unordered, their negations, false, true: low bits 00 or 11).
        elif re.match(r'vcmpp[sd] \$\{c\}, \{o0\}, \{o1\}, \{o2\}$', tpl):
            alts.append('if(((c&3)==0)|((c&3)==3)) ' + swapped(alts[0]))
        # Where VEX forms differ in which register is in ModRM.rm (a move's
        # load and store forms, or a swap), each that can have the two-byte
        # prefix (ModRM.rm below 8) goes before any with three.
        short = [a for a in alts if two_byte_vex(a)]
        if len({two_byte_vex(a) for a in short}) > 1:
            alts = [f'if({two_byte_vex(a)}<8) {a}' for a in short] + alts
        enc = alts[0] if len(alts) == 1 else '{ ' + ' ; '.join(alts) + ' }'
        # (Without an EVEX form, only the 16 registers VEX reaches.)
        if 'evex' not in enc:
            params = tuple(p.replace('"xe"', '"x"').replace('"ye"', '"y"') for p in params)
        yield g['name'], list(params), enc, tpl, g['rip'] + (' "nodecode"' if g.get('nodecode') else '')


def main():
    xed, mode = sys.argv[1], sys.argv[2]
    rev = os.popen(f'git -C {xed} log -1 --format=%h 2>/dev/null').read().strip() or 'unknown'
    out = [f"""; x86-64's instructions, for assembly and disassembly ("asmonly"). Written
; by gen_xed.py from the instruction tables of Intel XED (revision {rev},
; https://github.com/intelxed/xed), translated into this description's
; "enc" language, with AT&T templates read back from llvm-objdump (att.txt).
; Do not edit: change the script. XED's tables are under this license:
;
;   Copyright (c) Intel Corporation
;
;   Licensed under the Apache License, Version 2.0 (the "License"); you may
;   not use this file except in compliance with the License. You may obtain
;   a copy of the License at http://www.apache.org/licenses/LICENSE-2.0
;
;   Unless required by applicable law or agreed to in writing, software
;   distributed under the License is distributed on an "AS IS" BASIS,
;   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
;   See the License for the specific language governing permissions and
;   limitations under the License.
"""]
    if mode == 'raw':
        for k, (name, params, enc, holes, ext, iclass, rip) in enumerate(forms(xed)):
            asm = f'xf{k} ' + ', '.join(holes)
            out.append(f'define void @X_{name}({", ".join(params)}) "enc"="{enc}" "asm"="{asm.strip()}" '
                       f'"asmonly"' + rip + ' {\n  ret void\n}\n')
    else:
        for name, params, enc, tpl, rip in merged(xed):
            out.append(f'define void @X_{name}({", ".join(params)}) "enc"="{enc}" "asm"="{tpl}" '
                       f'"asmonly"' + rip + ' {\n  ret void\n}\n')
    sys.stdout.write('\n'.join(out))


if __name__ == "__main__":
    main()
