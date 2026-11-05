#!/usr/bin/env python3
"""Finds the instruction for each of SVE's intrinsics (llvm.aarch64.sve.*),
for gen_sve.py's intrinsics(): writes sve-intrinsics.txt.

As x86_64/discover_intrinsics.py does for x86's: the intrinsics, their
types and the features they need are read from Rust's core::arch (its
aarch64/sve and sve2 sources: each `link_name` extern, and the
`target_feature` of the function it is in). The instruction is what
LLVM's llc makes of a function passing its arguments to the intrinsic (a
black box: the arguments arrive in AAPCS64's registers, z0-z7, p0-p3,
x0-x7, so the registers of its code say which operand each is). The
arguments llc wants constant (immarg) are given the first of a few values
it takes. Moves before the instruction (mov, an unpredicated movprfx) are
followed; the instruction is then found among 81-scalable.ll's entries by
its template.

Each line of sve-intrinsics.txt is one intrinsic, ` ; ` between fields:

    NAME ; SIGNATURE ; FEATURES ; WHAT
    WHAT = entry ; ENTRY ; OPERANDS ; EXTRA ; llc: INSTRUCTION
         | skip ; REASON ; llc: INSTRUCTIONS

SIGNATURE is `RET(ARG, ...)`, an argument `T imm:V` when it is a constant
(V the value llc took).
ENTRY is the 81-scalable.ll entry; OPERANDS names its operands for the
intrinsic's (`zm=a2 zdn=d pg=a0`: argument 2, the result, argument 0, or
a constant `imm=4`), and EXTRA is `tied=aK` (the result is written over
argument K) and `const=aK:V` (a constant the instruction does not take,
for which the entry is for that value alone).

Usage: discover_sve.py CORE_ARCH_SRC LLC > sve-intrinsics.txt
"""
import glob
import itertools
import os
import re
import subprocess
import sys
import tempfile
from concurrent.futures import ThreadPoolExecutor

HERE = os.path.dirname(os.path.abspath(__file__))

# ---------------------------------------------------------------------------
# Rust's declarations

LANES = {'8': 16, '16': 8, '32': 4, '64': 2}
SCALARS = {'i8': 'i8', 'i16': 'i16', 'i32': 'i32', 'i64': 'i64', 'u8': 'i8', 'u16': 'i16', 'u32': 'i32',
           'u64': 'i64', 'f32': 'float', 'f64': 'double', 'bool': 'i1', 'usize': 'i64', 'isize': 'i64',
           'svpattern': 'i32', 'svprfop': 'i32', '()': 'void'}
# Rust's names of features, where LLVM's differ.
FEATURES = {'sve2-bitperm': 'sve-bitperm', 'sve2-aes': 'sve-aes', 'sve2-sha3': 'sve-sha3', 'sve2-sm4': 'sve-sm4'}


def ir_type(t):
    t = t.strip()
    if t in SCALARS:
        return SCALARS[t]
    if t.startswith('*'):
        return 'ptr'
    m = re.fullmatch(r'svbool(\d*)_t', t)
    if m:
        return f'<vscale x {int(m.group(1) or 16)} x i1>'
    m = re.fullmatch(r'sv(u?int|float|bfloat)(\d+)_t', t)
    if m:
        kind, bits = m.groups()
        if kind == 'float':
            elem = {'16': 'half', '32': 'float', '64': 'double'}[bits]
        else:
            elem = 'bfloat' if kind == 'bfloat' else f'i{bits}'
        return f'<vscale x {LANES[bits]} x {elem}>'
    # Rust's names of unpacked vectors: nxv2i32, a 32-bit lane in each of
    # a register's 64-bit ones.
    m = re.fullmatch(r'nxv(\d+)(i|f)(\d+)', t)
    if m:
        n, k, bits = m.groups()
        elem = f'i{bits}' if k == 'i' else {'16': 'half', '32': 'float', '64': 'double'}[bits]
        return f'<vscale x {n} x {elem}>'
    return None


def split_top(s):
    out, depth, cur = [], 0, ''
    for c in s:
        depth += c in '(<['
        depth -= c in ')>]'
        if c == ',' and depth == 0:
            out.append(cur)
            cur = ''
        else:
            cur += c
    out.append(cur)
    return [x.strip() for x in out if x.strip()]


def declarations(src):
    """(name, ret, args, features) of each intrinsic Rust declares."""
    seen = set()
    for f in sorted(glob.glob(f'{src}/aarch64/sve/*.rs') + glob.glob(f'{src}/aarch64/sve2/*.rs')):
        s = open(f).read()
        for m in re.finditer(r'link_name = "(llvm\.aarch64\.sve\.[^"]+)"\s*\)\]\s*fn \w+\s*\((.*?)\)\s*(?:->\s*([^;]+))?;', s, re.S):
            name = m.group(1)
            if name in seen:
                continue
            seen.add(name)
            args = [ir_type(a.split(':', 1)[1]) for a in split_top(m.group(2))]
            ret = ir_type(m.group(3) or '()')
            before = s[:m.start()]
            fm = list(re.finditer(r'target_feature\(enable = "([^"]+)"\)', before))
            feats = ','.join(FEATURES.get(x, x) for x in fm[-1].group(1).split(',')) if fm else 'sve'
            if any(x in feats for x in ('sve-aes', 'sve-sha3', 'sve-sm4', 'sve-bitperm')) and 'sve2' not in feats:
                feats += ',sve2'
            yield name, ret, args, feats


# ---------------------------------------------------------------------------
# llc

CONSTS = [1, 2, 0, 3, 4, 90, 31, 8, 7, 16]


def module(name, ret, args, consts):
    params, call = [], []
    for k, a in enumerate(args):
        if k in consts:
            call.append(f'{a} {consts[k]}')
        else:
            params.append(f'{a} %a{k}')
            call.append(f'{a} %a{k}')
    text = f'declare {ret} @{name}({", ".join(args)})\ndefine {ret} @f({", ".join(params)}) {{\n'
    if ret == 'void':
        text += f'  call void @{name}({", ".join(call)})\n  ret void\n}}\n'
    else:
        text += f'  %r = call {ret} @{name}({", ".join(call)})\n  ret {ret} %r\n}}\n'
    return text


def llc(path, text, feats):
    with tempfile.NamedTemporaryFile('w', suffix='.ll', delete=False) as f:
        f.write(text)
    try:
        attrs = ','.join('+' + x for x in feats.split(','))
        p = subprocess.run([path, '-O2', '-mtriple=aarch64-linux-gnu', f'-mattr={attrs}', '-o', '-', f.name],
                           capture_output=True, text=True, timeout=120)
    finally:
        os.unlink(f.name)
    if p.returncode != 0:
        return None, (p.stderr.strip().splitlines() or ['?'])[0]
    code, on = [], False
    for line in p.stdout.splitlines():
        if line.startswith('f:'):
            on = True
        elif on and (line.startswith('.Lfunc_end') or line.strip().startswith('.size')):
            break
        elif on:
            s = ' '.join(line.split('//')[0].split())
            if s and not s.startswith('.'):
                code.append(s)
    return code, ''


def compile_one(path, name, ret, args, feats):
    """(constants given, llc's code), or (None, why): the constants, for
    the integer arguments llc wants constant, the first values it takes."""
    ints = [k for k, t in enumerate(args) if re.fullmatch(r'i\d+', t)]
    consts = {}
    why = ''
    for _ in range(len(ints) + 1):
        code, err = llc(path, module(name, ret, args, consts), feats)
        if code is not None:
            return (consts, code), ''
        why = err
        if 'immarg' not in err:
            # (A constant out of range: the next value for the last given.)
            if consts and ('Cannot select' in err or 'range' in err):
                k = max(consts)
                i = CONSTS.index(consts[k])
                if i + 1 < len(CONSTS):
                    consts[k] = CONSTS[i + 1]
                    continue
            break
        free = [k for k in ints if k not in consts]
        if not free:
            break
        consts[free[-1] if len(free) == 1 else free[0]] = CONSTS[0]
    # Values for every constant tried in turn, for an immarg of a range.
    if 'immarg' in why or 'Cannot select' in why:
        cands = [k for k in ints]
        for n in range(1, min(3, len(cands)) + 1):
            for ks in itertools.combinations(cands, n):
                for vals in itertools.product(CONSTS[:6], repeat=n):
                    c = dict(zip(ks, vals))
                    code, err = llc(path, module(name, ret, args, c), feats)
                    if code is not None:
                        return (c, code), ''
    return None, why


# ---------------------------------------------------------------------------
# Registers and 81-scalable.ll

def reg(text):
    """A register's (file, number, kind): ('v', 3, 'z') for z3 or s3."""
    t = text.lower()
    m = re.fullmatch(r'(z|v|q|d|s|h|b)(\d+)', t)
    if m:
        return 'v', int(m.group(2)), 'z' if m.group(1) == 'z' else m.group(1)
    m = re.fullmatch(r'(pn|p)(\d+)', t)
    if m:
        return 'p', int(m.group(2)), m.group(1)
    m = re.fullmatch(r'(x|w)(\d+)', t)
    if m:
        return 'r', int(m.group(2)), m.group(1)
    if t in ('sp', 'wsp'):
        return 'r', 31, 'sp'
    if t in ('xzr', 'wzr'):
        return 'r', 31, 'zr'
    return None


def fits_class(r, cls):
    """Whether register `r` (as `reg` gives it) is one of class `cls`'s."""
    kind = r[2]
    return {'z': cls == 'z', 'p': cls in ('p', 'pg'), 'pn': cls == 'pn', 'x': cls in ('x', 'xsp'),
            'w': cls in ('w', 'wsp'), 'sp': cls in ('xsp', 'wsp'), 'zr': cls in ('x', 'w')}.get(kind, cls == kind)


NUM = r'-?(?:0x[0-9a-f]+|[0-9]+)(?:\.[0-9]+)?'
REG = r'[a-z]+[0-9]*'


def entries():
    """81-scalable.ll's entries by mnemonic: (name, regex, holes, params)."""
    out = {}
    for line in open(os.path.join(HERE, '81-scalable.ll')):
        m = re.match(r'define void @([\w.]+)\((.*?)\) "enc"="[^"]*" "asm"="([^"]*)"', line)
        if not m:
            continue
        name, ptext, asm = m.groups()
        params = {n: (k, c) for k, c, n in re.findall(r'"(reg|imm|sym)"(?:="(\w+)")? %([\w.]+)', ptext)}
        rx, holes = '', []
        for part in re.split(r'(\{[^}]*\})', asm):
            if not part:
                continue
            if part == '{lb}':
                rx += r'\{\s*'
            elif part == '{rb}':
                rx += r'\s*\}'
            elif part.startswith('{='):
                rx += f'({NUM})'
                holes.append(('expr', part[2:-1]))
            elif part.startswith('{'):
                inner = part[1:-1]
                n, _, mod = inner.partition(':')
                kind = params.get(n, ('imm', None))[0]
                rx += f'({REG})' if kind == 'reg' else f'({NUM})'
                holes.append(('op', n, mod))
            else:
                lit = re.escape(part.lower())
                lit = lit.replace(r'\ ', r'\s*').replace(',', r'\s*,\s*').replace(r'\-', r'\s*-\s*')
                rx += lit
        mn = asm.split(' ')[0].lower()
        out.setdefault(mn, []).append((name, re.compile('^' + rx + '$'), holes, params))
    return out


def number(t):
    t = t.lower()
    try:
        return int(t, 16) if t.lstrip('-').startswith('0x') else int(t)
    except ValueError:
        return float(t)


# ---------------------------------------------------------------------------
# Matching

def abi(ret, args, consts):
    """Where AAPCS64 puts each argument, and the result."""
    nx, nv, np = 0, 0, 0
    where = {}
    for k, t in enumerate(args):
        if k in consts:
            continue
        if t.endswith('x i1>'):
            if np >= 4:
                return None
            where[k] = ('p', np)
            np += 1
        elif t.startswith('<') or t in ('float', 'double', 'half', 'bfloat'):
            if nv >= 8:
                return None
            where[k] = ('v', nv)
            nv += 1
        elif t == 'ptr' or re.fullmatch(r'i(8|16|32|64|1)', t):
            if nx >= 8:
                return None
            where[k] = ('r', nx)
            nx += 1
        else:
            return None
    if ret == 'void':
        res = None
    elif ret.endswith('x i1>'):
        res = ('p', 0)
    elif ret.startswith('<') or ret in ('float', 'double', 'half', 'bfloat'):
        res = ('v', 0)
    elif re.fullmatch(r'i\d+', ret):
        res = ('r', 0)
    else:
        return None
    return where, res


def operands(text):
    mn, _, rest = text.partition(' ')
    return mn.lower(), rest.strip()


def move(text):
    """A move between registers: (from, to), or None. movprfx unpredicated
    is one (its next instruction writes over what it moved)."""
    mn, rest = operands(text)
    ops = split_top(rest)
    if mn in ('mov', 'movprfx', 'fmov', 'orr') and len(ops) == 2:
        a, b = [reg(o.split('.')[0].split('/')[0]) for o in ops]
        if a and b and '/' not in rest:
            return b, a
    return None


def match(code, ret, args, consts, ents):
    place = abi(ret, args, consts)
    if place is None:
        return 'skip', 'types'
    where, res = place
    body = [c for c in code if c != 'ret']
    holds = {loc: k for k, loc in where.items()}
    while len(body) > 1 and move(body[0]):
        src, dst = move(body[0])
        holds[dst[:2]] = holds.get(src[:2])
        body = body[1:]
    moved = None
    if len(body) == 2 and res and move(body[1]) and move(body[1])[1][:2] == res:
        moved = move(body[1])[0]
        body = body[:1]
    if len(body) != 1:
        return 'skip', f'{len(body)} instructions'
    text = body[0].lower()
    # (A list of three or four registers, as llc writes it, a range: as
    # the templates write it, each.)
    def listed(m):
        a, b, t = int(m.group(1)), int(m.group(3)), m.group(2)
        return '{ ' + ', '.join(f'z{k % 32}.{t}' for k in range(a, b + (32 if b < a else 0) + 1)) + ' }'
    text = re.sub(r'\{\s*z(\d+)\.(\w+)\s*-\s*z(\d+)\.\w+\s*\}', listed, text)
    mn, _ = operands(text)
    cands = ents.get(mn, [])
    if not cands:
        return 'skip', 'mnemonic not in 81-scalable.ll'
    for name, rx, holes, params in cands:
        m = rx.match(text)
        if not m:
            continue
        vals = {}
        ok = True
        for g, h in zip(m.groups(), holes):
            if h[0] == 'expr':
                continue
            n, mod = h[1], h[2]
            kind, cls = params.get(n, ('imm', None))
            if kind == 'reg':
                r = reg(g)
                if r is None or not fits_class(r, cls):
                    ok = False
                    break
                if mod.startswith('+'):
                    continue
                if n in vals and vals[n] != r:
                    ok = False
                    break
                vals[n] = r
            else:
                v = number(g)
                if n in vals and vals[n] != v:
                    ok = False
                    break
                vals[n] = v
        if not ok:
            continue
        order = [h[1] for h in holes if h[0] == 'op' and not h[2].startswith('+')]
        names = list(dict.fromkeys(order))
        mapping, extra, used = {}, [], set()
        # The destination: the first operand, if it is a register where the
        # result is (or moved from).
        dst = None
        if res and names and params.get(names[0], ('imm',))[0] == 'reg':
            r = vals[names[0]]
            if r[:2] == (moved[:2] if moved else res):
                dst = names[0]
                mapping[dst] = 'd'
                if order.count(dst) > 1:
                    k = holds.get(r[:2])
                    if k is None:
                        return 'skip', f'{name}: written over no argument'
                    extra.append(f'tied=a{k}')
                    used.add(k)
        if res and dst is None:
            continue
        for n in names:
            if n == dst:
                continue
            kind = params[n][0] if n in params else 'imm'
            v = vals[n]
            if kind == 'reg':
                if v[2] == 'zr':
                    mapping[n] = 'zr'
                    continue
                k = holds.get(v[:2])
                if k is None:
                    break
                mapping[n] = f'a{k}'
                used.add(k)
            else:
                ks = [k for k, c in consts.items() if c == v]
                if len(ks) == 1:
                    mapping[n] = f'a{ks[0]}'
                    used.add(ks[0])
                else:
                    mapping[n] = str(v)
        else:
            for k in sorted(set(consts) - used):
                extra.append(f'const=a{k}:{consts[k]}')
                used.add(k)
            # An argument in the destination's register the instruction
            # does not otherwise take is what it writes over (merging, p0/m;
            # SVE2's top halves, addhnt): tied to it.
            if dst and not any(e.startswith('tied=') for e in extra):
                k = holds.get(vals[dst][:2])
                if k is not None and k not in used:
                    extra.append(f'tied=a{k}')
                    used.add(k)
            # Any other the instruction does not take, its result does not
            # depend on (a governing predicate of a form whose inactive
            # lanes are undefined): the entry takes it, unused.
            unused = set(range(len(args))) - used
            for k in sorted(unused):
                extra.append(f'unused=a{k}')
            unused = set()
            if unused:
                return 'skip', 'arguments the instruction does not take: ' + ','.join(map(str, sorted(unused)))
            ops = ' '.join(f'{k}={v}' for k, v in sorted(mapping.items()))
            return 'entry', f'{name} ; {ops} ; {" ".join(extra)}'
    return 'skip', 'no 81-scalable.ll entry is written so'


def main():
    src, llc_path = sys.argv[1:3]
    ents = entries()
    decls = list(declarations(src))

    def one(d):
        name, ret, args, feats = d
        if ret is None or None in args:
            return f'{name} ; ? ; {feats} ; skip ; types Rust declares that are not here (tuples, svcount)'
        got, why = compile_one(llc_path, name, ret, args, feats)
        if got is None:
            return f'{name} ; {ret}({", ".join(args)}) ; {feats} ; skip ; llc: {why[:160]}'
        consts, code = got
        sig = f'{ret}({", ".join(t + (f" imm:{consts[k]}" if k in consts else "") for k, t in enumerate(args))})'
        what, rest = match(code, ret, args, consts, ents)
        return f'{name} ; {sig} ; {feats} ; {what} ; {rest} ; llc: {" / ".join(code)}'
    with ThreadPoolExecutor(16) as ex:
        lines = list(ex.map(one, decls))
    print("# Generated by discover_sve.py (see there): SVE's intrinsics and their instructions.")
    for line in sorted(lines):
        print(line)


if __name__ == '__main__':
    main()
