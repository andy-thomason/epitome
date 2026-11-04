#!/usr/bin/env python3
"""Finds the instruction for each of x86's own intrinsics (llvm.x86.*), for
gen_sel.py's intrinsics(): writes intrinsics.txt.

The intrinsics, their types and the features they need are read from
Rust's core::arch (stdarch's x86 and x86_64 sources: each `link_name`
extern, and the `target_feature` of the first function calling it). The
instruction is what LLVM's llc makes of a function passing its arguments
to the intrinsic (a black box: its arguments arrive in the C calling
convention's registers, so the registers of its code say which operand
each is). The arguments llc wants constant (immarg) are given 4. The
instruction is then found among 02-xed.ll's entries, and Intel's XED
(whose tables 02-xed.ll is made from) says which of its operands it
writes, reads or both, and which registers it uses unnamed.

Rust declares some intrinsics with its own types where LLVM's differ
(masks as integers, BF16 as i16); those are tried as vectors of i1 and
of bfloat when llc says the types are wrong.

Each line of intrinsics.txt is one intrinsic, ` ; ` between fields:

    NAME ; SIGNATURE ; FEATURES ; WHAT
    WHAT = entry ; TEMPLATE ; CLASSES ; OPERANDS ; EXTRA ; llc: INSTRUCTION
         | skip ; REASON ; llc: INSTRUCTIONS

SIGNATURE is `RET(ARG, ...)`, an argument `T imm` when it is a constant.
TEMPLATE and CLASSES are those of 02-xed.ll's entry (as gen_sel.py's
xed() looks one up); OPERANDS names its operands for the intrinsic's
(`o0=a1 o1=d c=a2`: argument 1, the result, argument 2), and EXTRA is
`tied=aK` (the result is written over argument K) and `flags` (it
changes the flags), space-separated.

Usage: discover_intrinsics.py CORE_ARCH_SRC LLC XED_DIR > intrinsics.txt
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

RUST_TYPES = {
    '__m128i': '<2 x i64>', '__m256i': '<4 x i64>', '__m512i': '<8 x i64>',
    '__m128': '<4 x float>', '__m256': '<8 x float>', '__m512': '<16 x float>',
    '__m128d': '<2 x double>', '__m256d': '<4 x double>', '__m512d': '<8 x double>',
    '__m128h': '<8 x half>', '__m256h': '<16 x half>', '__m512h': '<32 x half>',
    '__m128bh': '<8 x bfloat>', '__m256bh': '<16 x bfloat>', '__m512bh': '<32 x bfloat>',
    '__mmask8': 'i8', '__mmask16': 'i16', '__mmask32': 'i32', '__mmask64': 'i64',
    'f16': 'half', 'f32': 'float', 'f64': 'double', 'bf16': 'bfloat', 'bool': 'i1', '()': 'void',
}
ELEMS = {'f16': 'half', 'f32': 'float', 'f64': 'double', 'bf16': 'bfloat'}
# Rust's names of features, where LLVM's differ.
FEATURES = {'pclmulqdq': 'pclmul', 'rdrand': 'rdrnd', 'bmi1': 'bmi', 'cmpxchg16b': 'cx16'}


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


def ir_type(t):
    t = t.strip()
    if t in RUST_TYPES:
        return RUST_TYPES[t]
    if t.startswith('*'):
        return 'ptr'
    m = re.fullmatch(r'[iu](\d+)', t)
    if m:
        return f'i{m.group(1)}'
    m = re.fullmatch(r'([iuf]\d+|bf16)x(\d+)', t)
    if m:
        e, n = m.groups()
        return f'<{n} x {ELEMS.get(e, "i" + e[1:])}>'
    if t.startswith('(') and t.endswith(')'):
        return '{ ' + ', '.join(ir_type(x) for x in split_top(t[1:-1])) + ' }'
    return None


def declarations(src):
    """(name, ret, args, features) of each intrinsic Rust declares, in the
    order its sources do."""
    files = sorted(glob.glob(f'{src}/x86/*.rs') + glob.glob(f'{src}/x86_64/*.rs'))
    externs, callers = [], []
    for f in files:
        s = open(f).read()
        for blk in re.finditer(r'extern "unadjusted" \{(.*?)\n\}', s, re.S):
            for m in re.finditer(r'#\[link_name = "(llvm\.x86\.[^"]+)"\]\s*(?:#\[[^\]]*\]\s*)*(?:pub )?(?:safe |unsafe )?'
                                 r'fn (\w+)\s*\((.*?)\)\s*(?:->\s*([^;]+))?;', blk.group(1), re.S):
                args = [ir_type(a.split(':', 1)[1]) for a in split_top(m.group(3))]
                externs.append((m.group(1), m.group(2), args, ir_type(m.group(4) or '()'), f))
        for m in re.finditer(r'((?:#\[(?:[^\[\]]|\[[^\]]*\])*\]|//[^\n]*\n|\s)*)pub (?:const )?(?:unsafe )?fn \w+[^{]*\{'
                             r'(.*?)\n\}\n', s, re.S):
            callers.append((m.group(1), m.group(2), f))
    seen = set()
    for name, rust, args, ret, home in externs:
        if name in seen:
            continue
        seen.add(name)
        feats = ''
        # (Its own file's callers first: avx2.rs calls SSE2's too.)
        for attrs, body, _ in sorted(callers, key=lambda c: c[2] != home):
            if re.search(rf'\b{rust}\s*\(', body):
                fm = re.search(r'target_feature\(enable = "([^"]+)"\)', attrs)
                feats = ','.join(FEATURES.get(x, x) for x in fm.group(1).split(',')) if fm else ''
                break
        yield name, ret, args, feats


# ---------------------------------------------------------------------------
# llc

def lanes(t):
    m = re.match(r'<(\d+) x', t or '')
    return int(m.group(1)) if m else None


def variants(ret, args):
    """The signature, then with LLVM's types where Rust's may differ: a
    mask (the result, or the last argument) as a vector of i1 (as many as
    its bits, or a vector's lanes), BF16 as bfloat."""
    bf = lambda t: re.sub(r'<(\d+) x i16>', r'<\1 x bfloat>', t)
    vl = sorted({lanes(a) for a in args + [ret] if lanes(a)})

    def masks(t):
        m = re.fullmatch(r'i(8|16|32|64)', t)
        return [t] + ([f'<{k} x i1>' for k in [int(m.group(1))] + vl if k <= int(m.group(1))] if m else [])
    seen = set()
    for r in masks(ret):
        for f in (lambda t: t, bf):
            for last_mask in (False, True):
                a = [f(x) for x in args]
                if last_mask and len(a) > 1 and len(masks(a[-1])) > 1:
                    a[-1] = masks(a[-1])[-1]
                key = (f(r), tuple(a))
                if key not in seen:
                    seen.add(key)
                    yield key


def module(name, ret, args, consts):
    params, call = [], []
    for k, a in enumerate(args):
        if k in consts:
            call.append(f'{a} 4')
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
        cmd = [path, '-O2', '-mtriple=x86_64-unknown-linux-gnu', '-mcpu=x86-64', '-o', '-', f.name]
        if feats:
            cmd.insert(3, '-mattr=' + ','.join('+' + x for x in feats.split(',')))
        p = subprocess.run(cmd, capture_output=True, text=True, timeout=120)
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
            s = ' '.join(line.split('#')[0].split())
            if s and not s.startswith('.'):
                code.append(s)
    return code, ''


def compile_one(path, name, ret, args, feats):
    """(ret, args, constant arguments, llc's code) of the first signature
    and choice of constants llc builds, or (None, why)."""
    why = ''
    for r, a in variants(ret, list(args)):
        a = list(a)
        ints = [k for k, t in enumerate(a) if re.fullmatch(r'i\d+', t)]
        for n in range(4):
            for consts in itertools.combinations(ints, n):
                code, err = llc(path, module(name, r, a, set(consts)), feats)
                if code is not None:
                    return (r, a, consts, code), ''
                why = err
                if 'immarg' not in err:
                    break
            else:
                continue
            break
        if 'type' not in why and 'signature' not in why:
            break
    return None, why


# ---------------------------------------------------------------------------
# 02-xed.ll and XED

GPRS = [['rax', 'eax', 'ax', 'al'], ['rcx', 'ecx', 'cx', 'cl'], ['rdx', 'edx', 'dx', 'dl'], ['rbx', 'ebx', 'bx', 'bl'],
        ['rsi', 'esi', 'si', 'sil'], ['rdi', 'edi', 'di', 'dil'], ['rbp', 'ebp', 'bp', 'bpl'], ['rsp', 'esp', 'sp', 'spl']] + \
       [[f'r{k}', f'r{k}d', f'r{k}w', f'r{k}b'] for k in range(8, 16)]
GPR_CLASS = ['r64', 'r32', 'r16', 'r8']


def reg(name):
    """A register's (file, number, class): ('v', 3, 'x') for %xmm3."""
    m = re.fullmatch(r'%([xyz])mm(\d+)', name)
    if m:
        return 'v', int(m.group(2)), m.group(1)
    for k, names in enumerate(GPRS):
        if name[1:] in names:
            return 'r', k, GPR_CLASS[names.index(name[1:])]
    return None


def abi(ret, args, consts):
    """Where the C calling convention puts each argument (by index), and the
    result."""
    ints = [5, 4, 2, 1, 8, 9]   # rdi, rsi, rdx, rcx, r8, r9
    vecs = 0
    where = {}
    for k, t in enumerate(args):
        if k in consts:
            continue
        if t.startswith('<') or t in ('float', 'double', 'half', 'bfloat'):
            if 'i1>' in t:
                return None
            where[k] = ('v', vecs)
            vecs += 1
        elif t == 'ptr' or re.fullmatch(r'i(8|16|32|64)', t):
            where[k] = ('r', ints.pop(0))
        else:
            return None
    if ret == 'void':
        res = None
    elif ret.startswith('<') and 'i1>' not in ret or ret in ('float', 'double', 'half', 'bfloat'):
        res = ('v', 0)
    elif re.fullmatch(r'i(8|16|32|64)', ret):
        res = ('r', 0)
    else:
        return None
    return where, res


def operands(text):
    """An instruction's mnemonic and operands."""
    mn, _, rest = text.partition(' ')
    return mn, split_top(rest)


def xed_entries():
    """02-xed.ll's entries by mnemonic: (name, template operands, params)."""
    out = {}
    for line in open(os.path.join(HERE, '02-xed.ll')):
        m = re.match(r'define void @(\w+)\((.*?)\) "enc"="[^"]*" "asm"="([^"]*)"', line)
        if m:
            mn, ops = operands(m.group(3))
            params = {n: c for _, c, n in re.findall(r'"(reg|imm|sym)"(?:="(\w+)")? %([\w.]+)', m.group(2))}
            out.setdefault(mn, []).append((m.group(1), m.group(3), ops, params))
    return out


def xed_forms(xed_dir):
    sys.path.insert(0, HERE)
    import gen_xed
    forms = {}
    for r, n, f in gen_xed.all_forms(xed_dir):
        expl, impl = [], []
        for t in f['OPERANDS'].split():
            o = gen_xed.Op(t)
            acc = next((a for a in o.attrs if a in ('r', 'w', 'rw', 'cr', 'cw', 'rcw', 'crw')), '')
            (impl if o.supp or o.impl else expl).append((o.val or o.slot, acc))
        forms[n] = (expl, impl)
    return forms


# Registers an instruction may use unnamed without its entry saying so
# (beyond the flags, which it says).
IGNORED = {'XED_REG_MXCSR', 'XED_REG_XCR0', 'XED_REG_TSC', 'XED_REG_TSCAUX'}
FLAGS = {'XED_REG_RFLAGS', 'XED_REG_EFLAGS', 'rFLAGS()', 'XED_REG_FLAGS'}
CLASSES = {'x': ('x', 'xe'), 'y': ('y', 'ye'), 'z': ('z',), 'r64': ('r64',), 'r32': ('r32',), 'r16': ('r16',),
           'r8': ('r8',)}


def match(code, ret, args, consts, entries, forms):
    """The line for an intrinsic llc compiles to `code`."""
    place = abi(ret, args, consts)
    if place is None:
        return 'skip', 'types'
    where, res = place
    body = [c for c in code if c not in ('retq', 'vzeroupper')]
    # The result moved to where it is returned.
    moved = None
    if len(body) == 2 and res:
        mn, ops = operands(body[1])
        if re.fullmatch(r'v?mov(aps|apd|ups|dqa|dqa64|dqu|q|d|l)', mn) and len(ops) == 2 and reg(ops[1]) \
                and reg(ops[1])[:2] == res:
            moved = reg(ops[0])
            body = body[:1]
    if len(body) != 1:
        return 'skip', f'{len(body)} instructions'
    mn, ops = operands(body[0].removeprefix('{vex} '))
    if any('{' in o for o in ops):
        return 'skip', 'masks, rounding or broadcasts'
    if mn not in entries:
        return 'skip', 'mnemonic not in 02-xed.ll'
    const_args = list(consts)
    for name, tmpl, tops, params in entries[mn]:
        if len(tops) != len(ops):
            continue
        names, ok = {}, True
        for o, t in zip(ops, tops):
            if t.startswith('%') or t.startswith('$') and not t.startswith('${'):
                ok = o == t
            elif t.startswith('{o') and t.endswith('}'):
                r = reg(o)
                ok = r is not None and params.get(t[1:-1]) in CLASSES.get(r[2], ())
                names[t[1:-1]] = r
            elif t.startswith('${'):
                ok = o.startswith('$')
                names[t[2:-1]] = 'imm'
            elif t == '{m.disp}({m.base})':
                m = re.fullmatch(r'\((%\w+)\)', o)
                ok = m is not None and reg(m.group(1)) is not None
                if ok:
                    names['m.base'] = reg(m.group(1))
                    names['m.disp'] = 'zero'
            else:
                ok = False
            if not ok:
                break
        if not ok:
            continue
        # Intel's order is AT&T's reversed.
        n = int(re.search(r'_(\d+)(?:_[a-z]+)*$', name).group(1))
        expl, impl = forms[n]
        # (An unmasked form's mask, k0, is not written.)
        if name.endswith('_u'):
            expl = [(v, a) for v, a in expl if 'MASK' not in v]
        if len(expl) != len(tops):
            return 'skip', f'{name}: XED operands'
        access = list(reversed([a for _, a in expl]))
        extra = []
        for r, a in impl:
            if r in FLAGS or 'FLAGS' in r:
                if 'w' in a:
                    extra.append('flags')
            elif r not in IGNORED:
                return 'skip', f'{name}: uses {r} unnamed'
        mapping = {}
        dst = None
        if res:
            last = tops[-1]
            if not (last.startswith('{o') and 'w' in access[-1]):
                return 'skip', f'{name}: no register result'
            dst = last[1:-1]
            r = names[dst]
            if r[:2] != (moved[:2] if moved else res):
                return 'skip', f'{name}: result elsewhere'
            mapping[dst] = 'd'
            if 'r' in access[-1]:
                tied = [k for k, w in where.items() if w == r[:2]]
                if len(tied) != 1:
                    return 'skip', f'{name}: written over no argument'
                extra.insert(0, f'tied=a{tied[0]}')
        imms = iter(const_args)
        used = set()
        for t, r in names.items():
            if t == dst:
                continue
            if r == 'zero':
                mapping[t] = '0'
            elif r == 'imm':
                k = next(imms, None)
                if k is None:
                    return 'skip', f'{name}: immediates'
                mapping[t] = f'a{k}'
                used.add(k)
            else:
                k = [k for k, w in where.items() if w == r[:2]]
                if len(k) != 1:
                    return 'skip', f'{name}: an operand no argument is in'
                mapping[t] = f'a{k[0]}'
                used.add(k[0])
                if 'w' in access[tops.index('{' + t + '}')] if '{' + t + '}' in tops else False:
                    return 'skip', f'{name}: writes an argument'
        if res is None and any('w' in a for a, t in zip(access, tops) if t.startswith('{o')):
            return 'skip', f'{name}: writes a register'
        tied = [e for e in extra if e.startswith('tied=')]
        unused = set(range(len(args))) - used - {int(t[6:]) for t in tied}
        if unused:
            return 'skip', 'arguments the instruction does not take: ' + ','.join(map(str, sorted(unused)))
        classes = ' '.join(params[t[1:-1]] for t in tops if t.startswith('{o'))
        ops_s = ' '.join(f'{k}={v}' for k, v in sorted(mapping.items()))
        return 'entry', f'{tmpl} ; {classes} ; {ops_s} ; {" ".join(extra)}'
    return 'skip', 'no 02-xed.ll entry is written so'


def main():
    src, llc_path, xed_dir = sys.argv[1:4]
    entries = xed_entries()
    forms = xed_forms(xed_dir)
    decls = list(declarations(src))

    def one(d):
        name, ret, args, feats = d
        if ret is None or None in args:
            return f'{name} ; ? ; {feats} ; skip ; Rust\'s types for it are not LLVM\'s (AMX tiles, Key Locker)'
        # (No feature: x86-64's own, SSE2.)
        feats = feats or 'sse2'
        got, why = compile_one(llc_path, name, ret, args, feats)
        if got is None:
            return f'{name} ; {ret}({", ".join(args)}) ; {feats} ; skip ; llc: {why[:160]}'
        r, a, consts, code = got
        sig = f'{r}({", ".join(t + (" imm" if k in consts else "") for k, t in enumerate(a))})'
        what, rest = match(code, r, a, consts, entries, forms)
        return f'{name} ; {sig} ; {feats} ; {what} ; {rest} ; llc: {" / ".join(code)}'
    with ThreadPoolExecutor(16) as ex:
        lines = list(ex.map(one, decls))
    print('# Generated by discover_intrinsics.py (see there): x86\'s intrinsics and their instructions.')
    for line in sorted(lines):
        print(line)


if __name__ == '__main__':
    main()
