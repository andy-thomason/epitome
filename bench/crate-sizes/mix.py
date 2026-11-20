#!/usr/bin/env python3
"""The instruction mix of a build's functions, for classifying excesses.

    bench/crate-sizes/mix.py EXE SYMBOL-SUBSTRING [--code]

For each text symbol containing the substring: bytes, instructions, stack
loads and stores (sp/x29 on AArch64, rsp/rbp on x86-64), other memory
operations, calls, branches, traps, vector instructions, moves between
registers, and AArch64 frame addresses made in x15 (offsets out of
reach). --code prints the disassembly too.
"""
import re
import subprocess
import sys

KEYS = ('insts', 'stack_ld', 'stack_st', 'frame_addr', 'mem', 'calls', 'branches', 'traps', 'vector', 'moves')


def classify(ins):
    """Counts of the kinds of instruction in `ins` (objdump's text)."""
    c = dict.fromkeys(KEYS, 0)
    c['insts'] = len(ins)
    for i in ins:
        op = i.split()[0] if i.split() else ''
        stack = re.search(r'\[(sp|x29|x15)\b|\((%rsp|%rbp)\)|%rsp\)|%rbp\)', i)
        if op.startswith(('ldr', 'ldp', 'ldur')) or (op.startswith('mov') and '(' in i.split(',')[0]):
            c['stack_ld' if stack else 'mem'] += 1
        elif op.startswith(('str', 'stp', 'stur')) or (op.startswith('mov') and '(' in i.split(',')[-1]):
            c['stack_st' if stack else 'mem'] += 1
        elif op in ('bl', 'blr', 'call', 'callq'):
            c['calls'] += 1
        elif op in ('brk', 'ud2', 'int3'):
            c['traps'] += 1
        elif op.startswith(('b', 'j', 'cb', 'tb')) and op not in ('bic', 'bics', 'bfi', 'bfxil', 'bsr', 'bsf', 'bswap', 'bt'):
            c['branches'] += 1
        # AArch64's frame offsets out of reach: `mov x15, #off; add x15, sp, x15`.
        if re.match(r'(mov\s+x15, #|add\s+x15, sp, x15)', i):
            c['frame_addr'] += 1
        if re.search(r'\b[qv]\d+|%[xyz]mm', i):
            c['vector'] += 1
        if (op == 'mov' and re.match(r'mov\s+[wx]\d+, [wx]\d+$', i)) or re.match(r'movq?\s+%r\w+, %r\w+$', i):
            c['moves'] += 1
    return c


def disassemble(exe):
    """{symbol: [(line, instruction)]} of the executable's text."""
    out = subprocess.run(['objdump', '-d', '--no-show-raw-insn', exe], capture_output=True, text=True).stdout
    funcs = {}
    for f in re.split(r'\n(?=[0-9a-f]+ <)', out):
        m = re.match(r'([0-9a-f]+) <(.*)>:', f.split('\n', 1)[0])
        if not m:
            continue
        lines = [l for l in f.split('\n')[1:] if re.match(r'\s*[0-9a-f]+:', l)]
        name = m.group(2)
        if sys.platform == 'darwin' and name.startswith('_'):
            name = name[1:]
        funcs[name] = [(l, l.split(':', 1)[1].strip()) for l in lines]
    return funcs


if __name__ == '__main__':
    exe, pat = sys.argv[1], sys.argv[2]
    for name, lines in disassemble(exe).items():
        if pat not in name:
            continue
        c = classify([i for _, i in lines])
        print(f'{name}: ' + ', '.join(f'{k} {v}' for k, v in c.items()))
        if '--code' in sys.argv:
            print('\n'.join(l for l, _ in lines))
