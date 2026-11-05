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


def main():
    out.append("""; SVE's instructions for selection. Written by gen_sve.py; do not edit.
; The conventions are those of 00-base.ll; docs/design/sve.md has the plan.
""")
    copies_and_memory()
    arithmetic()
    sys.stdout.write('\n'.join(out))


main()
