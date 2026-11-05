#!/usr/bin/env python3
"""Finds the instruction for each of SME's intrinsics (llvm.aarch64.sme.*),
for gen_sve.py's intrinsics(): writes sme-intrinsics.txt, in
discover_sve.py's format (see there), whose matching it uses.

Rust's core::arch has no SME, so the intrinsics are found from C's (Arm's
ACLE, `arm_sme.h`), with clang as a black box: its AST dump lists the
functions the header declares and their prototypes; each is called from a
streaming function sharing ZA (and ZT0), its arguments the wrapper's (the
integer ones llc wants constant given 0, or the least of the range clang
says), and the IR clang makes of the call says which LLVM intrinsic it is,
of what types. The features are those clang asks for. Then, as for SVE,
llc's code of a function passing its arguments to the intrinsic (made
streaming and sharing ZA, as llc wants) is found among 81-scalable.ll's
entries.

Usage: discover_sme.py CLANG LLC > sme-intrinsics.txt (clang's own llc: llc-22 for clang 22)
"""
import os
import re
import subprocess
import sys
import tempfile
from concurrent.futures import ThreadPoolExecutor

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import discover_sve as ds  # noqa: E402

TARGET = '--target=aarch64-unknown-linux-gnu'
BASE = ['sme', 'sve2']


def acle_functions(clang):
    """(name, ret, params) of each function arm_sme.h declares, as C types."""
    with tempfile.TemporaryDirectory() as d:
        c = os.path.join(d, 'decls.c')
        open(c, 'w').write('#include <arm_sme.h>\n')
        p = subprocess.run([clang, TARGET, '-march=armv9-a+sme2', '-fsyntax-only', '-Xclang', '-ast-dump',
                            '-Xclang', '-ast-dump-filter', '-Xclang', 'sv', c],
                           capture_output=True, text=True, timeout=600)
    out = []
    for line in p.stdout.splitlines():
        m = re.search(r"FunctionDecl 0x[0-9a-f]+ <([^>]*)> \S+ (\w+) '([^(']*)\((.*?)\)([^']*)'", line)
        if not m:
            continue
        name, ret, params = m.group(2), m.group(3).strip(), m.group(4)
        # (Of ZA, ZT0 and streaming mode's lengths: the rest are SVE's.)
        if not ('za' in name or 'zt' in name or name.startswith('svcnts')):
            continue
        params = [] if params.strip() in ('', 'void') else [x.strip() for x in params.split(',')]
        out.append((name, ret, params))
    return out


def wrapper(name, ret, params, consts):
    """C of a streaming function sharing ZA and ZT0 that calls `name`."""
    ps, args = [], []
    for k, t in enumerate(params):
        if k in consts:
            args.append(str(consts[k]))
        else:
            ps.append(f'{t} a{k}')
            args.append(f'a{k}')
    body = f'return {name}({", ".join(args)});' if ret != 'void' else f'{name}({", ".join(args)});'
    zt0 = ' __arm_inout("zt0")' if 'zt' in name or 'luti' in name else ''
    return ('#include <arm_sme.h>\n#include <arm_sve.h>\n'
            f'{ret} w({", ".join(ps) or "void"}) __arm_streaming __arm_inout("za"){zt0} {{ {body} }}\n')


def clang_ir(clang, text, feats):
    with tempfile.TemporaryDirectory() as d:
        c = os.path.join(d, 'w.c')
        open(c, 'w').write(text)
        p = subprocess.run([clang, TARGET, '-march=armv9-a+' + '+'.join(feats), '-O1', '-S', '-emit-llvm', c, '-o', '-'],
                           capture_output=True, text=True, timeout=120)
    return p.returncode == 0, p.stdout, p.stderr


def lower(clang, fn):
    """The LLVM intrinsics clang calls for `fn`, with their signatures and
    the features: [(name, ret, args, feats)], or a reason."""
    name, ret, params = fn
    ints = [k for k, t in enumerate(params) if re.fullmatch(r'u?int\d+_t|unsigned|int|uint\d+_t|int\d+_t', t)]
    consts, feats = {}, list(BASE)
    for _ in range(3 * len(params) + 6):
        ok, ir, err = clang_ir(clang, wrapper(name, ret, params, consts), feats)
        if ok:
            break
        need = re.findall(r"requires (?:target feature )?'([^']+)'", err) + \
            re.findall(r"needs target feature ([\w,.-]+)", err)
        if need:
            new = [x for f in need for x in f.split(',') if x not in feats]
            if not new:
                return None, 'features'
            feats += new
            continue
        rng = re.search(r'outside the valid range \[(-?\d+), (-?\d+)\]', err)
        if rng and consts:
            k = max(consts)
            consts[k] = int(rng.group(1))
            continue
        if 'must be a constant integer' in err or 'constant' in err:
            free = [k for k in ints if k not in consts]
            if not free:
                return None, (err.strip().splitlines() or ['?'])[0][:120]
            consts[free[0]] = 0
            continue
        return None, (err.strip().splitlines() or ['?'])[0][:120]
    else:
        return None, 'no form clang takes'
    out = []
    for m in re.finditer(r'^declare (.*?)@(llvm\.aarch64\.sme\.[\w.]+)\((.*)\)', ir, re.M):
        rty = m.group(1).split()[-1] if m.group(1).strip() else 'void'
        rty = re.sub(r'^.*?(<[^>]*>|\{.*\}|\S+)\s*$', r'\1', m.group(1).strip())
        args = [re.sub(r'\s+(immarg|noundef|nocapture|readonly|writeonly|captures\(none\))', '', a).strip()
                for a in ds.split_top(m.group(3))]
        out.append((m.group(2), rty, args, ','.join(feats)))
    return out, ''


def module(name, ret, args, consts):
    """discover_sve's module, the function made streaming and sharing ZA
    and ZT0, as llc wants for SME's intrinsics."""
    text = MODULE(name, ret, args, consts)
    text = text.replace(') {\n', ') #0 {\n', 1)
    return text + 'attributes #0 = { "aarch64_pstate_sm_enabled" "aarch64_inout_za" "aarch64_inout_zt0" }\n'


MODULE = ds.module
ds.module = module



def main():
    clang, llc_path = sys.argv[1:3]
    ents = ds.entries()
    fns = acle_functions(clang)
    with ThreadPoolExecutor(16) as ex:
        lowered = list(ex.map(lambda f: lower(clang, f), fns))
    decls, failed = {}, []
    for fn, (got, why) in zip(fns, lowered):
        if got is None:
            failed.append((fn[0], why))
            continue
        for d in got:
            if d[0] not in decls or len(d[3]) < len(decls[d[0]][3]):
                decls[d[0]] = d

    def one(d):
        name, ret, args, feats = d
        if any(a.startswith('{') for a in args) or ret.startswith('{'):
            return f'{name} ; {ret}({", ".join(args)}) ; {feats} ; skip ; tuples of vectors'
        got, why = ds.compile_one(llc_path, name, ret, args, feats)
        if got is None:
            return f'{name} ; {ret}({", ".join(args)}) ; {feats} ; skip ; llc: {why[:160]}'
        consts, code = got
        sig = f'{ret}({", ".join(t + (f" imm:{consts[k]}" if k in consts else "") for k, t in enumerate(args))})'
        what, rest = ds.match(code, ret, args, consts, ents)
        return f'{name} ; {sig} ; {feats} ; {what} ; {rest} ; llc: {" / ".join(code)}'
    with ThreadPoolExecutor(16) as ex:
        lines = list(ex.map(one, sorted(decls.values())))
    print("# Generated by discover_sme.py (see there): SME's intrinsics and their instructions.")
    print(f"# {len(fns)} ACLE functions, {len(failed)} clang would not call as written here.")
    for line in sorted(lines):
        print(line)


if __name__ == '__main__':
    main()
