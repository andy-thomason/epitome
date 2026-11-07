#!/usr/bin/env python3
"""Writes 10-integer.ll: x86-64's integer instructions for selection, with
bodies (what each does, as IR), from tables. The encodings use the
fragments of 00-base.ll; the AT&T templates are as llvm-mc writes them
(tests/x86.rs checks every entry against it).

x86's arithmetic is two-address: the destination is also the first
source (a "tied" operand). Most of it writes the flags, which the code
generator is told ("clobbers"="flags"). `lea` adds without touching them,
three-address, for the code generator's own adds (roles addi, add).

Usage: gen_sel.py > 10-integer.ll
"""

import os
import sys

out = []

# Width: (IR type, class, REX.W, suffix, bits).
W = {64: ('i64', 'r64', 1, 'q', 64), 32: ('i32', 'r32', 0, 'l', 32)}
CC = [('o', 0), ('no', 1), ('b', 2), ('ae', 3), ('e', 4), ('ne', 5), ('be', 6), ('a', 7), ('s', 8), ('ns', 9),
      ('p', 10), ('np', 11), ('l', 12), ('ge', 13), ('le', 14), ('g', 15)]


def bits(v, n=8):
    return format(v, f'0{n}b')


def entry(ret, name, params, enc, asm, body, attrs='', decode=False):
    """An entry: `body` is the IR lines between braces. (None is read by the
    disassembler, 02-xed.ll's entries are, but those `decode` says.)"""
    if not decode and '"nodecode"' not in attrs:
        attrs += ' "nodecode"'
    out.append(f'define {ret} @{name}({", ".join(params)}) "enc"="{enc}" "asm"="{asm}"{attrs} {{\n'
               + ''.join(f'  {line}\n' for line in body) + '}\n')


FLAGS = ' "clobbers"="flags"'


def alu():
    # add, or, adc, sbb, and, sub, xor, cmp: their numbers.
    ops = [('add', 0, 'add'), ('or', 1, 'or'), ('and', 4, 'and'), ('sub', 5, 'sub'), ('xor', 6, 'xor')]
    for w in (64, 32):
        ty, cls, rw, sfx, _ = W[w]
        zext = '' if w == 64 else ' "zext"="32"'
        for mn, n, ir in ops:
            N = mn.upper() + str(w)
            # op m, d: d = d op m (01 + 8n /r: ModRM.reg the source).
            entry(ty, f'{N}rr', [f'{ty} "reg"="{cls}" "tied" %n', f'{ty} "reg"="{cls}" %m'],
                  f'rex({rw}, m, 0, d) | {bits(n * 8 + 1)} | rr(m, d)', f'{mn}{sfx} {{m}}, {{d}}',
                  [f'%r = {ir} {ty} %n, %m', f'ret {ty} %r'],
                  FLAGS + zext + (' "role"="sub"' if (mn, w) == ('sub', 64) else '')
                  + (' "role"="add2.r64"' if (mn, w) == ('add', 64) else ''))
            # op $imm, d: a byte if it fits (83 /n), else 32 bits (81 /n).
            # (The accumulator's own form, 05 + 8n, where the byte does not
            # hold the immediate: as llvm-mc writes it.)
            entry(ty, f'{N}ri', [f'{ty} "reg"="{cls}" "tied" %n', f'{ty} "imm" %imm'],
                  f'{{ rex({rw}, 0, 0, d) | 10000011 | rr({n}, d) | ib:s8=imm ; '
                  f'acc:0=d rex({rw}, 0, 0, 0) | {bits(n * 8 + 5)} | iv:s32=imm ; '
                  f'rex({rw}, 0, 0, d) | 10000001 | rr({n}, d) | iv:s32=imm }}',
                  f'{mn}{sfx} ${{imm}}, {{d}}', [f'%r = {ir} {ty} %n, %imm', f'ret {ty} %r'],
                  FLAGS + zext + (' "role"="add2i.r64"' if (mn, w) == ('add', 64) else ''))
        # (Roles add2 and add2i, the two-address adds: where one follows a
        # copy of its source, and the flags it sets are dead, peephole.rs
        # makes the two one lea, which is three-address.)
        # imul m, d (0F AF /r: ModRM.reg the destination).
        entry(ty, f'IMUL{w}rr', [f'{ty} "reg"="{cls}" "tied" %n', f'{ty} "reg"="{cls}" %m'],
              f'rex({rw}, d, 0, m) | 00001111 | 10101111 | rr(d, m)', f'imul{sfx} {{m}}, {{d}}',
              ['%r = mul ' + ty + ' %n, %m', f'ret {ty} %r'], FLAGS + zext)
        # imul $imm, n, d: three-address.
        entry(ty, f'IMUL{w}rri', [f'{ty} "reg"="{cls}" %n', f'{ty} "imm" %imm'],
              f'{{ rex({rw}, d, 0, n) | 01101011 | rr(d, n) | ib:s8=imm ; '
              f'rex({rw}, d, 0, n) | 01101001 | rr(d, n) | iv:s32=imm }}',
              f'imul{sfx} ${{imm}}, {{n}}, {{d}}', [f'%r = mul {ty} %n, %imm', f'ret {ty} %r'], FLAGS + zext)
        # Shifts: by an immediate (C1 /r ib), or by cl (D3 /r).
        for mn, r, ir in [('shl', 4, 'shl'), ('shr', 5, 'lshr'), ('sar', 7, 'ashr')]:
            N = mn.upper() + str(w)
            # (By one, D1 /r, as llvm-mc writes it.)
            entry(ty, f'{N}ri', [f'{ty} "reg"="{cls}" "tied" %n', f'{ty} "imm" %imm'],
                  f'{{ one:0=imm-1 rex({rw}, 0, 0, d) | 11010001 | rr({r}, d) ; '
                  f'rex({rw}, 0, 0, d) | 11000001 | rr({r}, d) | ib:8=imm }}', f'{mn}{sfx} ${{imm}}, {{d}}',
                  [f'%r = {ir} {ty} %n, %imm', f'ret {ty} %r'], FLAGS + zext)
            entry(ty, f'{N}rCL', [f'{ty} "reg"="{cls}" "tied" %n', f'{ty} "reg"="rcx" %c'],
                  f'rex({rw}, 0, 0, d) | 11010011 | rr({r}, d)', f'{mn}{sfx} %cl, {{d}}',
                  [f'%r = {ir} {ty} %n, %c', f'ret {ty} %r'], FLAGS + zext)
        for mn, r, body in [('neg', 3, f'%r = sub {ty} 0, %n'), ('not', 2, f'%r = xor {ty} %n, -1')]:
            entry(ty, f'{mn.upper()}{w}r', [f'{ty} "reg"="{cls}" "tied" %n'],
                  f'rex({rw}, 0, 0, d) | 11110111 | rr({r}, d)', f'{mn}{sfx} {{d}}', [body, f'ret {ty} %r'],
                  (FLAGS if mn == 'neg' else '') + zext)


def moves():
    for w in (64, 32):
        ty, cls, rw, sfx, _ = W[w]
        zext = '' if w == 64 else ' "zext"="32"'
        entry(ty, f'MOV{w}rr', [f'{ty} "reg"="{cls}" %n'], f'rex({rw}, n, 0, d) | 10001001 | rr(n, d)',
              f'mov{sfx} {{n}}, {{d}}', [f'ret {ty} %n'], f' "copy"="{cls}"' + zext)
    # Constants: movl zero-extends (B8+r id), movq sign-extends 32 bits
    # (C7 /0 id), movabsq has all 64.
    entry('i32', 'MOV32ri', ['i32 "imm" %imm'], '{ rex(0, 0, 0, d) | 10111 o:3=d&7 | iv:s32=imm ; '
          'rex(0, 0, 0, d) | 10111 o:3=d&7 | iv:32=imm }', 'movl ${imm}, {d}', ['ret i32 %imm'], ' "zext"="32"')
    entry('i64', 'MOV64ri32', ['i64 "imm" %imm'], 'rex(0, 0, 0, d) | 10111 o:3=d&7 | iv:32=imm',
          'movl ${imm}, {d:r32}', ['ret i64 %imm'], ' "nodecode"')
    entry('i64', 'MOV64ri', ['i64 "imm" %imm'], 'rex(1, 0, 0, d) | 11000111 | rr(0, d) | iv:s32=imm',
          'movq ${imm}, {d}', ['ret i64 %imm'])
    entry('i64', 'MOVABS64ri', ['i64 "imm" %imm'], 'rex(1, 0, 0, d) | 10111 o:3=d&7 | io:s64=imm',
          'movabsq ${imm}, {d}', ['ret i64 %imm'])
    # Extensions. A byte register's REX is there whenever it is spl-dil.
    rex8 = lambda r, b, w=0: (f'{{ if({b}<4) rex({w}, {r}, 0, {b}) ; 0100 W:1={w} R:1=({r})>>3 X:1=0 B:1=({b})>>3 }}'
                               if w == 0 else f'rex(1, {r}, 0, {b})')
    for (mn, ext, src, op, dst, roles) in [
            ('movzbl', 'zext', 'i8', '10110110', 32, ',zext.8'), ('movzwl', 'zext', 'i16', '10110111', 32, ',zext.16'),
            ('movsbl', 'sext', 'i8', '10111110', 32, ',sext.8'), ('movswl', 'sext', 'i16', '10111111', 32, ',sext.16'),
            ('movzbl', 'zext', 'i8', '10110110', 64, ''), ('movzwl', 'zext', 'i16', '10110111', 64, ''),
            ('movsbq', 'sext', 'i8', '10111110', 64, ''), ('movswq', 'sext', 'i16', '10111111', 64, '')]:
        ty, cls, rw, sfx, _ = W[dst]
        view = 'r8' if src == 'i8' else 'r16'
        # (Zero-extending to 64 bits is the 32-bit move's: it clears the top.)
        w = 1 if dst == 64 and ext == 'sext' else 0
        d = '{d}' if w else '{d:r32}'
        pre = rex8('d', 'n', w) if src == 'i8' else f'rex({w}, d, 0, n)'
        zb = ' "zext"="' + ('8' if src == 'i8' else '16') + '"' if ext == 'zext' else ''
        role = f' "role"="{roles[1:]}"' if roles else ''
        entry(ty, f'{mn.upper()}{dst}', [f'{src} "reg"="r32" %n'], f'{pre} | 00001111 | {op} | rr(d, n)',
              f'{mn} {{n:{view}}}, {d}', [f'%r = {ext} {src} %n to {ty}', f'ret {ty} %r'], zb + role)
    # A boolean widened: its lowest bit (what is above it is undefined: a
    # truncation leaves it).
    entry('i32', 'ZEXT1', ['i1 "reg"="r32" "tied" %n'], 'rex(0, 0, 0, d) | 10000011 | rr(4, d) | 00000001',
          'andl $1, {d}', ['%r = zext i1 %n to i32', 'ret i32 %r'], ' "zext"="1" "role"="zext.1"' + FLAGS)
    entry('i64', 'ZEXT1q', ['i1 "reg"="r32" "tied" %n'], 'rex(0, 0, 0, d) | 10000011 | rr(4, d) | 00000001',
          'andl $1, {d:r32}', ['%r = zext i1 %n to i64', 'ret i64 %r'], ' "zext"="1" "dclass"="r64"' + FLAGS)
    entry('i64', 'MOVSLQ', ['i32 "reg"="r32" %n'], 'rex(1, d, 0, n) | 01100011 | rr(d, n)', 'movslq {n}, {d}',
          ['%r = sext i32 %n to i64', 'ret i64 %r'], ' "role"="sext.32"')
    # Addresses, three-address and leaving the flags alone: the code
    # generator's adds.
    entry('ptr', 'LEA64ri', ['ptr "reg"="r64" %n', 'i64 "imm" %imm'],
          'rex(1, d, 4, n) | 10001101 | rm(d, n, 4, 1, imm, 0, 1)', 'leaq {imm}({n}), {d}',
          ['%r = getelementptr i8, ptr %n, i64 %imm', 'ret ptr %r'], ' "role"="addi"')
    entry('ptr', 'LEA64rr', ['ptr "reg"="r64" %n', 'i64 "reg"="r64" %m'],
          '{ if(m!=4) rex(1, d, m, n) | 10001101 | rm(d, n, m, 1, 0, 0, 1) }', 'leaq ({n},{m}), {d}',
          ['%r = getelementptr i8, ptr %n, i64 %m', 'ret ptr %r'], ' "role"="add"')
    entry('i64', 'LEA64add', ['i64 "reg"="r64" %n', 'i64 "reg"="r64" %m'],
          '{ if(m!=4) rex(1, d, m, n) | 10001101 | rm(d, n, m, 1, 0, 0, 1) }', 'leaq ({n},{m}), {d}',
          ['%r = add i64 %n, %m', 'ret i64 %r'], ' "nodecode"')
    entry('i64', 'LEA64addi', ['i64 "reg"="r64" %n', 'i64 "imm" %imm'],
          'rex(1, d, 4, n) | 10001101 | rm(d, n, 4, 1, imm, 0, 1)', 'leaq {imm}({n}), {d}',
          ['%r = add i64 %n, %imm', 'ret i64 %r'], ' "nodecode"')
    # A global's address, PC-relative; or its GOT entry's contents.
    entry('ptr', 'LEArip', ['ptr "sym" %s'], 'rex(1, d, 0, 0) | 10001101 | rm(d, 16, 4, 1, 0, s, 1)',
          'leaq {s}(%rip), {d}', ['ret ptr %s'], ' "reloc"="pc32x" "role"="addr" "nomatch" "nodecode"')
    entry('ptr', 'MOVgot', ['ptr "sym" %s'], 'rex(1, d, 0, 0) | 10001011 | rm(d, 16, 4, 1, 0, s, 1)',
          'movq {s}(%rip), {d}', ['ret ptr %s'], ' "reloc"="gotpcrel" "role"="got" "nomatch" "nodecode"')
    # Thread-local variables (initial exec): the offset from the GOT, and
    # the thread pointer, the word at fs:0 (read back by this entry: the
    # generated ones have no segment prefixes).
    entry('i64', 'MOVgottpoff', ['ptr "sym" %s'], 'rex(1, d, 0, 0) | 10001011 | rm(d, 16, 4, 1, 0, s, 1)',
          'movq {s}(%rip), {d}', ['ret i64 0'], ' "reloc"="gottpoff" "role"="tls.ie" "nomatch" "nodecode"')
    entry('ptr', 'MOVfs0', [], '01100100 | rex(1, d, 0, 0) | 10001011 | 00 r:3=d&7 100 | 00100101 | '
          '00000000 | 00000000 | 00000000 | 00000000', 'movq %fs:0, {d}', ['%p = call ptr @llvm.thread.pointer.p0()', 'ret ptr %p'],
          ' "role"="thread.pointer"', decode=True)
    out.append('declare ptr @llvm.thread.pointer.p0()\n')


def memory():
    """Loads and stores at a register plus a displacement."""
    rex8 = lambda r, x, b: (f'{{ if({r}<4) rex(0, {r}, {x}, {b}) ; 0100 0 R:1=({r})>>3 X:1=({x})>>3 B:1=({b})>>3 }}')
    # (By size, x for 8 bytes, and by the class spills and saves are in.)
    loads = [('i64', 'movq', '10001011', 1, 'load.x,load.r64', '', ''), ('i32', 'movl', '10001011', 0, 'load.w', '', '32'),
             ('i8', 'movzbl', '00001111 | 10110110', 0, 'load.b', '', '8'),
             ('i16', 'movzwl', '00001111 | 10110111', 0, 'load.h', '', '16')]
    for ty, mn, op, w, role, _, zext in loads:
        d = '{d}' if ty == 'i64' else '{d:r32}' if ty in ('i8', 'i16') else '{d}'
        cls = 'r64' if ty == 'i64' else 'r32'
        entry(ty, f'LD{ty.upper()}', ['ptr "reg"="r64" %n', 'i64 "imm" %off'],
              f'rex({w}, d, 4, n) | {op} | rm(d, n, 4, 1, off, 0, 1)', f'{mn} {{off}}({{n}}), {d}',
              ['%a = getelementptr i8, ptr %n, i64 %off', f'%r = load {ty}, ptr %a', f'ret {ty} %r'],
              f' "role"="{role}"' + (f' "zext"="{zext}"' if zext else '') + f' "dclass"="{cls}"')
    entry('ptr', 'LDP', ['ptr "reg"="r64" %n', 'i64 "imm" %off'], 'rex(1, d, 4, n) | 10001011 | rm(d, n, 4, 1, off, 0, 1)',
          'movq {off}({n}), {d}', ['%a = getelementptr i8, ptr %n, i64 %off', '%r = load ptr, ptr %a', 'ret ptr %r'],
          ' "nodecode"')
    # Sign-extending loads.
    for ty, src, mn, op, w in [('i32', 'i8', 'movsbl', '00001111 | 10111110', 0),
                               ('i32', 'i16', 'movswl', '00001111 | 10111111', 0),
                               ('i64', 'i8', 'movsbq', '00001111 | 10111110', 1),
                               ('i64', 'i16', 'movswq', '00001111 | 10111111', 1),
                               ('i64', 'i32', 'movslq', '01100011', 1)]:
        entry(ty, f'LD{src.upper()}S{ty[1:]}', ['ptr "reg"="r64" %n', 'i64 "imm" %off'],
              f'rex({w}, d, 4, n) | {op} | rm(d, n, 4, 1, off, 0, 1)', f'{mn} {{off}}({{n}}), {{d}}',
              ['%a = getelementptr i8, ptr %n, i64 %off', f'%v = load {src}, ptr %a', f'%r = sext {src} %v to {ty}',
               f'ret {ty} %r'])
    stores = [('i64', 'movq', '10001001', 1, 'store.x,store.r64', '{t}', 'r64'), ('i32', 'movl', '10001001', 0, 'store.w', '{t}', 'r32'),
              ('i16', 'movw', '10001001', 0, 'store.h', '{t:r16}', 'r32'), ('i8', 'movb', '10001000', 0, 'store.b', '{t:r8}', 'r32')]
    for ty, mn, op, w, role, t, cls in stores:
        pre = '01100110 | ' if ty == 'i16' else ''
        rx = rex8('t', 4, 'n') if ty == 'i8' else f'rex({w}, t, 4, n)'
        entry('void', f'ST{ty.upper()}', [f'{ty} "reg"="{cls}" %t', 'ptr "reg"="r64" %n', 'i64 "imm" %off'],
              f'{pre}{rx} | {op} | rm(t, n, 4, 1, off, 0, 1)', f'{mn} {t}, {{off}}({{n}})',
              ['%a = getelementptr i8, ptr %n, i64 %off', f'store {ty} %t, ptr %a', 'ret void'], f' "role"="{role}"')
    entry('void', 'STP', ['ptr "reg"="r64" %t', 'ptr "reg"="r64" %n', 'i64 "imm" %off'],
          'rex(1, t, 4, n) | 10001001 | rm(t, n, 4, 1, off, 0, 1)', 'movq {t}, {off}({n})',
          ['%a = getelementptr i8, ptr %n, i64 %off', 'store ptr %t, ptr %a', 'ret void'], ' "nodecode"')


def compares():
    for w in (64, 32):
        ty, cls, rw, sfx, _ = W[w]
        ext = [] if w == 64 else [f'%a = sext i32 %n to i64', f'%b = sext i32 %m to i64']
        a, b = ('%n', '%m') if w == 64 else ('%a', '%b')
        # cmp m, n: the flags of n - m (39 /r).
        entry('%flags', f'CMP{w}rr', [f'{ty} "reg"="{cls}" %n', f'{ty} "reg"="{cls}" %m'],
              f'rex({rw}, m, 0, n) | 00111001 | rr(m, n)', f'cmp{sfx} {{m}}, {{n}}',
              ext + [f'%f0 = insertvalue %flags poison, i64 {a}, 0', f'%f = insertvalue %flags %f0, i64 {b}, 1',
                     'ret %flags %f'], ' "sets"="zcso"')
        ext2 = [] if w == 64 else [f'%a = sext i32 %n to i64', f'%b = sext i32 %imm to i64']
        b2 = '%imm' if w == 64 else '%b'
        entry('%flags', f'CMP{w}ri', [f'{ty} "reg"="{cls}" %n', f'{ty} "imm" %imm'],
              f'{{ rex({rw}, 0, 0, n) | 10000011 | rr(7, n) | ib:s8=imm ; '
              f'acc:0=n rex({rw}, 0, 0, 0) | 00111101 | iv:s32=imm ; '
              f'rex({rw}, 0, 0, n) | 10000001 | rr(7, n) | iv:s32=imm }}', f'cmp{sfx} ${{imm}}, {{n}}',
              ext2 + [f'%f0 = insertvalue %flags poison, i64 {a}, 0', f'%f = insertvalue %flags %f0, i64 {b2}, 1',
                      'ret %flags %f'], f' "sets"="zcso" "role"="cmpi.{cls}"')
        # (A compare with 0 is a test of the register with itself, a byte
        # shorter, with the same flags: roles cmpi and test, for
        # peephole.rs's short forms.)
        # test m, n: the flags of n & m compared with 0.
        x = [f'%r = and {ty} %n, %m'] + ([] if w == 64 else ['%x = sext i32 %r to i64'])
        entry('%flags', f'TEST{w}rr', [f'{ty} "reg"="{cls}" %n', f'{ty} "reg"="{cls}" %m'],
              f'rex({rw}, m, 0, n) | 10000101 | rr(m, n)', f'test{sfx} {{m}}, {{n}}',
              x + [f'%f0 = insertvalue %flags poison, i64 {"%r" if w == 64 else "%x"}, 0',
                   '%f = insertvalue %flags %f0, i64 0, 1', 'ret %flags %f'], f' "sets"="zcso" "role"="test.{cls}"')


def control():
    entry('void', 'JMP', ['ptr "label" %dest'], '{ 11101011 | rel:s8=dest-end ; 11101001 | rel:s32=dest-end }',
          'jmp {dest}', ['indirectbr ptr %dest, []'], ' "role"="jump"')
    entry('void', 'JCC', ['i1 "cond" %c', 'ptr "label" %dest'],
          '{ 0111 cc:4=c | rel:s8=dest-end ; 00001111 | 1000 cc:4=c | rel:s32=dest-end }', 'j{c} {dest}',
          ['br i1 %c, label %taken, label %next', 'taken:', 'indirectbr ptr %dest, []', 'next:', 'ret void'],
          ' "inverse"="JCC"')
    entry('void', 'CALL', ['ptr "sym" %s'], '11101000 | rel:s32=s-end', 'callq {s}', ['ret void'],
          ' "reloc"="plt32" "role"="call" "nomatch"')
    entry('void', 'TAILJMP', ['ptr "sym" %s'], '11101001 | rel:s32=s-end', 'jmp {s}', ['ret void'],
          ' "reloc"="plt32" "role"="tailcall" "nomatch" "nodecode"')
    entry('void', 'CALLr', ['ptr "reg"="r64" %n'], 'rex(0, 0, 0, n) | 11111111 | rr(2, n)', 'callq *{n}', ['ret void'],
          ' "role"="callr" "nomatch"')
    entry('void', 'RET', [], '11000011', 'retq', ['ret void'], ' "role"="ret"')
    entry('void', 'UD2', [], '00001111 | 00001011', 'ud2', ['unreachable'], ' "role"="trap"')
    # A spin loop's hint (core::hint::spin_loop).
    entry('void', 'PAUSE', [], '11110011 | 10010000', 'pause', ['call void @llvm.x86.sse2.pause()', 'ret void'])
    out.append('declare void @llvm.x86.sse2.pause()\n')
    # setcc writes a byte: 0 or 1.
    entry('i1', 'SETCC', ['i1 "cond" %c'],
          '{ if(d<4) 00001111 | 1001 cc:4=c | 11 000 rm:3=d&7 ; '
          '0100 0 0 0 B:1=d>>3 | 00001111 | 1001 cc:4=c | 11 000 rm:3=d&7 }',
          'set{c} {d:r8}', ['ret i1 %c'], ' "dclass"="r32"')
    # select: the second value, then the first moved over it if c.
    for w in (64, 32):
        ty, cls, rw, sfx, _ = W[w]
        entry(ty, f'CMOV{w}', ['i1 "cond" %c', f'{ty} "reg"="{cls}" %n', f'{ty} "reg"="{cls}" "tied" %m'],
              f'rex({rw}, d, 0, n) | 00001111 | 0100 cc:4=c | rr(d, n)', f'cmov{{c}}{sfx} {{n}}, {{d}}',
              [f'%r = select i1 %c, {ty} %n, {ty} %m', f'ret {ty} %r'], '' if w == 64 else ' "zext"="32"')
    entry('ptr', 'CMOVP', ['i1 "cond" %c', 'ptr "reg"="r64" %n', 'ptr "reg"="r64" "tied" %m'],
          'rex(1, d, 0, n) | 00001111 | 0100 cc:4=c | rr(d, n)', 'cmov{c}q {n}, {d}',
          ['%r = select i1 %c, ptr %n, ptr %m', 'ret ptr %r'], ' "nodecode"')


def division():
    """Division: rdx:rax (or edx:eax) by a register, the quotient in rax and
    the remainder in rdx: fixed registers. Recipes widen the dividend
    first (cqto, or a zero rdx)."""
    for w in (64, 32):
        ty, cls, rw, sfx, _ = W[w]
        ax, dx = ('rax', 'rdx')
        sign = 'cqto' if w == 64 else 'cltd'
        entry(ty, f'SIGN{w}', [f'{ty} "reg"="{ax}" %a'], ('01001000 | 10011001' if w == 64 else '10011001'), sign,
              [f'%r = ashr {ty} %a, {w - 1}', f'ret {ty} %r'], f' "dclass"="{dx}" "nomatch"')
        for mn, r, op, res, q in [('idiv', 7, 'sdiv', ax, 'q'), ('idiv', 7, 'srem', dx, 'r'),
                                  ('div', 6, 'udiv', ax, 'q'), ('div', 6, 'urem', dx, 'r')]:
            other = dx if res == ax else ax
            entry(ty, f'{mn.upper()}{w}{q}', [f'{ty} "reg"="{ax}" %lo', f'{ty} "reg"="{dx}" %hi', f'{ty} "reg"="{cls}" %b'],
                  f'rex({rw}, 0, 0, b) | 11110111 | rr({r}, b)', f'{mn}{sfx} {{b}}',
                  [f'%r = {op} {ty} %lo, %b', f'ret {ty} %r'],
                  f' "dclass"="{res}" "clobbers"="%{other},flags" "nomatch"' + ('' if q == 'q' else ' "nodecode"'))
    # The recipes.
    for w in (64, 32):
        ty, cls, rw, sfx, _ = W[w]
        for op, signed, mn, q in [('sdiv', True, 'IDIV', 'q'), ('srem', True, 'IDIV', 'r'),
                                  ('udiv', False, 'DIV', 'q'), ('urem', False, 'DIV', 'r')]:
            out.append(f'define {ty} @{op}.{w}({ty} "reg"="{cls}" %a, {ty} "reg"="{cls}" %b) "recipe"="{op}.{w}.expand" {{\n'
                       f'  %r = {op} {ty} %a, %b\n  ret {ty} %r\n}}\n')
            hi = f'%h = call {ty} @SIGN{w}({ty} %a)' if signed else f'%h = call {ty} @MOV{w}zero()'
            out.append(f'define {ty} @{op}.{w}.expand({ty} %a, {ty} %b) {{\n  {hi}\n'
                       f'  %r = call {ty} @{mn}{w}{q}({ty} %a, {ty} %h, {ty} %b)\n  ret {ty} %r\n}}\n')
        # rdx = 0, for unsigned division (a move, leaving the flags).
        entry(ty, f'MOV{w}zero', [], 'rex(0, 0, 0, d) | 10111 o:3=d&7 | 00000000 | 00000000 | 00000000 | 00000000',
              'movl $0, {d:r32}', [f'ret {ty} 0'], ' "nomatch" "nodecode"')


def indexed_memory():
    """Loads and address arithmetic at a base plus an index scaled by 1,
    2, 4 or 8: what an array's element is (`(%rdi,%rdx,4)`). The index is
    never rsp (4 is no index), which allocation never gives. (Not stores:
    with their value, three registers, more than spill code's scratch
    registers if all are spilled; a store's address is a lea.)"""
    rex8 = lambda r, x, b: (f'{{ if({r}<4) rex(0, {r}, {x}, {b}) ; 0100 0 R:1=({r})>>3 X:1=({x})>>3 B:1=({b})>>3 }}')
    loads = [('i64', 'movq', '10001011', 1, '{d}', 'r64', '', None), ('i32', 'movl', '10001011', 0, '{d}', 'r32', '32', None),
             ('i8', 'movzbl', '00001111 | 10110110', 0, '{d:r32}', 'r32', '8', None),
             ('i16', 'movzwl', '00001111 | 10110111', 0, '{d:r32}', 'r32', '16', None),
             ('ptr', 'movq', '10001011', 1, '{d}', 'r64', '', None)]
    sloads = [('i32', 'i8', 'movsbl', '00001111 | 10111110', 0), ('i32', 'i16', 'movswl', '00001111 | 10111111', 0),
              ('i64', 'i8', 'movsbq', '00001111 | 10111110', 1), ('i64', 'i16', 'movswq', '00001111 | 10111111', 1),
              ('i64', 'i32', 'movslq', '01100011', 1)]
    fl = [('double', '11110010 | ', 'movsd', 'd'), ('float', '11110011 | ', 'movss', 's')]
    for k, sc in enumerate([1, 2, 4, 8]):
        # The address: n plus the index shifted.
        off = ['%a = getelementptr i8, ptr %n, i64 %m.index'] if k == 0 else \
              [f'%o = shl i64 %m.index, {k}', '%a = getelementptr i8, ptr %n, i64 %o']
        prm = ['ptr "reg"="r64" %n', 'i64 "reg"="r64" %m.index']
        mem = f'({{n}},{{m.index}},{sc})'
        for ty, mn, op, w, d, cls, zext, _ in loads:
            entry(ty, f'LD{ty.upper()}X{sc}', prm, f'rex({w}, d, m.index, n) | {op} | rm(d, n, m.index, {sc}, 0, 0, 1)',
                  f'{mn} {mem}, {d}', off + [f'%r = load {ty}, ptr %a', f'ret {ty} %r'],
                  f' "dclass"="{cls}"' + (f' "zext"="{zext}"' if zext else ''))
        for ty, src, mn, op, w in sloads:
            entry(ty, f'LD{src.upper()}S{ty[1:]}X{sc}', prm, f'rex({w}, d, m.index, n) | {op} | rm(d, n, m.index, {sc}, 0, 0, 1)',
                  f'{mn} {mem}, {{d}}', off + [f'%v = load {src}, ptr %a', f'%r = sext {src} %v to {ty}', f'ret {ty} %r'])
        for ty, pre, mn, cls in fl:
            entry(ty, f'LD{cls.upper()}X{sc}', prm, f'{pre}rex(0, d, m.index, n) | 00001111 | 00010000 | rm(d, n, m.index, {sc}, 0, 0, 1)',
                  f'{mn} {mem}, {{d}}', off + [f'%r = load {ty}, ptr %a', f'ret {ty} %r'], f' "dclass"="{cls}"')
        # The address itself (lea), but for scale 1, which LEA64rr is.
        if k > 0:
            entry('ptr', f'LEA64X{sc}', prm, f'rex(1, d, m.index, n) | 10001101 | rm(d, n, m.index, {sc}, 0, 0, 1)', f'leaq {mem}, {{d}}',
                  off + ['ret ptr %a'])


def rip_memory():
    """Loads and stores of a global's own address, RIP-relative
    (`movl counter(%rip), %eax`): one instruction, not its address made
    first. (Not of what the GOT reaches: selection takes these only for
    globals of this module.)"""
    rex8 = lambda r: f'{{ if({r}<4) rex(0, {r}, 0, 0) ; 0100 0 R:1=({r})>>3 0 0 }}'
    loads = [('i64', 'movq', '10001011', 1, '{d}', 'r64', ''), ('i32', 'movl', '10001011', 0, '{d}', 'r32', '32'),
             ('i8', 'movzbl', '00001111 | 10110110', 0, '{d:r32}', 'r32', '8'),
             ('i16', 'movzwl', '00001111 | 10110111', 0, '{d:r32}', 'r32', '16'),
             ('ptr', 'movq', '10001011', 1, '{d}', 'r64', '')]
    for ty, mn, op, w, d, cls, zext in loads:
        entry(ty, f'LD{ty.upper()}rip', ['ptr "sym" %s'], f'rex({w}, d, 0, 0) | {op} | rm(d, 16, 4, 1, 0, s, 1)',
              f'{mn} {{s}}(%rip), {d}', [f'%r = load {ty}, ptr %s', f'ret {ty} %r'],
              f' "reloc"="pc32x" "dclass"="{cls}"' + (f' "zext"="{zext}"' if zext else ''))
    for ty, src, mn, op, w in [('i32', 'i8', 'movsbl', '00001111 | 10111110', 0), ('i32', 'i16', 'movswl', '00001111 | 10111111', 0),
                               ('i64', 'i8', 'movsbq', '00001111 | 10111110', 1), ('i64', 'i16', 'movswq', '00001111 | 10111111', 1),
                               ('i64', 'i32', 'movslq', '01100011', 1)]:
        entry(ty, f'LD{src.upper()}S{ty[1:]}rip', ['ptr "sym" %s'], f'rex({w}, d, 0, 0) | {op} | rm(d, 16, 4, 1, 0, s, 1)',
              f'{mn} {{s}}(%rip), {{d}}', [f'%v = load {src}, ptr %s', f'%r = sext {src} %v to {ty}', f'ret {ty} %r'],
              ' "reloc"="pc32x"')
    for ty, mn, op, w, t, cls in [('i64', 'movq', '10001001', 1, '{t}', 'r64'), ('i32', 'movl', '10001001', 0, '{t}', 'r32'),
                                  ('i16', 'movw', '10001001', 0, '{t:r16}', 'r32'), ('i8', 'movb', '10001000', 0, '{t:r8}', 'r32'),
                                  ('ptr', 'movq', '10001001', 1, '{t}', 'r64')]:
        pre = '01100110 | ' if ty == 'i16' else ''
        rx = rex8('t') if ty == 'i8' else f'rex({w}, t, 0, 0)'
        entry('void', f'ST{ty.upper()}rip', [f'{ty} "reg"="{cls}" %t', 'ptr "sym" %s'], f'{pre}{rx} | {op} | rm(t, 16, 4, 1, 0, s, 1)',
              f'{mn} {t}, {{s}}(%rip)', [f'store {ty} %t, ptr %s', 'ret void'], ' "reloc"="pc32x"')
    for ty, pre, mn, cls in [('double', '11110010 | ', 'movsd', 'd'), ('float', '11110011 | ', 'movss', 's')]:
        entry(ty, f'LD{cls.upper()}rip', ['ptr "sym" %s'], f'{pre}rex(0, d, 0, 0) | 00001111 | 00010000 | rm(d, 16, 4, 1, 0, s, 1)',
              f'{mn} {{s}}(%rip), {{d}}', [f'%r = load {ty}, ptr %s', f'ret {ty} %r'], f' "reloc"="pc32x" "dclass"="{cls}"')
        entry('void', f'ST{cls.upper()}rip', [f'{ty} "reg"="{cls}" %t', 'ptr "sym" %s'],
              f'{pre}rex(0, t, 0, 0) | 00001111 | 00010001 | rm(t, 16, 4, 1, 0, s, 1)', f'{mn} {{t}}, {{s}}(%rip)',
              [f'store {ty} %t, ptr %s', 'ret void'], ' "reloc"="pc32x"')


def load_op():
    """Arithmetic and comparisons of a register with memory at a base plus
    a displacement (`addl 8(%rdi), %eax`, `cmpl $0, 16(%rax)`): the load
    folded in, one instruction and one register fewer."""
    for w in (64, 32):
        ty, cls, rw, sfx, _ = W[w]
        zext = '' if w == 64 else ' "zext"="32"'
        prm = ['ptr "reg"="r64" %n', 'i64 "imm" %off']
        ld = ['%a = getelementptr i8, ptr %n, i64 %off', f'%v = load {ty}, ptr %a']
        for mn, k, ir in [('add', 0, 'add'), ('or', 1, 'or'), ('and', 4, 'and'), ('sub', 5, 'sub'), ('xor', 6, 'xor')]:
            entry(ty, f'{mn.upper()}{w}rm', [f'{ty} "reg"="{cls}" "tied" %x'] + prm,
                  f'rex({rw}, d, 4, n) | {bits(k * 8 + 3)} | rm(d, n, 4, 1, off, 0, 1)', f'{mn}{sfx} {{off}}({{n}}), {{d}}',
                  ld + [f'%r = {ir} {ty} %x, %v', f'ret {ty} %r'], FLAGS + zext)
        entry(ty, f'IMUL{w}rm', [f'{ty} "reg"="{cls}" "tied" %x'] + prm,
              f'rex({rw}, d, 4, n) | 00001111 | 10101111 | rm(d, n, 4, 1, off, 0, 1)', f'imul{sfx} {{off}}({{n}}), {{d}}',
              ld + [f'%r = mul {ty} %x, %v', f'ret {ty} %r'], FLAGS + zext)
        # cmp: the flags of x - the memory (3B /r), and of the memory less
        # an immediate (83 or 81 /7).
        ext = (lambda a, b: [f'%ea = sext i32 {a} to i64', f'%eb = sext i32 {b} to i64']) if w == 32 else (lambda a, b: [])
        A = lambda a: '%ea' if w == 32 else a
        B = lambda b: '%eb' if w == 32 else b
        entry('%flags', f'CMP{w}rm', [f'{ty} "reg"="{cls}" %x'] + prm,
              f'rex({rw}, x, 4, n) | 00111011 | rm(x, n, 4, 1, off, 0, 1)', f'cmp{sfx} {{off}}({{n}}), {{x}}',
              ld + ext('%x', '%v') + [f'%f0 = insertvalue %flags poison, i64 {A("%x")}, 0',
                                       f'%f = insertvalue %flags %f0, i64 {B("%v")}, 1', 'ret %flags %f'], ' "sets"="zcso"')
        entry('%flags', f'CMP{w}mi', prm + [f'{ty} "imm" %imm'],
              f'{{ rex({rw}, 0, 4, n) | 10000011 | rm(7, n, 4, 1, off, 0, 1) | ib:s8=imm ; '
              f'rex({rw}, 0, 4, n) | 10000001 | rm(7, n, 4, 1, off, 0, 1) | iv:s32=imm }}',
              f'cmp{sfx} ${{imm}}, {{off}}({{n}})',
              ld + ext('%v', '%imm') + [f'%f0 = insertvalue %flags poison, i64 {A("%v")}, 0',
                                         f'%f = insertvalue %flags %f0, i64 {B("%imm")}, 1', 'ret %flags %f'], ' "sets"="zcso"')
    for ty, (pre, sfx, cls, _) in FP.items():
        T = sfx.upper()
        prm = ['ptr "reg"="r64" %n', 'i64 "imm" %off']
        ld = ['%a = getelementptr i8, ptr %n, i64 %off', f'%v = load {ty}, ptr %a']
        for mn, op, ir in [('add', 0x58, 'fadd'), ('sub', 0x5C, 'fsub'), ('mul', 0x59, 'fmul'), ('div', 0x5E, 'fdiv')]:
            entry(ty, f'{mn.upper()}{T}rm', [f'{ty} "reg"="{cls}" "tied" %x'] + prm,
                  f'{pre}rex(0, d, 4, n) | 00001111 | {bits(op)} | rm(d, n, 4, 1, off, 0, 1)', f'{mn}{sfx} {{off}}({{n}}), {{d}}',
                  ld + [f'%r = {ir} {ty} %x, %v', f'ret {ty} %r'], f' "dclass"="{cls}"')


def recipes():
    """Minimum and maximum: compare, then move one over the other if it
    wins."""
    decl = []
    for w in (64, 32):
        ty, cls, rw, sfx, _ = W[w]
        for op, cc in [('smin', 'l'), ('smax', 'g'), ('umin', 'b'), ('umax', 'a')]:
            decl.append(f'declare {ty} @llvm.{op}.i{w}({ty}, {ty})\n')
            out.append(f'define {ty} @{op}.{w}({ty} "reg"="{cls}" %a, {ty} "reg"="{cls}" %b) "recipe"="{op}.{w}.expand" {{\n'
                       f'  %r = call {ty} @llvm.{op}.i{w}({ty} %a, {ty} %b)\n  ret {ty} %r\n}}\n')
            out.append(f'define {ty} @{op}.{w}.expand({ty} %a, {ty} %b) {{\n'
                       f'  %f = call %flags @CMP{w}rr({ty} %a, {ty} %b)\n'
                       f'  %c = call i1 @cc.{cc}(%flags %f)\n'
                       f'  %r = call {ty} @CMOV{w}(i1 %c, {ty} %a, {ty} %b)\n  ret {ty} %r\n}}\n')
    out.extend(decl)
    # A boolean in a register: its low byte tested.
    entry('%flags', 'TESTB1', ['i1 "reg"="r32" %n'],
          '{ acc:0=n 10101000 | 00000001 ; if(n<4) 11110110 | rr(0, n) | 00000001 ; '
          '0100 0 0 0 B:1=n>>3 | 11110110 | rr(0, n) | 00000001 }',
          'testb $1, {n:r8}', ['%x = zext i1 %n to i64', '%f0 = insertvalue %flags poison, i64 %x, 0',
                               '%f = insertvalue %flags %f0, i64 0, 1', 'ret %flags %f'], ' "sets"="zcso" "nomatch"')
    for ty, cls, w in [('i64', 'r64', 64), ('i32', 'r32', 32), ('ptr', 'r64', 'p')]:
        cm = 'CMOVP' if w == 'p' else f'CMOV{w}'
        out.append(f'define {ty} @select.{w}(i1 "reg"="r32" %c, {ty} "reg"="{cls}" %a, {ty} "reg"="{cls}" %b) '
                   f'"recipe"="select.{w}.expand" {{\n  %r = select i1 %c, {ty} %a, {ty} %b\n  ret {ty} %r\n}}\n')
        out.append(f'define {ty} @select.{w}.expand(i1 %c, {ty} %a, {ty} %b) {{\n'
                   f'  %f = call %flags @TESTB1(i1 %c)\n  %ne = call i1 @cc.ne(%flags %f)\n'
                   f'  %r = call {ty} @{cm}(i1 %ne, {ty} %a, {ty} %b)\n  ret {ty} %r\n}}\n')
    # A boolean sign-extended: 0 or -1, its lowest bit negated (what is
    # above it is undefined: a truncation leaves it).
    for ty in ('i8', 'i16', 'i32', 'i64'):
        w = 1 if ty == 'i64' else 0
        entry(ty, f'ZEXT1to{ty}', ['i1 "reg"="r32" "tied" %n'], 'rex(0, 0, 0, d) | 10000011 | rr(4, d) | 00000001',
              'andl $1, {d:r32}', [f'%r = zext i1 %n to {ty}', f'ret {ty} %r'], ' "clobbers"="flags" "nomatch"' +
              (' "dclass"="r32"' if ty in ('i8', 'i16') else ' "dclass"="r64"' if w else ''))
        entry(ty, f'NEGx{ty}', [f'{ty} "reg"="{"r64" if w else "r32"}" "tied" %n'],
              f'rex({w}, 0, 0, d) | 11110111 | rr(3, d)', f'neg{"q" if w else "l"} {{d}}',
              [f'%r = sub {ty} 0, %n', f'ret {ty} %r'], ' "clobbers"="flags" "nomatch"' +
              (' "dclass"="r32"' if ty in ('i8', 'i16') else ''))
        cls = 'r64' if w else 'r32'
        out.append(f'define {ty} @sext.1.{ty}(i1 "reg"="r32" %n) "recipe"="sext.1.{ty}.expand" {{\n'
                   f'  %r = sext i1 %n to {ty}\n  ret {ty} %r\n}}\n')
        out.append(f'define {ty} @sext.1.{ty}.expand(i1 %n) {{\n  %z = call {ty} @ZEXT1to{ty}(i1 %n)\n'
                   f'  %r = call {ty} @NEGx{ty}({ty} %z)\n  ret {ty} %r\n}}\n')
    # The high half of a 64-bit product: rdx, of rax by a register.
    for mn, r, ext in [('imul', 5, 'sext'), ('mul', 4, 'zext')]:
        entry('i64', f'{mn.upper()}H64', ['i64 "reg"="rax" %n', 'i64 "reg"="r64" %m'],
              f'rex(1, 0, 0, m) | 11110111 | rr({r}, m)', f'{mn}q {{m}}',
              [f'%a = {ext} i64 %n to i128', f'%b = {ext} i64 %m to i128', '%p = mul i128 %a, %b',
               '%h = lshr i128 %p, 64', '%r = trunc i128 %h to i64', 'ret i64 %r'],
              ' "dclass"="rdx" "clobbers"="%rax,flags"')
    # A 32-bit value widened with zeros: its move (which clears the top).
    entry('i64', 'MOVZX32', ['i32 "reg"="r32" %n'], 'rex(0, n, 0, d) | 10001001 | rr(n, d)', 'movl {n}, {d:r32}',
          ['%r = zext i32 %n to i64', 'ret i64 %r'], ' "zext"="32"')


# Floating point (SSE2): the scalar types, their prefix, their class.
FP = {'double': ('11110010 | ', 'sd', 'd', 'q'), 'float': ('11110011 | ', 'ss', 's', 'l')}


def floats():
    out.append("""; Floating point, SSE2's scalar instructions. The flags of a comparison
; (ucomisd) are ZF, PF and CF: unordered sets all three, less than CF,
; equal ZF. Each of the eight predicates a condition tests is one condition
; code; oeq and une are two (prepare splits them: "fcmp-split").
%fflags = type { double, double }
; The flags of the operands the other way round: ucomisd n, m compares m
; with n. Its conditions test the predicates whose operands are swapped:
; olt is ogt the other way round, ugt ult.
%fflagsr = type { double, double }
""")
    for name, pred, code, cc in [('olt', 'olt', 7, 'a'), ('ole', 'ole', 3, 'ae'), ('ugt', 'ugt', 2, 'b'),
                                 ('uge', 'uge', 6, 'be')]:
        out.append(f'define i1 @cc.fr.{name}(%fflagsr %f) "cond"="{code}" "asm"="{cc}" "reads"="zcp" {{\n'
                   f'  %a = extractvalue %fflagsr %f, 0\n  %b = extractvalue %fflagsr %f, 1\n'
                   f'  %c = fcmp {pred} double %a, %b\n  ret i1 %c\n}}\n')
    for name, pred, code, cc in [('ogt', 'ogt', 7, 'a'), ('oge', 'oge', 3, 'ae'), ('ult', 'ult', 2, 'b'),
                                 ('ule', 'ule', 6, 'be'), ('ueq', 'ueq', 4, 'e'), ('one', 'one', 5, 'ne'),
                                 ('uno', 'uno', 10, 'p'), ('ord', 'ord', 11, 'np')]:
        out.append(f'define i1 @cc.f.{name}(%fflags %f) "cond"="{code}" "asm"="{cc}" "reads"="zcp" {{\n'
                   f'  %a = extractvalue %fflags %f, 0\n  %b = extractvalue %fflags %f, 1\n'
                   f'  %c = fcmp {pred} double %a, %b\n  ret i1 %c\n}}\n')
    for ty, (pre, sfx, cls, isfx) in FP.items():
        T = sfx.upper()
        wide = ['%a = fpext float %n to double', '%b = fpext float %m to double'] if ty == 'float' else []
        a, b = ('%a', '%b') if ty == 'float' else ('%n', '%m')
        p66 = '01100110 | ' if ty == 'double' else ''
        # ucomisd m, n: the flags of n against m; and the other way round.
        entry('%fflags', f'UCOMI{T}', [f'{ty} "reg"="{cls}" %n', f'{ty} "reg"="{cls}" %m'],
              f'{p66}rex(0, n, 0, m) | 00001111 | 00101110 | rr(n, m)', f'ucomi{sfx} {{m}}, {{n}}',
              wide + [f'%f0 = insertvalue %fflags poison, double {a}, 0', f'%f = insertvalue %fflags %f0, double {b}, 1',
                      'ret %fflags %f'], ' "sets"="zcp"')
        entry('%fflagsr', f'UCOMI{T}rev', [f'{ty} "reg"="{cls}" %n', f'{ty} "reg"="{cls}" %m'],
              f'{p66}rex(0, m, 0, n) | 00001111 | 00101110 | rr(m, n)', f'ucomi{sfx} {{n}}, {{m}}',
              wide + [f'%f0 = insertvalue %fflagsr poison, double {a}, 0', f'%f = insertvalue %fflagsr %f0, double {b}, 1',
                      'ret %fflagsr %f'], ' "sets"="zcp"')
        for mn, op, ir in [('add', 0x58, 'fadd'), ('sub', 0x5C, 'fsub'), ('mul', 0x59, 'fmul'), ('div', 0x5E, 'fdiv')]:
            entry(ty, f'{mn.upper()}{T}', [f'{ty} "reg"="{cls}" "tied" %n', f'{ty} "reg"="{cls}" %m'],
                  f'{pre}rex(0, d, 0, m) | 00001111 | {bits(op)} | rr(d, m)', f'{mn}{sfx} {{m}}, {{d}}',
                  [f'%r = {ir} {ty} %n, %m', f'ret {ty} %r'])
        intr = 'f64' if ty == 'double' else 'f32'
        entry(ty, f'SQRT{T}', [f'{ty} "reg"="{cls}" %n'], f'{pre}rex(0, d, 0, n) | 00001111 | 01010001 | rr(d, n)',
              f'sqrt{sfx} {{n}}, {{d}}', [f'%r = call {ty} @llvm.sqrt.{intr}({ty} %n)', f'ret {ty} %r'])
        out.append(f'declare {ty} @llvm.sqrt.{intr}({ty})\n')
        # Loads and stores.
        entry(ty, f'LD{T}', ['ptr "reg"="r64" %n', 'i64 "imm" %off'],
              f'{pre}rex(0, d, 4, n) | 00001111 | 00010000 | rm(d, n, 4, 1, off, 0, 1)', f'mov{sfx} {{off}}({{n}}), {{d}}',
              ['%a = getelementptr i8, ptr %n, i64 %off', f'%r = load {ty}, ptr %a', f'ret {ty} %r'],
              f' "role"="load.{cls}"')
        entry('void', f'ST{T}', [f'{ty} "reg"="{cls}" %t', 'ptr "reg"="r64" %n', 'i64 "imm" %off'],
              f'{pre}rex(0, t, 4, n) | 00001111 | 00010001 | rm(t, n, 4, 1, off, 0, 1)', f'mov{sfx} {{t}}, {{off}}({{n}})',
              ['%a = getelementptr i8, ptr %n, i64 %off', f'store {ty} %t, ptr %a', 'ret void'], f' "role"="store.{cls}"')
        # Between the files: the bits.
        ity, icls, w, mv = ('i64', 'r64', 1, 'movq') if ty == 'double' else ('i32', 'r32', 0, 'movd')
        entry(ty, f'MOV{T}from{ity}', [f'{ity} "reg"="{icls}" %n'],
              f'01100110 | rex({w}, d, 0, n) | 00001111 | 01101110 | rr(d, n)', f'{mv} {{n}}, {{d}}',
              [f'%r = bitcast {ity} %n to {ty}', f'ret {ty} %r'], f' "role"="move.{cls}.{icls}"')
        entry(ity, f'MOV{ity}from{T}', [f'{ty} "reg"="{cls}" %n'],
              f'01100110 | rex({w}, n, 0, d) | 00001111 | 01111110 | rr(n, d)', f'{mv} {{n}}, {{d}}',
              [f'%r = bitcast {ty} %n to {ity}', f'ret {ity} %r'], f' "role"="move.{icls}.{cls}"')
        # Conversions from integers (cvtsi2sd: signed; an unsigned 32-bit
        # value is its 64-bit register, zero-extended).
        for src, w, ext in [('i32', 0, 'sitofp'), ('i64', 1, 'sitofp'), ('i32', 1, 'uitofp')]:
            scls = 'r64' if src == 'i64' else 'r32'
            n = '{n:r64}' if (src == 'i32' and w == 1) else '{n}'
            entry(ty, f'CVT{ext.upper()}{src.upper()}{T}', [f'{src} "reg"="{scls}" %n'],
                  f'{pre}rex({w}, d, 0, n) | 00001111 | 00101010 | rr(d, n)', f'cvtsi2{sfx} {n}, {{d}}',
                  [f'%r = {ext} {src} %n to {ty}', f'ret {ty} %r'])
        # To integers, truncating (an unsigned 32-bit result is the low
        # half of the 64-bit conversion).
        for dst, w, ext in [('i32', 0, 'fptosi'), ('i64', 1, 'fptosi'), ('i32', 1, 'fptoui')]:
            dcls = 'r64' if dst == 'i64' else 'r32'
            d = '{d:r64}' if (dst == 'i32' and w == 1) else '{d}'
            entry(dst, f'CVTT{ext.upper()}{T}{dst.upper()}', [f'{ty} "reg"="{cls}" %n'],
                  f'{pre}rex({w}, d, 0, n) | 00001111 | 00101100 | rr(d, n)', f'cvtt{sfx}2si {{n}}, {d}',
                  [f'%r = {ext} {ty} %n to {dst}', f'ret {dst} %r'], f' "dclass"="{dcls}"')
        # select: through the integer registers.
        out.append(f'define {ty} @select.{cls}(i1 "reg"="r32" %c, {ty} "reg"="{cls}" %a, {ty} "reg"="{cls}" %b) '
                   f'"recipe"="select.{cls}.expand" {{\n  %r = select i1 %c, {ty} %a, {ty} %b\n  ret {ty} %r\n}}\n')
        cm = 'CMOV64' if ty == 'double' else 'CMOV32'
        out.append(f'define {ty} @select.{cls}.expand(i1 %c, {ty} %a, {ty} %b) {{\n'
                   f'  %ai = call {ity} @MOV{ity}from{T}({ty} %a)\n  %bi = call {ity} @MOV{ity}from{T}({ty} %b)\n'
                   f'  %f = call %flags @TESTB1(i1 %c)\n  %ne = call i1 @cc.ne(%flags %f)\n'
                   f'  %ri = call {ity} @{cm}(i1 %ne, {ity} %ai, {ity} %bi)\n'
                   f'  %r = call {ty} @MOV{T}from{ity}({ity} %ri)\n  ret {ty} %r\n}}\n')
        # Negation and absolute value: the sign bit flipped or cleared,
        # in the integer registers.
        bitsw = 64 if ty == 'double' else 32
        sign = -(1 << (bitsw - 1))
        xor, andr, movk = (('XOR64rr', 'AND64rr', 'MOVABS64ri') if ty == 'double' else ('XOR32rr', 'AND32rr', 'MOV32ri'))
        for op, body, k in [('fneg', f'%r = fneg {ty} %a', sign), ('fabs', f'%r = call {ty} @llvm.fabs.{intr}({ty} %a)',
                                                                     (1 << (bitsw - 1)) - 1)]:
            out.append(f'define {ty} @{op}.{cls}({ty} "reg"="{cls}" %a) "recipe"="{op}.{cls}.expand" {{\n'
                       f'  {body}\n  ret {ty} %r\n}}\n')
            inst = xor if op == 'fneg' else andr
            out.append(f'define {ty} @{op}.{cls}.expand({ty} %a) {{\n'
                       f'  %ai = call {ity} @MOV{ity}from{T}({ty} %a)\n  %k = call {ity} @{movk}({ity} {k})\n'
                       f'  %x = call {ity} @{inst}({ity} %ai, {ity} %k)\n'
                       f'  %r = call {ty} @MOV{T}from{ity}({ity} %x)\n  ret {ty} %r\n}}\n')
        out.append(f'declare {ty} @llvm.fabs.{intr}({ty})\n')
        # fmuladd: a multiply, then an add (fusing is allowed, not needed).
        out.append(f'declare {ty} @llvm.fmuladd.{intr}({ty}, {ty}, {ty})\n')
        out.append(f'define {ty} @fmuladd.{cls}({ty} "reg"="{cls}" %a, {ty} "reg"="{cls}" %b, {ty} "reg"="{cls}" %c) '
                   f'"recipe"="fmuladd.{cls}.expand" {{\n  %r = call {ty} @llvm.fmuladd.{intr}({ty} %a, {ty} %b, {ty} %c)\n'
                   f'  ret {ty} %r\n}}\n')
        out.append(f'define {ty} @fmuladd.{cls}.expand({ty} %a, {ty} %b, {ty} %c) {{\n'
                   f'  %p = call {ty} @MUL{T}({ty} %a, {ty} %b)\n  %r = call {ty} @ADD{T}({ty} %p, {ty} %c)\n'
                   f'  ret {ty} %r\n}}\n')
        # An unsigned 64-bit value: converted signed if its top bit is
        # clear; else halved (keeping the low bit, for rounding),
        # converted, and doubled.
        out.append(f'define {ty} @uitofp.64.{cls}(i64 "reg"="r64" %x) "recipe"="uitofp.64.{cls}.expand" {{\n'
                   f'  %r = uitofp i64 %x to {ty}\n  ret {ty} %r\n}}\n')
        out.append(f'define {ty} @uitofp.64.{cls}.expand(i64 %x) {{\n'
                   f'  %h = call i64 @SHR64ri(i64 %x, i64 1)\n  %l = call i64 @AND64ri(i64 %x, i64 1)\n'
                   f'  %o = call i64 @OR64rr(i64 %h, i64 %l)\n'
                   f'  %f1 = call %flags @TEST64rr(i64 %x, i64 %x)\n  %n1 = call i1 @cc.l(%flags %f1)\n'
                   f'  %t = call i64 @CMOV64(i1 %n1, i64 %o, i64 %x)\n'
                   f'  %c = call {ty} @CVTSITOFPI64{T}(i64 %t)\n  %c2 = call {ty} @ADD{T}({ty} %c, {ty} %c)\n'
                   f'  %ci = call {ity} @MOV{ity}from{T}({ty} %c)\n  %c2i = call {ity} @MOV{ity}from{T}({ty} %c2)\n'
                   f'  %f2 = call %flags @TEST64rr(i64 %x, i64 %x)\n  %n2 = call i1 @cc.l(%flags %f2)\n'
                   f'  %ri = call {ity} @{"CMOV64" if ty == "double" else "CMOV32"}(i1 %n2, {ity} %c2i, {ity} %ci)\n'
                   f'  %r = call {ty} @MOV{T}from{ity}({ity} %ri)\n  ret {ty} %r\n}}\n')
        # To an unsigned 64-bit value: converted signed, and again less
        # 2^63; a value of 2^63 or more gives the first 1<<63 (the
        # indefinite result), and then the second's bits go in too.
        big = '0x43E0000000000000' if ty == 'double' else '0x5F000000'
        out.append(f'define i64 @fptoui.64.{cls}({ty} "reg"="{cls}" %x) "recipe"="fptoui.64.{cls}.expand" {{\n'
                   f'  %r = fptoui {ty} %x to i64\n  ret i64 %r\n}}\n')
        out.append(f'define i64 @fptoui.64.{cls}.expand({ty} %x) {{\n'
                   f'  %a = call i64 @CVTTFPTOSI{T}I64({ty} %x)\n'
                   f'  %ki = call {ity} @{"MOVABS64ri" if ty == "double" else "MOV32ri"}({ity} {int(big, 16)})\n'
                   f'  %k = call {ty} @MOV{T}from{ity}({ity} %ki)\n  %y = call {ty} @SUB{T}({ty} %x, {ty} %k)\n'
                   f'  %b = call i64 @CVTTFPTOSI{T}I64({ty} %y)\n  %s = call i64 @SAR64ri(i64 %a, i64 63)\n'
                   f'  %m = call i64 @AND64rr(i64 %b, i64 %s)\n  %r = call i64 @OR64rr(i64 %a, i64 %m)\n'
                   f'  ret i64 %r\n}}\n')
    # A half's bits in an xmm register's low 16 (as halves are passed):
    # in with movd, out with pextrw (which zero-extends).
    entry('float', 'MOVDh', ['i16 "reg"="r32" %n'], '01100110 | rex(0, d, 0, n) | 00001111 | 01101110 | rr(d, n)',
          'movd {n}, {d}', ['%r = bitcast i32 0 to float', 'ret float %r'], ' "role"="move.h.w" "nomatch" "nodecode"')
    entry('i16', 'PEXTRWh', ['float "reg"="s" %n'], '01100110 | rex(0, d, 0, n) | 00001111 | 11000101 | rr(d, n) | 00000000',
          'pextrw $0, {n}, {d}', ['ret i16 0'], ' "dclass"="r32" "zext"="16" "role"="move.w.h" "nomatch" "nodecode"')
    # Between the two widths.
    entry('double', 'CVTSS2SD', ['float "reg"="s" %n'], '11110011 | rex(0, d, 0, n) | 00001111 | 01011010 | rr(d, n)',
          'cvtss2sd {n}, {d}', ['%r = fpext float %n to double', 'ret double %r'])
    entry('float', 'CVTSD2SS', ['double "reg"="d" %n'], '11110010 | rex(0, d, 0, n) | 00001111 | 01011010 | rr(d, n)',
          'cvtsd2ss {n}, {d}', ['%r = fptrunc double %n to float', 'ret float %r'])


def atomics():
    """Atomics. x86 orders memory strongly (TSO): loads of any ordering
    and release stores are plain moves; a sequentially consistent store
    is an xchg (`seq-cst-stores`); read-modify-writes are xchg (locked
    without a prefix), lock xadd, and lock cmpxchg (with the value
    expected and the old one in rax), whatever their ordering."""
    rex8 = lambda r, b: f'{{ if({r}<4) rex(0, {r}, 4, {b}) ; 0100 0 R:1=({r})>>3 X:1=0 B:1=({b})>>3 }}'
    # (Type, suffix, REX.W, class, view, 16-bit prefix.)
    widths = [('i8', 'b', 0, 'r32', ':r8', ''), ('i16', 'w', 0, 'r32', ':r16', '01100110 | '),
              ('i32', 'l', 0, 'r32', '', ''), ('i64', 'q', 1, 'r64', '', '')]
    orders = ['monotonic', 'acquire', 'release', 'seq_cst']
    for ty, sfx, w, cls, view, p16 in widths:
        byte = ty == 'i8'
        bits_ = ty[1:]
        rx = (lambda r: rex8(r, 'n')) if byte else (lambda r: f'rex({w}, {r}, 4, n)')
        # Acquire loads and release stores.
        ld = {'i8': ('movzbl', '00001111 | 10110110', '{d:r32}'), 'i16': ('movzwl', '00001111 | 10110111', '{d:r32}'),
              'i32': ('movl', '10001011', '{d}'), 'i64': ('movq', '10001011', '{d}')}[ty]
        lw = 1 if ty == 'i64' else 0
        entry(ty, f'LDA{ty.upper()}', ['ptr "reg"="r64" %n'], f'rex({lw}, d, 4, n) | {ld[1]} | rm(d, n, 4, 1, 0, 0, 1)',
              f'{ld[0]} 0({{n}}), {ld[2]}', [f'%r = load atomic {ty}, ptr %n acquire, align {int(bits_) // 8}', f'ret {ty} %r'],
              f' "dclass"="{cls}" "nodecode"' + ('' if ty == 'i64' else f' "zext"="{bits_}"' if byte or ty == 'i16' else ''))
        op = '10001000' if byte else '10001001'
        entry('void', f'STR{ty.upper()}', [f'{ty} "reg"="{cls}" %t', 'ptr "reg"="r64" %n'],
              f'{p16}{rx("t")} | {op} | rm(t, n, 4, 1, 0, 0, 1)', f'mov{sfx} {{t{view}}}, 0({{n}})',
              [f'store atomic {ty} %t, ptr %n release, align {int(bits_) // 8}', 'ret void'], ' "nodecode"')
        for k, order in enumerate(orders):
            # xchg m, (n): m the old value after.
            entry(ty, f'XCHG{ty.upper()}{k}', [f'{ty} "reg"="{cls}" "tied" %m', 'ptr "reg"="r64" %n'],
                  f'{p16}{rx("d")} | {"10000110" if byte else "10000111"} | rm(d, n, 4, 1, 0, 0, 1)',
                  f'xchg{sfx} {{d{view}}}, 0({{n}})', [f'%r = atomicrmw xchg ptr %n, {ty} %m {order}', f'ret {ty} %r'],
                  ' "nodecode"')
            entry(ty, f'XADD{ty.upper()}{k}', [f'{ty} "reg"="{cls}" "tied" %m', 'ptr "reg"="r64" %n'],
                  f'{p16}11110000 | {rx("d")} | 00001111 | {"11000000" if byte else "11000001"} | rm(d, n, 4, 1, 0, 0, 1)',
                  f'lock xadd{sfx} {{d{view}}}, 0({{n}})', [f'%r = atomicrmw add ptr %n, {ty} %m {order}', f'ret {ty} %r'],
                  ' "clobbers"="flags" "nodecode"')
            entry(ty, f'CMPXCHG{ty.upper()}{k}', [f'{ty} "reg"="rax" "tied" %old', f'{ty} "reg"="{cls}" %new',
                                                    'ptr "reg"="r64" %n'],
                  f'{p16}11110000 | {rx("new")} | 00001111 | {"10110000" if byte else "10110001"} | rm(new, n, 4, 1, 0, 0, 1)',
                  f'lock cmpxchg{sfx} {{new{view}}}, 0({{n}})',
                  [f'%r = call {ty} @llvm.epitome.cmpxchg.{ty}(ptr %n, {ty} %old, {ty} %new, i32 {k})', f'ret {ty} %r'],
                  ' "dclass"="rax" "clobbers"="flags" "nodecode"')
        out.append(f'declare {ty} @llvm.epitome.cmpxchg.{ty}(ptr, {ty}, {ty}, i32)\n')
    entry('ptr', 'LDAP', ['ptr "reg"="r64" %n'], 'rex(1, d, 4, n) | 10001011 | rm(d, n, 4, 1, 0, 0, 1)',
          'movq 0({n}), {d}', ['%r = load atomic ptr, ptr %n acquire, align 8', 'ret ptr %r'], ' "nodecode"')
    entry('void', 'STRP', ['ptr "reg"="r64" %t', 'ptr "reg"="r64" %n'], 'rex(1, t, 4, n) | 10001001 | rm(t, n, 4, 1, 0, 0, 1)',
          'movq {t}, 0({n})', ['store atomic ptr %t, ptr %n release, align 8', 'ret void'], ' "nodecode"')
    # Fences: only a sequentially consistent one orders anything on x86,
    # but acquire ones are mfence too (the code generator has one head for
    # the others).
    entry('void', 'MFENCE', [], '00001111 | 10101110 | 11110000', 'mfence', ['fence seq_cst', 'ret void'], ' "nodecode"')
    entry('void', 'MFENCEA', [], '00001111 | 10101110 | 11110000', 'mfence', ['fence acquire', 'ret void'], ' "nodecode"')


def x87():
    """x86_fp80 (C's long double) on the x87 unit. Its values are kept in
    memory (prepare's x87.rs): each x87 instruction is an intrinsic of its
    own, called in order, with the stack empty between operations; a
    comparison is a recipe that loads both values, compares and pops."""
    for name, op, r, asm in [('FLDT', '11011011', 5, 'fldt'), ('FSTPT', '11011011', 7, 'fstpt'),
                             ('FLDL', '11011101', 0, 'fldl'), ('FSTPL', '11011101', 3, 'fstpl'),
                             ('FLDS', '11011001', 0, 'flds'), ('FSTPS', '11011001', 3, 'fstps'),
                             ('FILDLL', '11011111', 5, 'fildll'), ('FISTPLL', '11011111', 7, 'fistpll'),
                             ('FNSTCW', '11011001', 7, 'fnstcw'), ('FLDCW', '11011001', 5, 'fldcw')]:
        entry('void', name, ['ptr "reg"="r64" %n'], f'rex(0, 0, 4, n) | {op} | rm({r}, n, 4, 1, 0, 0, 1)',
              f'{asm} 0({{n}})', [f'call void @llvm.epitome.x87.{asm}(ptr %n)', 'ret void'])
        out.append(f'declare void @llvm.epitome.x87.{asm}(ptr)\n')
    # On the top two: the one pushed first (st(1)) op the other (st(0)),
    # popped into st(0). (AT&T's fsubrp is Intel's fsubp: st(1) - st(0).)
    for name, enc, asm, op in [('FADDP', '11011110 | 11000001', 'faddp %st, %st(1)', 'fadd'),
                               ('FSUBP', '11011110 | 11101001', 'fsubrp %st, %st(1)', 'fsub'),
                               ('FMULP', '11011110 | 11001001', 'fmulp %st, %st(1)', 'fmul'),
                               ('FDIVP', '11011110 | 11111001', 'fdivrp %st, %st(1)', 'fdiv'),
                               ('FCHS', '11011001 | 11100000', 'fchs', 'fchs'),
                               ('FABS', '11011001 | 11100001', 'fabs', 'fabs'),
                               ('FPOP', '11011101 | 11011000', 'fstp %st(0)', 'pop')]:
        entry('void', name, [], enc, asm, [f'call void @llvm.epitome.x87.{op}()', 'ret void'])
        out.append(f'declare void @llvm.epitome.x87.{op}()\n')
    # Compares st(0) with st(1) as ucomisd does, and pops one.
    entry('%fflags', 'FUCOMPI', [], '11011111 | 11101001', 'fucompi %st(1), %st',
          ['ret %fflags poison'], ' "sets"="zcp" "nomatch"')
    # pred(*a, *b), for the predicates one condition tests (prepare swaps
    # the others' operands, and splits oeq and une).
    for p in ['ogt', 'oge', 'ult', 'ule', 'ueq', 'one', 'uno', 'ord']:
        out.append(f'declare i1 @llvm.epitome.x87.cmp.{p}(ptr, ptr)\n')
        out.append(f'define i1 @x87.cmp.{p}(ptr "reg"="r64" %a, ptr "reg"="r64" %b) "recipe"="x87.cmp.{p}.expand" {{\n'
                   f'  %r = call i1 @llvm.epitome.x87.cmp.{p}(ptr %a, ptr %b)\n  ret i1 %r\n}}\n')
        out.append(f'define i1 @x87.cmp.{p}.expand(ptr %a, ptr %b) {{\n'
                   f'  call void @FLDT(ptr %b)\n  call void @FLDT(ptr %a)\n  %f = call %fflags @FUCOMPI()\n'
                   f'  call void @FPOP()\n  %c = call i1 @cc.f.{p}(%fflags %f)\n  %r = call i1 @SETCC(i1 %c)\n'
                   f'  ret i1 %r\n}}\n')


def sse_vectors():
    """SSE2's vectors, for selection: 128 bits in xv, 64 in xh (an xmm
    register's low half, as LLVM widens them). What SSE2 has no instruction
    for is done lane by lane (legalize.rs)."""
    P66 = '01100110 | '
    def leg(pre, op, r, m, w=0):
        return f'{pre}rex({w}, {r}, 0, {m}) | 00001111 | {op} | rr({r}, {m})'
    def mem(pre, op, r, w=0):
        return f'{pre}rex({w}, {r}, 4, n) | 00001111 | {op} | rm({r}, n, 4, 1, off, 0, 1)'
    INT = [('<16 x i8>', 'b', 8, 16, 'xv'), ('<8 x i16>', 'w', 16, 8, 'xv'), ('<4 x i32>', 'd', 32, 4, 'xv'),
           ('<2 x i64>', 'q', 64, 2, 'xv'), ('<8 x i8>', 'b', 8, 8, 'xh'), ('<4 x i16>', 'w', 16, 4, 'xh'),
           ('<2 x i32>', 'd', 32, 2, 'xh')]
    FLT = [('<4 x float>', 'ps', 'float', 4, 'xv'), ('<2 x double>', 'pd', 'double', 2, 'xv'),
           ('<2 x float>', 'ps', 'float', 2, 'xh')]
    tag = lambda ty: ty.strip('<>').replace(' x ', 'x').replace('i', 'i').replace('float', 'f32').replace('double', 'f64')
    # Memory, copies and zero. (Spills by class: load.xv, store.xv.)
    for ty, cls in [(t, c) for t, *_, c in INT] + [(t, c) for t, *_, c in FLT]:
        T = tag(ty)
        role = cls == 'xv' and ty == '<2 x i64>' or cls == 'xh' and ty == '<2 x i32>'
        # (Vectors passed on the stack: load.q and store.q, too.)
        if cls == 'xv':
            ld, st = (mem('', '00010000', 'd'), 'movups {off}({n}), {d}'), (mem('', '00010001', 't'), 'movups {t}, {off}({n})')
        else:
            ld = (mem('11110011 | ', '01111110', 'd'), 'movq {off}({n}), {d}')
            st = (mem(P66, '11010110', 't'), 'movq {t}, {off}({n})')
        entry(ty, f'LDV.{T}', ['ptr "reg"="r64" %n', 'i64 "imm" %off'], ld[0], ld[1],
              ['%a = getelementptr i8, ptr %n, i64 %off', f'%r = load {ty}, ptr %a', f'ret {ty} %r'],
              f' "dclass"="{cls}"' + (f' "role"="load.{cls}' + (',load.q' if cls == 'xv' else '') + '"' if role else ''))
        entry('void', f'STV.{T}', [f'{ty} "reg"="{cls}" %t', 'ptr "reg"="r64" %n', 'i64 "imm" %off'], st[0], st[1],
              ['%a = getelementptr i8, ptr %n, i64 %off', f'store {ty} %t, ptr %a', 'ret void'],
              f' "role"="store.{cls}' + (',store.q' if cls == 'xv' else '') + '"' if role else '')
    for cls, ty in [('xv', '<2 x i64>'), ('xh', '<2 x i32>')]:
        entry(ty, f'MOVAPS{cls}', [f'{ty} "reg"="{cls}" %n'], 'rex(0, d, 0, n) | 00001111 | 00101000 | rr(d, n)',
              'movaps {n}, {d}', [f'ret {ty} %n'], f' "copy"="{cls}" "dclass"="{cls}"')
    entry('<2 x i64>', 'PXORzero', [], leg(P66, '11101111', 'd', 'd'), 'pxor {d}, {d}', ['ret <2 x i64> zeroinitializer'],
          ' "dclass"="xv" "role"="movi.zero" "nomatch"')
    # Integer lanes: two-address, the first source the destination.
    def two(ty, cls, name, pre, op, mn, body, rty=None, dcls=None):
        rty = rty or ty
        entry(rty, name, [f'{ty} "reg"="{cls}" "tied" %n', f'{ty} "reg"="{cls}" %m'], leg(pre, op, 'd', 'm'),
              f'{mn} {{m}}, {{d}}', body, f' "dclass"="{dcls or cls}"')
    for ty, sfx, eb, n, cls in INT:
        T = tag(ty)
        ops = [('add', 'padd', {8: 'FC', 16: 'FD', 32: 'FE', 64: 'D4'}), ('sub', 'psub', {8: 'F8', 16: 'F9', 32: 'FA', 64: 'FB'})]
        if eb == 16:
            ops.append(('mul', 'pmull', {16: 'D5'}))
        for ir, mn, opc in ops:
            two(ty, cls, f'{mn.upper()}{sfx.upper()}.{T}', P66, bits(int(opc[eb], 16)), f'{mn}{sfx}',
                [f'%r = {ir} {ty} %n, %m', f'ret {ty} %r'])
        for ir, mn, opc in [('and', 'pand', 0xDB), ('or', 'por', 0xEB), ('xor', 'pxor', 0xEF)]:
            two(ty, cls, f'{mn.upper()}.{T}', P66, bits(opc), mn, [f'%r = {ir} {ty} %n, %m', f'ret {ty} %r'])
        # Shifts by an immediate (none of bytes; no arithmetic one of i64).
        for ir, r in [('shl', 6), ('lshr', 2), ('ashr', 4)]:
            if eb == 8 or (ir == 'ashr' and eb == 64):
                continue
            op = {16: '01110001', 32: '01110010', 64: '01110011'}[eb]
            mn = {'shl': 'psll', 'lshr': 'psrl', 'ashr': 'psra'}[ir] + sfx
            entry(ty, f'{mn.upper()}i.{T}', [f'{ty} "reg"="{cls}" "tied" %n', f'{ty} "imm" %sh'],
                  f'{P66}rex(0, 0, 0, d) | 00001111 | {op} | rr({r}, d) | ib:8=sh', f'{mn} ${{sh}}, {{d}}',
                  [f'%r = {ir} {ty} %n, %sh', f'ret {ty} %r'], f' "dclass"="{cls}"')
        # Comparisons, to masks: eq and sgt (not of i64: SSE4).
        M = f'<{n} x i1>'
        if eb < 64:
            k = {8: 0, 16: 1, 32: 2}[eb]
            two(ty, cls, f'PCMPEQ{sfx.upper()}.{T}', P66, bits(0x74 + k), f'pcmpeq{sfx}',
                [f'%r = icmp eq {ty} %n, %m', f'ret {M} %r'], M, 'xv')
            two(ty, cls, f'PCMPGT{sfx.upper()}.{T}', P66, bits(0x64 + k), f'pcmpgt{sfx}',
                [f'%r = icmp sgt {ty} %n, %m', f'ret {M} %r'], M, 'xv')
        # A mask's lanes as 0 or 1: its top bit shifted down.
        if eb > 8:
            op = {16: '01110001', 32: '01110010', 64: '01110011'}[eb]
            entry(ty, f'MASK.{T}', [f'{M} "reg"="xv" "tied" %n'],
                  f'{P66}rex(0, 0, 0, d) | 00001111 | {op} | rr(2, d) | ib:8={eb - 1}', f'psrl{sfx} ${eb - 1}, {{d}}',
                  [f'%r = zext {M} %n to {ty}', f'ret {ty} %r'], f' "dclass"="{cls}"')
    # Masks' logic.
    for n, cls in [(16, 'xv'), (8, 'xv'), (4, 'xv'), (2, 'xv')]:
        M = f'<{n} x i1>'
        for ir, mn, opc in [('and', 'pand', 0xDB), ('or', 'por', 0xEB), ('xor', 'pxor', 0xEF)]:
            two(M, cls, f'{mn.upper()}.m{n}', P66, bits(opc), mn, [f'%r = {ir} {M} %n, %m', f'ret {M} %r'])
    # The plain bitwise operations recipes use, and andn: ~n & m.
    for cls, ty in [('xv', '<2 x i64>'), ('xh', '<2 x i32>')]:
        T = tag(ty)
        two(ty, cls, f'PANDN.{cls}', P66, bits(0xDF), 'pandn',
            [f'%x = xor {ty} %n, splat ({ty.split()[-1][:-1]} -1)', f'%r = and {ty} %x, %m', f'ret {ty} %r'])
    # Floating point.
    for ty, sfx, et, n, cls in FLT:
        T = tag(ty)
        pre = P66 if sfx == 'pd' else ''
        for ir, opc in [('fadd', 0x58), ('fsub', 0x5C), ('fmul', 0x59), ('fdiv', 0x5E)]:
            two(ty, cls, f'{ir.upper()}{sfx.upper()}.{T}', pre, bits(opc), f'{ir[1:]}{sfx}',
                [f'%r = {ir} {ty} %n, %m', f'ret {ty} %r'])
        it = f'v{n}f{32 if et == "float" else 64}'
        entry(ty, f'SQRT{sfx.upper()}.{T}', [f'{ty} "reg"="{cls}" %n'], leg(pre, '01010001', 'd', 'n'), f'sqrt{sfx} {{n}}, {{d}}',
              [f'%r = call {ty} @llvm.sqrt.{it}({ty} %n)', f'ret {ty} %r'], f' "dclass"="{cls}"')
        out.append(f'declare {ty} @llvm.sqrt.{it}({ty})\n')
        # cmpps's predicates; the others with the operands swapped.
        M = f'<{n} x i1>'
        for pred, k, swap in [('oeq', 0, 0), ('olt', 1, 0), ('ole', 2, 0), ('uno', 3, 0), ('une', 4, 0), ('uge', 5, 0),
                              ('ugt', 6, 0), ('ord', 7, 0), ('ogt', 1, 1), ('oge', 2, 1), ('ult', 6, 1), ('ule', 5, 1)]:
            a, b = ('%m', '%n') if swap else ('%n', '%m')
            entry(M, f'CMP{pred.upper()}{sfx.upper()}.{T}', [f'{ty} "reg"="{cls}" "tied" %n', f'{ty} "reg"="{cls}" %m'],
                  leg(pre, '11000010', 'd', 'm') + f' | {bits(k)}', f'cmp{sfx} ${k}, {{m}}, {{d}}',
                  [f'%r = fcmp {pred} {ty} {a}, {b}', f'ret {M} %r'], ' "dclass"="xv"')
    # Conversions between 32-bit integers and floats, lane for lane.
    for ity, fty, cls in [('<4 x i32>', '<4 x float>', 'xv'), ('<2 x i32>', '<2 x float>', 'xh')]:
        entry(fty, f'CVTDQ2PS.{tag(fty)}', [f'{ity} "reg"="{cls}" %n'], leg('', '01011011', 'd', 'n'), 'cvtdq2ps {n}, {d}',
              [f'%r = sitofp {ity} %n to {fty}', f'ret {fty} %r'], f' "dclass"="{cls}"')
        entry(ity, f'CVTTPS2DQ.{tag(ity)}', [f'{fty} "reg"="{cls}" %n'], leg('11110011 | ', '01011011', 'd', 'n'),
              'cvttps2dq {n}, {d}', [f'%r = fptosi {fty} %n to {ity}', f'ret {ity} %r'], f' "dclass"="{cls}"')
    entry('<2 x double>', 'CVTPS2PD', ['<2 x float> "reg"="xh" %n'], leg('', '01011010', 'd', 'n'), 'cvtps2pd {n}, {d}',
          ['%r = fpext <2 x float> %n to <2 x double>', 'ret <2 x double> %r'], ' "dclass"="xv"')
    entry('<2 x float>', 'CVTPD2PS', ['<2 x double> "reg"="xv" %n'], leg(P66, '01011010', 'd', 'n'), 'cvtpd2ps {n}, {d}',
          ['%r = fptrunc <2 x double> %n to <2 x float>', 'ret <2 x float> %r'], ' "dclass"="xh"')
    entry('<2 x double>', 'CVTDQ2PD', ['<2 x i32> "reg"="xh" %n'], leg('11110011 | ', '11100110', 'd', 'n'), 'cvtdq2pd {n}, {d}',
          ['%r = sitofp <2 x i32> %n to <2 x double>', 'ret <2 x double> %r'], ' "dclass"="xv"')
    entry('<2 x i32>', 'CVTTPD2DQ', ['<2 x double> "reg"="xv" %n'], leg(P66, '11100110', 'd', 'n'), 'cvttpd2dq {n}, {d}',
          ['%r = fptosi <2 x double> %n to <2 x i32>', 'ret <2 x i32> %r'], ' "dclass"="xh"')
    # Selects by a mask: (c & a) | (~c & b).
    for ty, *rest in INT + FLT:
        cls = rest[-1]
        n = int(ty[1:].split()[0])
        M = f'<{n} x i1>'
        wide, el = ('<2 x i64>', 'i64') if cls == 'xv' else ('<2 x i32>', 'i32')
        T = tag(ty)
        ety = ty.split()[-1][:-1]
        lane = {'float': 'i32', 'double': 'i64'}.get(ety, ety)
        ity = f'<{n} x {lane}>'
        cast = (f'  %ti = bitcast {ty} %a to {wide}\n  %fi = bitcast {ty} %b to {wide}\n')
        out.append(f'define {ty} @vselect.{T}({M} "reg"="xv" %c, {ty} "reg"="{cls}" %a, {ty} "reg"="{cls}" %b) '
                   f'"recipe"="vselect.{T}.expand" {{\n  %r = select {M} %c, {ty} %a, {ty} %b\n  ret {ty} %r\n}}\n')
        pand, pandn, por = (('PAND.2xi64', 'PANDN.xv', 'POR.2xi64') if cls == 'xv' else ('PAND.2xi32', 'PANDN.xh', 'POR.2xi32'))
        out.append(f'define {ty} @vselect.{T}.expand({M} %c, {ty} %a, {ty} %b) {{\n'
                   f'  %cs = sext {M} %c to {ity}\n  %ci = bitcast {ity} %cs to {wide}\n' + cast +
                   f'  %x = call {wide} @{pand}({wide} %ci, {wide} %ti)\n'
                   f'  %y = call {wide} @{pandn}({wide} %ci, {wide} %fi)\n'
                   f'  %z = call {wide} @{por}({wide} %x, {wide} %y)\n  %r = bitcast {wide} %z to {ty}\n  ret {ty} %r\n}}\n')
    # Lanes. i16: pextrw and pinsrw.
    for ty, cls in [('<8 x i16>', 'xv'), ('<4 x i16>', 'xh')]:
        T = tag(ty)
        entry('i16', f'PEXTRW.{T}', [f'{ty} "reg"="{cls}" %v', 'i64 "imm" %k'], leg(P66, '11000101', 'd', 'v') + ' | ib:8=k',
              'pextrw ${k}, {v}, {d:r32}', [f'%r = extractelement {ty} %v, i64 %k', 'ret i16 %r'],
              ' "dclass"="r32" "zext"="16"')
        entry(ty, f'PINSRW.{T}', [f'{ty} "reg"="{cls}" "tied" %v', 'i16 "reg"="r32" %e', 'i64 "imm" %k'],
              leg(P66, '11000100', 'd', 'e') + ' | ib:8=k', 'pinsrw ${k}, {e:r32}, {d}',
              [f'%r = insertelement {ty} %v, i16 %e, i64 %k', f'ret {ty} %r'], f' "dclass"="{cls}"')
    # Lane 0 of 32- and 64-bit integers: movd, movq; lane k, a pshufd first.
    for ty, cls in [('<4 x i32>', 'xv'), ('<2 x i32>', 'xh')]:
        T = tag(ty)
        entry('i32', f'MOVDlane0.{T}', [f'{ty} "reg"="{cls}" %v'], f'{P66}rex(0, v, 0, d) | 00001111 | 01111110 | rr(v, d)',
              'movd {v}, {d}', [f'%r = extractelement {ty} %v, i64 0', 'ret i32 %r'], ' "dclass"="r32"')
        entry(ty, f'PSHUFDlane.{T}', [f'{ty} "reg"="{cls}" %v', 'i64 "imm" %k'], leg(P66, '01110000', 'd', 'v') + ' | ib:8=k',
              'pshufd ${k}, {v}, {d}', [f'ret {ty} %v'], f' "dclass"="{cls}" "nomatch"')
        out.append(f'define i32 @lane.{T}({ty} "reg"="{cls}" %v, i64 %k) "recipe"="lane.{T}.expand" {{\n'
                   f'  %r = extractelement {ty} %v, i64 %k\n  ret i32 %r\n}}\n')
        out.append(f'define i32 @lane.{T}.expand({ty} %v, i64 %k) {{\n  %s = call {ty} @PSHUFDlane.{T}({ty} %v, i64 %k)\n'
                   f'  %r = call i32 @MOVDlane0.{T}({ty} %s)\n  ret i32 %r\n}}\n')
    entry('i64', 'MOVQlane0', ['<2 x i64> "reg"="xv" %v'], f'{P66}rex(1, v, 0, d) | 00001111 | 01111110 | rr(v, d)',
          'movq {v}, {d}', ['%r = extractelement <2 x i64> %v, i64 0', 'ret i64 %r'], ' "dclass"="r64"')
    entry('<2 x i64>', 'PUNPCKHQDQself', ['<2 x i64> "reg"="xv" "tied" %v'], leg(P66, '01101101', 'd', 'd'),
          'punpckhqdq {d}, {d}', ['ret <2 x i64> %v'], ' "dclass"="xv" "nomatch"')
    out.append('define i64 @lane1.2xi64(<2 x i64> "reg"="xv" %v) "recipe"="lane1.2xi64.expand" {\n'
               '  %r = extractelement <2 x i64> %v, i64 1\n  ret i64 %r\n}\n')
    out.append('define i64 @lane1.2xi64.expand(<2 x i64> %v) {\n  %s = call <2 x i64> @PUNPCKHQDQself(<2 x i64> %v)\n'
               '  %r = call i64 @MOVQlane0(<2 x i64> %s)\n  ret i64 %r\n}\n')
    # Floats: lane k brought to lane 0 (pshufd), where the float is.
    for ty, cls in [('<4 x float>', 'xv'), ('<2 x float>', 'xh')]:
        entry('float', f'PSHUFDf.{tag(ty)}', [f'{ty} "reg"="{cls}" %v', 'i64 "imm" %k'], leg(P66, '01110000', 'd', 'v') + ' | ib:8=k',
              'pshufd ${k}, {v}, {d}', [f'%r = extractelement {ty} %v, i64 %k', 'ret float %r'], ' "dclass"="s"')
    entry('double', 'MOVAPDlane0', ['<2 x double> "reg"="xv" %v'], 'rex(0, d, 0, v) | 00001111 | 00101000 | rr(d, v)',
          'movaps {v}, {d}', ['%r = extractelement <2 x double> %v, i64 0', 'ret double %r'], ' "dclass"="d"')
    entry('double', 'UNPCKHPDlane1', ['<2 x double> "reg"="xv" "tied" %v'], leg(P66, '00010101', 'd', 'd'),
          'unpckhpd {d}, {d}', ['%r = extractelement <2 x double> %v, i64 1', 'ret double %r'], ' "dclass"="d"')
    # Splats.
    def splat_body(ty, et, x):
        return (f'  %i = insertelement {ty} poison, {et} {x}, i64 0\n'
                f'  %r = shufflevector {ty} %i, {ty} poison, {ty.replace(et, "i32")} zeroinitializer\n')
    entry('<4 x i32>', 'MOVDto', ['i32 "reg"="r32" %n'], f'{P66}rex(0, d, 0, n) | 00001111 | 01101110 | rr(d, n)',
          'movd {n}, {d}', ['%r = insertelement <4 x i32> zeroinitializer, i32 %n, i64 0', 'ret <4 x i32> %r'], ' "dclass"="xv"')
    entry('<2 x i64>', 'MOVQto', ['i64 "reg"="r64" %n'], f'{P66}rex(1, d, 0, n) | 00001111 | 01101110 | rr(d, n)',
          'movq {n}, {d}', ['%r = insertelement <2 x i64> zeroinitializer, i64 %n, i64 0', 'ret <2 x i64> %r'],
          ' "dclass"="xv" "role"="vec.from.x"')
    # A 64-bit vector's bits from and to an integer register.
    entry('<8 x i8>', 'MOVQtoh', ['i64 "reg"="r64" %n'], f'{P66}rex(1, d, 0, n) | 00001111 | 01101110 | rr(d, n)',
          'movq {n}, {d}', ['%r = bitcast i64 %n to <8 x i8>', 'ret <8 x i8> %r'], ' "dclass"="xh"')
    entry('i64', 'MOVQfromh', ['<8 x i8> "reg"="xh" %v'], f'{P66}rex(1, v, 0, d) | 00001111 | 01111110 | rr(v, d)',
          'movq {v}, {d}', ['%r = bitcast <8 x i8> %v to i64', 'ret i64 %r'], ' "dclass"="r64"')
    # (Constants are made of 64-bit words: the second joined above the
    # first.)
    entry('<2 x i64>', 'PUNPCKLQDQ', ['<2 x i64> "reg"="xv" "tied" %n', '<2 x i64> "reg"="xv" %m'], leg(P66, '01101100', 'd', 'm'),
          'punpcklqdq {m}, {d}', ['ret <2 x i64> %n'], ' "dclass"="xv" "role"="vec.join.d" "nomatch"')
    # A mask as the integers of its lanes (all ones or zeros), and back:
    # the same bits (legalize.rs, for lanes done one at a time).
    for n, w, cls in [(16, 8, 'xv'), (8, 16, 'xv'), (4, 32, 'xv'), (2, 64, 'xv'), (8, 8, 'xh'), (4, 16, 'xh'), (2, 32, 'xh')]:
        M, I = f'<{n} x i1>', f'<{n} x i{w}>'
        entry(I, f'BITS.{n}x{w}', [f'{M} "reg"="xv" %m'], 'rex(0, d, 0, m) | 00001111 | 00101000 | rr(d, m)',
              'movaps {m}, {d}', [f'%r = call {I} @llvm.epitome.mask.bits.v{n}i{w}({M} %m)', f'ret {I} %r'], f' "dclass"="{cls}"')
        entry(M, f'MASK.{n}x{w}', [f'{I} "reg"="{cls}" %v'], 'rex(0, d, 0, v) | 00001111 | 00101000 | rr(d, v)',
              'movaps {v}, {d}', [f'%r = call {M} @llvm.epitome.bits.mask.v{n}i{w}({I} %v)', f'ret {M} %r'], ' "dclass"="xv"')
        out.append(f'declare {I} @llvm.epitome.mask.bits.v{n}i{w}({M})\ndeclare {M} @llvm.epitome.bits.mask.v{n}i{w}({I})\n')
    entry('<4 x i32>', 'PSHUFD', ['<4 x i32> "reg"="xv" %v', 'i64 "imm" %k'], leg(P66, '01110000', 'd', 'v') + ' | ib:8=k',
          'pshufd ${k}, {v}, {d}', ['ret <4 x i32> %v'], ' "dclass"="xv" "nomatch"')
    entry('<8 x i16>', 'PSHUFLW', ['<8 x i16> "reg"="xv" %v', 'i64 "imm" %k'], leg('11110010 | ', '01110000', 'd', 'v') + ' | ib:8=k',
          'pshuflw ${k}, {v}, {d}', ['ret <8 x i16> %v'], ' "dclass"="xv" "nomatch"')
    entry('<16 x i8>', 'PUNPCKLBWself', ['<16 x i8> "reg"="xv" "tied" %v'], leg(P66, '01100000', 'd', 'd'),
          'punpcklbw {d}, {d}', ['ret <16 x i8> %v'], ' "dclass"="xv" "nomatch"')
    for ty, et, cls, steps in [('<4 x i32>', 'i32', 'xv', ['d']), ('<2 x i32>', 'i32', 'xh', ['d']),
                               ('<2 x i64>', 'i64', 'xv', ['q']), ('<8 x i16>', 'i16', 'xv', ['w']),
                               ('<4 x i16>', 'i16', 'xh', ['w']), ('<16 x i8>', 'i8', 'xv', ['b']), ('<8 x i8>', 'i8', 'xh', ['b']),
                               ('<4 x float>', 'float', 'xv', ['f']), ('<2 x float>', 'float', 'xh', ['f']),
                               ('<2 x double>', 'double', 'xv', ['g'])]:
        T = tag(ty)
        src = {'i32': 'i32 "reg"="r32"', 'i16': 'i16 "reg"="r32"', 'i8': 'i8 "reg"="r32"', 'i64': 'i64 "reg"="r64"',
               'float': 'float "reg"="s"', 'double': 'double "reg"="d"'}[et]
        out.append(f'define {ty} @splat.{T}({src} %x) "recipe"="splat.{T}.expand" {{\n' + splat_body(ty, et, '%x') +
                   f'  ret {ty} %r\n}}\n')
        if et in ('i32', 'i16', 'i8'):
            x = '%x' if et == 'i32' else '%w'
            pre = '' if et == 'i32' else f'  %w = zext {et} %x to i32\n'
            body = pre + f'  %v = call <4 x i32> @MOVDto(i32 {x})\n'
            if et == 'i8':
                body += ('  %vb = bitcast <4 x i32> %v to <16 x i8>\n  %u = call <16 x i8> @PUNPCKLBWself(<16 x i8> %vb)\n'
                         '  %v2 = bitcast <16 x i8> %u to <4 x i32>\n')
                v = '%v2'
            else:
                v = '%v'
            if et != 'i32':
                body += (f'  %h = bitcast <4 x i32> {v} to <8 x i16>\n  %l = call <8 x i16> @PSHUFLW(<8 x i16> %h, i64 0)\n'
                         '  %v3 = bitcast <8 x i16> %l to <4 x i32>\n')
                v = '%v3'
            body += f'  %s = call <4 x i32> @PSHUFD(<4 x i32> {v}, i64 0)\n  %r = bitcast <4 x i32> %s to {ty}\n'
        elif et == 'i64':
            body = '  %v = call <2 x i64> @MOVQto(i64 %x)\n  %vi = bitcast <2 x i64> %v to <4 x i32>\n' \
                   '  %s = call <4 x i32> @PSHUFD(<4 x i32> %vi, i64 68)\n  %r = bitcast <4 x i32> %s to <2 x i64>\n'
        elif et == 'float':
            body = '  %vi = bitcast float %x to <4 x i32>\n  %s = call <4 x i32> @PSHUFD(<4 x i32> %vi, i64 0)\n' \
                   f'  %r = bitcast <4 x i32> %s to {ty}\n'
        else:
            body = '  %vi = bitcast double %x to <4 x i32>\n  %s = call <4 x i32> @PSHUFD(<4 x i32> %vi, i64 68)\n' \
                   f'  %r = bitcast <4 x i32> %s to {ty}\n'
        out.append(f'define {ty} @splat.{T}.expand({src.split()[0]} %x) {{\n' + body + f'  ret {ty} %r\n}}\n')
    # Halves of 128-bit vectors (as legalisation splits them): the low one
    # in place, the high one moved down; and two joined.
    for full, half, n in [('<16 x i8>', '<8 x i8>', 16), ('<8 x i16>', '<4 x i16>', 8), ('<4 x i32>', '<2 x i32>', 4),
                          ('<4 x float>', '<2 x float>', 4)]:
        h = n // 2
        lo = ', '.join(f'i32 {k}' for k in range(h))
        hi = ', '.join(f'i32 {k}' for k in range(h, n))
        both = ', '.join(f'i32 {k}' for k in range(n))
        entry(half, f'LOW.{tag(full)}', [f'{full} "reg"="xv" %v'], 'rex(0, d, 0, v) | 00001111 | 00101000 | rr(d, v)',
              'movaps {v}, {d}', [f'%r = shufflevector {full} %v, {full} poison, <{h} x i32> <{lo}>', f'ret {half} %r'],
              ' "dclass"="xh"')
        entry(half, f'HIGH.{tag(full)}', [f'{full} "reg"="xv" %v'], leg(P66, '01110000', 'd', 'v') + ' | 11101110',
              'pshufd $238, {v}, {d}', [f'%r = shufflevector {full} %v, {full} poison, <{h} x i32> <{hi}>', f'ret {half} %r'],
              ' "dclass"="xh"')
        entry(full, f'JOIN.{tag(full)}', [f'{half} "reg"="xh" "tied" %a', f'{half} "reg"="xh" %b'], leg(P66, '01101100', 'd', 'b'),
              'punpcklqdq {b}, {d}', [f'%r = shufflevector {half} %a, {half} %b, <{n} x i32> <{both}>', f'ret {full} %r'],
              ' "dclass"="xv"')
    # Widening a 64-bit vector's lanes: interleaved with zeros (zext), or
    # with themselves and shifted down (sext; 32 to 64 bits with their signs).
    for nar, wid, k, unp in [('<8 x i8>', '<8 x i16>', 8, '01100000'), ('<4 x i16>', '<4 x i32>', 16, '01100001'),
                             ('<2 x i32>', '<2 x i64>', 32, '01100010')]:
        T = tag(nar)
        mn = {8: 'punpcklbw', 16: 'punpcklwd', 32: 'punpckldq'}[k]
        entry(wid, f'UNPCKLz.{T}', [f'{nar} "reg"="xh" "tied" %v', '<2 x i64> "reg"="xv" %z'], leg(P66, unp, 'd', 'z'),
              f'{mn} {{z}}, {{d}}', [f'ret {wid} poison'], ' "dclass"="xv" "nomatch"')
        entry(wid, f'UNPCKLs.{T}', [f'{nar} "reg"="xh" "tied" %v'], leg(P66, unp, 'd', 'd'),
              f'{mn} {{d}}, {{d}}', [f'ret {wid} poison'], ' "dclass"="xv" "nomatch"')
        out.append(f'define {wid} @zext.{T}({nar} "reg"="xh" %v) "recipe"="zext.{T}.expand" {{\n'
                   f'  %r = zext {nar} %v to {wid}\n  ret {wid} %r\n}}\n')
        out.append(f'define {wid} @zext.{T}.expand({nar} %v) {{\n  %z = call <2 x i64> @PXORzero()\n'
                   f'  %r = call {wid} @UNPCKLz.{T}({nar} %v, <2 x i64> %z)\n  ret {wid} %r\n}}\n')
        out.append(f'define {wid} @sext.{T}({nar} "reg"="xh" %v) "recipe"="sext.{T}.expand" {{\n'
                   f'  %r = sext {nar} %v to {wid}\n  ret {wid} %r\n}}\n')
        if k < 32:
            out.append(f'define {wid} @sext.{T}.expand({nar} %v) {{\n  %u = call {wid} @UNPCKLs.{T}({nar} %v)\n'
                       f'  %r = call {wid} @PSRA{"W" if k == 8 else "D"}i.{tag(wid)}({wid} %u, {wid.split()[-1][:-1]} {k})\n'
                       f'  ret {wid} %r\n}}\n')
        else:
            out.append(f'define {wid} @sext.{T}.expand({nar} %v) {{\n  %z = call <2 x i64> @PXORzero()\n'
                       f'  %zi = bitcast <2 x i64> %z to <2 x i32>\n  %s = call <2 x i1> @PCMPGTD.2xi32(<2 x i32> %zi, <2 x i32> %v)\n'
                       f'  %si = sext <2 x i1> %s to <2 x i32>\n  %sw = bitcast <2 x i32> %si to <2 x i64>\n'
                       f'  %r = call {wid} @UNPCKLz.{T}({nar} %v, <2 x i64> %sw)\n  ret {wid} %r\n}}\n')


def sse_recipes():
    """What SSE2 does in a few instructions: the other comparisons, the
    least and greatest, multiplying 32-bit lanes, and a sign's bits."""
    P66 = '01100110 | '
    def leg(pre, op, r, m):
        return f'{pre}rex(0, {r}, 0, {m}) | 00001111 | {op} | rr({r}, {m})'
    tag = lambda ty: ty.strip('<>').replace(' x ', 'x').replace('float', 'f32').replace('double', 'f64')
    # All ones (any register compared with itself).
    entry('<2 x i64>', 'ALLONES', [], leg(P66, '01110110', 'd', 'd'), 'pcmpeqd {d}, {d}',
          ['ret <2 x i64> splat (i64 -1)'], ' "dclass"="xv" "nomatch"')
    def recipe(name, ret, params, body, expand):
        ps = ', '.join(f'{t} "reg"="{c}" %{n}' for t, c, n in params)
        plain = ', '.join(f'{t} %{n}' for t, c, n in params)
        out.append(f'define {ret} @{name}({ps}) "recipe"="{name}.expand" {{\n' + ''.join(f'  {l}\n' for l in body) + '}\n')
        out.append(f'define {ret} @{name}.expand({plain}) {{\n' + ''.join(f'  {l}\n' for l in expand) + '}\n')
    INT = [('<16 x i8>', 'b', 8, 16, 'xv'), ('<8 x i16>', 'w', 16, 8, 'xv'), ('<4 x i32>', 'd', 32, 4, 'xv'),
           ('<8 x i8>', 'b', 8, 8, 'xh'), ('<4 x i16>', 'w', 16, 4, 'xh'), ('<2 x i32>', 'd', 32, 2, 'xh')]
    for ty, sfx, eb, n, cls in INT:
        T = tag(ty)
        M = f'<{n} x i1>'
        S = sfx.upper()
        eq, gt = f'PCMPEQ{S}.{T}', f'PCMPGT{S}.{T}'
        wide = '<2 x i64>'
        notm = lambda x: [f'%xi = sext {M} {x} to {ty}', f'%xw = bitcast {ty} %xi to {wide}',
                          '%on = call <2 x i64> @ALLONES()', f'%nw = call {wide} @PXOR.2xi64({wide} %xw, {wide} %on)',
                          f'%ni = bitcast {wide} %nw to {ty}', f'%r = trunc {ty} %ni to {M}', f'ret {M} %r']
        # slt: sgt the other way round.
        recipe(f'slt.{T}', M, [(ty, cls, 'a'), (ty, cls, 'b')], [f'%r = icmp slt {ty} %a, %b', f'ret {M} %r'],
               [f'%r = call {M} @{gt}({ty} %b, {ty} %a)', f'ret {M} %r'])
        # ne, sge, sle: the opposite comparison, inverted.
        for pred, inner, a, b in [('ne', eq, '%a', '%b'), ('sge', gt, '%b', '%a'), ('sle', gt, '%a', '%b')]:
            recipe(f'{pred}.{T}', M, [(ty, cls, 'a'), (ty, cls, 'b')], [f'%r = icmp {pred} {ty} %a, %b', f'ret {M} %r'],
                   [f'%c = call {M} @{inner}({ty} {a}, {ty} {b})'] + notm('%c'))
        # Unsigned: bytes by their greatest (a >= b when max(a, b) == a),
        # words by saturating subtraction (a <= b when a - b is 0), and
        # 32-bit lanes with their sign bits flipped, compared signed.
        if eb == 8:
            mx = f'PMAXUB.{T}'
            entry(ty, mx, [f'{ty} "reg"="{cls}" "tied" %n', f'{ty} "reg"="{cls}" %m'], leg(P66, '11011110', 'd', 'm'),
                  'pmaxub {m}, {d}', [f'%r = call {ty} @llvm.umax.v{n}i8({ty} %n, {ty} %m)', f'ret {ty} %r'], f' "dclass"="{cls}"')
            entry(ty, f'PMINUB.{T}', [f'{ty} "reg"="{cls}" "tied" %n', f'{ty} "reg"="{cls}" %m'], leg(P66, '11011010', 'd', 'm'),
                  'pminub {m}, {d}', [f'%r = call {ty} @llvm.umin.v{n}i8({ty} %n, {ty} %m)', f'ret {ty} %r'], f' "dclass"="{cls}"')
            out.append(f'declare {ty} @llvm.umax.v{n}i8({ty}, {ty})\ndeclare {ty} @llvm.umin.v{n}i8({ty}, {ty})\n')
            for pred, x, y, inv in [('uge', '%a', '%b', False), ('ule', '%b', '%a', False), ('ult', '%a', '%b', True),
                                    ('ugt', '%b', '%a', True)]:
                body = [f'%m = call {ty} @{mx}({ty} {x}, {ty} {y})', f'%c = call {M} @{eq}({ty} %m, {ty} {x})']
                recipe(f'{pred}.{T}', M, [(ty, cls, 'a'), (ty, cls, 'b')], [f'%r = icmp {pred} {ty} %a, %b', f'ret {M} %r'],
                       body + (notm('%c') if inv else [f'ret {M} %c']))
        elif eb == 16:
            sub = f'PSUBUSW.{T}'
            entry(ty, sub, [f'{ty} "reg"="{cls}" "tied" %n', f'{ty} "reg"="{cls}" %m'], leg(P66, '11011001', 'd', 'm'),
                  'psubusw {m}, {d}', [f'%r = call {ty} @llvm.usub.sat.v{n}i16({ty} %n, {ty} %m)', f'ret {ty} %r'],
                  f' "dclass"="{cls}"')
            out.append(f'declare {ty} @llvm.usub.sat.v{n}i16({ty}, {ty})\n')
            for name, mn, op, ir in [('PMAXSW', 'pmaxsw', '11101110', 'smax'), ('PMINSW', 'pminsw', '11101010', 'smin')]:
                entry(ty, f'{name}.{T}', [f'{ty} "reg"="{cls}" "tied" %n', f'{ty} "reg"="{cls}" %m'], leg(P66, op, 'd', 'm'),
                      f'{mn} {{m}}, {{d}}', [f'%r = call {ty} @llvm.{ir}.v{n}i16({ty} %n, {ty} %m)', f'ret {ty} %r'],
                      f' "dclass"="{cls}"')
                out.append(f'declare {ty} @llvm.{ir}.v{n}i16({ty}, {ty})\n')
            for pred, x, y, inv in [('ule', '%a', '%b', False), ('uge', '%b', '%a', False), ('ugt', '%a', '%b', True),
                                    ('ult', '%b', '%a', True)]:
                body = ['%z = call <2 x i64> @PXORzero()', f'%zt = bitcast <2 x i64> %z to {ty}',
                        f'%d = call {ty} @{sub}({ty} {x}, {ty} {y})', f'%c = call {M} @{eq}({ty} %d, {ty} %zt)']
                recipe(f'{pred}.{T}', M, [(ty, cls, 'a'), (ty, cls, 'b')], [f'%r = icmp {pred} {ty} %a, %b', f'ret {M} %r'],
                       body + (notm('%c') if inv else [f'ret {M} %c']))
        else:
            sh = f'PSLLDi.{T}'
            for pred, sp, x, y, inv in [('ugt', 'sgt', '%fa', '%fb', False), ('ult', 'sgt', '%fb', '%fa', False),
                                        ('ule', 'sgt', '%fa', '%fb', True), ('uge', 'sgt', '%fb', '%fa', True)]:
                body = ['%o = call <2 x i64> @ALLONES()', f'%ot = bitcast <2 x i64> %o to {ty}',
                        f'%sg = call {ty} @{sh}({ty} %ot, i32 31)', f'%fa = call {ty} @PXOR.{T}({ty} %a, {ty} %sg)',
                        f'%fb = call {ty} @PXOR.{T}({ty} %b, {ty} %sg)', f'%c = call {M} @{gt}({ty} {x}, {ty} {y})']
                recipe(f'{pred}.{T}', M, [(ty, cls, 'a'), (ty, cls, 'b')], [f'%r = icmp {pred} {ty} %a, %b', f'ret {M} %r'],
                       body + (notm('%c') if inv else [f'ret {M} %c']))
    # 32-bit lanes multiplied: the even lanes' and the odd lanes' 64-bit
    # products (pmuludq), their low halves gathered.
    entry('<2 x i64>', 'PMULUDQ', ['<2 x i64> "reg"="xv" "tied" %n', '<2 x i64> "reg"="xv" %m'], leg(P66, '11110100', 'd', 'm'),
          'pmuludq {m}, {d}', ['ret <2 x i64> %n'], ' "dclass"="xv" "nomatch"')
    entry('<4 x i32>', 'PUNPCKLDQ', ['<4 x i32> "reg"="xv" "tied" %n', '<4 x i32> "reg"="xv" %m'], leg(P66, '01100010', 'd', 'm'),
          'punpckldq {m}, {d}', ['ret <4 x i32> %n'], ' "dclass"="xv" "nomatch"')
    for ty, cls in [('<4 x i32>', 'xv'), ('<2 x i32>', 'xh')]:
        T = tag(ty)
        recipe(f'mul.{T}', ty, [(ty, cls, 'a'), (ty, cls, 'b')], [f'%r = mul {ty} %a, %b', f'ret {ty} %r'],
               [f'%aw = bitcast {ty} %a to <2 x i64>', f'%bw = bitcast {ty} %b to <2 x i64>',
                '%p0 = call <2 x i64> @PMULUDQ(<2 x i64> %aw, <2 x i64> %bw)',
                '%a1 = call <2 x i64> @PSRLQi.2xi64(<2 x i64> %aw, i64 32)', '%b1 = call <2 x i64> @PSRLQi.2xi64(<2 x i64> %bw, i64 32)',
                '%p1 = call <2 x i64> @PMULUDQ(<2 x i64> %a1, <2 x i64> %b1)',
                '%q0 = bitcast <2 x i64> %p0 to <4 x i32>', '%q1 = bitcast <2 x i64> %p1 to <4 x i32>',
                '%s0 = call <4 x i32> @PSHUFD(<4 x i32> %q0, i64 8)', '%s1 = call <4 x i32> @PSHUFD(<4 x i32> %q1, i64 8)',
                '%u = call <4 x i32> @PUNPCKLDQ(<4 x i32> %s0, <4 x i32> %s1)', f'%r = bitcast <4 x i32> %u to {ty}',
                f'ret {ty} %r'])
    # A float's sign: flipped (xor) or cleared (and), with masks made of all
    # ones shifted.
    for ty, cls, et, w in [('<4 x float>', 'xv', 'f32', 32), ('<2 x double>', 'xv', 'f64', 64), ('<2 x float>', 'xh', 'f32', 32)]:
        T = tag(ty)
        n = int(ty[1:].split()[0])
        lane = 'q' if w == 64 else 'd'
        shl, shr = (f'PSLLQi.2xi64', f'PSRLQi.2xi64') if w == 64 else (f'PSLLDi.4xi32', f'PSRLDi.4xi32')
        it = '<2 x i64>' if w == 64 else '<4 x i32>'
        recipe(f'fneg.{T}', ty, [(ty, cls, 'a')], [f'%r = fneg {ty} %a', f'ret {ty} %r'],
               ['%o = call <2 x i64> @ALLONES()', f'%oi = bitcast <2 x i64> %o to {it}',
                f'%s = call {it} @{shl}({it} %oi, i32 {w - 1})', '%sw = bitcast ' + it + ' %s to <2 x i64>',
                f'%aw = bitcast {ty} %a to <2 x i64>', '%x = call <2 x i64> @PXOR.2xi64(<2 x i64> %aw, <2 x i64> %sw)',
                f'%r = bitcast <2 x i64> %x to {ty}', f'ret {ty} %r'])
        recipe(f'fabs.{T}', ty, [(ty, cls, 'a')], [f'%r = call {ty} @llvm.fabs.v{n}{et}({ty} %a)', f'ret {ty} %r'],
               ['%o = call <2 x i64> @ALLONES()', f'%oi = bitcast <2 x i64> %o to {it}',
                f'%s = call {it} @{shr}({it} %oi, i32 1)', '%sw = bitcast ' + it + ' %s to <2 x i64>',
                f'%aw = bitcast {ty} %a to <2 x i64>', '%x = call <2 x i64> @PAND.2xi64(<2 x i64> %aw, <2 x i64> %sw)',
                f'%r = bitcast <2 x i64> %x to {ty}', f'ret {ty} %r'])
        out.append(f'declare {ty} @llvm.fabs.v{n}{et}({ty})\n')


XED = None


def xed(asm, classes):
    """The encoding of 02-xed.ll's entry written `asm` whose registers are
    of `classes` (in order), and its operands' names: VEX's encodings as
    llvm-mc writes them, its swapped sources included."""
    global XED
    if XED is None:
        import re
        XED = {}
        for line in open(os.path.join(os.path.dirname(os.path.abspath(__file__)), '02-xed.ll')):
            m = re.match(r'define void @\w+\((.*?)\) "enc"="([^"]*)" "asm"="([^"]*)"', line)
            if m:
                ps = re.findall(r'"(reg|imm|sym)"(?:="(\w+)")? %([\w.]+)', m.group(1))
                XED.setdefault(m.group(3), []).append((ps, m.group(2)))
    # (VEX's alone are x and y; with EVEX's too, xe and ye.)
    plain = lambda cs: [c.rstrip('e') for c in cs]
    for exact in (True, False):
        for ps, enc in XED.get(asm, []):
            regs = [c for k, c, n in ps if k == 'reg' and n not in ('m.base', 'm.index')]
            if regs == list(classes) if exact else plain(regs) == plain(classes):
                return enc, [n for _, _, n in ps]
    raise KeyError(f'no XED entry {asm!r} of {classes}')


def renamed(enc, names):
    """`enc` with its operands renamed (`o2` to `d`)."""
    import re
    return re.sub(r'(?<![\w.])(o\d|c|m\.disp|m\.base)(?![\w.])', lambda m: names.get(m.group(1), m.group(1)), enc)


def avx2():
    """AVX and AVX2's 256-bit vectors, in yv (ymm0-15), for functions with
    avx2: VEX encodings from 02-xed.ll. Masks of them are in yv too."""
    feat = ' "feature"="avx2"'
    INT = [('<32 x i8>', 'b', 8, 32), ('<16 x i16>', 'w', 16, 16), ('<8 x i32>', 'd', 32, 8), ('<4 x i64>', 'q', 64, 4)]
    FLT = [('<8 x float>', 'ps', 'float', 8, 32), ('<4 x double>', 'pd', 'double', 4, 64)]
    tag = lambda ty: ty.strip('<>').replace(' x ', 'x').replace('float', 'f32').replace('double', 'f64')
    Y = 'ye'
    def three(ty, name, asm, body, rty=None, dcls='yv', classes=(Y, Y, Y), extra=''):
        # AT&T: src2, src1, dst; o0 src2 (m), o1 src1 (n), o2 dst.
        enc, _ = xed(asm, classes)
        enc = renamed(enc, {'o0': 'm', 'o1': 'n', 'o2': 'd'})
        entry(rty or ty, name, [f'{ty} "reg"="yv" %n', f'{ty} "reg"="yv" %m'], enc,
              asm.replace('{o0}', '{m}').replace('{o1}', '{n}').replace('{o2}', '{d}'), body,
              f' "dclass"="{dcls}"' + feat + extra)
    # Memory, copies, zero.
    for ty, *_ in INT + FLT:
        T = tag(ty)
        role = ty == '<4 x i64>'
        enc, _ = xed('vmovups {m.disp}({m.base}), {o0}', (Y,))
        entry(ty, f'VLD.{T}', ['ptr "reg"="r64" %n', 'i64 "imm" %off'], renamed(enc, {'o0': 'd', 'm.disp': 'off', 'm.base': 'n'}),
              'vmovups {off}({n}), {d}', ['%a = getelementptr i8, ptr %n, i64 %off', f'%r = load {ty}, ptr %a', f'ret {ty} %r'],
              ' "dclass"="yv"' + (' "role"="load.yv"' if role else '') + feat)
        enc, _ = xed('vmovups {o0}, {m.disp}({m.base})', (Y,))
        entry('void', f'VST.{T}', [f'{ty} "reg"="yv" %t', 'ptr "reg"="r64" %n', 'i64 "imm" %off'],
              renamed(enc, {'o0': 't', 'm.disp': 'off', 'm.base': 'n'}), 'vmovups {t}, {off}({n})',
              ['%a = getelementptr i8, ptr %n, i64 %off', f'store {ty} %t, ptr %a', 'ret void'],
              (' "role"="store.yv"' if role else '') + feat)
    enc, _ = xed('vmovaps {o0}, {o1}', (Y, Y))
    entry('<4 x i64>', 'VMOVAPSyv', ['<4 x i64> "reg"="yv" %n'], renamed(enc, {'o0': 'n', 'o1': 'd'}), 'vmovaps {n}, {d}',
          ['ret <4 x i64> %n'], ' "copy"="yv" "dclass"="yv"')
    enc, _ = xed('vpxor {o0}, {o1}, {o2}', (Y, Y, Y))
    entry('<4 x i64>', 'VPXORzero', [], renamed(enc, {'o0': 'd', 'o1': 'd', 'o2': 'd'}), 'vpxor {d}, {d}, {d}',
          ['ret <4 x i64> zeroinitializer'], ' "dclass"="yv" "role"="movi.zero.yv" "nomatch"')
    # The high 128 bits of a constant, inserted.
    enc, _ = xed('vinserti128 ${c}, {o0}, {o1}, {o2}', ('x', 'y', 'y'))
    entry('<4 x i64>', 'VINSERTI128hi', ['<4 x i64> "reg"="yv" "tied" %n', '<2 x i64> "reg"="xv" %h'],
          renamed(enc, {'c': '1', 'o0': 'h', 'o1': 'n', 'o2': 'd'}).replace('i1:8=1', '00000001'),
          'vinserti128 $1, {h}, {d}, {d}', ['ret <4 x i64> %n'], ' "dclass"="yv" "role"="vec.join.q" "nomatch"')
    # Integer lanes.
    for ty, sfx, eb, n in INT:
        T = tag(ty)
        S = sfx.upper()
        ops = [('add', 'vpadd'), ('sub', 'vpsub')]
        if eb == 16:
            ops.append(('mul', 'vpmull'))
        if eb == 32:
            ops.append(('mul', 'vpmull'))
        for ir, mn in ops:
            three(ty, f'{mn.upper()}{S}.{T}', f'{mn}{sfx} {{o0}}, {{o1}}, {{o2}}', [f'%r = {ir} {ty} %n, %m', f'ret {ty} %r'])
        for ir, mn in [('and', 'vpand'), ('or', 'vpor'), ('xor', 'vpxor')]:
            three(ty, f'{mn.upper()}.{T}', f'{mn} {{o0}}, {{o1}}, {{o2}}', [f'%r = {ir} {ty} %n, %m', f'ret {ty} %r'])
        M = f'<{n} x i1>'
        three(ty, f'VPCMPEQ{S}.{T}', f'vpcmpeq{sfx} {{o0}}, {{o1}}, {{o2}}', [f'%r = icmp eq {ty} %n, %m', f'ret {M} %r'], M)
        three(ty, f'VPCMPGT{S}.{T}', f'vpcmpgt{sfx} {{o0}}, {{o1}}, {{o2}}', [f'%r = icmp sgt {ty} %n, %m', f'ret {M} %r'], M)
        if eb < 64:
            for ir, mn in [('smin', 'vpmins'), ('smax', 'vpmaxs'), ('umin', 'vpminu'), ('umax', 'vpmaxu')]:
                three(ty, f'{mn.upper()}{S}.{T}', f'{mn}{sfx} {{o0}}, {{o1}}, {{o2}}',
                      [f'%r = call {ty} @llvm.{ir}.v{n}i{eb}({ty} %n, {ty} %m)', f'ret {ty} %r'])
                out.append(f'declare {ty} @llvm.{ir}.v{n}i{eb}({ty}, {ty})\n')
        # Shifts by an immediate, and (32 and 64 bits) by a vector.
        for ir, mn in [('shl', 'vpsll'), ('lshr', 'vpsrl'), ('ashr', 'vpsra')]:
            if eb == 8 or (ir == 'ashr' and eb == 64):
                continue
            enc, _ = xed(f'{mn}{sfx} ${{c}}, {{o0}}, {{o1}}', (Y, Y))
            entry(ty, f'{mn.upper()}{S}i.{T}', [f'{ty} "reg"="yv" %n', f'{ty} "imm" %sh'],
                  renamed(enc, {'c': 'sh', 'o0': 'n', 'o1': 'd'}), f'{mn}{sfx} ${{sh}}, {{n}}, {{d}}',
                  [f'%r = {ir} {ty} %n, %sh', f'ret {ty} %r'], ' "dclass"="yv"' + feat)
            if eb >= 32 and not (ir == 'ashr' and eb == 64):
                three(ty, f'{mn.upper()}V{S}.{T}', f'{mn}v{sfx} {{o0}}, {{o1}}, {{o2}}', [f'%r = {ir} {ty} %n, %m', f'ret {ty} %r'])
        # A mask's lanes as 0 or 1.
        if eb > 8:
            enc, _ = xed(f'vpsrl{sfx} ${{c}}, {{o0}}, {{o1}}', (Y, Y))
            entry(ty, f'VMASK.{T}', [f'{M} "reg"="yv" %n'], renamed(enc, {'c': str(eb - 1), 'o0': 'n', 'o1': 'd'}).replace(f'i1:8={eb - 1}', bits(eb - 1)),
                  f'vpsrl{sfx} ${eb - 1}, {{n}}, {{d}}', [f'%r = zext {M} %n to {ty}', f'ret {ty} %r'], ' "dclass"="yv"' + feat)
        # Selects by a mask: each byte by its top bit, which a mask's lanes
        # all have.
        enc, _ = xed('vpblendvb {o0}, {o1}, {o2}, {o3}', ('y', 'y', 'y', 'y'))
        entry(ty, f'VPBLENDVB.{T}', [f'{M} "reg"="yv" %c', f'{ty} "reg"="yv" %a', f'{ty} "reg"="yv" %b'],
              renamed(enc, {'o0': 'c', 'o1': 'a', 'o2': 'b', 'o3': 'd'}).replace('o3', 'd'), 'vpblendvb {c}, {a}, {b}, {d}',
              [f'%r = select {M} %c, {ty} %a, {ty} %b', f'ret {ty} %r'], ' "dclass"="yv"' + feat)
    # Shifts of 128-bit vectors by vectors.
    for ty, sfx in [('<4 x i32>', 'd'), ('<2 x i64>', 'q')]:
        for ir, mn in [('shl', 'vpsll'), ('lshr', 'vpsrl'), ('ashr', 'vpsra')]:
            if ir == 'ashr' and sfx == 'q':
                continue
            enc, _ = xed(f'{mn}v{sfx} {{o0}}, {{o1}}, {{o2}}', ('xe', 'xe', 'xe'))
            entry(ty, f'{mn.upper()}V{sfx.upper()}x.{tag(ty)}', [f'{ty} "reg"="xv" %n', f'{ty} "reg"="xv" %m'],
                  renamed(enc, {'o0': 'm', 'o1': 'n', 'o2': 'd'}), f'{mn}v{sfx} {{m}}, {{n}}, {{d}}',
                  [f'%r = {ir} {ty} %n, %m', f'ret {ty} %r'], ' "dclass"="xv"' + feat)
    # Floating point.
    for ty, sfx, et, n, eb in FLT:
        T = tag(ty)
        S = sfx.upper()
        for ir, mn in [('fadd', 'vadd'), ('fsub', 'vsub'), ('fmul', 'vmul'), ('fdiv', 'vdiv')]:
            three(ty, f'{mn.upper()}{S}.{T}', f'{mn}{sfx} {{o0}}, {{o1}}, {{o2}}', [f'%r = {ir} {ty} %n, %m', f'ret {ty} %r'])
        it = f'v{n}f{eb}'
        enc, _ = xed(f'vsqrt{sfx} {{o0}}, {{o1}}', (Y, Y))
        entry(ty, f'VSQRT{S}.{T}', [f'{ty} "reg"="yv" %n'], renamed(enc, {'o0': 'n', 'o1': 'd'}), f'vsqrt{sfx} {{n}}, {{d}}',
              [f'%r = call {ty} @llvm.sqrt.{it}({ty} %n)', f'ret {ty} %r'], ' "dclass"="yv"' + feat)
        out.append(f'declare {ty} @llvm.sqrt.{it}({ty})\n')
        M = f'<{n} x i1>'
        enc, _ = xed(f'vcmp{sfx} ${{c}}, {{o0}}, {{o1}}, {{o2}}', ('y', 'y', 'y'))
        for pred, k in [('oeq', 0), ('olt', 1), ('ole', 2), ('uno', 3), ('une', 4), ('uge', 5), ('ugt', 6), ('ord', 7),
                        ('ueq', 8), ('ult', 9), ('ule', 10), ('one', 12), ('oge', 13), ('ogt', 14)]:
            entry(M, f'VCMP{pred.upper()}{S}.{T}', [f'{ty} "reg"="yv" %n', f'{ty} "reg"="yv" %m'],
                  renamed(enc, {'o0': 'm', 'o1': 'n', 'o2': 'd', 'c': str(k)}),
                  f'vcmp{sfx} ${k}, {{m}}, {{n}}, {{d}}', [f'%r = fcmp {pred} {ty} %n, %m', f'ret {M} %r'], ' "dclass"="yv"' + feat)
        enc, _ = xed(f'vblendv{sfx} {{o0}}, {{o1}}, {{o2}}, {{o3}}', ('y', 'y', 'y', 'y'))
        entry(ty, f'VBLENDV{S}.{T}', [f'{M} "reg"="yv" %c', f'{ty} "reg"="yv" %a', f'{ty} "reg"="yv" %b'],
              renamed(enc, {'o0': 'c', 'o1': 'a', 'o2': 'b', 'o3': 'd'}).replace('o3', 'd'), f'vblendv{sfx} {{c}}, {{a}}, {{b}}, {{d}}',
              [f'%r = select {M} %c, {ty} %a, {ty} %b', f'ret {ty} %r'], ' "dclass"="yv"' + feat)
        # Fused multiply-add: d = n * m + d.
        enc, _ = xed(f'vfmadd231{sfx} {{o0}}, {{o1}}, {{o2}}', (Y, Y, Y))
        for intr in ('fma', 'fmuladd'):
            entry(ty, f'VFMADD231{S}.{intr}.{T}', [f'{ty} "reg"="yv" %n', f'{ty} "reg"="yv" %m', f'{ty} "reg"="yv" "tied" %acc'],
                  renamed(enc, {'o0': 'm', 'o1': 'n', 'o2': 'd'}), f'vfmadd231{sfx} {{m}}, {{n}}, {{d}}',
                  [f'%r = call {ty} @llvm.{intr}.{it}({ty} %n, {ty} %m, {ty} %acc)', f'ret {ty} %r'],
                  ' "dclass"="yv" "feature"="avx2,fma"')
            out.append(f'declare {ty} @llvm.{intr}.{it}({ty}, {ty}, {ty})\n')
    # Conversions.
    for asm, classes, src, dst, ir in [('vcvtdq2ps {o0}, {o1}', (Y, Y), '<8 x i32>', '<8 x float>', 'sitofp'),
                                       ('vcvttps2dq {o0}, {o1}', (Y, Y), '<8 x float>', '<8 x i32>', 'fptosi'),
                                       ('vcvtps2pd {o0}, {o1}', ('xe', Y), '<4 x float>', '<4 x double>', 'fpext'),
                                       ('vcvtdq2pd {o0}, {o1}', ('xe', Y), '<4 x i32>', '<4 x double>', 'sitofp')]:
        enc, _ = xed(asm, classes)
        scls = 'xv' if classes[0] == 'xe' else 'yv'
        entry(dst, f'{asm.split()[0].upper()}.{tag(dst)}', [f'{src} "reg"="{scls}" %n'], renamed(enc, {'o0': 'n', 'o1': 'd'}),
              asm.replace('{o0}', '{n}').replace('{o1}', '{d}'), [f'%r = {ir} {src} %n to {dst}', f'ret {dst} %r'],
              ' "dclass"="yv"' + feat)
    # Halves (legalisation's): the low in place, the high extracted; two
    # joined; and 128-bit lanes widened to 256 (vpmovsx, vpmovzx).
    for full, half, n in [('<32 x i8>', '<16 x i8>', 32), ('<16 x i16>', '<8 x i16>', 16), ('<8 x i32>', '<4 x i32>', 8),
                          ('<4 x i64>', '<2 x i64>', 4), ('<8 x float>', '<4 x float>', 8), ('<4 x double>', '<2 x double>', 4)]:
        h = n // 2
        lo = ', '.join(f'i32 {k}' for k in range(h))
        hi = ', '.join(f'i32 {k}' for k in range(h, n))
        both = ', '.join(f'i32 {k}' for k in range(n))
        T = tag(full)
        entry(half, f'VLOW.{T}', [f'{full} "reg"="yv" %v'], 'rex(0, d, 0, v) | 00001111 | 00101000 | rr(d, v)',
              'movaps {v:x}, {d}', [f'%r = shufflevector {full} %v, {full} poison, <{h} x i32> <{lo}>', f'ret {half} %r'],
              ' "dclass"="xv"' + feat)
        enc, _ = xed('vextracti128 ${c}, {o0}, {o1}', ('y', 'x'))
        entry(half, f'VHIGH.{T}', [f'{full} "reg"="yv" %v'], renamed(enc, {'o0': 'v', 'o1': 'd'}).replace('i1:8=c', '00000001'),
              'vextracti128 $1, {v}, {d}', [f'%r = shufflevector {full} %v, {full} poison, <{h} x i32> <{hi}>', f'ret {half} %r'],
              ' "dclass"="xv"' + feat)
        enc, _ = xed('vinserti128 ${c}, {o0}, {o1}, {o2}', ('x', 'y', 'y'))
        entry(full, f'VJOIN.{T}', [f'{half} "reg"="xv" %a', f'{half} "reg"="xv" %b'],
              renamed(enc, {'o0': 'b', 'o1': 'a', 'o2': 'd'}).replace('i1:8=c', '00000001'), 'vinserti128 $1, {b}, {a:y}, {d}',
              [f'%r = shufflevector {half} %a, {half} %b, <{n} x i32> <{both}>', f'ret {full} %r'], ' "dclass"="yv"' + feat)
    for src, dst, mn in [('<16 x i8>', '<16 x i16>', 'bw'), ('<8 x i16>', '<8 x i32>', 'wd'), ('<4 x i32>', '<4 x i64>', 'dq')]:
        for ext, pre in [('sext', 'vpmovsx'), ('zext', 'vpmovzx')]:
            enc, _ = xed(f'{pre}{mn} {{o0}}, {{o1}}', ('xe', Y))
            entry(dst, f'{pre.upper()}{mn.upper()}', [f'{src} "reg"="xv" %n'], renamed(enc, {'o0': 'n', 'o1': 'd'}),
                  f'{pre}{mn} {{n}}, {{d}}', [f'%r = {ext} {src} %n to {dst}', f'ret {dst} %r'], ' "dclass"="yv"' + feat)
    # Masks' bits, as copies (lanewise.rs).
    for n, w in [(32, 8), (16, 16), (8, 32), (4, 64)]:
        M, I = f'<{n} x i1>', f'<{n} x i{w}>'
        enc, _ = xed('vmovaps {o0}, {o1}', (Y, Y))
        entry(I, f'VBITS.{n}x{w}', [f'{M} "reg"="yv" %m'], renamed(enc, {'o0': 'm', 'o1': 'd'}), 'vmovaps {m}, {d}',
              [f'%r = call {I} @llvm.epitome.mask.bits.v{n}i{w}({M} %m)', f'ret {I} %r'], ' "dclass"="yv"' + feat)
        entry(M, f'VMASK.{n}x{w}', [f'{I} "reg"="yv" %v'], renamed(enc, {'o0': 'v', 'o1': 'd'}), 'vmovaps {v}, {d}',
              [f'%r = call {M} @llvm.epitome.bits.mask.v{n}i{w}({I} %v)', f'ret {M} %r'], ' "dclass"="yv"' + feat)
        out.append(f'declare {I} @llvm.epitome.mask.bits.v{n}i{w}({M})\ndeclare {M} @llvm.epitome.bits.mask.v{n}i{w}({I})\n')
    # Splats, from an xmm register's lane 0 (vpbroadcast, vbroadcastss).
    for ty, et, mn, src, srccls in [('<8 x i32>', 'i32', 'vpbroadcastd', 'i32 "reg"="r32"', 'r32'),
                                    ('<4 x i64>', 'i64', 'vpbroadcastq', 'i64 "reg"="r64"', 'r64'),
                                    ('<16 x i16>', 'i16', 'vpbroadcastw', 'i16 "reg"="r32"', 'r32'),
                                    ('<32 x i8>', 'i8', 'vpbroadcastb', 'i8 "reg"="r32"', 'r32'),
                                    ('<8 x float>', 'float', 'vbroadcastss', 'float "reg"="s"', 's'),
                                    ('<4 x double>', 'double', 'vbroadcastsd', 'double "reg"="d"', 'd')]:
        T = tag(ty)
        enc, _ = xed(f'{mn} {{o0}}, {{o1}}', ('xe', Y))
        entry(ty, f'{mn.upper()}', ['<2 x i64> "reg"="xv" %n'], renamed(enc, {'o0': 'n', 'o1': 'd'}), f'{mn} {{n}}, {{d}}',
              [f'ret {ty} poison'], ' "dclass"="yv" "nomatch"' + feat)
        body = (f'  %i = insertelement {ty} poison, {et} %x, i64 0\n'
                f'  %r = shufflevector {ty} %i, {ty} poison, <{ty[1:].split()[0]} x i32> zeroinitializer\n  ret {ty} %r\n')
        out.append(f'define {ty} @vsplat.{T}({src} %x) "recipe"="vsplat.{T}.expand" "feature"="avx2" {{\n{body}}}\n')
        if srccls == 'r64':
            move = '  %v = call <2 x i64> @MOVQto(i64 %x)\n'
        elif srccls == 'r32':
            move = (f'  %w = zext {et} %x to i32\n' if et != 'i32' else '') + \
                   f'  %v4 = call <4 x i32> @MOVDto(i32 {"%w" if et != "i32" else "%x"})\n  %v = bitcast <4 x i32> %v4 to <2 x i64>\n'
        else:
            move = f'  %v = bitcast {et} %x to <2 x i64>\n'
        out.append(f'define {ty} @vsplat.{T}.expand({et} %x) {{\n{move}  %r = call {ty} @{mn.upper()}(<2 x i64> %v)\n'
                   f'  ret {ty} %r\n}}\n')


def avx512():
    """AVX-512's 512-bit vectors, in zv (zmm0-31), for functions with
    avx512f: what matrix kernels need (memory, copies, zero, broadcasts,
    adds and fused multiply-adds), and AVX512-BF16's conversion and dot
    product (as intrinsics of the code generator's own: pairs of BF16 are
    i32 lanes). EVEX encodings from 02-xed.ll."""
    feat = ' "feature"="avx512f"'
    bf = ' "feature"="avx512f,avx512bf16"'
    Z = 'z'
    # (Memory of every 512-bit vector: the intrinsics take them all.)
    for ty in ('<16 x float>', '<16 x i32>', '<64 x i8>', '<32 x i16>', '<8 x i64>', '<8 x double>',
               '<32 x half>', '<32 x bfloat>'):
        T = ty.strip('<>').replace(' x ', 'x').replace('float', 'f32').replace('double', 'f64')
        role = ty == '<16 x i32>'
        enc, _ = xed('vmovups {m.disp}({m.base}), {o0}', (Z,))
        entry(ty, f'VLDZ.{T}', ['ptr "reg"="r64" %n', 'i64 "imm" %off'], renamed(enc, {'o0': 'd', 'm.disp': 'off', 'm.base': 'n'}),
              'vmovups {off}({n}), {d}', ['%a = getelementptr i8, ptr %n, i64 %off', f'%r = load {ty}, ptr %a', f'ret {ty} %r'],
              ' "dclass"="zv"' + (' "role"="load.zv"' if role else '') + feat)
        enc, _ = xed('vmovups {o0}, {m.disp}({m.base})', (Z,))
        entry('void', f'VSTZ.{T}', [f'{ty} "reg"="zv" %t', 'ptr "reg"="r64" %n', 'i64 "imm" %off'],
              renamed(enc, {'o0': 't', 'm.disp': 'off', 'm.base': 'n'}), 'vmovups {t}, {off}({n})',
              ['%a = getelementptr i8, ptr %n, i64 %off', f'store {ty} %t, ptr %a', 'ret void'],
              (' "role"="store.zv"' if role else '') + feat)
    enc, _ = xed('vmovaps {o0}, {o1}', (Z, Z))
    entry('<16 x i32>', 'VMOVAPSzv', ['<16 x i32> "reg"="zv" %n'], renamed(enc, {'o0': 'n', 'o1': 'd'}), 'vmovaps {n}, {d}',
          ['ret <16 x i32> %n'], ' "copy"="zv" "dclass"="zv"' + feat)
    # The high 256 bits of a constant, inserted.
    enc, _ = xed('vinserti64x4 ${c}, {o0}, {o1}, {o2}', ('ye', Z, Z))
    entry('<8 x i64>', 'VINSERTI64X4hi', ['<8 x i64> "reg"="zv" "tied" %n', '<4 x i64> "reg"="yv" %h'],
          renamed(enc, {'c': '1', 'o0': 'h', 'o1': 'n', 'o2': 'd'}).replace('i1:8=1', '00000001'),
          'vinserti64x4 $1, {h}, {d}, {d}', ['ret <8 x i64> %n'], ' "dclass"="zv" "role"="vec.join.y" "nomatch"' + feat)
    enc, _ = xed('vpxord {o0}, {o1}, {o2}', (Z, Z, Z))
    for ty in ('<16 x float>', '<16 x i32>'):
        T = ty.strip('<>').replace(' x ', 'x').replace('float', 'f32')
        entry(ty, f'VPXORDzero.{T}', [], renamed(enc, {'o0': 'd', 'o1': 'd', 'o2': 'd'}), 'vpxord {d}, {d}, {d}',
              [f'ret {ty} zeroinitializer'], ' "dclass"="zv"' + (' "role"="movi.zero.zv"' if ty == '<16 x i32>' else '') + feat)
    # Broadcasts of an element in memory.
    for ty, et, mn in [('<16 x i32>', 'i32', 'vpbroadcastd'), ('<16 x float>', 'float', 'vbroadcastss')]:
        T = ty.strip('<>').replace(' x ', 'x').replace('float', 'f32')
        enc, _ = xed(f'{mn} {{m.disp}}({{m.base}}), {{o0}}', (Z,))
        entry(ty, f'{mn.upper()}m.{T}', ['ptr "reg"="r64" %n', 'i64 "imm" %off'],
              renamed(enc, {'o0': 'd', 'm.disp': 'off', 'm.base': 'n'}), f'{mn} {{off}}({{n}}), {{d}}',
              ['%a = getelementptr i8, ptr %n, i64 %off', f'%x = load {et}, ptr %a',
               f'%i = insertelement {ty} poison, {et} %x, i64 0',
               f'%r = shufflevector {ty} %i, {ty} poison, <16 x i32> zeroinitializer', f'ret {ty} %r'],
              ' "dclass"="zv"' + feat)
    # Arithmetic on floats: AT&T's src2, src1, dst.
    for ir, mn in [('fadd', 'vaddps'), ('fsub', 'vsubps'), ('fmul', 'vmulps')]:
        enc, _ = xed(f'{mn} {{o0}}, {{o1}}, {{o2}}', (Z, Z, Z))
        entry('<16 x float>', f'{mn.upper()}.16xf32', ['<16 x float> "reg"="zv" %n', '<16 x float> "reg"="zv" %m'],
              renamed(enc, {'o0': 'm', 'o1': 'n', 'o2': 'd'}), f'{mn} {{m}}, {{n}}, {{d}}',
              [f'%r = {ir} <16 x float> %n, %m', 'ret <16 x float> %r'], ' "dclass"="zv"' + feat)
    enc, _ = xed('vfmadd231ps {o0}, {o1}, {o2}', (Z, Z, Z))
    for intr in ('fma', 'fmuladd'):
        entry('<16 x float>', f'VFMADD231PS.{intr}.16xf32',
              ['<16 x float> "reg"="zv" %n', '<16 x float> "reg"="zv" %m', '<16 x float> "reg"="zv" "tied" %acc'],
              renamed(enc, {'o0': 'm', 'o1': 'n', 'o2': 'd'}), 'vfmadd231ps {m}, {n}, {d}',
              [f'%r = call <16 x float> @llvm.{intr}.v16f32(<16 x float> %n, <16 x float> %m, <16 x float> %acc)',
               'ret <16 x float> %r'], ' "dclass"="zv"' + feat)
        out.append(f'declare <16 x float> @llvm.{intr}.v16f32(<16 x float>, <16 x float>, <16 x float>)\n')
    # BF16: sixteen floats rounded (to nearest, even) to sixteen BF16, a
    # ymm's worth; and acc + the products of pairs of BF16, each lane's two
    # added in.
    enc, _ = xed('vcvtneps2bf16 {o0}, {o1}', (Z, 'ye'))
    entry('<8 x i32>', 'VCVTNEPS2BF16.16xf32', ['<16 x float> "reg"="zv" %n'], renamed(enc, {'o0': 'n', 'o1': 'd'}),
          'vcvtneps2bf16 {n}, {d}', ['%r = call <8 x i32> @llvm.epitome.x86.cvtneps2bf16.512(<16 x float> %n)', 'ret <8 x i32> %r'],
          ' "dclass"="yv"' + bf)
    out.append('declare <8 x i32> @llvm.epitome.x86.cvtneps2bf16.512(<16 x float>)\n')
    enc, _ = xed('vdpbf16ps {o0}, {o1}, {o2}', (Z, Z, Z))
    entry('<16 x float>', 'VDPBF16PS.16xf32',
          ['<16 x float> "reg"="zv" "tied" %acc', '<16 x i32> "reg"="zv" %n', '<16 x i32> "reg"="zv" %m'],
          renamed(enc, {'o0': 'm', 'o1': 'n', 'o2': 'd'}), 'vdpbf16ps {m}, {n}, {d}',
          ['%r = call <16 x float> @llvm.epitome.x86.dpbf16ps.512(<16 x float> %acc, <16 x i32> %n, <16 x i32> %m)',
           'ret <16 x float> %r'], ' "dclass"="zv"' + bf)
    out.append('declare <16 x float> @llvm.epitome.x86.dpbf16ps.512(<16 x float>, <16 x i32>, <16 x i32>)\n')


def avx512_int():
    """AVX-512's integer lanes in zv, and the masks of 512-bit vectors in k
    registers (kw: k1-k7, as an instruction's mask cannot be k0; isel.rs
    keeps a mask there when every comparison and select it meets is of
    512-bit vectors). Bytes and words need avx512bw; leading zeros,
    avx512cd (and avx512vl at 128 and 256 bits)."""
    Z = 'z'
    F = ' "feature"="avx512f"'
    BW = ' "feature"="avx512f,avx512bw"'
    INT = [('<64 x i8>', 'b', 8, 64), ('<32 x i16>', 'w', 16, 32), ('<16 x i32>', 'd', 32, 16), ('<8 x i64>', 'q', 64, 8)]
    tag = lambda ty: ty.strip('<>').replace(' x ', 'x').replace('float', 'f32').replace('double', 'f64')
    imm = lambda enc, k: enc.replace(f'i1:8={k}', bits(k))

    def three(ty, name, asm, body, feat, rty=None, dcls='zv'):
        # AT&T: src2, src1, dst; o0 src2 (m), o1 src1 (n), o2 dst.
        enc, _ = xed(asm, (Z, Z, Z))
        entry(rty or ty, name, [f'{ty} "reg"="zv" %n', f'{ty} "reg"="zv" %m'], renamed(enc, {'o0': 'm', 'o1': 'n', 'o2': 'd'}),
              asm.replace('{o0}', '{m}').replace('{o1}', '{n}').replace('{o2}', '{d}'), body, f' "dclass"="{dcls}"' + feat)

    for ty, sfx, eb, n in INT:
        T = tag(ty)
        S = sfx.upper()
        feat = BW if eb <= 16 else F
        M = f'<{n} x i1>'
        # (Whole-register logic is of doublewords or quadwords, any lanes.)
        lg = 'q' if eb == 64 else 'd'
        ops = [('add', 'vpadd'), ('sub', 'vpsub')]
        for ir, mn in ops:
            three(ty, f'{mn.upper()}{S}z.{T}', f'{mn}{sfx} {{o0}}, {{o1}}, {{o2}}', [f'%r = {ir} {ty} %n, %m', f'ret {ty} %r'], feat)
        if eb == 16:
            three(ty, f'VPMULLWz.{T}', 'vpmullw {o0}, {o1}, {o2}', [f'%r = mul {ty} %n, %m', f'ret {ty} %r'], BW)
        if eb == 32:
            three(ty, f'VPMULLDz.{T}', 'vpmulld {o0}, {o1}, {o2}', [f'%r = mul {ty} %n, %m', f'ret {ty} %r'], F)
        if eb == 64:
            three(ty, f'VPMULLQz.{T}', 'vpmullq {o0}, {o1}, {o2}', [f'%r = mul {ty} %n, %m', f'ret {ty} %r'],
                  ' "feature"="avx512f,avx512dq"')
        for ir, mn in [('and', 'vpand'), ('or', 'vpor'), ('xor', 'vpxor')]:
            three(ty, f'{mn.upper()}{lg.upper()}z.{T}', f'{mn}{lg} {{o0}}, {{o1}}, {{o2}}', [f'%r = {ir} {ty} %n, %m', f'ret {ty} %r'], F)
        # n & ~m is vpandn's (~src1) & src2.
        three(ty, f'VPANDN{lg.upper()}z.{T}', f'vpandn{lg} {{o0}}, {{o1}}, {{o2}}',
              [f'%x = xor {ty} %n, splat (i{eb} -1)', f'%r = and {ty} %x, %m', f'ret {ty} %r'], F)
        # Comparisons into a mask register: vpcmp's predicates (0 eq, 1 lt,
        # 2 le, 4 ne, 5 nlt, 6 nle), signed and unsigned.
        for pred, mn, k in [('eq', 'vpcmp', 0), ('ne', 'vpcmp', 4), ('slt', 'vpcmp', 1), ('sle', 'vpcmp', 2),
                            ('sge', 'vpcmp', 5), ('sgt', 'vpcmp', 6), ('ult', 'vpcmpu', 1), ('ule', 'vpcmpu', 2),
                            ('uge', 'vpcmpu', 5), ('ugt', 'vpcmpu', 6)]:
            enc, _ = xed(f'{mn}{sfx} ${{c}}, {{o0}}, {{o1}}, {{o2}}', (Z, Z, 'k'))
            entry(M, f'{mn.upper()}{S}.{pred}.{T}', [f'{ty} "reg"="zv" %n', f'{ty} "reg"="zv" %m'],
                  imm(renamed(enc, {'o0': 'm', 'o1': 'n', 'o2': 'd', 'c': str(k)}), k), f'{mn}{sfx} ${k}, {{m}}, {{n}}, {{d}}',
                  [f'%r = icmp {pred} {ty} %n, %m', f'ret {M} %r'], ' "dclass"="kw"' + feat)
        # Selects by a mask: where its bit is set, the first.
        enc, _ = xed(f'vpblendm{sfx} {{o0}}, {{o1}}, {{o2}} {{lb}}{{o3}}{{rb}}', (Z, Z, Z, 'k'))
        entry(ty, f'VPBLENDM{S}.{T}', [f'{M} "reg"="kw" %k', f'{ty} "reg"="zv" %a', f'{ty} "reg"="zv" %b'],
              renamed(enc, {'o0': 'a', 'o1': 'b', 'o2': 'd', 'o3': 'k'}), f'vpblendm{sfx} {{a}}, {{b}}, {{d}} {{lb}}{{k}}{{rb}}',
              [f'%r = select {M} %k, {ty} %a, {ty} %b', f'ret {ty} %r'], ' "dclass"="zv"' + feat)
        # Shifts by an immediate (no bytes'), and by a vector.
        for ir, mn in [('shl', 'vpsll'), ('lshr', 'vpsrl'), ('ashr', 'vpsra')]:
            if eb == 8:
                continue
            enc, _ = xed(f'{mn}{sfx} ${{c}}, {{o0}}, {{o1}}', (Z, Z))
            entry(ty, f'{mn.upper()}{S}iz.{T}', [f'{ty} "reg"="zv" %n', f'{ty} "imm" %sh'],
                  renamed(enc, {'c': 'sh', 'o0': 'n', 'o1': 'd'}), f'{mn}{sfx} ${{sh}}, {{n}}, {{d}}',
                  [f'%r = {ir} {ty} %n, %sh', f'ret {ty} %r'], ' "dclass"="zv"' + feat)
            three(ty, f'{mn.upper()}V{S}z.{T}', f'{mn}v{sfx} {{o0}}, {{o1}}, {{o2}}', [f'%r = {ir} {ty} %n, %m', f'ret {ty} %r'], feat)
        for ir, mn in [('smin', 'vpmins'), ('smax', 'vpmaxs'), ('umin', 'vpminu'), ('umax', 'vpmaxu')]:
            three(ty, f'{mn.upper()}{S}z.{T}', f'{mn}{sfx} {{o0}}, {{o1}}, {{o2}}',
                  [f'%r = call {ty} @llvm.{ir}.v{n}i{eb}({ty} %n, {ty} %m)', f'ret {ty} %r'], feat)
            out.append(f'declare {ty} @llvm.{ir}.v{n}i{eb}({ty}, {ty})\n')
        enc, _ = xed(f'vpabs{sfx} {{o0}}, {{o1}}', (Z, Z))
        for flag in ('false', 'true'):
            entry(ty, f'VPABS{S}z.{flag}.{T}', [f'{ty} "reg"="zv" %n'], renamed(enc, {'o0': 'n', 'o1': 'd'}), f'vpabs{sfx} {{n}}, {{d}}',
                  [f'%r = call {ty} @llvm.abs.v{n}i{eb}({ty} %n, i1 {flag})', f'ret {ty} %r'], ' "dclass"="zv"' + feat)
        out.append(f'declare {ty} @llvm.abs.v{n}i{eb}({ty}, i1)\n')
        # A mask's lanes as all ones or zeros: doublewords and quadwords
        # by a ternary function of all ones, its other lanes zeroed (which
        # needs no avx512dq), bytes and words by vpmovm2. (Its register is
        # read, but any value will do.)
        if eb >= 32:
            enc, _ = xed(f'vpternlog{sfx} ${{c}}, {{o0}}, {{o1}}, {{o2}} {{lb}}{{o3}}{{rb}} {{lb}}z{{rb}}', (Z, Z, Z, 'k'))
            enc = imm(renamed(enc, {'o0': 'd', 'o1': 'd', 'o2': 'd', 'o3': 'k', 'c': '255'}), 255)
            asm = f'vpternlog{sfx} $255, {{d}}, {{d}}, {{d}} {{lb}}{{k}}{{rb}} {{lb}}z{{rb}}'
        else:
            enc, _ = xed(f'vpmovm2{sfx} {{o0}}, {{o1}}', ('k', Z))
            enc = renamed(enc, {'o0': 'k', 'o1': 'd'})
            asm = f'vpmovm2{sfx} {{k}}, {{d}}'
        entry(ty, f'VPMOVM2{S}.{T}', [f'{M} "reg"="kw" %k'], enc, asm, [f'%r = sext {M} %k to {ty}', f'ret {ty} %r'],
              ' "dclass"="zv"' + feat)
        entry(ty, f'VMASKBITS.{T}', [f'{M} "reg"="kw" %k'], enc, asm,
              [f'%r = call {ty} @llvm.epitome.mask.bits.v{n}i{eb}({M} %k)', f'ret {ty} %r'], ' "dclass"="zv"' + feat)
        # And as 0 or 1: their absolute values.
        out.append(f'define {ty} @zextk.{T}({M} "reg"="kw" %k) "recipe"="zextk.{T}.expand"{feat} {{\n'
                   f'  %r = zext {M} %k to {ty}\n  ret {ty} %r\n}}\n')
        out.append(f'define {ty} @zextk.{T}.expand({M} %k) {{\n  %s = call {ty} @VPMOVM2{S}.{T}({M} %k)\n'
                   f'  %r = call {ty} @VPABS{S}z.false.{T}({ty} %s)\n  ret {ty} %r\n}}\n')
        # Lanes as a mask: those not zero.
        enc, _ = xed(f'vptestm{sfx} {{o0}}, {{o1}}, {{o2}}', (Z, Z, 'k'))
        entry(M, f'VBITSMASK.{T}', [f'{ty} "reg"="zv" %n'], renamed(enc, {'o0': 'n', 'o1': 'n', 'o2': 'd'}),
              f'vptestm{sfx} {{n}}, {{n}}, {{d}}', [f'%r = call {M} @llvm.epitome.bits.mask.v{n}i{eb}({ty} %n)', f'ret {M} %r'],
              ' "dclass"="kw"' + feat)
        out.append(f'declare {ty} @llvm.epitome.mask.bits.v{n}i{eb}({M})\ndeclare {M} @llvm.epitome.bits.mask.v{n}i{eb}({ty})\n')
        # Splats of a general register's value.
        et = f'i{eb}'
        src = 'r64' if eb == 64 else 'r32'
        enc, _ = xed(f'vpbroadcast{sfx} {{o0}}, {{o1}}', (src, Z))
        entry(ty, f'VPBROADCAST{S}r.{T}', [f'{et} "reg"="{src}" %x'], renamed(enc, {'o0': 'x', 'o1': 'd'}),
              f'vpbroadcast{sfx} {{x}}, {{d}}',
              [f'%i = insertelement {ty} poison, {et} %x, i64 0',
               f'%r = shufflevector {ty} %i, {ty} poison, <{n} x i32> zeroinitializer', f'ret {ty} %r'],
              f' "dclass"="zv" "role"="dup.zv.{eb}"' + feat)
    # Floats' comparisons into a mask register (vcmp's predicates, as
    # AVX's), and selects by one.
    for ty, sfx, n in [('<16 x float>', 'ps', 16), ('<8 x double>', 'pd', 8)]:
        T = tag(ty)
        M = f'<{n} x i1>'
        enc, _ = xed(f'vcmp{sfx} ${{c}}, {{o0}}, {{o1}}, {{o2}}', (Z, Z, 'k'))
        for pred, k in [('oeq', 0), ('olt', 1), ('ole', 2), ('uno', 3), ('une', 4), ('uge', 5), ('ugt', 6), ('ord', 7),
                        ('ueq', 8), ('ult', 9), ('ule', 10), ('one', 12), ('oge', 13), ('ogt', 14)]:
            entry(M, f'VCMP{pred.upper()}{sfx.upper()}z.{T}', [f'{ty} "reg"="zv" %n', f'{ty} "reg"="zv" %m'],
                  imm(renamed(enc, {'o0': 'm', 'o1': 'n', 'o2': 'd', 'c': str(k)}), k),
                  f'vcmp{sfx} ${k}, {{m}}, {{n}}, {{d}}', [f'%r = fcmp {pred} {ty} %n, %m', f'ret {M} %r'], ' "dclass"="kw"' + F)
        enc, _ = xed(f'vblendm{sfx} {{o0}}, {{o1}}, {{o2}} {{lb}}{{o3}}{{rb}}', (Z, Z, Z, 'k'))
        entry(ty, f'VBLENDM{sfx.upper()}.{T}', [f'{M} "reg"="kw" %k', f'{ty} "reg"="zv" %a', f'{ty} "reg"="zv" %b'],
              renamed(enc, {'o0': 'a', 'o1': 'b', 'o2': 'd', 'o3': 'k'}), f'vblendm{sfx} {{a}}, {{b}}, {{d}} {{lb}}{{k}}{{rb}}',
              [f'%r = select {M} %k, {ty} %a, {ty} %b', f'ret {ty} %r'], ' "dclass"="zv"' + F)
    # The masks themselves: as integers (kmovw for 2 to 16 lanes, which
    # avx512f has; kmovd and kmovq with avx512bw), in memory, and their
    # logic.
    for n, mn, ity, rcls, feat in [(2, 'kmovw', 'i2', 'r32', F), (4, 'kmovw', 'i4', 'r32', F), (8, 'kmovw', 'i8', 'r32', F), (16, 'kmovw', 'i16', 'r32', F),
                                   (32, 'kmovd', 'i32', 'r32', BW), (64, 'kmovq', 'i64', 'r64', BW)]:
        M = f'<{n} x i1>'
        enc, _ = xed(f'{mn} {{o0}}, {{o1}}', (rcls, 'k'))
        # (Integers of 2 and 4 bits are held in 32, as prepare.rs's
        # promote_odd leaves them: from those.)
        src = 'i32' if n < 8 else ity
        entry(M, f'{mn.upper()}kr.{n}', [f'{src} "reg"="{rcls}" %n'], renamed(enc, {'o0': 'n', 'o1': 'd'}), f'{mn} {{n}}, {{d}}',
              [f'%r = bitcast {src} %n to {M}', f'ret {M} %r'], ' "dclass"="kw"' + feat)
        enc, _ = xed(f'{mn} {{o0}}, {{o1}}', ('k', rcls))
        entry(ity, f'{mn.upper()}rk.{n}', [f'{M} "reg"="kw" %n'], renamed(enc, {'o0': 'n', 'o1': 'd'}), f'{mn} {{n}}, {{d}}',
              [f'%r = bitcast {M} %n to {ity}', f'ret {ity} %r'],
              f' "dclass"="{rcls}"' + (' "zext"="16"' if n == 16 else '') + feat)
        sfx = {'kmovw': 'w', 'kmovd': 'd', 'kmovq': 'q'}[mn]
        for ir, kmn in [('and', 'kand'), ('or', 'kor'), ('xor', 'kxor')]:
            enc, _ = xed(f'{kmn}{sfx} {{o0}}, {{o1}}, {{o2}}', ('k', 'k', 'k'))
            entry(M, f'{kmn.upper()}{sfx.upper()}.{n}', [f'{M} "reg"="kw" %n', f'{M} "reg"="kw" %m'],
                  renamed(enc, {'o0': 'm', 'o1': 'n', 'o2': 'd'}), f'{kmn}{sfx} {{m}}, {{n}}, {{d}}',
                  [f'%r = {ir} {M} %n, %m', f'ret {M} %r'], ' "dclass"="kw"' + feat)
        enc, _ = xed(f'kandn{sfx} {{o0}}, {{o1}}, {{o2}}', ('k', 'k', 'k'))
        entry(M, f'KANDN{sfx.upper()}.{n}', [f'{M} "reg"="kw" %n', f'{M} "reg"="kw" %m'],
              renamed(enc, {'o0': 'm', 'o1': 'n', 'o2': 'd'}), f'kandn{sfx} {{m}}, {{n}}, {{d}}',
              [f'%x = xor {M} %n, splat (i1 true)', f'%r = and {M} %x, %m', f'ret {M} %r'], ' "dclass"="kw"' + feat)
        enc, _ = xed(f'knot{sfx} {{o0}}, {{o1}}', ('k', 'k'))
        entry(M, f'KNOT{sfx.upper()}.{n}', [f'{M} "reg"="kw" %n'], renamed(enc, {'o0': 'n', 'o1': 'd'}), f'knot{sfx} {{n}}, {{d}}',
              [f'%r = xor {M} %n, splat (i1 true)', f'ret {M} %r'], ' "dclass"="kw"' + feat)
    # Leading zeros (avx512cd), at 512 bits and, with avx512vl, 128 and 256;
    # and the bits set (avx512vpopcntdq).
    for eb, sfx in [(32, 'd'), (64, 'q')]:
        for cls, sel, bits_ in [(Z, 'zv', 512), ('ye', 'yv', 256), ('xe', 'xv', 128)]:
            n = bits_ // eb
            ty = f'<{n} x i{eb}>'
            T = tag(ty)
            vl = '' if bits_ == 512 else ',avx512vl'
            enc, _ = xed(f'vplzcnt{sfx} {{o0}}, {{o1}}', (cls, cls))
            entry(ty, f'VPLZCNT{sfx.upper()}.{T}', [f'{ty} "reg"="{sel}" %n'], renamed(enc, {'o0': 'n', 'o1': 'd'}),
                  f'vplzcnt{sfx} {{n}}, {{d}}', [f'%r = call {ty} @llvm.ctlz.v{n}i{eb}({ty} %n, i1 false)', f'ret {ty} %r'],
                  f' "dclass"="{sel}" "feature"="avx512f,avx512cd{vl}"')
            out.append(f'declare {ty} @llvm.ctlz.v{n}i{eb}({ty}, i1)\n')
            enc, _ = xed(f'vpopcnt{sfx} {{o0}}, {{o1}}', (cls, cls))
            entry(ty, f'VPOPCNT{sfx.upper()}.{T}', [f'{ty} "reg"="{sel}" %n'], renamed(enc, {'o0': 'n', 'o1': 'd'}),
                  f'vpopcnt{sfx} {{n}}, {{d}}', [f'%r = call {ty} @llvm.ctpop.v{n}i{eb}({ty} %n)', f'ret {ty} %r'],
                  f' "dclass"="{sel}" "feature"="avx512f,avx512vpopcntdq{vl}"')
            out.append(f'declare {ty} @llvm.ctpop.v{n}i{eb}({ty})\n')


def avx512vl_masks():
    """With avx512vl, the masks of 128- and 256-bit vectors in k registers
    too (kw; isel.rs keeps a mask there when every comparison and select it
    meets has an entry making or taking it there, at any of these widths):
    comparisons into them, selects by them, and their lanes as all ones or
    zeros and back. Bytes and words need avx512bw as well."""
    VL = ' "feature"="avx512f,avx512vl"'
    VLBW = ' "feature"="avx512f,avx512vl,avx512bw"'
    tag = lambda ty: ty.strip('<>').replace(' x ', 'x').replace('float', 'f32').replace('double', 'f64')
    imm = lambda enc, k: enc.replace(f'i1:8={k}', bits(k))
    for bits_, cls, sel in [(256, 'ye', 'yv'), (128, 'xe', 'xv')]:
        for sfx, eb in [('b', 8), ('w', 16), ('d', 32), ('q', 64)]:
            n = bits_ // eb
            ty = f'<{n} x i{eb}>'
            T = tag(ty)
            S = sfx.upper()
            M = f'<{n} x i1>'
            feat = VLBW if eb <= 16 else VL
            for pred, mn, k in [('eq', 'vpcmp', 0), ('ne', 'vpcmp', 4), ('slt', 'vpcmp', 1), ('sle', 'vpcmp', 2),
                                ('sge', 'vpcmp', 5), ('sgt', 'vpcmp', 6), ('ult', 'vpcmpu', 1), ('ule', 'vpcmpu', 2),
                                ('uge', 'vpcmpu', 5), ('ugt', 'vpcmpu', 6)]:
                enc, _ = xed(f'{mn}{sfx} ${{c}}, {{o0}}, {{o1}}, {{o2}}', (cls, cls, 'k'))
                entry(M, f'{mn.upper()}{S}.{pred}.{T}', [f'{ty} "reg"="{sel}" %n', f'{ty} "reg"="{sel}" %m'],
                      imm(renamed(enc, {'o0': 'm', 'o1': 'n', 'o2': 'd', 'c': str(k)}), k),
                      f'{mn}{sfx} ${k}, {{m}}, {{n}}, {{d}}', [f'%r = icmp {pred} {ty} %n, %m', f'ret {M} %r'],
                      ' "dclass"="kw"' + feat)
            enc, _ = xed(f'vpblendm{sfx} {{o0}}, {{o1}}, {{o2}} {{lb}}{{o3}}{{rb}}', (cls, cls, cls, 'k'))
            entry(ty, f'VPBLENDM{S}.{T}', [f'{M} "reg"="kw" %k', f'{ty} "reg"="{sel}" %a', f'{ty} "reg"="{sel}" %b'],
                  renamed(enc, {'o0': 'a', 'o1': 'b', 'o2': 'd', 'o3': 'k'}),
                  f'vpblendm{sfx} {{a}}, {{b}}, {{d}} {{lb}}{{k}}{{rb}}',
                  [f'%r = select {M} %k, {ty} %a, {ty} %b', f'ret {ty} %r'], f' "dclass"="{sel}"' + feat)
            # Lanes of all ones or zeros, as at 512 bits; 0 or 1 as their
            # absolute values (vpabsq is AVX-512's).
            if eb >= 32:
                enc, _ = xed(f'vpternlog{sfx} ${{c}}, {{o0}}, {{o1}}, {{o2}} {{lb}}{{o3}}{{rb}} {{lb}}z{{rb}}',
                             (cls, cls, cls, 'k'))
                enc = imm(renamed(enc, {'o0': 'd', 'o1': 'd', 'o2': 'd', 'o3': 'k', 'c': '255'}), 255)
                asm = f'vpternlog{sfx} $255, {{d}}, {{d}}, {{d}} {{lb}}{{k}}{{rb}} {{lb}}z{{rb}}'
            else:
                enc, _ = xed(f'vpmovm2{sfx} {{o0}}, {{o1}}', ('k', cls))
                enc = renamed(enc, {'o0': 'k', 'o1': 'd'})
                asm = f'vpmovm2{sfx} {{k}}, {{d}}'
            entry(ty, f'VPMOVM2{S}.{T}', [f'{M} "reg"="kw" %k'], enc, asm, [f'%r = sext {M} %k to {ty}', f'ret {ty} %r'],
                  f' "dclass"="{sel}"' + feat)
            entry(ty, f'VMASKBITS.{T}', [f'{M} "reg"="kw" %k'], enc, asm,
                  [f'%r = call {ty} @llvm.epitome.mask.bits.v{n}i{eb}({M} %k)', f'ret {ty} %r'], f' "dclass"="{sel}"' + feat)
            enc, _ = xed(f'vpabs{sfx} {{o0}}, {{o1}}', (cls, cls))
            entry(ty, f'VPABS{S}k.{T}', [f'{ty} "reg"="{sel}" %n'], renamed(enc, {'o0': 'n', 'o1': 'd'}),
                  f'vpabs{sfx} {{n}}, {{d}}', [f'ret {ty} %n'], f' "dclass"="{sel}" "nomatch"' + feat)
            out.append(f'define {ty} @zextk.{T}({M} "reg"="kw" %k) "recipe"="zextk.{T}.expand"{feat} {{\n'
                       f'  %r = zext {M} %k to {ty}\n  ret {ty} %r\n}}\n')
            out.append(f'define {ty} @zextk.{T}.expand({M} %k) {{\n  %s = call {ty} @VPMOVM2{S}.{T}({M} %k)\n'
                       f'  %r = call {ty} @VPABS{S}k.{T}({ty} %s)\n  ret {ty} %r\n}}\n')
            enc, _ = xed(f'vptestm{sfx} {{o0}}, {{o1}}, {{o2}}', (cls, cls, 'k'))
            entry(M, f'VBITSMASK.{T}', [f'{ty} "reg"="{sel}" %n'], renamed(enc, {'o0': 'n', 'o1': 'n', 'o2': 'd'}),
                  f'vptestm{sfx} {{n}}, {{n}}, {{d}}',
                  [f'%r = call {M} @llvm.epitome.bits.mask.v{n}i{eb}({ty} %n)', f'ret {M} %r'], ' "dclass"="kw"' + feat)
        for fty, sfx, n in [(f'<{bits_ // 32} x float>', 'ps', bits_ // 32), (f'<{bits_ // 64} x double>', 'pd', bits_ // 64)]:
            T = tag(fty)
            M = f'<{n} x i1>'
            enc, _ = xed(f'vcmp{sfx} ${{c}}, {{o0}}, {{o1}}, {{o2}}', (cls, cls, 'k'))
            for pred, k in [('oeq', 0), ('olt', 1), ('ole', 2), ('uno', 3), ('une', 4), ('uge', 5), ('ugt', 6), ('ord', 7),
                            ('ueq', 8), ('ult', 9), ('ule', 10), ('one', 12), ('oge', 13), ('ogt', 14)]:
                entry(M, f'VCMP{pred.upper()}{sfx.upper()}k.{T}', [f'{fty} "reg"="{sel}" %n', f'{fty} "reg"="{sel}" %m'],
                      imm(renamed(enc, {'o0': 'm', 'o1': 'n', 'o2': 'd', 'c': str(k)}), k),
                      f'vcmp{sfx} ${k}, {{m}}, {{n}}, {{d}}', [f'%r = fcmp {pred} {fty} %n, %m', f'ret {M} %r'],
                      ' "dclass"="kw"' + VL)
            enc, _ = xed(f'vblendm{sfx} {{o0}}, {{o1}}, {{o2}} {{lb}}{{o3}}{{rb}}', (cls, cls, cls, 'k'))
            entry(fty, f'VBLENDM{sfx.upper()}.{T}', [f'{M} "reg"="kw" %k', f'{fty} "reg"="{sel}" %a', f'{fty} "reg"="{sel}" %b'],
                  renamed(enc, {'o0': 'a', 'o1': 'b', 'o2': 'd', 'o3': 'k'}),
                  f'vblendm{sfx} {{a}}, {{b}}, {{d}} {{lb}}{{k}}{{rb}}',
                  [f'%r = select {M} %k, {fty} %a, {fty} %b', f'ret {fty} %r'], f' "dclass"="{sel}"' + VL)


def reductions():
    """Sums across lanes: halves added until one lane is left (pshufd's
    high lanes brought down), as SSE2 and AVX2 have no across-lanes add."""
    def recipe(name, ret, vty, cls, body, feature=''):
        n, el = vty[1:].split(' x ')
        el = el[:-1]
        out.append(f'declare {ret} @llvm.vector.reduce.add.v{n}{el}({vty})\n')
        out.append(f'define {ret} @{name}({vty} "reg"="{cls}" %v) "recipe"="{name}.expand"{feature} {{\n'
                   f'  %r = call {ret} @llvm.vector.reduce.add.v{n}{el}({vty} %v)\n  ret {ret} %r\n}}\n')
        out.append(f'define {ret} @{name}.expand({vty} %v) {{\n' + ''.join(f'  {l}\n' for l in body) + '}\n')
    four = lambda v: ['%s1 = call <4 x i32> @PSHUFD(<4 x i32> ' + v + ', i64 14)',
                      '%a1 = call <4 x i32> @PADDD.4xi32(<4 x i32> ' + v + ', <4 x i32> %s1)',
                      '%s2 = call <4 x i32> @PSHUFD(<4 x i32> %a1, i64 1)',
                      '%a2 = call <4 x i32> @PADDD.4xi32(<4 x i32> %a1, <4 x i32> %s2)',
                      '%r = call i32 @MOVDlane0.4xi32(<4 x i32> %a2)', 'ret i32 %r']
    two = lambda v: ['%s1 = call <2 x i64> @PUNPCKHQDQself(<2 x i64> ' + v + ')',
                     '%a1 = call <2 x i64> @PADDQ.2xi64(<2 x i64> ' + v + ', <2 x i64> %s1)',
                     '%r = call i64 @MOVQlane0(<2 x i64> %a1)', 'ret i64 %r']
    recipe('reduce.add.4xi32', 'i32', '<4 x i32>', 'xv', four('%v'))
    recipe('reduce.add.2xi64', 'i64', '<2 x i64>', 'xv', two('%v'))
    recipe('reduce.add.8xi32', 'i32', '<8 x i32>', 'yv',
           ['%l = call <4 x i32> @VLOW.8xi32(<8 x i32> %v)', '%h = call <4 x i32> @VHIGH.8xi32(<8 x i32> %v)',
            '%s = call <4 x i32> @PADDD.4xi32(<4 x i32> %l, <4 x i32> %h)'] + four('%s'), ' "feature"="avx2"')
    recipe('reduce.add.4xi64', 'i64', '<4 x i64>', 'yv',
           ['%l = call <2 x i64> @VLOW.4xi64(<4 x i64> %v)', '%h = call <2 x i64> @VHIGH.4xi64(<4 x i64> %v)',
            '%s = call <2 x i64> @PADDQ.2xi64(<2 x i64> %l, <2 x i64> %h)'] + two('%s'), ' "feature"="avx2"')


def mul64():
    """64-bit lanes multiplied (SSE2 and AVX2 have 32 by 32 to 64 only):
    lo(a)lo(b) + ((hi(a)lo(b) + lo(a)hi(b)) << 32)."""
    def body(pfx, ty, mul, srl, sll, add):
        return [f'%h = call {ty} @{srl}({ty} %a, i64 32)', f'%g = call {ty} @{srl}({ty} %b, i64 32)',
                f'%p = call {ty} @{mul}({ty} %a, {ty} %b)', f'%q = call {ty} @{mul}({ty} %h, {ty} %b)',
                f'%u = call {ty} @{mul}({ty} %a, {ty} %g)', f'%s = call {ty} @{add}({ty} %q, {ty} %u)',
                f'%t = call {ty} @{sll}({ty} %s, i64 32)', f'%r = call {ty} @{add}({ty} %p, {ty} %t)', f'ret {ty} %r']
    out.append('define <2 x i64> @mul.2xi64(<2 x i64> "reg"="xv" %a, <2 x i64> "reg"="xv" %b) "recipe"="mul.2xi64.expand" {\n'
               '  %r = mul <2 x i64> %a, %b\n  ret <2 x i64> %r\n}\n')
    out.append('define <2 x i64> @mul.2xi64.expand(<2 x i64> %a, <2 x i64> %b) {\n' +
               ''.join(f'  {l}\n' for l in body('', '<2 x i64>', 'PMULUDQ', 'PSRLQi.2xi64', 'PSLLQi.2xi64', 'PADDQ.2xi64')) + '}\n')
    enc, _ = xed('vpmuludq {o0}, {o1}, {o2}', ('ye', 'ye', 'ye'))
    entry('<4 x i64>', 'VPMULUDQ', ['<4 x i64> "reg"="yv" %n', '<4 x i64> "reg"="yv" %m'], renamed(enc, {'o0': 'm', 'o1': 'n', 'o2': 'd'}),
          'vpmuludq {m}, {n}, {d}', ['ret <4 x i64> %n'], ' "dclass"="yv" "nomatch" "feature"="avx2"')
    out.append('define <4 x i64> @mul.4xi64(<4 x i64> "reg"="yv" %a, <4 x i64> "reg"="yv" %b) "recipe"="mul.4xi64.expand" "feature"="avx2" {\n'
               '  %r = mul <4 x i64> %a, %b\n  ret <4 x i64> %r\n}\n')
    out.append('define <4 x i64> @mul.4xi64.expand(<4 x i64> %a, <4 x i64> %b) {\n' +
               ''.join(f'  {l}\n' for l in body('', '<4 x i64>', 'VPMULUDQ', 'VPSRLQi.4xi64', 'VPSLLQi.4xi64', 'VPADDQ.4xi64')) + '}\n')


def more_compares():
    """AVX2's other comparisons (as SSE2's: swapped, inverted, unsigned by
    the greatest or with signs flipped), and absolute values (AVX2's
    vpabs; SSE2's of the sign spread: (x ^ s) - s)."""
    tag = lambda ty: ty.strip('<>').replace(' x ', 'x')
    feat = ' "feature"="avx2"'
    enc, _ = xed('vpcmpeqd {o0}, {o1}, {o2}', ('y', 'y', 'y'))
    entry('<4 x i64>', 'VALLONES', [], renamed(enc, {'o0': 'd', 'o1': 'd', 'o2': 'd'}), 'vpcmpeqd {d}, {d}, {d}',
          ['ret <4 x i64> splat (i64 -1)'], ' "dclass"="yv" "nomatch"' + feat)
    def recipe(name, ret, params, body, expand):
        ps = ', '.join(f'{t} "reg"="yv" %{n}' for t, n in params)
        plain = ', '.join(f'{t} %{n}' for t, n in params)
        out.append(f'define {ret} @{name}({ps}) "recipe"="{name}.expand"{feat} {{\n' + ''.join(f'  {l}\n' for l in body) + '}\n')
        out.append(f'define {ret} @{name}.expand({plain}) {{\n' + ''.join(f'  {l}\n' for l in expand) + '}\n')
    for ty, sfx, eb, n in [('<32 x i8>', 'b', 8, 32), ('<16 x i16>', 'w', 16, 16), ('<8 x i32>', 'd', 32, 8), ('<4 x i64>', 'q', 64, 4)]:
        T = tag(ty)
        S = sfx.upper()
        M = f'<{n} x i1>'
        eq, gt = f'VPCMPEQ{S}.{T}', f'VPCMPGT{S}.{T}'
        notm = lambda x: [f'%xi = sext {M} {x} to {ty}', '%on = call <4 x i64> @VALLONES()', f'%ot = bitcast <4 x i64> %on to {ty}',
                          f'%ni = call {ty} @VPXOR.{T}({ty} %xi, {ty} %ot)', f'%r = trunc {ty} %ni to {M}', f'ret {M} %r']
        p2 = [(ty, 'a'), (ty, 'b')]
        recipe(f'vslt.{T}', M, p2, [f'%r = icmp slt {ty} %a, %b', f'ret {M} %r'], [f'%r = call {M} @{gt}({ty} %b, {ty} %a)', f'ret {M} %r'])
        for pred, inner, a, b in [('ne', eq, '%a', '%b'), ('sge', gt, '%b', '%a'), ('sle', gt, '%a', '%b')]:
            recipe(f'v{pred}.{T}', M, p2, [f'%r = icmp {pred} {ty} %a, %b', f'ret {M} %r'],
                   [f'%c = call {M} @{inner}({ty} {a}, {ty} {b})'] + notm('%c'))
        if eb < 64:
            mx = f'VPMAXU{S}.{T}'
            for pred, x, y, inv in [('uge', '%a', '%b', False), ('ule', '%b', '%a', False), ('ult', '%a', '%b', True), ('ugt', '%b', '%a', True)]:
                body = [f'%m = call {ty} @{mx}({ty} {x}, {ty} {y})', f'%c = call {M} @{eq}({ty} %m, {ty} {x})']
                recipe(f'v{pred}.{T}', M, p2, [f'%r = icmp {pred} {ty} %a, %b', f'ret {M} %r'], body + (notm('%c') if inv else [f'ret {M} %c']))
        else:
            for pred, x, y, inv in [('ugt', '%fa', '%fb', False), ('ult', '%fb', '%fa', False), ('ule', '%fa', '%fb', True), ('uge', '%fb', '%fa', True)]:
                body = ['%o = call <4 x i64> @VALLONES()', '%sg = call <4 x i64> @VPSLLQi.4xi64(<4 x i64> %o, i64 63)',
                        '%fa = call <4 x i64> @VPXOR.4xi64(<4 x i64> %a, <4 x i64> %sg)',
                        '%fb = call <4 x i64> @VPXOR.4xi64(<4 x i64> %b, <4 x i64> %sg)', f'%c = call {M} @{gt}({ty} {x}, {ty} {y})']
                recipe(f'v{pred}.{T}', M, p2, [f'%r = icmp {pred} {ty} %a, %b', f'ret {M} %r'], body + (notm('%c') if inv else [f'ret {M} %c']))
        if eb < 64:
            enc, _ = xed(f'vpabs{sfx} {{o0}}, {{o1}}', ('ye', 'ye'))
            for flag in ('false', 'true'):
                entry(ty, f'VPABS{S}.{flag}.{T}', [f'{ty} "reg"="yv" %n'], renamed(enc, {'o0': 'n', 'o1': 'd'}), f'vpabs{sfx} {{n}}, {{d}}',
                      [f'%r = call {ty} @llvm.abs.v{n}i{eb}({ty} %n, i1 {flag})', f'ret {ty} %r'], ' "dclass"="yv"' + feat)
            out.append(f'declare {ty} @llvm.abs.v{n}i{eb}({ty}, i1)\n')
    # SSE2's absolute values of 16- and 32-bit lanes.
    for ty, cls, eb, n, sra, sub, xor in [('<4 x i32>', 'xv', 32, 4, 'PSRADi.4xi32', 'PSUBD.4xi32', 'PXOR.4xi32'),
                                         ('<8 x i16>', 'xv', 16, 8, 'PSRAWi.8xi16', 'PSUBW.8xi16', 'PXOR.8xi16'),
                                         ('<2 x i32>', 'xh', 32, 2, 'PSRADi.2xi32', 'PSUBD.2xi32', 'PXOR.2xi32')]:
        T = tag(ty)
        out.append(f'declare {ty} @llvm.abs.v{n}i{eb}({ty}, i1)\n')
        for flag in ('false', 'true'):
            out.append(f'define {ty} @abs.{flag}.{T}({ty} "reg"="{cls}" %x) "recipe"="abs.{flag}.{T}.expand" {{\n'
                       f'  %r = call {ty} @llvm.abs.v{n}i{eb}({ty} %x, i1 {flag})\n  ret {ty} %r\n}}\n')
            out.append(f'define {ty} @abs.{flag}.{T}.expand({ty} %x) {{\n  %s = call {ty} @{sra}({ty} %x, i32 {eb - 1})\n'
                       f'  %y = call {ty} @{xor}({ty} %x, {ty} %s)\n  %r = call {ty} @{sub}({ty} %y, {ty} %s)\n  ret {ty} %r\n}}\n')


def float_minmax():
    """IEEE 754's maximum and minimum (llvm.maximum, llvm.minimum) of
    vectors: x86's maxps and minps return the second operand when either
    is a NaN and when the two are equal (so of +0 and -0), so these are
    compares and selects instead: the greater (or lesser), or when equal
    the two's bits and-ed (or or-ed), which makes +0 the greater of the
    zeros, and a NaN (their sum) when either is one."""
    for ty, it, cls, feat in [('<4 x float>', 'v4f32', 'xv', ''), ('<2 x double>', 'v2f64', 'xv', ''),
                              ('<8 x float>', 'v8f32', 'yv', ' "feature"="avx2"'), ('<4 x double>', 'v4f64', 'yv', ' "feature"="avx2"')]:
        n = ty[1:].split()[0]
        M = f'<{n} x i1>'
        T = f'{n}x{"f32" if "float" in ty else "f64"}'
        S = 'PS' if 'float' in ty else 'PD'
        if cls == 'xv':
            I, cmp, add, AND, OR = '<2 x i64>', f'CMP{{p}}{S}.{T}', f'FADD{S}.{T}', 'PAND.2xi64', 'POR.2xi64'
            def select(r, c, a, b):
                return (f'  %{r}c = bitcast {M} %{c} to {I}\n  %{r}a = bitcast {ty} %{a} to {I}\n  %{r}b = bitcast {ty} %{b} to {I}\n'
                        f'  %{r}x = call {I} @PAND.2xi64({I} %{r}c, {I} %{r}a)\n  %{r}y = call {I} @PANDN.xv({I} %{r}c, {I} %{r}b)\n'
                        f'  %{r}i = call {I} @POR.2xi64({I} %{r}x, {I} %{r}y)\n  %{r} = bitcast {I} %{r}i to {ty}\n')
        else:
            I, cmp, add, AND, OR = '<4 x i64>', f'VCMP{{p}}{S}.{T}', f'VADD{S}.{T}', 'VPAND.4xi64', 'VPOR.4xi64'
            def select(r, c, a, b):
                return f'  %{r} = call {ty} @VBLENDV{S}.{T}({M} %{c}, {ty} %{a}, {ty} %{b})\n'
        # (SSE's greater-than is its less-than, the operands swapped: its
        # entry's are.)
        for op, (pred, x, y), bits in [('maximum', ('OGT', 'a', 'b') if cls == 'yv' else ('OLT', 'b', 'a'), AND),
                                       ('minimum', ('OLT', 'a', 'b'), OR)]:
            out.append(f'define {ty} @{op}.{T}({ty} "reg"="{cls}" %a, {ty} "reg"="{cls}" %b) "recipe"="{op}.{T}.expand"{feat} {{\n'
                       f'  %r = call {ty} @llvm.{op}.{it}({ty} %a, {ty} %b)\n  ret {ty} %r\n}}\n')
            out.append(f'define {ty} @{op}.{T}.expand({ty} %a, {ty} %b) {{\n'
                       f'  %w = call {M} @{cmp.format(p=pred)}({ty} %{x}, {ty} %{y})\n'
                       + select('x', 'w', 'a', 'b') +
                       f'  %e = call {M} @{cmp.format(p="OEQ")}({ty} %a, {ty} %b)\n'
                       f'  %ai = bitcast {ty} %a to {I}\n  %bi = bitcast {ty} %b to {I}\n'
                       f'  %ci = call {I} @{bits}({I} %ai, {I} %bi)\n  %c = bitcast {I} %ci to {ty}\n'
                       + select('y', 'e', 'c', 'x') +
                       f'  %u = call {M} @{cmp.format(p="UNO")}({ty} %a, {ty} %b)\n'
                       f'  %nan = call {ty} @{add}({ty} %a, {ty} %b)\n'
                       + select('r', 'u', 'nan', 'y') +
                       f'  ret {ty} %r\n}}\n')
            out.append(f'declare {ty} @llvm.{op}.{it}({ty}, {ty})\n')

def vector_copies():
    """Copies of the vector and mask registers (the register allocator's)."""
    entry('<2 x i64>', 'MOVAPSrr', ['<2 x i64> "reg"="x" %n'], 'rex(0, d, 0, n) | 00001111 | 00101000 | rr(d, n)',
          'movaps {n}, {d}', ['ret <2 x i64> %n'], ' "copy"="x" "dclass"="x"')
    for cls, ty, l, ll in [('xe', '<2 x i64>', 0, 0), ('ye', '<4 x i64>', 1, 1)]:
        # (The store form where only it has the two-byte prefix, as llvm-mc
        # writes it.)
        entry(ty, f'VMOVAPS{cls}', [f'{ty} "reg"="{cls}" %n'],
              f'{{ if(n<8) vex(d, 0, n, 1, 0, 0, {l}, 0) | 00101000 | rr(d, n) ; '
              f'if(d<8) vex(n, 0, d, 1, 0, 0, {l}, 0) | 00101001 | rr(n, d) ; '
              f'vex(d, 0, n, 1, 0, 0, {l}, 0) | 00101000 | rr(d, n) ; '
              f'evexr(d, n, 1, 0, 0, {ll}, 0, 0, 0, 0) | 00101000 | rr(d, n) }}',
              'vmovaps {n}, {d}', [f'ret {ty} %n'], f' "copy"="{cls}" "dclass"="{cls}"')
    entry('<8 x i64>', 'VMOVAPSz', ['<8 x i64> "reg"="z" %n'], 'evexr(d, n, 1, 0, 0, 2, 0, 0, 0, 0) | 00101000 | rr(d, n)',
          'vmovaps {n}, {d}', ['ret <8 x i64> %n'], ' "copy"="z" "dclass"="z"')
    entry('i64', 'KMOVQ', ['i64 "reg"="k" %n'], 'vex(d, 0, n, 1, 1, 0, 0, 0) | 10010000 | rr(d, n)', 'kmovq {n}, {d}',
          ['ret i64 %n'], ' "copy"="k" "dclass"="k"')
    # AVX-512's masks (k1-k7) to and from the integer registers, and their
    # spills: kmovq, all 64 bits (AVX512BW's; every machine with AVX-512
    # but the Xeon Phi has it). Only for the allocator.
    enc, _ = xed('kmovq {o0}, {o1}', ('r64', 'k'))
    entry('i64', 'KMOVQkr', ['i64 "reg"="r64" %n'], renamed(enc, {'o0': 'n', 'o1': 'd'}), 'kmovq {n}, {d}', ['ret i64 %n'],
          ' "dclass"="k" "role"="move.k.r64" "nomatch"')
    enc, _ = xed('kmovq {o0}, {o1}', ('k', 'r64'))
    entry('i64', 'KMOVQrk', ['i64 "reg"="k" %n'], renamed(enc, {'o0': 'n', 'o1': 'd'}), 'kmovq {n}, {d}', ['ret i64 %n'],
          ' "dclass"="r64" "role"="move.r64.k" "nomatch"')
    enc, _ = xed('kmovq {m.disp}({m.base}), {o0}', ('k',))
    entry('i64', 'KMOVQld', ['ptr "reg"="r64" %n', 'i64 "imm" %off'], renamed(enc, {'o0': 'd', 'm.disp': 'off', 'm.base': 'n'}),
          'kmovq {off}({n}), {d}', ['%a = getelementptr i8, ptr %n, i64 %off', '%r = load i64, ptr %a', 'ret i64 %r'],
          ' "dclass"="k" "role"="load.k" "nomatch"')
    enc, _ = xed('kmovq {o0}, {m.disp}({m.base})', ('k',))
    entry('void', 'KMOVQst', ['i64 "reg"="k" %t', 'ptr "reg"="r64" %n', 'i64 "imm" %off'],
          renamed(enc, {'o0': 't', 'm.disp': 'off', 'm.base': 'n'}), 'kmovq {t}, {off}({n})',
          ['%a = getelementptr i8, ptr %n, i64 %off', 'store i64 %t, ptr %a', 'ret void'], ' "role"="store.k" "nomatch"')


def intrinsics():
    """x86's own intrinsics (llvm.x86.*) that are one instruction of
    02-xed.ll's, as discover_intrinsics.py found them (intrinsics.txt): the
    body calls the intrinsic, and the instruction's operands are its
    arguments and result; the features are those Rust's core::arch asks
    for it."""
    import re
    sel = {'x': 'xv', 'xe': 'xv', 'y': 'yv', 'ye': 'yv', 'z': 'zv'}
    declared = set(re.findall(r'declare [^@]*@([\w.]+)\(', ''.join(out)))

    def split_top(s):
        parts, depth, cur = [], 0, ''
        for ch in s:
            depth += ch in '<{('
            depth -= ch in '>})'
            if ch == ',' and depth == 0:
                parts.append(cur.strip())
                cur = ''
            else:
                cur += ch
        return parts + ([cur.strip()] if cur.strip() else [])
    for line in open(os.path.join(os.path.dirname(os.path.abspath(__file__)), 'intrinsics.txt')):
        f = line.rstrip('\n').split(' ; ')
        if line.startswith('#') or len(f) < 9 or f[3] != 'entry' or f[0] in declared:
            continue
        # (Not yet those of half: x86 has no half values yet.)
        if 'half' in f[1]:
            continue
        name, sig, feats, _, asm, classes, operands, extra = f[:8]
        ret, args = sig[:-1].split('(', 1)
        args = split_top(args)
        enc, _ = xed(asm, classes.split())
        names = dict(o.split('=') for o in operands.split())
        extra = extra.split()
        tied = next((e[5:] for e in extra if e.startswith('tied=')), None)
        # Operands in registers the instruction takes unnamed (eax, xmm0).
        fixed = dict(e[6:].split(':') for e in extra if e.startswith('fixed='))
        clobbers = [c for e in extra if e.startswith('clobbers=') for c in e[9:].split(',')]
        clobbers += ['flags'] if 'flags' in extra else []
        # Constants the instruction is for one value of (a rounding).
        consts = dict(e[6:].split(':') for e in extra if e.startswith('const='))
        # The class of each register operand, by its name in the template.
        regs = re.findall(r'\{(o\d+)\}', asm)
        cls = dict(zip(regs, classes.split()))
        of = {v: k for k, v in names.items()}
        params, call = [], []
        for k, a in enumerate(args):
            ty, imm = (a[:-4], True) if a.endswith(' imm') else (a, False)
            if f'a{k}' in consts:
                call.append(f'{ty} {consts[f"a{k}"]}')
                continue
            call.append(f'{ty} %a{k}')
            if imm:
                params.append(f'{ty} "imm" %a{k}')
            elif f'a{k}' in fixed:
                params.append(f'{ty} "reg"="{fixed[f"a{k}"]}" %a{k}')
            elif of.get(f'a{k}') == 'm.base':
                params.append(f'{ty} "reg"="r64" %a{k}')
            else:
                o = of.get(f'a{k}') or of['d']
                params.append(f'{ty} "reg"="{sel.get(cls[o], cls[o])}"' + (' "tied"' if tied == f'a{k}' else '') + f' %a{k}')
        if 'd' in fixed:
            attrs = f' "dclass"="{fixed["d"]}"'
        else:
            attrs = f' "dclass"="{sel.get(cls[of["d"]], cls[of["d"]])}"' if 'd' in of else ''
        attrs += f' "feature"="{feats}"' + (f' "clobbers"="{",".join(clobbers)}"' if clobbers else '')
        template = asm.replace('{m.disp}({m.base})', '0({m.base})')
        for o, v in names.items():
            template = template.replace('${' + o + '}', '${' + v + '}').replace('{' + o + '}', '{' + v + '}')
        if ret == 'void':
            body = [f'call void @{name}({", ".join(call)})', 'ret void']
        else:
            body = [f'%r = call {ret} @{name}({", ".join(call)})', f'ret {ret} %r']
        # (A byte immediate is read either way, as an i8 is written: Rust's
        # 0x80 is -128.)
        enc = renamed(enc, names)
        for k, a in enumerate(args):
            if a.endswith(' imm'):
                enc = re.sub(rf'\bi(\d):8=a{k}\b', rf'{{ i\1:8=a{k} ; i\1:s8=a{k} }}', enc)
        entry(ret, 'X86.' + name[len('llvm.x86.'):], params, enc, template, body, attrs)
        types = ', '.join(a.removesuffix(' imm') for a in args)
        out.append(f'declare {ret} @{name}({types})\n')
        declared.add(name)


def main():
    out.append("""; x86-64's integer instructions for selection. Written by gen_sel.py; do
; not edit. The conventions are those of codegen/targets/aarch64/00-base.ll,
; and docs/design/x86.md has the plan.
""")
    alu()
    moves()
    memory()
    indexed_memory()
    rip_memory()
    compares()
    control()
    division()
    recipes()
    floats()
    load_op()
    atomics()
    x87()
    sse_vectors()
    sse_recipes()
    avx2()
    avx512()
    avx512_int()
    avx512vl_masks()
    reductions()
    mul64()
    more_compares()
    float_minmax()
    vector_copies()
    intrinsics()
    sys.stdout.write('\n'.join(out))


main()
