# Sums each tag's objects in target/vs-llc/<triple>/ (see run.sh):
# instructions, register moves, loads and stores addressed from the stack
# or frame pointer, and on x86-64 bytes; and each as a ratio of llc's.
import collections, os, re, subprocess, sys

out, objdump = sys.argv[1], sys.argv[2]
VECTOR = {'rand_chacha', 'memchr', 'base64', 'crc32fast', 'hashbrown'}

def counts(path, x86):
    # (x86's bytes are its encodings; AArch64's words are not wanted.)
    flags = ['-d'] if x86 else ['-d', '--no-show-raw-insn']
    d = subprocess.run([objdump, *flags, path], capture_output=True, text=True).stdout
    c = collections.Counter()
    for line in d.splitlines():
        m = re.match(r'\s*[0-9a-f]+:\s+((?:[0-9a-f]{2} )+)?\s*(\S+)\s*(.*)', line)
        if not m or line.rstrip().endswith(':'):
            continue
        # (Without the comment: x86's after '#', AArch64's after ';'.)
        op, args = m.group(2), m.group(3).split('#' if x86 else ';')[0].strip()
        c['insts'] += 1
        if x86:
            c['bytes'] += len((m.group(1) or '').split())
            if op.startswith('mov') or op.startswith('vmov'):
                if '(' not in args and '$' not in args:
                    c['moves'] += 1
                elif '(%rsp)' in args or '(%rbp)' in args:
                    c['loads' if '(' in args.split(',')[0] else 'stores'] += 1
        else:
            if op in ('mov', 'fmov') and '#' not in args and 'sp' not in args:
                c['moves'] += 1
            if '[sp' in args or '[x29' in args:
                pair = op in ('ldp', 'stp')
                if op.startswith('ld'):
                    c['loads'] += 1 + pair
                elif op.startswith('st'):
                    c['stores'] += 1 + pair
    return c

for triple in sorted(os.listdir(out)):
    d = os.path.join(out, triple)
    if triple == 'sample' or not os.path.isdir(d):
        continue
    x86 = triple.startswith('x86')
    objs = [f[:-2].split('-', 1) for f in os.listdir(d) if f.endswith('.o')]
    tags = sorted({t for t, _ in objs} - {'llc'})
    crates = sorted({n for t, n in objs if t == 'llc'} - (VECTOR if x86 else set()))
    keys = ['insts', 'moves', 'loads', 'stores'] + (['bytes'] if x86 else [])
    tot = {}
    for t in ['llc'] + tags:
        tot[t] = collections.Counter()
        for n in crates:
            p = os.path.join(d, f'{t}-{n}.o')
            if os.path.exists(p) and os.path.getsize(p) > 0:
                tot[t] += counts(p, x86)
    print(triple + (' (scalar crates)' if x86 else ''))
    size = 'bytes' if x86 else 'insts'
    for t in ['llc'] + tags:
        print(f'  {t:10}', '  '.join(f'{k} {tot[t][k]}' for k in keys),
              f' {tot[t][size] / max(1, tot["llc"][size]):.3f}x llc')
