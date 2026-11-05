#!/usr/bin/env python3
"""Writes 82-sve.ll: SVE's instructions for selection, with bodies (what
each does, as IR of scalable vectors), from tables. The encodings are
81-scalable.ll's (written from Arm's specification, every one checked
against clang), found by their assembly templates.

Scalable vectors are in z0-z31 (class z), whatever the machine's vector
length. Little-endian, a whole register's load or store (ldr, str) is the
lanes' in order, of any lane width: it serves every type, with no
predicate. Entries need the feature sve.

Usage: gen_sve.py > 82-sve.ll
"""
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
out = []
ASM = None


def enc(asm, names):
    """81-scalable.ll's encoding for template `asm` (its operands named as
    there), with its operands renamed."""
    global ASM
    if ASM is None:
        ASM = {}
        for line in open(os.path.join(HERE, '81-scalable.ll')):
            m = re.match(r'define void @[\w.]+\(.*?\) "enc"="([^"]*)" "asm"="([^"]*)"', line)
            if m:
                ASM.setdefault(m.group(2), m.group(1))
    if asm not in ASM:
        raise KeyError(f'no entry of 81-scalable.ll is written {asm!r}')
    return re.sub(r'(?<![\w.])([a-z][a-z0-9_]*)(?![\w.(])', lambda m: names.get(m.group(1), m.group(1)), ASM[asm])


def entry(ret, name, params, e, asm, body, attrs=''):
    out.append(f'define {ret} @{name}({", ".join(params)}) "feature"="sve" "enc"="{e}" "asm"="{asm}"{attrs} '
               '"nodecode" {\n' + ''.join(f'  {line}\n' for line in body) + '}\n')


# Types: (IR type, lane suffix, name).
INT = [('<vscale x 16 x i8>', 'b'), ('<vscale x 8 x i16>', 'h'), ('<vscale x 4 x i32>', 's'),
       ('<vscale x 2 x i64>', 'd')]
FLT = [('<vscale x 8 x half>', 'h'), ('<vscale x 4 x float>', 's'), ('<vscale x 2 x double>', 'd')]
ALL = INT + FLT + [('<vscale x 8 x bfloat>', 'h')]
tag = lambda ty: re.sub(r'<vscale x (\d+) x (\w+)>', r'nx\1\2', ty)


def copies_and_memory():
    """Copies; spills and their frames; whole registers to and from
    memory."""
    z = '<vscale x 16 x i8>'
    entry(z, 'MOVz', [f'{z} "reg"="z" %n'], enc('orr {zd}.d, {zn}.d, {zm}.d', {'zd': 'd', 'zn': 'n', 'zm': 'n'}),
          'mov {d}.d, {n}.d', [f'ret {z} %n'], ' "copy"="z"')
    # Spills: at a slot's offset in vector lengths (frame.rs).
    entry(z, 'LDRzspill', ['ptr "reg"="xsp" %n', 'i64 "imm" %off'],
          enc('ldr {zt}, [{xn_sp}, #{imm}, mul vl]', {'zt': 'd', 'xn_sp': 'n', 'imm': 'off'}),
          'ldr {d}, [{n}, #{off}, mul vl]', [f'%r = load {z}, ptr %n', f'ret {z} %r'], ' "role"="load.z" "nomatch"')
    entry('void', 'STRzspill', [f'{z} "reg"="z" %t', 'ptr "reg"="xsp" %n', 'i64 "imm" %off'],
          enc('str {zt}, [{xn_sp}, #{imm}, mul vl]', {'zt': 't', 'xn_sp': 'n', 'imm': 'off'}),
          'str {t}, [{n}, #{off}, mul vl]', [f'store {z} %t, ptr %n', 'ret void'], ' "role"="store.z" "nomatch"')
    # Predicates: copies (mov, orr of a predicate with itself, by itself)
    # and spills, in eighths of a vector length.
    pr = '<vscale x 16 x i1>'
    entry(pr, 'MOVp', [f'{pr} "reg"="p" %n'],
          enc('orr {pd}.b, {pg}/z, {pn}.b, {pm}.b', {'pd': 'd', 'pg': 'n', 'pn': 'n', 'pm': 'n'}),
          'mov {d}.b, {n}.b', [f'ret {pr} %n'], ' "copy"="p"')
    entry(pr, 'LDRpspill', ['ptr "reg"="xsp" %n', 'i64 "imm" %off'],
          enc('ldr {pt}, [{xn_sp}, #{imm}, mul vl]', {'pt': 'd', 'xn_sp': 'n', 'imm': 'off'}),
          'ldr {d}, [{n}, #{off}, mul vl]', [f'%r = load {pr}, ptr %n', f'ret {pr} %r'],
          ' "role"="load.p,load.pg" "nomatch"')
    entry('void', 'STRpspill', [f'{pr} "reg"="p" %t', 'ptr "reg"="xsp" %n', 'i64 "imm" %off'],
          enc('str {pt}, [{xn_sp}, #{imm}, mul vl]', {'pt': 't', 'xn_sp': 'n', 'imm': 'off'}),
          'str {t}, [{n}, #{off}, mul vl]', [f'store {pr} %t, ptr %n', 'ret void'], ' "role"="store.p,store.pg" "nomatch"')
    # Predicates' constants: all lanes on, of a width, and none.
    for n, lane in [(16, 'b'), (8, 'h'), (4, 's'), (2, 'd')]:
        ty = f'<vscale x {n} x i1>'
        entry(ty, f'PTRUE{lane}', [], enc(f'ptrue {{pd}}.{lane}', {'pd': 'd'}), f'ptrue {{d}}.{lane}',
              [f'ret {ty} splat (i1 true)'], f' "dclass"="p" "role"="ptrue.{lane}" "nomatch"')
    entry(pr, 'PFALSE', [], enc('pfalse {pd}.b', {'pd': 'd'}), 'pfalse {d}.b', [f'ret {pr} zeroinitializer'],
          ' "dclass"="p" "role"="pfalse" "nomatch"')
    # Whole predicates to and from memory (svbool's: a bit for each byte of
    # a vector).
    entry(pr, 'LDRp', ['ptr "reg"="xsp" %n'], enc('ldr {pt}, [{xn_sp}]', {'pt': 'd', 'xn_sp': 'n'}), 'ldr {d}, [{n}]',
          [f'%r = load {pr}, ptr %n', f'ret {pr} %r'])
    entry('void', 'STRp', [f'{pr} "reg"="p" %t', 'ptr "reg"="xsp" %n'], enc('str {pt}, [{xn_sp}]', {'pt': 't', 'xn_sp': 'n'}),
          'str {t}, [{n}]', [f'store {pr} %t, ptr %n', 'ret void'])
    # A predicate of fewer lanes from and to svbool: the same register
    # (from), and with the bits that are not lanes cleared (to: and with
    # an all-true predicate of its lanes).
    mov = enc('orr {pd}.b, {pg}/z, {pn}.b, {pm}.b', {'pd': 'd', 'pg': 'n', 'pn': 'n', 'pm': 'n'})
    for n, lane in [(8, 'h'), (4, 's'), (2, 'd')]:
        ty = f'<vscale x {n} x i1>'
        out.append(f'declare {ty} @llvm.aarch64.sve.convert.from.svbool.nxv{n}i1({pr})\n'
                   f'declare {pr} @llvm.aarch64.sve.convert.to.svbool.nxv{n}i1({ty})\n')
        entry(ty, f'FROMSVBOOL{lane}', [f'{pr} "reg"="p" %n'], mov, 'mov {d}.b, {n}.b',
              [f'%r = call {ty} @llvm.aarch64.sve.convert.from.svbool.nxv{n}i1({pr} %n)', f'ret {ty} %r'], ' "dclass"="p"')
        entry(pr, f'PTRUE16{lane}', [], enc(f'ptrue {{pd}}.{lane}', {'pd': 'd'}), f'ptrue {{d}}.{lane}',
              [f'ret {pr} splat (i1 true)'], ' "dclass"="p" "nomatch"')
        entry(pr, f'ANDZ16{lane}', [f'{pr} "reg"="pg" %g', f'{ty} "reg"="p" %a'],
              enc('and {pd}.b, {pg}/z, {pn}.b, {pm}.b', {'pd': 'd', 'pg': 'g', 'pn': 'a', 'pm': 'a'}),
              'and {d}.b, {g}/z, {a}.b, {a}.b', [f'ret {pr} %g'], ' "dclass"="p" "nomatch"')
        out.append(f'define {pr} @to.svbool.{lane}({ty} "reg"="p" %a) "recipe"="to.svbool.{lane}.expand" {{\n'
                   f'  %r = call {pr} @llvm.aarch64.sve.convert.to.svbool.nxv{n}i1({ty} %a)\n  ret {pr} %r\n}}\n'
                   f'define {pr} @to.svbool.{lane}.expand({ty} %a) {{\n  %t = call {pr} @PTRUE16{lane}()\n'
                   f'  %r = call {pr} @ANDZ16{lane}({pr} %t, {ty} %a)\n  ret {pr} %r\n}}\n')
    # sp by vector lengths.
    entry('i64', 'ADDVL', ['i64 "reg"="xsp" %n', 'i64 "imm" %k'],
          enc('addvl {xd_sp}, {xn_sp}, #{imm}', {'xd_sp': 'd', 'xn_sp': 'n', 'imm': 'k'}),
          'addvl {d}, {n}, #{k}', ['ret i64 %n'], ' "dclass"="xsp" "role"="addvl" "nomatch"')
    for ty, _ in ALL:
        T = tag(ty)
        entry(ty, f'LDRz.{T}', ['ptr "reg"="xsp" %n'], enc('ldr {zt}, [{xn_sp}]', {'zt': 'd', 'xn_sp': 'n'}),
              'ldr {d}, [{n}]', [f'%r = load {ty}, ptr %n', f'ret {ty} %r'])
        entry('void', f'STRz.{T}', [f'{ty} "reg"="z" %t', 'ptr "reg"="xsp" %n'],
              enc('str {zt}, [{xn_sp}]', {'zt': 't', 'xn_sp': 'n'}), 'str {t}, [{n}]',
              [f'store {ty} %t, ptr %n', 'ret void'])


def arithmetic():
    """Unpredicated arithmetic: of integers, lane by lane; logic, of any
    lanes (as .d); of floats."""
    for ty, s in INT:
        T = tag(ty)
        for op in ('add', 'sub'):
            entry(ty, f'{op.upper()}z.{T}', [f'{ty} "reg"="z" %n', f'{ty} "reg"="z" %m'],
                  enc(f'{op} {{zd}}.{s}, {{zn}}.{s}, {{zm}}.{s}', {'zd': 'd', 'zn': 'n', 'zm': 'm'}),
                  f'{op} {{d}}.{s}, {{n}}.{s}, {{m}}.{s}', [f'%r = {op} {ty} %n, %m', f'ret {ty} %r'])
        for op in ('and', 'orr', 'eor'):
            ir = {'orr': 'or', 'eor': 'xor'}.get(op, op)
            entry(ty, f'{op.upper()}z.{T}', [f'{ty} "reg"="z" %n', f'{ty} "reg"="z" %m'],
                  enc(f'{op} {{zd}}.d, {{zn}}.d, {{zm}}.d', {'zd': 'd', 'zn': 'n', 'zm': 'm'}),
                  f'{op} {{d}}.d, {{n}}.d, {{m}}.d', [f'%r = {ir} {ty} %n, %m', f'ret {ty} %r'])
    for ty, s in FLT:
        T = tag(ty)
        for op in ('fadd', 'fsub', 'fmul'):
            entry(ty, f'{op.upper()}z.{T}', [f'{ty} "reg"="z" %n', f'{ty} "reg"="z" %m'],
                  enc(f'{op} {{zd}}.{s}, {{zn}}.{s}, {{zm}}.{s}', {'zd': 'd', 'zn': 'n', 'zm': 'm'}),
                  f'{op} {{d}}.{s}, {{n}}.{s}, {{m}}.{s}', [f'%r = {op} {ty} %n, %m', f'ret {ty} %r'])


ENTRIES = None


def entry_of(name):
    """81-scalable.ll's entry `name`: its params (name: (kind, class)),
    encoding and template."""
    global ENTRIES
    if ENTRIES is None:
        ENTRIES = {}
        for line in open(os.path.join(HERE, '81-scalable.ll')):
            m = re.match(r'define void @([\w.]+)\((.*?)\) "enc"="([^"]*)" "asm"="([^"]*)"', line)
            if m:
                params = {n: (k, c) for k, c, n in re.findall(r'"(reg|imm|sym)"(?:="(\w+)")? %([\w.]+)', m.group(2))}
                ENTRIES[m.group(1)] = (params, m.group(3), m.group(4))
    return ENTRIES[name]


def fixed(e, name, v):
    """Encoding `e` with operand `name` the constant `v`: its fields'
    bits written in."""
    def tok(t):
        m = re.fullmatch(r'([A-Za-z0-9_]+):(s?)(\d+)(?:=(.*))?', t)
        if not m:
            return t
        field, signed, width, expr = m.groups()
        expr = expr if expr is not None else field
        if not re.search(rf'(?<![\w.]){re.escape(name)}(?![\w.])', expr):
            return t
        x = eval(re.sub(rf'(?<![\w.]){re.escape(name)}(?![\w.])', f'({v})', expr), {})
        return format(x & ((1 << int(width)) - 1), f'0{width}b')
    return ' '.join(tok(t) for t in e.split(' '))


def class_of(ty):
    """The class an argument of type `ty` is in, where no operand says."""
    if ty.endswith('x i1>'):
        return 'p'
    if ty.startswith('<'):
        return 'z'
    return {'float': 's', 'double': 'd', 'i64': 'x', 'ptr': 'x'}.get(ty, 'w')


def intrinsics():
    """SVE's intrinsics (llvm.aarch64.sve.*) that are one of 81-scalable.ll's
    instructions, as discover_sve.py found them (sve-intrinsics.txt): the
    body calls the intrinsic, the operands are its arguments and result;
    the features are those Rust's core::arch asks for it."""
    declared = set()
    for line in open(os.path.join(HERE, 'sve-intrinsics.txt')):
        f = line.rstrip('\n').split(' ; ')
        if line.startswith('#') or len(f) < 8 or f[3] != 'entry':
            continue
        name, sig, feats, _, ename, operands, extra = f[:7]
        # (Not of half or bfloat scalars, which have no class here yet.)
        ret, args = sig[:-1].split('(', 1)
        args = split_top(args)
        params81, e, asm = entry_of(ename)
        names = dict(o.split('=') for o in operands.split())
        extra = extra.split()
        tied = next((x[5:] for x in extra if x.startswith('tied=')), None)
        consts = dict(x[6:].split(':') for x in extra if x.startswith('const='))
        of = {v: k for k, v in names.items()}
        # Operands that are constants, or the zero register, written in.
        rename = {}
        for n, v in names.items():
            if v == 'zr':
                e = fixed(e, n, 31)
                asm = asm.replace('{' + n + '}', 'xzr' if params81[n][1] in ('x', 'xsp') else 'wzr')
            elif re.fullmatch(r'-?\d+', v):
                e = fixed(e, n, int(v))
                asm = asm.replace('{' + n + '}', v)
            else:
                rename[n] = v
        params, call = [], []
        bad = False
        for k, a in enumerate(args):
            ty = a.split(' imm')[0]
            ak = f'a{k}'
            if ak in consts:
                call.append(f'{ty} {consts[ak]}')
                continue
            call.append(f'{ty} %{ak}')
            if ' imm' in a and ak in of:
                params.append(f'{ty} "imm" %{ak}')
            elif ak in of:
                params.append(f'{ty} "reg"="{params81[of[ak]][1]}" %{ak}')
            elif ak == tied:
                params.append(f'{ty} "reg"="{params81[of["d"]][1]}" "tied" %{ak}')
            else:
                params.append(f'{ty} "reg"="{class_of(ty)}" %{ak}')
        if any(c in ('h', 'b', 'q') and ('"reg"="' + c + '"') in ' '.join(params) for c in 'hbq'):
            bad = True
        dclass = params81[of['d']][1] if 'd' in of else None
        if bad or dclass in ('h', 'b', 'q') or 'half' in sig or 'bfloat' in sig and not ret.startswith('<'):
            continue
        e = re.sub(r'(?<![\w.])([a-z][a-z0-9_]*)(?![\w.(])', lambda m: rename.get(m.group(1), m.group(1)), e)
        for n, v in rename.items():
            asm = re.sub(r'\{' + re.escape(n) + r'(\}|:)', '{' + v + r'\1', asm)
            asm = re.sub(r'(\{=[^}]*)(?<![\w.])' + re.escape(n) + r'(?![\w.])', r'\1' + v, asm)
        body = ([f'call void @{name}({", ".join(call)})', 'ret void'] if ret == 'void'
                else [f'%r = call {ret} @{name}({", ".join(call)})', f'ret {ret} %r'])
        attrs = f' "dclass"="{dclass}"' if dclass else ''
        out.append(f'define {ret} @SVE.{name[len("llvm.aarch64.sve."):]}({", ".join(params)}) "feature"="{feats}" '
                   f'"enc"="{e}" "asm"="{asm}"{attrs} "nodecode" {{\n' + ''.join(f'  {b}\n' for b in body) + '}\n')
        if name not in declared:
            types = ', '.join(a.split(' imm')[0] for a in args)
            out.append(f'declare {ret} @{name}({types})\n')
            declared.add(name)


def split_top(s):
    out, depth, cur = [], 0, ''
    for c in s:
        depth += c in '<{('
        depth -= c in '>})'
        if c == ',' and depth == 0:
            out.append(cur.strip())
            cur = ''
        else:
            cur += c
    return out + ([cur.strip()] if cur.strip() else [])


def main():
    out.append("""; SVE's instructions for selection. Written by gen_sve.py; do not edit.
; The conventions are those of 00-base.ll; docs/design/sve.md has the plan.
""")
    copies_and_memory()
    arithmetic()
    intrinsics()
    sys.stdout.write('\n'.join(out))


main()
