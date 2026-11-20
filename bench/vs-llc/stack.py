# x86-64's stack traffic, by kind, for each tag's objects in
# target/vs-llc/x86_64-unknown-linux-gnu/ (see run.sh), or for objects
# named on the command line; the numbers of docs/design/push-pop.md.
#
#   python3 bench/vs-llc/stack.py LLVM-OBJDUMP target/vs-llc [TAG...]
#   python3 bench/vs-llc/stack.py LLVM-OBJDUMP prog.o ...
#
# For each: accesses of memory at rsp or rbp (moves to and from registers,
# immediates stored, other instructions' operands, leaq) with their bytes
# and how many need a 32-bit offset; push and pop; and three patterns:
#   forwardable  a load of a slot that a store of the same width wrote
#                earlier in the block, with no call, branch target, leaq
#                of a slot or (where one escapes) store through another
#                pointer between: the load (and often the store) need not
#                be there at all.
#   once         a slot stored once and loaded once (by 8-byte moves,
#                nothing else touching it): the candidates for a push and
#                a pop, by whether a call or another access of the stack
#                comes between, and whether they are in one block.
#   imm via reg  an immediate made in a register only to be stored.
import collections, os, re, subprocess, sys

LINE = re.compile(r'\s*([0-9a-f]+):\s+((?:[0-9a-f]{2} )+)\s*(\S+)\s*(.*)')
VECTOR = {'rand_chacha', 'memchr', 'base64', 'crc32fast', 'hashbrown'}
W = {'q': 8, 'l': 4, 'w': 2, 'b': 1}
R = {}
for n in ['ax', 'bx', 'cx', 'dx', 'si', 'di', 'bp', 'sp']:
    for v in ('r' + n, 'e' + n, n, n[0] + 'l' if n[1] == 'x' else n + 'l'):
        R[v] = n
for k in range(8, 16):
    for v in (f'r{k}', f'r{k}d', f'r{k}w', f'r{k}b'):
        R[v] = f'r{k}'


def regs(s):
    return {R[x] for x in re.findall(r'%([a-z0-9]+)', s) if x in R}


def slot(a):
    m = re.fullmatch(r'(-?0x[0-9a-f]+|-?\d+)?\(%rsp\)', a.strip())
    return (int(m.group(1), 0) if m.group(1) else 0) if m else None


def width(op, args):
    if op in ('movups', 'movaps', 'movdqu', 'movdqa', 'vmovups', 'vmovdqu', 'vmovaps', 'vmovdqa'):
        return 32 if any('ymm' in a for a in args) else 16
    if op in ('movsd', 'movq') and any('xmm' in a for a in args):
        return 8
    if op in ('movss', 'movd'):
        return 4
    if op.startswith('mov') and op[-1] in W and not op.startswith(('movz', 'movs')):
        return W[op[-1]]
    return None


def funcs(objdump, path):
    dis = subprocess.run([objdump, '-d', path], capture_output=True, text=True).stdout
    fs, f = [], None
    for line in dis.splitlines():
        if re.match(r'^[0-9a-f]+ <', line):
            f = []
            fs.append(f)
            continue
        m = LINE.match(line)
        if m and f is not None:
            args = m.group(4).split('#')[0].strip()
            parts = [a.strip() for a in re.split(r',(?![^(]*\))', args)] if args else []
            f.append((int(m.group(1), 16), len(m.group(2).split()), m.group(3), parts))
    return fs


def kinds(f, c):
    for a, nb, op, args in f:
        joined = ', '.join(args)
        c['all'] += 1
        c['all bytes'] += nb
        if op.startswith(('push', 'pop')):
            k = op + (' imm' if '$' in joined else '')
        elif '(%rsp' not in joined and '(%rbp' not in joined:
            continue
        elif op.startswith('lea'):
            k = 'leaq'
        elif op.startswith(('mov', 'vmov')) and len(args) == 2:
            vec = 'xmm' in joined or 'ymm' in joined
            if '(' in args[0]:
                k = 'load' + (' vector' if vec else '')
            elif '$' in args[0]:
                k = 'store imm'
            else:
                k = 'store' + (' vector' if vec else '')
        else:
            k = 'other operand'
        c[k] += 1
        c[k + ' bytes'] += nb
        disp = re.search(r'(-?0x[0-9a-f]+|-?\d+)?\(%r[sb]p', joined)
        if disp and disp.group(1) and not -128 <= int(disp.group(1), 0) < 128:
            c[k + ' disp32'] += 1


def targets_of(f):
    t = set()
    for a, nb, op, args in f:
        if op.startswith('j') and args:
            m = re.match(r'0x([0-9a-f]+)', args[0])
            if m:
                t.add(int(m.group(1), 16))
    return t


def forwardable(f, c):
    targets = targets_of(f)
    escapes = any(op.startswith('lea') and '(%rsp' in ' '.join(args) for _, _, op, args in f)
    last = {}
    for a, nb, op, args in f:
        if a in targets:
            last = {}
        if op.startswith(('call', 'j', 'lea', 'ret', 'rep')):
            last = {}
            continue
        w = width(op, args)
        if w and len(args) == 2 and slot(args[1]) is not None and args[0].startswith('%'):
            s = slot(args[1])
            for b in range(s, s + w):
                last[b] = (s, w)
            continue
        if w and len(args) == 2 and slot(args[0]) is not None and args[1].startswith('%'):
            s = slot(args[0])
            if last.get(s) == (s, w) and all(last.get(b) == (s, w) for b in range(s, s + w)):
                c['forwardable load'] += 1
                c['forwardable load bytes'] += nb
            continue
        if args:
            s = slot(args[-1])
            if s is not None:
                for b in [b for b in last if s - 32 < b < s + 32]:
                    del last[b]
            elif escapes and len(args) == 2 and '(' in args[1] and op.startswith('mov'):
                last = {}


def once(f, c):
    targets = targets_of(f)
    st, ld, other, pos = collections.Counter(), collections.Counter(), set(), {}
    for i, (a, nb, op, args) in enumerate(f):
        for k, x in enumerate(args):
            s = slot(x)
            if s is None:
                continue
            pos.setdefault(s, []).append(i)
            if op == 'movq' and len(args) == 2 and k == 1 and args[0].startswith('%'):
                st[s] += 1
            elif op == 'movq' and len(args) == 2 and k == 0 and args[1].startswith('%'):
                ld[s] += 1
            else:
                other.add(s)
    for s in st:
        if st[s] != 1 or ld[s] != 1 or s in other or len(pos[s]) != 2:
            continue
        i, j = pos[s]
        if slot(f[i][3][1]) != s:
            continue
        mid = f[i + 1:j]
        block = not any(x[0] in targets for x in f[i + 1:j + 1]) and not any(x[2].startswith('j') for x in mid)
        k = 'once: ' + ('one block' if block else 'across blocks')
        if any(x[2].startswith('call') for x in mid):
            k += ', a call between'
        if any('%rsp' in ' '.join(x[3]) for x in mid):
            k += ', rsp used between'
        c[k] += 1
        c[k + ' bytes'] += f[i][1] + f[j][1]


def imm_via_reg(f, c):
    for i, (nb, op, args) in enumerate((x[1], x[2], ', '.join(x[3])) for x in f):
        m = re.fullmatch(r'\$(-?0x[0-9a-f]+|-?\d+), %([a-z0-9]+)', args)
        zero = op == 'xorl' and re.fullmatch(r'%(\w+), %\1', args)
        if not ((op in ('movl', 'movq', 'movabsq') and m) or zero):
            continue
        r = R.get(m.group(2) if m else args.split(',')[0][1:])
        if r is None:
            continue
        stores, total, dead = 0, nb, False
        for _, nb2, op2, parts in f[i + 1:i + 12]:
            if op2.startswith(('j', 'call', 'ret')):
                break
            if (op2.startswith('mov') and len(parts) == 2 and '(' in parts[1] and parts[0].startswith('%')
                    and R.get(parts[0][1:]) == r and r not in regs(parts[1])):
                stores += 1
                total += nb2
                continue
            if r in regs(', '.join(parts)):
                dead = (op2.startswith(('movl', 'movq', 'lea', 'xorl')) and len(parts) == 2
                        and R.get(parts[1][1:]) == r and r not in regs(parts[0]))
                break
        else:
            dead = True
        if stores and dead:
            c['imm via reg'] += stores
            c['imm via reg bytes'] += total


def report(name, paths, objdump):
    c = collections.Counter()
    for p in paths:
        for f in funcs(objdump, p):
            kinds(f, c)
            forwardable(f, c)
            once(f, c)
            imm_via_reg(f, c)
    print(f"{name}: {c['all']} instructions, {c['all bytes']} bytes")
    for k in sorted(k for k in c if not k.startswith('all') and not k.endswith((' bytes', ' disp32'))):
        d = f"  ({c[k + ' disp32']} disp32)" if c[k + ' disp32'] else ''
        print(f'  {k:42} {c[k]:6}  {c[k + " bytes"]:7} bytes{d}')


objdump, where, tags = sys.argv[1], sys.argv[2], sys.argv[3:]
if where.endswith('.o'):
    for p in [where] + tags:
        report(p, [p], objdump)
else:
    d = os.path.join(where, 'x86_64-unknown-linux-gnu')
    objs = [f[:-2].split('-', 1) for f in os.listdir(d) if f.endswith('.o')]
    crates = sorted({n for t, n in objs if t == 'llc'} - VECTOR)
    for t in tags or ['llc'] + sorted({t for t, _ in objs} - {'llc'}):
        report(t, [os.path.join(d, f'{t}-{n}.o') for n in crates if os.path.exists(os.path.join(d, f'{t}-{n}.o'))], objdump)
