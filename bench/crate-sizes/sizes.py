#!/usr/bin/env python3
"""Code sizes of the programs build.sh made: Epitome's against rustc's.

    bench/crate-sizes/sizes.py [OUT] [--top N] [--crate NAME] [--all]

OUT is build.sh's directory (default target/crate-sizes). For each crate it
prints the size of the text section of each build, the bytes of the
functions written in the crate and its demo program (the crate's code),
and the functions where Epitome's code is biggest against rustc -O's,
matched by name. --all lists every function of the crate's code, and --mix the
instruction mix of the crate's code (mix.py).

A function's size is the distance to the next symbol in the text section
(alignment padding included, as for both compilers). Epitome's functions
are put down to crates by `epitome-rustc2 --fn-crates` (ep-*.crates);
rustc's by the first crate root in their v0 symbol (the crate the item, or
its impl, is written in). Names are matched without paths, generic
arguments and hashes: `<Buffer>::format`, `<f32 as Sealed>::is_nonfinite`,
`format64`, `main::{closure}`; generic instances of one function add up.
"""
import os
import re
import subprocess
import sys

ARGS = [a for a in sys.argv[1:]]


def opt(name, default=None):
    if name in ARGS:
        i = ARGS.index(name)
        v = ARGS[i + 1]
        del ARGS[i:i + 2]
        return v
    return default


TOP = int(opt('--top', '12'))
ONLY = opt('--crate')
ALL = '--all' in ARGS
MIX = '--mix' in ARGS
ARGS = [a for a in ARGS if a not in ('--all', '--mix')]
HERE = os.path.dirname(os.path.abspath(__file__))
OUT = ARGS[0] if ARGS else os.path.join(HERE, '..', '..', 'target', 'crate-sizes')
DEMANGLE = os.path.join(OUT, 'tools', 'demangle')


def run(*cmd, inp=None):
    return subprocess.run(cmd, input=inp, capture_output=True, text=True, check=True).stdout


def text_section(exe):
    """(address, size) of the text section."""
    for line in run('objdump', '-h', exe).splitlines():
        f = line.split()
        if len(f) >= 4 and f[1] in ('__text', '.text'):
            return int(f[3], 16), int(f[2], 16)
    raise SystemExit(f'{exe}: no text section')


def functions(exe):
    """{symbol: size} of the text section's symbols."""
    lo, size = text_section(exe)
    hi = lo + size
    syms = {}
    for line in run('nm', '-n', '--defined-only', exe).splitlines():
        f = line.split(None, 2)
        if len(f) != 3 or f[1] not in 'tT':
            continue
        a = int(f[0], 16)
        if lo <= a < hi:
            syms.setdefault(a, f[2])  # the first of aliases
    addrs = sorted(syms)
    out = {}
    for i, a in enumerate(addrs):
        nxt = addrs[i + 1] if i + 1 < len(addrs) else hi
        name = syms[a]
        if sys.platform == 'darwin' and name.startswith('_'):
            name = name[1:]
        out[name] = out.get(name, 0) + nxt - a
    return out


def demangle(names):
    if not names:
        return {}
    if not os.access(DEMANGLE, os.X_OK):
        return {n: n for n in names}
    lines = run(DEMANGLE, inp='\n'.join(names) + '\n').splitlines()
    return dict(zip(names, lines))


def strip_generics(s):
    """Drops generic arguments: `<` after an identifier or `::`."""
    out = []
    i = 0
    while i < len(s):
        c = s[i]
        prev = ''.join(out).rstrip()
        if c == '<' and (prev[-1:].isalnum() or prev[-1:] in ('_', ':', ']')) and not prev.endswith(' as'):
            if prev.endswith('::'):
                del out[len(out) - 2:]
            depth = 0
            while i < len(s):
                if s[i] == '<':
                    depth += 1
                elif s[i] == '>' and s[i - 1] != '-':
                    depth -= 1
                    if depth == 0:
                        break
                i += 1
            i += 1
            continue
        out.append(c)
        i += 1
    return ''.join(out)


def key(title):
    """The name functions are matched by (see the module's comment)."""
    t = title
    m = re.match(r'closure at node \d+ in (.*)', t)
    closure = bool(m)
    if m:
        t = m.group(1)
    m = re.match(r'fn (.*) \([^()]*\)$', t)
    if m:
        t = m.group(1)
    if closure:
        t += '::{closure}'
    t = re.sub(r'\{closure#\d+\}', '{closure}', t)
    t = re.sub(r'\.(llvm\.\d+|cold(\.\d+)?)$', '', t)
    t = strip_generics(t)
    t = re.sub(r'\b[A-Za-z_]\w*::(?=[A-Za-z_<])', '', t)
    return t


V0_CRATE = re.compile(r'C(?:s[0-9A-Za-z]*_)?(\d+)')


def rustc_crate(sym):
    """The crate a v0 symbol's item is written in (the first crate root);
    for a legacy symbol (the precompiled std's), its first path segment."""
    s = sym.lstrip('_')
    if s.startswith('R'):
        m = V0_CRATE.search(s)
        if m:
            n = int(m.group(1))
            return s[m.end():m.end() + n]
    if s.startswith('ZN'):
        m = re.match(r'ZN(\d+)', s)
        if m:
            return s[m.end():m.end() + int(m.group(1))]
    return '?'


HASH = re.compile(r'^(.*?\.[0-9a-f]{16})(\..*)?$')


def ours(exe, crates_file):
    """[(crate, key, size)] of an Epitome build."""
    info = {}
    if os.path.exists(crates_file):
        for line in open(crates_file):
            f = line.rstrip('\n').split('\t')
            if len(f) >= 3:
                info[f[0]] = (f[1], f[2])
    rows = []
    for sym, size in functions(exe).items():
        base = sym
        if base not in info:
            m = HASH.match(sym)
            if m:
                base = m.group(1)
        if base in info:
            krate, title = info[base]
            rows.append((krate, key(title), size, sym))
        else:
            rows.append(('?', sym, size, sym))
    return rows


def theirs(exe):
    fs = functions(exe)
    names = demangle(list(fs))
    return [(rustc_crate(s), key(names[s]), size, s) for s, size in fs.items()]


def kb(n):
    return f'{n / 1024:.1f} KB'


def crates():
    for line in open(os.path.join(HERE, 'crates.txt')):
        f = line.split()
        if f and not f[0].startswith('#'):
            yield f[0]


def status():
    st = {}
    p = os.path.join(OUT, 'status.txt')
    if os.path.exists(p):
        for line in open(p):
            f = line.split(None, 2)
            if len(f) >= 3:
                st[(f[0], f[1])] = f[2].strip()
    return st


MIXES = {}


def main():
    st = status()
    summary = []
    hello = {}
    for t in ('ep-default', 'ep-ndebug', 'ep-abort', 'rustc-O3', 'rustc-Os'):
        e = os.path.join(OUT, 'hello', t + '.exe')
        if os.path.exists(e):
            hello[t] = text_section(e)[1]
    for name in crates():
        if ONLY and name != ONLY:
            continue
        d = os.path.join(OUT, name)
        exes = {t: os.path.join(d, t + '.exe') for t in ('ep-default', 'ep-ndebug', 'ep-abort', 'rustc-O3', 'rustc-Os')}
        if not all(os.path.exists(e) for e in exes.values()):
            print(f'## {name}\n\nnot built: ' + '; '.join(f'{t} {st.get((name, t), "missing")}' for t in exes) + '\n')
            continue
        own = {name, name + '_demo'}
        texts = {t: text_section(e)[1] for t, e in exes.items()}
        rows = {t: (ours(e, os.path.join(d, t + '.crates')) if t.startswith('ep') else theirs(e)) for t, e in exes.items()}
        code = {t: sum(r[2] for r in rs if r[0] in own) for t, rs in rows.items()}
        runs = st.get((name, 'ep-ndebug'), '?')
        print(f'## {name}\n')
        print(f'epitome-rustc2 output: {runs}\n')
        print('| | ep default | ep ndebug | ep immediate-abort | rustc -O | rustc -Os | ndebug / rustc -O |')
        print('|---|---|---|---|---|---|---|')
        print('| text | ' + ' | '.join(kb(texts[t]) for t in exes) + f' | {texts["ep-ndebug"] / texts["rustc-O3"]:.2f} |')
        print("| crate's code | " + ' | '.join(f'{code[t]:,}' for t in exes) +
              f' | {code["ep-ndebug"] / max(code["rustc-O3"], 1):.2f} |')
        if len(hello) == len(exes):
            over = {t: texts[t] - hello[t] for t in exes}
            print("| text over hello world | " + ' | '.join(f'{over[t]:,}' for t in exes) +
                  f' | {over["ep-ndebug"] / max(over["rustc-O3"], 1):.2f} |')
            texts.update({t + '+': over[t] for t in exes})
        print()
        summary.append((name, runs, texts, code))
        if MIX:
            sys.path.insert(0, HERE)
            import mix
            print('| | ' + ' | '.join(mix.KEYS) + ' |')
            print('|---|' + '---|' * len(mix.KEYS))
            for t in ('ep-ndebug', 'rustc-O3'):
                dis = mix.disassemble(exes[t])
                tot = dict.fromkeys(mix.KEYS, 0)
                for krate, _, _, sym in rows[t]:
                    if krate in own and sym in dis:
                        for k, v in mix.classify([i for _, i in dis[sym]]).items():
                            tot[k] += v
                print(f'| {t} | ' + ' | '.join(f'{tot[k]:,}' for k in mix.KEYS) + ' |')
                MIXES.setdefault(t, dict.fromkeys(mix.KEYS, 0))
                for k in mix.KEYS:
                    MIXES[t][k] += tot[k]
            print()
        # By function: ours (ndebug) against rustc -O.
        a, b = {}, {}
        for k, rs in (('ep-ndebug', a), ('rustc-O3', b)):
            for krate, fk, size, _ in rows[k]:
                if krate in own:
                    rs[fk] = rs.get(fk, 0) + size
        keys = sorted(set(a) | set(b), key=lambda k: -(a.get(k, 0) - b.get(k, 0)))
        if not ALL:
            keys = keys[:TOP]
        print('| function | ep | rustc -O | ep - rustc |')
        print('|---|---|---|---|')
        for k in keys:
            x, y = a.get(k), b.get(k)
            fmt = lambda v: '—' if v is None else f'{v:,}'
            print(f'| `{k}` | {fmt(x)} | {fmt(y)} | {(x or 0) - (y or 0):+,} |')
        print()
    if len(summary) > 1:
        print('## Summary\n')
        if hello:
            print(f'Hello world\'s text: ' + ', '.join(f'{t} {v:,}' for t, v in hello.items()) + '.\n')
        print("| crate | runs | text over hello: ep ndebug | rustc -O | crate's code: ep default | ep ndebug | ep abort | rustc -O | rustc -Os | ndebug / rustc -O |")
        print('|---|---|---|---|---|---|---|---|---|---|')
        tot = {}
        for name, runs, t, c in summary:
            print(f'| {name} | {runs} | {t.get("ep-ndebug+", 0):,} | {t.get("rustc-O3+", 0):,} | '
                  f'{c["ep-default"]:,} | {c["ep-ndebug"]:,} | {c["ep-abort"]:,} | {c["rustc-O3"]:,} | {c["rustc-Os"]:,} | '
                  f'{c["ep-ndebug"] / max(c["rustc-O3"], 1):.2f} |')
            for k, v in c.items():
                tot[k] = tot.get(k, 0) + v
        print(f'| all | | | | {tot["ep-default"]:,} | {tot["ep-ndebug"]:,} | {tot["ep-abort"]:,} | {tot["rustc-O3"]:,} | '
              f'{tot["rustc-Os"]:,} | {tot["ep-ndebug"] / max(tot["rustc-O3"], 1):.2f} |')
        print()
    if MIXES:
        import mix
        print("Instruction mix of the crates' code, all crates:\n")
        print('| | ' + ' | '.join(mix.KEYS) + ' |')
        print('|---|' + '---|' * len(mix.KEYS))
        for t, tot in MIXES.items():
            print(f'| {t} | ' + ' | '.join(f'{tot[k]:,}' for k in mix.KEYS) + ' |')
        print()


main()
