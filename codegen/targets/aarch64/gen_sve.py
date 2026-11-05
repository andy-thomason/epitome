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


def recipe(name, ret, params, body, expand):
    """A recipe: IR (`body`, of `params`: (type, class, name)) that is no
    one instruction, and the entries it is made of (`expand`)."""
    ps = ', '.join(f'{t} "reg"="{c}" %{n}' for t, c, n in params)
    plain = ', '.join(f'{t} %{n}' for t, _, n in params)
    out.append(f'define {ret} @{name}({ps}) "feature"="sve" "recipe"="{name}.expand" {{\n'
               + ''.join(f'  {line}\n' for line in body) + '}\n')
    out.append(f'define {ret} @{name}.expand({plain}) {{\n' + ''.join(f'  {line}\n' for line in expand) + '}\n')


def narrow_reduction(intr, ty, l, t, P):
    """A reduction to a byte or half (orv b0): its result in the s register
    of the same number, as b and h values have no class to be copied out
    of, and the instruction clears the bits above them."""
    for line in open(os.path.join(HERE, 'sve-intrinsics.txt')):
        f = line.rstrip('\n').split(' ; ')
        if f[0] == f'llvm.aarch64.sve.{intr}.{t}' and len(f) >= 8 and f[3] == 'entry':
            break
    else:
        return
    _, e, asm = entry_of(f[4])
    names = dict(o.split('=') for o in f[5].split())
    rename = {n: {'a0': 'p', 'a1': 'a', 'd': 'd'}[v] for n, v in names.items()}
    e = re.sub(r'(?<![\w.])([a-z][a-z0-9_]*)(?![\w.(])', lambda m: rename.get(m.group(1), m.group(1)), e)
    for n, v in rename.items():
        asm = re.sub(r'\{' + re.escape(n) + r'\}', '{' + v + (':' + l if v == 'd' else '') + '}', asm)
    entry('i32', f'{intr.upper()}{l}', [f'{P} "reg"="pg" %p', f'{ty} "reg"="z" %a'], e, asm, ['ret i32 0'],
          ' "dclass"="s" "nomatch"')


def generic():
    """Generic IR of scalable vectors, as the vectoriser makes it: what has
    no unpredicated instruction is the predicated one under an all-true
    predicate of its lanes (ptrue), from SVE's intrinsics' entries."""
    have = {line.split('@')[1].split('(')[0] for line in out if line.startswith('define') and '@SVE.' in line}
    # dup.x makes scalable constants: a role for isel to find it by.
    for i, line in enumerate(out):
        if line.startswith('define') and '@SVE.dup.x.' in line:
            t = line.split('@SVE.dup.x.')[1].split('(')[0]
            out[i] = line.replace(' "nodecode"', f' "role"="splat.{t}" "nodecode"', 1)
    lanes = {'b': 16, 'h': 8, 's': 4, 'd': 2}
    INTS = [('<vscale x 16 x i8>', 'b', 'nxv16i8'), ('<vscale x 8 x i16>', 'h', 'nxv8i16'),
            ('<vscale x 4 x i32>', 's', 'nxv4i32'), ('<vscale x 2 x i64>', 'd', 'nxv2i64')]
    FLTS = [('<vscale x 4 x float>', 's', 'nxv4f32', 'float'), ('<vscale x 2 x double>', 'd', 'nxv2f64', 'double')]

    def pred(lane):
        return f'<vscale x {lanes[lane]} x i1>'

    def predicated(name, ty, lane, intr, ir, args, call_args):
        """`ir` (lines) as SVE's intrinsic `intr` under ptrue."""
        if f'SVE.{intr}' not in have:
            return
        P = pred(lane)
        recipe(name, ty, args, ir, [f'%p = call {P} @PTRUE{lane}()', f'%r = call {ty} @SVE.{intr}({call_args})',
                                    f'ret {ty} %r'])

    for ty, l, t in INTS:
        P = pred(l)
        two = [(ty, 'z', 'a'), (ty, 'z', 'b')]
        for op, intr in [('mul', 'mul'), ('sdiv', 'sdiv'), ('udiv', 'udiv'), ('shl', 'lsl'), ('lshr', 'lsr'),
                         ('ashr', 'asr')]:
            predicated(f'sve.{op}.{t}', ty, l, f'{intr}.{t}', [f'%r = {op} {ty} %a, %b', f'ret {ty} %r'], two,
                       f'{P} %p, {ty} %a, {ty} %b')
        for op in ('smax', 'smin', 'umax', 'umin'):
            out.append(f'declare {ty} @llvm.{op}.{t}({ty}, {ty})\n')
            predicated(f'sve.{op}.{t}', ty, l, f'{op}.{t}', [f'%r = call {ty} @llvm.{op}.{t}({ty} %a, {ty} %b)',
                                                           f'ret {ty} %r'], two, f'{P} %p, {ty} %a, {ty} %b')
        # Comparisons, into a predicate; those SVE has not, the other way.
        for pr, intr, swap in [('eq', 'cmpeq', False), ('ne', 'cmpne', False), ('sgt', 'cmpgt', False),
                               ('sge', 'cmpge', False), ('ugt', 'cmphi', False), ('uge', 'cmphs', False),
                               ('slt', 'cmpgt', True), ('sle', 'cmpge', True), ('ult', 'cmphi', True),
                               ('ule', 'cmphs', True)]:
            if f'SVE.{intr}.{t}' not in have:
                continue
            x, y = ('%b', '%a') if swap else ('%a', '%b')
            recipe(f'sve.icmp.{pr}.{t}', P, two, [f'%r = icmp {pr} {ty} %a, %b', f'ret {P} %r'],
                   [f'%p = call {P} @PTRUE{l}()', f'%r = call {P} @SVE.{intr}.{t}({P} %p, {ty} {x}, {ty} {y})',
                    f'ret {P} %r'])
        # Selects, lane by lane by a predicate.
        size = format('bhsd'.index(l), '02b')
        entry(ty, f'SELz.{t}', [f'{P} "reg"="p" %c', f'{ty} "reg"="z" %a', f'{ty} "reg"="z" %b'],
              f'00000101{size}1 b:5 11 c:4 a:5 d:5', f'sel {{d}}.{l}, {{c}}, {{a}}.{l}, {{b}}.{l}',
              [f'%r = select {P} %c, {ty} %a, {ty} %b', f'ret {ty} %r'])
        # Reductions.
        for red, intr, wide in [('add', 'uaddv', True), ('smax', 'smaxv', False), ('smin', 'sminv', False),
                                ('umax', 'umaxv', False), ('umin', 'uminv', False), ('and', 'andv', False),
                                ('or', 'orv', False), ('xor', 'eorv', False)]:
            et = 'i' + str({'b': 8, 'h': 16, 's': 32, 'd': 64}[l])
            if not wide and l in 'bh':
                narrow_reduction(intr, ty, l, t, P)
                tail = [f'%v = call i32 @{intr.upper()}{l}({P} %p, {ty} %a)', f'%r = trunc i32 %v to {et}']
                out.append(f'declare {et} @llvm.vector.reduce.{red}.{t}({ty})\n')
                recipe(f'sve.reduce.{red}.{t}', et, [(ty, 'z', 'a')],
                       [f'%r = call {et} @llvm.vector.reduce.{red}.{t}({ty} %a)', f'ret {et} %r'],
                       [f'%p = call {P} @PTRUE{l}()'] + tail + [f'ret {et} %r'])
                continue
            if f'SVE.{intr}.{t}' not in have:
                continue
            out.append(f'declare {et} @llvm.vector.reduce.{red}.{t}({ty})\n')
            rt = 'i64' if wide else et
            tail = [f'%v = call i64 @SVE.{intr}.{t}({P} %p, {ty} %a)', f'%r = trunc i64 %v to {et}'] if wide and et != 'i64' \
                else [f'%r = call {rt} @SVE.{intr}.{t}({P} %p, {ty} %a)']
            recipe(f'sve.reduce.{red}.{t}', et, [(ty, 'z', 'a')], [f'%r = call {et} @llvm.vector.reduce.{red}.{t}({ty} %a)',
                                                                 f'ret {et} %r'],
                   [f'%p = call {P} @PTRUE{l}()'] + tail + [f'ret {et} %r'])
        # A splat of a scalar.
        et = {'s': 'i32', 'd': 'i64'}.get(l)
        if et and f'SVE.dup.x.{t}' in have:
            recipe(f'sve.splat.{t}', ty, [(et, 'w' if l == 's' else 'x', 'x')],
                   [f'%i = insertelement {ty} poison, {et} %x, i64 0',
                    f'%r = shufflevector {ty} %i, {ty} poison, <vscale x {lanes[l]} x i32> zeroinitializer',
                    f'ret {ty} %r'], [f'%r = call {ty} @SVE.dup.x.{t}({et} %x)', f'ret {ty} %r'])
    for ty, l, t, et in FLTS:
        P = pred(l)
        two = [(ty, 'z', 'a'), (ty, 'z', 'b')]
        predicated(f'sve.fdiv.{t}', ty, l, f'fdiv.{t}', [f'%r = fdiv {ty} %a, %b', f'ret {ty} %r'], two,
                   f'{P} %p, {ty} %a, {ty} %b')
        for op, intr in [('minnum', 'fminnm'), ('maxnum', 'fmaxnm')]:
            out.append(f'declare {ty} @llvm.{op}.{t}({ty}, {ty})\n')
            predicated(f'sve.{op}.{t}', ty, l, f'{intr}.{t}', [f'%r = call {ty} @llvm.{op}.{t}({ty} %a, {ty} %b)',
                                                             f'ret {ty} %r'], two, f'{P} %p, {ty} %a, {ty} %b')
        predicated(f'sve.fneg.{t}', ty, l, f'fneg.{t}', [f'%r = fneg {ty} %a', f'ret {ty} %r'], [(ty, 'z', 'a')],
                   f'{ty} %a, {P} %p, {ty} %a')
        for op, intr in [('fabs', 'fabs'), ('sqrt', 'fsqrt')]:
            out.append(f'declare {ty} @llvm.{op}.{t}({ty})\n')
            predicated(f'sve.{op}.{t}', ty, l, f'{intr}.{t}', [f'%r = call {ty} @llvm.{op}.{t}({ty} %a)', f'ret {ty} %r'],
                       [(ty, 'z', 'a')], f'{ty} %a, {P} %p, {ty} %a')
        # a * b + c: fmla, accumulating into c.
        for op in ('fmuladd', 'fma'):
            out.append(f'declare {ty} @llvm.{op}.{t}({ty}, {ty}, {ty})\n')
            predicated(f'sve.{op}.{t}', ty, l, f'fmla.{t}',
                       [f'%r = call {ty} @llvm.{op}.{t}({ty} %a, {ty} %b, {ty} %c)', f'ret {ty} %r'],
                       [(ty, 'z', 'a'), (ty, 'z', 'b'), (ty, 'z', 'c')], f'{P} %p, {ty} %c, {ty} %a, {ty} %b')
        for pr, intr, swap in [('oeq', 'fcmpeq', False), ('une', 'fcmpne', False), ('ogt', 'fcmpgt', False),
                               ('oge', 'fcmpge', False), ('olt', 'fcmpgt', True), ('ole', 'fcmpge', True),
                               ('uno', 'fcmpuo', False)]:
            if f'SVE.{intr}.{t}' not in have:
                continue
            x, y = ('%b', '%a') if swap else ('%a', '%b')
            recipe(f'sve.fcmp.{pr}.{t}', P, two, [f'%r = fcmp {pr} {ty} %a, %b', f'ret {P} %r'],
                   [f'%p = call {P} @PTRUE{l}()', f'%r = call {P} @SVE.{intr}.{t}({P} %p, {ty} {x}, {ty} {y})',
                    f'ret {P} %r'])
        size = format('bhsd'.index(l), '02b')
        entry(ty, f'SELz.{t}', [f'{P} "reg"="p" %c', f'{ty} "reg"="z" %a', f'{ty} "reg"="z" %b'],
              f'00000101{size}1 b:5 11 c:4 a:5 d:5', f'sel {{d}}.{l}, {{c}}, {{a}}.{l}, {{b}}.{l}',
              [f'%r = select {P} %c, {ty} %a, {ty} %b', f'ret {ty} %r'])
        # Ordered sums (fadda), with their start.
        if f'SVE.fadda.{t}' in have:
            out.append(f'declare {et} @llvm.vector.reduce.fadd.{t}({et}, {ty})\n')
            recipe(f'sve.reduce.fadd.{t}', et, [(et, l, 's'), (ty, 'z', 'a')],
                   [f'%r = call {et} @llvm.vector.reduce.fadd.{t}({et} %s, {ty} %a)', f'ret {et} %r'],
                   [f'%p = call {P} @PTRUE{l}()', f'%r = call {et} @SVE.fadda.{t}({P} %p, {et} %s, {ty} %a)',
                    f'ret {et} %r'])
        # Conversions of one width.
        it = INTS['bhsd'.index(l)]
        for op, intr in [('sitofp', f'scvtf.{t}.{it[2]}'), ('uitofp', f'ucvtf.{t}.{it[2]}')]:
            if f'SVE.{intr}' not in have:
                continue
            # The passthru is never read under ptrue: the source's bits.
            recipe(f'sve.{op}.{t}', ty, [(it[0], 'z', 'a')], [f'%r = {op} {it[0]} %a to {ty}', f'ret {ty} %r'],
                   [f'%p = call {P} @PTRUE{l}()', f'%q = bitcast {it[0]} %a to {ty}',
                    f'%r = call {ty} @SVE.{intr}({ty} %q, {P} %p, {it[0]} %a)', f'ret {ty} %r'])
        for op, intr in [('fptosi', f'fcvtzs.{it[2]}.{t}'), ('fptoui', f'fcvtzu.{it[2]}.{t}')]:
            if f'SVE.{intr}' not in have:
                continue
            recipe(f'sve.{op}.{t}', it[0], [(ty, 'z', 'a')], [f'%r = {op} {ty} %a to {it[0]}', f'ret {it[0]} %r'],
                   [f'%p = call {P} @PTRUE{l}()', f'%q = bitcast {ty} %a to {it[0]}', f'%r = call {it[0]} @SVE.{intr}({it[0]} %q, {P} %p, {ty} %a)',
                    f'ret {it[0]} %r'])
    # The vector length in 128-bit granules (vscale): rdvl's bytes, over 16.
    entry('i64', 'RDVL1', [], fixed(enc('rdvl {xd}, #{imm}', {'xd': 'd'}), 'imm', 1),
          'rdvl {d}, #1', ['ret i64 16'], ' "dclass"="x" "nomatch"')
    out.append('declare i64 @llvm.vscale.i64()\n')
    recipe('sve.vscale', 'i64', [], ['%r = call i64 @llvm.vscale.i64()', 'ret i64 %r'],
           ['%b = call i64 @RDVL1()', '%r = call i64 @LSRXri(i64 %b, i64 4)', 'ret i64 %r'])


def main():
    out.append("""; SVE's instructions for selection. Written by gen_sve.py; do not edit.
; The conventions are those of 00-base.ll; docs/design/sve.md has the plan.
""")
    copies_and_memory()
    arithmetic()
    intrinsics()
    generic()
    sys.stdout.write('\n'.join(out))


main()
