//! Every intrinsic, from catalogues: `tests/intrinsics/generic.txt` (LLVM's
//! generic intrinsics), `tests/intrinsics/aarch64.txt` (AArch64's own) and
//! `tests/intrinsics/x86.txt` (x86's own). Each line of one says what is
//! done with an intrinsic:
//!
//! - `test ; NAME ; SIGNATURE ; TYPES ; FEATURES`: for each of the types
//!   (none, when the signature has none to fill in), a function
//!   that loads the arguments, calls it and stores what it gives, built by
//!   us and by LLVM's `llc` (the one rustup's llvm-tools has) and run, on
//!   random and special values (`intrinsics/driver.c`); the two must
//!   store the same.
//! - `build ; NAME ; SIGNATURE ; TYPES`: built by both, not run (traps,
//!   what reads the machine's state: frames, counters, registers).
//! - `skip ; NAME ; REASON`: none of that, for a reason (made by passes
//!   before code generation, other platforms' runtimes, debug info...).
//!
//! FEATURES (optional) are the target's features a case needs, by LLVM's
//! names, comma-separated: the function has them, and those the machine
//! running the test has; a case needing one the machine lacks is built,
//! not run.
//!
//! `tests/intrinsics/generic-names.txt` and `aarch64-names.txt` are all the
//! intrinsics LLVM knows (from rustup's libLLVM, checked as intrinsics by
//! its verifier), and `x86-names.txt` x86's (those of a libLLVM's names
//! starting `llvm.x86.`, and those Rust's core::arch declares); each must
//! have a line. A `NAME` may end in `*` for all
//! the names starting so.
//!
//! A signature is `RESULT(ARG, ...)`, a type for each: `0`, `1`... are the
//! case's types (`TYPES` lists tuples of them, `|` between, `,` within, or
//! classes: `int`, `vint`...), `e0` the element type of `0`, `b0` its
//! shape of booleans, `n0` its shape of `i32`; `{A, B}` a struct; `void`.
//! An argument may be a constant (`i1 0`), `mem` (or `mem+K`, either
//! perhaps with attributes, `mem align 4`: a pointer into a scratch
//! buffer, whose bytes are stored too), `...T` (a `T` to one declared
//! `(...)`), or `ptrs0` (a
//! vector of pointers, one to each of the scratch buffer's elements of
//! `0`). A loaded argument may say what values it takes: `0:r` (floats in
//! the range of the integers), `0:p` (those not negative), `0:nz` (not
//! zero), `0:ltK` (below K). In the
//! name, `{0}` is LLVM's mangling of type `0`, and `{v0}` that of its
//! shape of pointers.
//!
//! The cases we get wrong or cannot build yet are listed in
//! `tests/intrinsics/generic-todo.txt` (and `aarch64-todo.txt`), one per
//! line as the failures name them: those are reported, not failures; any
//! other is one. `EPITOME_INTRINSICS_BLESS=1` writes the lists anew.
//!
//! The generic and AArch64 catalogues are run on an AArch64 Mac, the x86
//! one on x86-64 Linux, each with clang and rustup's llvm-tools. On x86-64
//! Linux the generic and AArch64 ones run too, under qemu (`qemu-aarch64`,
//! its `max` CPU: SVE2, SME and the rest of what it has), linked static
//! against Rust's own musl (rustup's `aarch64-unknown-linux-musl` target);
//! what they get wrong there is listed in `*-linux-todo.txt`.
//! `EPITOME_INTRINSICS` limits it to the names containing that text.

use std::collections::BTreeSet;
use std::fmt::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use epitome_codegen::target::Target;
use epitome_codegen::{Options, compile};

#[derive(Clone, Debug, PartialEq)]
enum Ty {
    Int(u32),
    Fp(&'static str),
    Vec(u32, Box<Ty>),
    Ptr,
    Void,
    Struct(Vec<Ty>),
}

fn fp_name(s: &str) -> Option<&'static str> {
    ["half", "bfloat", "float", "double", "fp128"].into_iter().find(|f| *f == s)
}

fn parse_ty(s: &str) -> Ty {
    let s = s.trim();
    if let Some(b) = s.strip_prefix('i').and_then(|b| b.parse().ok()) {
        return Ty::Int(b);
    }
    if let Some(f) = fp_name(s) {
        return Ty::Fp(f);
    }
    if s == "ptr" {
        return Ty::Ptr;
    }
    if s == "void" {
        return Ty::Void;
    }
    if let Some(inner) = s.strip_prefix('<').and_then(|s| s.strip_suffix('>')) {
        let (n, e) = inner.split_once(" x ").unwrap_or_else(|| panic!("bad vector type {s}"));
        return Ty::Vec(n.trim().parse().unwrap(), Box::new(parse_ty(e)));
    }
    if let Some(inner) = s.strip_prefix('{').and_then(|s| s.strip_suffix('}')) {
        return Ty::Struct(split_top(inner).iter().map(|f| parse_ty(f)).collect());
    }
    panic!("bad type {s}")
}

impl Ty {
    fn text(&self) -> String {
        match self {
            Ty::Int(b) => format!("i{b}"),
            Ty::Fp(f) => f.to_string(),
            Ty::Vec(n, e) => format!("<{n} x {}>", e.text()),
            Ty::Ptr => "ptr".into(),
            Ty::Void => "void".into(),
            Ty::Struct(fs) => format!("{{ {} }}", fs.iter().map(Ty::text).collect::<Vec<_>>().join(", ")),
        }
    }

    fn mangle(&self) -> String {
        match self {
            Ty::Int(b) => format!("i{b}"),
            Ty::Fp("half") => "f16".into(),
            Ty::Fp("bfloat") => "bf16".into(),
            Ty::Fp("float") => "f32".into(),
            Ty::Fp("double") => "f64".into(),
            Ty::Fp(_) => "f128".into(),
            Ty::Vec(n, e) => format!("v{n}{}", e.mangle()),
            Ty::Ptr => "p0".into(),
            Ty::Void => "isVoid".into(),
            Ty::Struct(fs) => format!("sl_{}s", fs.iter().map(Ty::mangle).collect::<String>()),
        }
    }

    fn elem(&self) -> Ty {
        match self {
            Ty::Vec(_, e) => (**e).clone(),
            t => t.clone(),
        }
    }

    /// The same shape (a scalar, or as many lanes), of `e`.
    fn shape_of(&self, e: Ty) -> Ty {
        match self {
            Ty::Vec(n, _) => Ty::Vec(*n, Box::new(e)),
            _ => e,
        }
    }

    /// How it is kept in memory here: integers in whole bytes, a power of
    /// two of them (booleans in a byte).
    fn storage(&self) -> Ty {
        match self {
            Ty::Int(b) => Ty::Int((*b).next_power_of_two().max(8)),
            Ty::Vec(n, e) => Ty::Vec(*n, Box::new(e.storage())),
            t => t.clone(),
        }
    }

    fn bytes(&self) -> u32 {
        match self {
            Ty::Int(b) => b.div_ceil(8),
            Ty::Fp("half" | "bfloat") => 2,
            Ty::Fp("float") => 4,
            Ty::Fp("double") | Ty::Ptr => 8,
            Ty::Fp(_) => 16,
            Ty::Vec(n, e) => n * e.bytes(),
            Ty::Void => 0,
            Ty::Struct(fs) => fs.iter().map(|f| f.bytes().next_multiple_of(16)).sum(),
        }
    }

    /// How the driver fills or prints it (`driver.c`), at `off`.
    fn desc(&self, off: u32, domain: &str) -> String {
        let (lanes, e) = match self {
            Ty::Vec(n, e) => (*n, (**e).clone()),
            t => (1, t.clone()),
        };
        let size = e.bytes();
        let kind = match (&e, domain) {
            (Ty::Fp("half" | "float" | "double"), "r") => "r",
            (Ty::Fp("half" | "float" | "double"), "p") => "p",
            (Ty::Fp("half" | "float" | "double"), _) => "f",
            (_, "nz") => "n",
            (_, d) if d.starts_with("lt") => "u",
            (Ty::Int(1), _) => "b",
            _ => "i",
        };
        let limit = domain.strip_prefix("lt").map(|l| format!("<{l}")).unwrap_or_default();
        format!("{kind}{size}*{lanes}@{off}{limit}")
    }
}

/// Splits at commas not inside brackets.
fn split_top(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut depth = 0;
    let mut cur = String::new();
    for c in s.chars() {
        match c {
            '<' | '{' | '(' | '[' => depth += 1,
            '>' | '}' | ')' | ']' => depth -= 1,
            ',' if depth == 0 => {
                out.push(cur.trim().to_string());
                cur.clear();
                continue;
            }
            _ => {}
        }
        cur.push(c);
    }
    if !cur.trim().is_empty() {
        out.push(cur.trim().to_string());
    }
    out
}

fn class(name: &str) -> Option<Vec<&'static str>> {
    Some(match name {
        "int" => vec!["i8", "i16", "i32", "i64", "i128"],
        "oddint" => vec!["i1", "i7", "i24", "i33"],
        "int16s" => vec!["i16", "i32", "i48", "i64", "i128"],
        "vint" => vec!["<8 x i8>", "<16 x i8>", "<4 x i16>", "<8 x i16>", "<2 x i32>", "<4 x i32>", "<2 x i64>"],
        "widevint" => vec!["<32 x i8>", "<8 x i32>", "<4 x i64>", "<3 x i32>", "<2 x i16>", "<1 x i64>"],
        "fp" => vec!["half", "float", "double"],
        "vfp" => vec!["<4 x half>", "<8 x half>", "<2 x float>", "<4 x float>", "<2 x double>"],
        "widevfp" => vec!["<8 x float>", "<3 x float>", "<1 x double>", "<4 x double>"],
        // NEON's: its vectors of integers, and of 64-bit ones too.
        "nint" => vec!["<8 x i8>", "<16 x i8>", "<4 x i16>", "<8 x i16>", "<2 x i32>", "<4 x i32>"],
        "nint64" => {
            vec!["<8 x i8>", "<16 x i8>", "<4 x i16>", "<8 x i16>", "<2 x i32>", "<4 x i32>", "<1 x i64>", "<2 x i64>"]
        }
        "nfp" => vec!["<2 x float>", "<4 x float>", "<2 x double>"],
        "nfp16" => vec!["<4 x half>", "<8 x half>", "<2 x float>", "<4 x float>", "<2 x double>"],
        // (Narrow, wide) pairs: lanes of half the width, as many.
        "narrow" => vec!["<8 x i8>, <8 x i16>", "<4 x i16>, <4 x i32>", "<2 x i32>, <2 x i64>"],
        // (Wide, narrow) pairs.
        "widen" => vec!["<8 x i16>, <8 x i8>", "<4 x i32>, <4 x i16>", "<2 x i64>, <2 x i32>"],
        // (Integers, floats) of one shape.
        "fcvt" => vec![
            "<2 x i32>, <2 x float>",
            "<4 x i32>, <4 x float>",
            "<2 x i64>, <2 x double>",
            "i32, float",
            "i64, double",
            "i32, double",
            "i64, float",
        ],
        _ => return None,
    })
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Status {
    Test,
    Build,
    Skip,
}

struct Entry {
    status: Status,
    name: String,
    sig: String,
    tuples: Vec<Vec<Ty>>,
    features: Vec<String>,
}

fn read_catalogue(path: &Path) -> Vec<Entry> {
    let text = std::fs::read_to_string(path).unwrap();
    let mut out = Vec::new();
    for (n, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        // (A skip's reason is the rest of the line.)
        let fields = if line.starts_with("skip") { 3 } else { 5 };
        let f: Vec<&str> = line.splitn(fields, ';').map(str::trim).collect();
        let status = match f[0] {
            "test" => Status::Test,
            "build" => Status::Build,
            "skip" => Status::Skip,
            s => panic!("{}:{}: status {s}", path.display(), n + 1),
        };
        if status == Status::Skip {
            assert!(f.len() == 3 && !f[2].is_empty(), "{}:{}: a skip needs a reason", path.display(), n + 1);
            out.push(Entry { status, name: f[1].into(), sig: String::new(), tuples: Vec::new(), features: Vec::new() });
            continue;
        }
        assert!((3..=5).contains(&f.len()), "{}:{}: fields", path.display(), n + 1);
        let mut tuples = Vec::new();
        if let Some(types) = f.get(3).filter(|t| !t.is_empty()) {
            for t in types.split('|').map(str::trim) {
                match class(t) {
                    Some(list) => {
                        tuples.extend(list.into_iter().map(|t| split_top(t).iter().map(|x| parse_ty(x)).collect()))
                    }
                    None => tuples.push(split_top(t).iter().map(|x| parse_ty(x)).collect()),
                }
            }
        } else {
            tuples.push(Vec::new());
        }
        let features = f.get(4).map_or(Vec::new(), |x| x.split(',').map(|x| x.trim().to_string()).collect());
        out.push(Entry { status, name: f[1].into(), sig: f[2].into(), tuples, features });
    }
    out
}

/// The base name an entry is for: its name without its manglings (`{0}`,
/// or written out: `.p0`, `.i64`, `.v4f32`).
fn base_name(name: &str) -> String {
    let mangling = |p: &str| {
        p.starts_with('{')
            || p.strip_prefix(['i', 'f', 'p']).is_some_and(|d| !d.is_empty() && d.bytes().all(|b| b.is_ascii_digit()))
            || p.strip_prefix('v').is_some_and(|d| d.starts_with(|c: char| c.is_ascii_digit()))
            || p == "bf16"
    };
    let mut parts: Vec<&str> = name.split('.').collect();
    while parts.len() > 2 && parts.last().is_some_and(|p| mangling(p)) {
        parts.pop();
    }
    parts.join(".")
}

fn resolve(tok: &str, tys: &[Ty]) -> Ty {
    let tok = tok.trim();
    if let Some(inner) = tok.strip_prefix('{').and_then(|s| s.strip_suffix('}')) {
        return Ty::Struct(split_top(inner).iter().map(|f| resolve(f, tys)).collect());
    }
    let idx = |s: &str| -> Option<usize> { s.parse().ok() };
    if let Some(k) = idx(tok) {
        return tys[k].clone();
    }
    if let Some(k) = tok.strip_prefix('e').and_then(idx) {
        return tys[k].elem();
    }
    if let Some(k) = tok.strip_prefix('b').and_then(idx) {
        return tys[k].shape_of(Ty::Int(1));
    }
    if let Some(k) = tok.strip_prefix('n').and_then(idx) {
        return tys[k].shape_of(Ty::Int(32));
    }
    parse_ty(tok)
}

/// One function: its IR, and what the driver puts in and prints.
struct Case {
    label: String,
    ir: String,
    input: String,
    output: String,
}

const SCRATCH: u32 = 64;

fn make_case(entry: &Entry, tys: &[Ty], sym: &str, host: &Host) -> Case {
    let mut name = entry.name.clone();
    for (k, t) in tys.iter().enumerate() {
        name = name.replace(&format!("{{{k}}}"), &t.mangle());
        name = name.replace(&format!("{{v{k}}}"), &t.shape_of(Ty::Ptr).mangle());
    }
    let (ret_s, args_s) = entry.sig.split_once('(').unwrap();
    let args_s = args_s.strip_suffix(')').unwrap();
    let ret = resolve(ret_s, tys);
    let mut body = String::new();
    let mut input = Vec::new();
    let mut in_off = 0u32;
    let mut args = Vec::new();
    let mut uses_mem = false;
    let mut varargs = false;
    // The arguments as the case's label names them.
    let mut parts: Vec<String> = Vec::new();
    let mut v = 0;
    let mut fresh = |p: &str| {
        v += 1;
        format!("%{p}{v}")
    };
    // The scratch buffer: after the result, in `out`.
    let out_mem = ret.bytes().next_multiple_of(16).max(16);
    for a in split_top(args_s) {
        if a == "mem" || a.starts_with("mem+") || a.starts_with("mem ") {
            uses_mem = true;
            let (m, attrs) = a.split_once(' ').unwrap_or((&a, ""));
            let k: u32 = m.strip_prefix("mem+").map_or(0, |k| k.parse().unwrap());
            let p = fresh("m");
            let _ = writeln!(body, "  {p} = getelementptr i8, ptr %out, i64 {}", out_mem + k);
            args.push(if attrs.is_empty() { format!("ptr {p}") } else { format!("ptr {attrs} {p}") });
            parts.push(format!("ptr@{k}"));
            continue;
        }
        // An argument to one declared `(...)`.
        if let Some(rest) = a.strip_prefix("...") {
            varargs = true;
            let t = resolve(rest, tys);
            in_off = in_off.next_multiple_of(16);
            input.push(t.storage().desc(in_off, ""));
            let (p, l) = (fresh("p"), fresh("l"));
            let _ = writeln!(body, "  {p} = getelementptr i8, ptr %in, i64 {in_off}");
            let _ = writeln!(body, "  {l} = load {}, ptr {p}, align 1", t.text());
            in_off += t.bytes();
            args.push(format!("{} {l}", t.text()));
            parts.push(format!("...{}", t.mangle()));
            continue;
        }
        if let Some(k) = a.strip_prefix("ptrs").and_then(|k| k.parse::<usize>().ok()) {
            uses_mem = true;
            let Ty::Vec(n, e) = &tys[k] else { panic!("ptrs of a scalar") };
            let idx: Vec<String> = (0..*n).map(|l| format!("i64 {}", out_mem + l * e.bytes())).collect();
            let p = fresh("ps");
            let _ = writeln!(body, "  {p} = getelementptr i8, ptr %out, <{n} x i64> <{}>", idx.join(", "));
            args.push(format!("<{n} x ptr> {p}"));
            parts.push(format!("v{n}p0@"));
            continue;
        }
        if a.starts_with("metadata") {
            args.push(a.to_string());
            parts.push(a.replace([' ', '"'], ""));
            continue;
        }
        let (tok, domain) = a.split_once(':').unwrap_or((&a, ""));
        // A constant: a type and its value.
        if tok.contains(' ') && !tok.starts_with('<') || tok.starts_with('<') && tok.contains("> ") {
            let (t, c) = if tok.starts_with('<') {
                let at = tok.find("> ").unwrap() + 1;
                (&tok[..at], &tok[at..])
            } else {
                tok.split_once(' ').unwrap()
            };
            let t = resolve(t, tys);
            // (`{e0}` in the value: the element type of `0`, for splats.)
            let mut c = c.trim().to_string();
            for (k, x) in tys.iter().enumerate() {
                c = c.replace(&format!("{{e{k}}}"), &x.elem().text());
            }
            args.push(format!("{} {c}", t.text()));
            parts.push(format!("{}={}", t.mangle(), c.replace(' ', "")));
            continue;
        }
        let t = resolve(tok, tys);
        let st = t.storage();
        in_off = in_off.next_multiple_of(16);
        input.push(st.desc(in_off, domain));
        let p = fresh("p");
        let l = fresh("l");
        let _ = writeln!(body, "  {p} = getelementptr i8, ptr %in, i64 {in_off}");
        let _ = writeln!(body, "  {l} = load {}, ptr {p}, align 1", st.text());
        in_off += st.bytes();
        if st != t {
            let x = fresh("t");
            let _ = writeln!(body, "  {x} = trunc {} {l} to {}", st.text(), t.text());
            args.push(format!("{} {x}", t.text()));
            parts.push(t.mangle());
        } else {
            args.push(format!("{} {l}", t.text()));
            parts.push(t.mangle());
        }
    }
    if uses_mem {
        in_off = in_off.next_multiple_of(16);
        input.push(format!("i1*{SCRATCH}@{in_off}"));
        // The scratch buffer starts as these bytes.
        let mut pre = String::new();
        for k in 0..SCRATCH / 16 {
            let _ = writeln!(pre, "  %sp{k} = getelementptr i8, ptr %in, i64 {}", in_off + 16 * k);
            let _ = writeln!(pre, "  %sv{k} = load <16 x i8>, ptr %sp{k}, align 1");
            let _ = writeln!(pre, "  %sq{k} = getelementptr i8, ptr %out, i64 {}", out_mem + 16 * k);
            let _ = writeln!(pre, "  store <16 x i8> %sv{k}, ptr %sq{k}, align 1");
        }
        body = pre + &body;
    }
    let arg_tys: Vec<String> = args.iter().map(|a| arg_type(a)).collect();
    let decl = if varargs {
        format!("declare {} @{name}(...)", ret.text())
    } else {
        format!("declare {} @{name}({})", ret.text(), arg_tys.join(", "))
    };
    let mut output = Vec::new();
    if ret == Ty::Void {
        let callee = if varargs { "void (...)" } else { "void" };
        let _ = writeln!(body, "  call {callee} @{name}({})", args.join(", "));
    } else {
        let _ = writeln!(body, "  %r = call {} @{name}({})", ret.text(), args.join(", "));
        let fields: Vec<(String, Ty)> = match &ret {
            Ty::Struct(fs) => fs
                .iter()
                .enumerate()
                .map(|(k, f)| {
                    let x = fresh("f");
                    let _ = writeln!(body, "  {x} = extractvalue {} %r, {k}", ret.text());
                    (x, f.clone())
                })
                .collect(),
            t => vec![("%r".to_string(), t.clone())],
        };
        let mut off = 0;
        for (x, t) in fields {
            let p = fresh("o");
            let _ = writeln!(body, "  {p} = getelementptr i8, ptr %out, i64 {off}");
            let st = t.storage();
            if t == Ty::Ptr {
                // A pointer, as where it is from the scratch buffer.
                let (a, b, d) = (fresh("a"), fresh("b"), fresh("d"));
                let _ = writeln!(body, "  {a} = ptrtoint ptr {x} to i64");
                let _ = writeln!(body, "  {b} = ptrtoint ptr %out to i64");
                let _ = writeln!(body, "  {d} = sub i64 {a}, {b}");
                let _ = writeln!(body, "  store i64 {d}, ptr {p}, align 1");
                output.push(format!("i8*1@{off}"));
            } else if st != t {
                let z = fresh("z");
                let _ = writeln!(body, "  {z} = zext {} {x} to {}", t.text(), st.text());
                let _ = writeln!(body, "  store {} {z}, ptr {p}, align 1", st.text());
                output.push(st.desc(off, ""));
            } else {
                let _ = writeln!(body, "  store {} {x}, ptr {p}, align 1", t.text());
                output.push(t.desc(off, ""));
            }
            off += st.bytes().next_multiple_of(16);
        }
    }
    if uses_mem {
        output.push(format!("i1*{SCRATCH}@{out_mem}"));
    }
    let mut features: Vec<String> = host.features.iter().map(|f| f.to_string()).collect();
    features.extend(entry.features.iter().filter(|f| !host.features.contains(&f.as_str())).cloned());
    let features = if features.is_empty() {
        String::new()
    } else {
        format!(" \"target-features\"=\"{}\"", features.iter().map(|f| format!("+{f}")).collect::<Vec<_>>().join(","))
    };
    let ir = format!(
        "target triple = \"{}\"\n{decl}\ndefine void @{sym}(ptr %out, ptr %in) #0 {{\n{body}  ret void\n}}\n\
         attributes #0 = {{ \"target-cpu\"=\"{}\"{features} }}\n",
        host.triple, host.cpu
    );
    // (Without spaces: the driver's lines start with it.)
    let label = format!("{name}({})", parts.join(","));
    Case { label, ir, input: input.join(" "), output: output.join(" ") }
}

/// The type of an argument written `TYPE VALUE`.
fn arg_type(a: &str) -> String {
    if a.starts_with('<') {
        let at = a.find("> ").map_or(a.len(), |k| k + 1);
        a[..at].to_string()
    } else if a.starts_with("metadata") {
        "metadata".into()
    } else {
        a.split_once(' ').map_or(a, |x| x.0).to_string()
    }
}

fn tool(name: &str) -> Option<PathBuf> {
    let host = if cfg!(target_arch = "aarch64") { "aarch64-apple-darwin" } else { "x86_64-unknown-linux-gnu" };
    let o = Command::new("rustc").arg("--print").arg("sysroot").output().ok()?;
    let root = String::from_utf8_lossy(&o.stdout).trim().to_string();
    let p = PathBuf::from(root).join("lib/rustlib").join(host).join("bin").join(name);
    p.exists().then_some(p)
}

/// What a catalogue's cases are built for, and the machine running them.
struct Host {
    target: Target,
    triple: &'static str,
    cpu: &'static str,
    /// The features the functions have: those of the machine, by LLVM's
    /// names (beyond the CPU's).
    features: Vec<&'static str>,
    /// Run under an emulator, linked as it needs.
    emulated: Option<Emulated>,
}

/// AArch64 Linux under qemu: the emulator, and what programs are linked
/// with (musl's start files and C library, Rust's compiler builtins, and
/// rust-lld).
struct Emulated {
    qemu: PathBuf,
    lld: PathBuf,
    musl: PathBuf,
    builtins: PathBuf,
}

/// What qemu's `max` CPU has (qemu 8), by LLVM's names.
const QEMU_MAX: &[&str] = &[
    "fp-armv8",
    "neon",
    "crc",
    "aes",
    "sha2",
    "sha3",
    "sm4",
    "lse",
    "rdm",
    "dotprod",
    "fullfp16",
    "fp16fml",
    "bf16",
    "i8mm",
    "f32mm",
    "f64mm",
    "sve",
    "sve2",
    "sve-aes",
    "sve-sha3",
    "sve-sm4",
    "sve-bitperm",
    "mte",
    "rand",
    "flagm",
    "sb",
    "ssbs",
    "predres",
    "bti",
    "pauth",
    "jsconv",
    "complxnum",
    "rcpc",
    "sme",
    "sme-f64f64",
    "sme-i16i64",
    "sme-fa64",
];

fn emulated_aarch64() -> Option<Emulated> {
    let path = std::env::var_os("PATH")?;
    let qemu = ["qemu-aarch64-static", "qemu-aarch64"]
        .iter()
        .flat_map(|q| std::env::split_paths(&path).map(move |d| d.join(q)))
        .find(|p| p.exists())?;
    let o = Command::new("rustc").arg("--print").arg("sysroot").output().ok()?;
    let root = PathBuf::from(String::from_utf8_lossy(&o.stdout).trim());
    let lib = root.join("lib/rustlib/aarch64-unknown-linux-musl/lib");
    let builtins = std::fs::read_dir(&lib).ok()?.flatten().map(|e| e.path()).find(|p| {
        p.extension().is_some_and(|x| x == "rlib")
            && p.file_name().is_some_and(|n| n.to_string_lossy().starts_with("libcompiler_builtins-"))
    })?;
    let lld = root.join("lib/rustlib/x86_64-unknown-linux-gnu/bin/gcc-ld/ld.lld");
    let musl = lib.join("self-contained");
    (lld.exists() && musl.join("libc.a").exists()).then_some(Emulated { qemu, lld, musl, builtins })
}

/// The machine's x86 features, by LLVM's names.
#[cfg(target_arch = "x86_64")]
fn x86_features() -> Vec<&'static str> {
    let mut v = Vec::new();
    macro_rules! detect {
        ($($rust:tt => $llvm:expr),* $(,)?) => {
            $(if std::arch::is_x86_feature_detected!($rust) {
                v.extend($llvm.split(','));
            })*
        };
    }
    detect!(
        "sse" => "sse", "sse2" => "sse2", "sse3" => "sse3", "ssse3" => "ssse3", "sse4.1" => "sse4.1",
        "sse4.2" => "sse4.2,crc32", "sse4a" => "sse4a", "popcnt" => "popcnt", "lzcnt" => "lzcnt",
        "bmi1" => "bmi", "bmi2" => "bmi2", "avx" => "avx", "avx2" => "avx2", "fma" => "fma", "f16c" => "f16c",
        "aes" => "aes", "pclmulqdq" => "pclmul", "sha" => "sha", "adx" => "adx", "rdrand" => "rdrnd",
        "rdseed" => "rdseed", "movbe" => "movbe", "xsave" => "xsave", "xsaveopt" => "xsaveopt", "xsavec" => "xsavec",
        "xsaves" => "xsaves", "fxsr" => "fxsr", "cmpxchg16b" => "cx16", "avx512f" => "avx512f", "avx512cd" => "avx512cd",
        "avx512bw" => "avx512bw", "avx512dq" => "avx512dq", "avx512vl" => "avx512vl", "avx512ifma" => "avx512ifma",
        "avx512vbmi" => "avx512vbmi", "avx512vbmi2" => "avx512vbmi2", "avx512vnni" => "avx512vnni",
        "avx512bitalg" => "avx512bitalg", "avx512vpopcntdq" => "avx512vpopcntdq", "avx512bf16" => "avx512bf16",
        "avx512fp16" => "avx512fp16", "avx512vp2intersect" => "avx512vp2intersect", "gfni" => "gfni",
        "vaes" => "vaes", "vpclmulqdq" => "vpclmulqdq", "avxvnni" => "avxvnni", "avxifma" => "avxifma",
        "avxneconvert" => "avxneconvert", "avxvnniint8" => "avxvnniint8", "avxvnniint16" => "avxvnniint16",
        "tbm" => "tbm", "rtm" => "rtm", "sha512" => "sha512", "sm3" => "sm3", "sm4" => "sm4", "kl" => "kl",
        "widekl" => "widekl",
    );
    v
}

#[cfg(not(target_arch = "x86_64"))]
fn x86_features() -> Vec<&'static str> {
    Vec::new()
}

/// The catalogue's host, if this machine is one.
fn host(which: &str) -> Option<Host> {
    if which == "x86" {
        (cfg!(target_arch = "x86_64") && cfg!(target_os = "linux")).then(|| Host {
            target: Target::x86_64(),
            triple: "x86_64-unknown-linux-gnu",
            cpu: "x86-64",
            features: x86_features(),
            emulated: None,
        })
    } else if cfg!(target_arch = "aarch64") && cfg!(target_os = "macos") {
        Some(Host {
            target: Target::aarch64(),
            triple: "arm64-apple-macosx14.0.0",
            cpu: "apple-m1",
            features: Vec::new(),
            emulated: None,
        })
    } else if cfg!(target_arch = "x86_64") && cfg!(target_os = "linux") {
        emulated_aarch64().map(|e| Host {
            target: Target::aarch64(),
            triple: "aarch64-unknown-linux-gnu",
            cpu: "generic",
            features: QEMU_MAX.to_vec(),
            emulated: Some(e),
        })
    } else {
        None
    }
}

/// The names `NAMES` lists that no line of `entries` is for.
fn uncovered(names: &Path, entries: &[Entry]) -> Vec<String> {
    let text = std::fs::read_to_string(names).unwrap();
    text.lines()
        .map(str::trim)
        .filter(|n| !n.is_empty() && !n.starts_with('#'))
        .filter(|n| {
            !entries.iter().any(|e| {
                let b = base_name(&e.name);
                match b.strip_suffix('*') {
                    Some(prefix) => n.starts_with(prefix),
                    None => *n == b || *n == e.name,
                }
            })
        })
        .map(str::to_string)
        .collect()
}

thread_local! {
    static QUIET: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Whether panics on this thread are printed (the tests run on threads of
/// their own, at once: the hook is the process's).
fn quiet_panics(quiet: bool) {
    static HOOK: std::sync::Once = std::sync::Once::new();
    HOOK.call_once(|| {
        let default = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            if !QUIET.with(|q| q.get()) {
                default(info);
            }
        }));
    });
    QUIET.with(|q| q.set(quiet));
}

fn run_catalogue(which: &str) {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/intrinsics");
    let entries = read_catalogue(&dir.join(format!("{which}.txt")));
    let missing = uncovered(&dir.join(format!("{which}-names.txt")), &entries);
    assert!(missing.is_empty(), "intrinsics with no line in {which}.txt:\n{}", missing.join("\n"));

    let llc = tool("llc");
    let host = host(which);
    let ok = host.is_some()
        && llc.is_some()
        && Command::new("clang").arg("--version").output().is_ok_and(|o| o.status.success());
    if !ok {
        let on = if which == "x86" { "x86-64 Linux" } else { "an AArch64 Mac (or x86-64 Linux, with qemu and musl)" };
        eprintln!("needs clang and rustup's llvm-tools (llc) on {on}; skipping");
        return;
    }
    let (llc, host) = (llc.unwrap(), host.unwrap());
    let only = std::env::var("EPITOME_INTRINSICS").ok();
    let tmp = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("intrinsics-{which}"));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();
    let t = &host.target;
    let mut cases: Vec<(Status, Case)> = Vec::new();
    // (Those the machine lacks a feature for are built, not run.)
    let mut unrunnable = 0;
    for e in &entries {
        if e.status == Status::Skip || only.as_ref().is_some_and(|o| !e.name.contains(o.as_str())) {
            continue;
        }
        let runs = e.features.iter().all(|f| host.features.contains(&f.as_str()));
        for tys in &e.tuples {
            let sym = format!("c{}", cases.len());
            let status = if e.status == Status::Test && !runs {
                unrunnable += 1;
                Status::Build
            } else {
                e.status
            };
            cases.push((status, make_case(e, tys, &sym, &host)));
        }
    }

    // Each built alone, by us and by llc. (Our compiler's panics are
    // caught, and reported as failures, not printed.)
    quiet_panics(true);
    // (Label, why.)
    let mut failures: Vec<(String, String)> = Vec::new();
    let mut reference_fails = Vec::new();
    let mut run = Vec::new();
    for (k, (status, c)) in cases.iter().enumerate() {
        let ll = tmp.join(format!("c{k}.ll"));
        std::fs::write(&ll, &c.ir).unwrap();
        let theirs = tmp.join(format!("c{k}.llc.o"));
        let st = Command::new(&llc)
            .args(["-O2", &format!("-mcpu={}", host.cpu), "-filetype=obj", "-o"])
            .arg(&theirs)
            .arg(&ll)
            .output()
            .unwrap();
        if !st.status.success() {
            reference_fails.push(format!(
                "{}: {}",
                c.label,
                String::from_utf8_lossy(&st.stderr).lines().next().unwrap_or("")
            ));
            continue;
        }
        let ours = tmp.join(format!("c{k}.ours.o"));
        let built = (|| -> Result<(), String> {
            let mut m = epitome_ir::parse_module(&c.ir).map_err(|e| format!("{e:?}"))?;
            let obj = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                compile(t, &mut m, &Options::default()).and_then(|x| x.object_file(t))
            }))
            .map_err(|_| "the compiler panicked".to_string())??;
            std::fs::write(&ours, obj).map_err(|e| e.to_string())
        })();
        match (status, built) {
            (_, Err(e)) => failures.push((c.label.clone(), e)),
            (Status::Test, Ok(())) => run.push(k),
            _ => {}
        }
    }

    quiet_panics(false);

    // Those to run, linked into one program each way.
    let table = |side: &str| -> PathBuf {
        let mut c = String::from(
            "struct Case { void (*f)(unsigned char *, const unsigned char *); const char *name, *in, *out; };\n",
        );
        for &k in &run {
            let _ = writeln!(c, "void c{k}(unsigned char *, const unsigned char *);");
        }
        c.push_str("struct Case cases[] = {\n");
        for &k in &run {
            let x = &cases[k].1;
            let label = x.label.replace('\\', "\\\\").replace('"', "\\\"");
            let _ = writeln!(c, "    {{c{k}, \"{label}\", \"{}\", \"{}\"}},", x.input, x.output);
        }
        let _ = writeln!(c, "    {{0, 0, 0, 0}}}};\nint ncases = {};", run.len());
        let p = tmp.join(format!("table.{side}.c"));
        std::fs::write(&p, c).unwrap();
        p
    };
    let outputs: Vec<String> = ["ours", "llc"]
        .iter()
        .map(|side| {
            let exe = tmp.join(format!("run.{side}"));
            let mut cmd = Command::new("clang");
            cmd.arg("-O1");
            if let Some(e) = &host.emulated {
                // (Static, against musl; the builtins ask for a personality
                // routine nothing here calls.)
                let stub = tmp.join("personality.c");
                std::fs::write(&stub, "void rust_eh_personality(void) {}\n").unwrap();
                cmd.args(["--target=aarch64-unknown-linux-musl", "-fuse-ld=lld", "-static", "-nostdlib"])
                    .arg(format!("--ld-path={}", e.lld.display()))
                    .arg(e.musl.join("crt1.o"))
                    .arg(e.musl.join("crti.o"))
                    .arg(&stub);
            }
            cmd.arg(dir.join("driver.c")).arg(table(side)).arg("-o").arg(&exe);
            for &k in &run {
                cmd.arg(tmp.join(format!("c{k}.{side}.o")));
            }
            if let Some(e) = &host.emulated {
                cmd.arg(e.musl.join("libc.a")).arg(&e.builtins).arg(e.musl.join("crtn.o"));
            }
            let st = cmd.status().unwrap();
            assert!(st.success(), "linking the {side} build failed");
            let o = match &host.emulated {
                Some(e) => Command::new(&e.qemu).args(["-cpu", "max"]).arg(&exe).output().unwrap(),
                None => Command::new(&exe).output().unwrap(),
            };
            assert!(o.status.success(), "the {side} build failed: {}", String::from_utf8_lossy(&o.stderr));
            String::from_utf8_lossy(&o.stdout).into_owned()
        })
        .collect();
    let mut differ = BTreeSet::new();
    for (a, b) in outputs[0].lines().zip(outputs[1].lines()) {
        if a != b {
            let case = a.split(' ').next().unwrap().to_string();
            if differ.insert(case.clone()) {
                failures.push((case, format!("differs:\n  ours: {a}\n  llc:  {b}")));
            }
        }
    }
    println!(
        "{which}: {} cases run, {} the same; {} llc cannot build; {unrunnable} built, not run (features the machine lacks)",
        run.len(),
        run.len() - differ.len(),
        reference_fails.len()
    );
    for r in &reference_fails {
        println!("  llc cannot build {r}");
    }
    let linux = if host.emulated.is_some() { "-linux" } else { "" };
    let todo_path = dir.join(format!("{which}{linux}-todo.txt"));
    if std::env::var("EPITOME_INTRINSICS_BLESS").is_ok() && only.is_none() {
        let mut list: Vec<&str> = failures.iter().map(|f| f.0.as_str()).collect();
        list.sort();
        list.dedup();
        let head =
            format!("# The {which} intrinsics' cases we get wrong or cannot build yet (see tests/intrinsics.rs).\n");
        std::fs::write(&todo_path, head + &list.join("\n") + "\n").unwrap();
    }
    let todo: BTreeSet<String> = std::fs::read_to_string(&todo_path)
        .unwrap_or_default()
        .lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
        .map(|l| l.trim().to_string())
        .collect();
    let failed: BTreeSet<&str> = failures.iter().map(|f| f.0.as_str()).collect();
    let fixed: Vec<&String> =
        todo.iter().filter(|t| !failed.contains(t.as_str()) && cases.iter().any(|c| c.1.label == **t)).collect();
    println!(
        "{which}: {} cases fail as {} expects",
        failures.iter().filter(|f| todo.contains(&f.0)).count(),
        todo_path.file_name().unwrap().to_string_lossy()
    );
    if !fixed.is_empty() {
        println!("listed in {which}-todo.txt but now right (take them out):");
        for f in fixed {
            println!("  {f}");
        }
    }
    let new: Vec<String> =
        failures.iter().filter(|f| !todo.contains(&f.0)).map(|(l, why)| format!("{l}: {why}")).collect();
    assert!(new.is_empty(), "{}", new.join("\n"));
}

#[test]
fn generic_intrinsics() {
    run_catalogue("generic");
}

#[test]
fn aarch64_intrinsics() {
    run_catalogue("aarch64");
}

#[test]
fn x86_intrinsics() {
    run_catalogue("x86");
}
