//! The C functions for exported subroutines (35.7, 35.8).
//!
//! C code calls an export by name, as an ordinary C function, so each
//! exported name has to be a symbol in the process before the libraries
//! calling it are loaded. Each stub has the C signature Annex H gives the
//! export (H.8.2), and passes a pointer to each argument's C value, and
//! one to where the result goes, to `dispatch`. The stubs are written as
//! C, built into a shared library with the C compiler (`EPITOME_SV_CC`,
//! else `CC`, else `cc`), and kept in the temporary directory under a
//! name made from their text, so that a design is built once.

use super::CKind;
use std::fmt::Write as _;
use std::path::PathBuf;

/// One exported name's C signature.
pub(super) struct Decl {
    pub name: String,
    /// Each parameter's C type, and whether it is a reference.
    pub params: Vec<(&'static str, bool)>,
    pub ret: &'static str,
}

/// The C type of a value passed by value (table H.1).
pub(super) fn c_type(k: &CKind) -> &'static str {
    match k {
        CKind::Int(1, true) => "signed char",
        CKind::Int(1, false) | CKind::Scalar => "unsigned char",
        CKind::Int(2, true) => "short",
        CKind::Int(2, false) => "unsigned short",
        CKind::Int(4, true) => "int",
        CKind::Int(4, false) => "unsigned int",
        CKind::Int(_, true) => "long long",
        CKind::Int(_, false) => "unsigned long long",
        CKind::Real => "double",
        CKind::ShortReal => "float",
        CKind::Str => "const char *",
        _ => "void *",
    }
}

pub(super) fn source(decls: &[Decl]) -> String {
    let mut s = String::from(
        "/* Export stubs written by epitome-sv. */\n\
         typedef void (*epitome_dpi_fn)(int, void **, void *);\n\
         __attribute__((visibility(\"default\"))) epitome_dpi_fn epitome_dpi_dispatch;\n",
    );
    for (k, d) in decls.iter().enumerate() {
        let params: Vec<String> = d.params.iter().enumerate().map(|(i, (t, _))| format!("{t} a{i}")).collect();
        let params = if params.is_empty() { "void".to_string() } else { params.join(", ") };
        let _ = writeln!(s, "__attribute__((visibility(\"default\"))) {} {}({params}) {{", d.ret, d.name);
        let n = d.params.len().max(1);
        let _ = writeln!(s, "    void *args[{n}];");
        for (i, (_, by_ref)) in d.params.iter().enumerate() {
            // A value's address; a reference is the address already.
            if *by_ref {
                let _ = writeln!(s, "    args[{i}] = a{i};");
            } else {
                let _ = writeln!(s, "    args[{i}] = (void *)&a{i};");
            }
        }
        if d.ret == "void" {
            let _ = writeln!(s, "    epitome_dpi_dispatch({k}, args, 0);");
        } else {
            let _ = writeln!(s, "    {} r = 0;", d.ret);
            let _ = writeln!(s, "    epitome_dpi_dispatch({k}, args, (void *)&r);");
            let _ = writeln!(s, "    return r;");
        }
        let _ = writeln!(s, "}}");
    }
    s
}

/// A 64-bit FNV-1a hash, to name the built library after its source.
fn fnv(s: &str) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for b in s.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100_0000_01b3);
    }
    h
}

/// Builds the stubs into a shared library, or finds them built.
pub(super) fn build(decls: &[Decl]) -> Result<PathBuf, String> {
    let src = source(decls);
    let ext = if cfg!(target_os = "macos") { "dylib" } else { "so" };
    let dir = std::env::temp_dir().join("epitome-sv-dpi");
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let lib = dir.join(format!("exports-{:016x}.{ext}", fnv(&src)));
    if lib.is_file() {
        return Ok(lib);
    }
    // Built under a name of its own and renamed, so that runs at the same
    // time do not see half a file.
    let tag = format!("{}-{:?}", std::process::id(), std::thread::current().id()).replace(|c: char| !c.is_ascii_alphanumeric(), "");
    let c = dir.join(format!("exports-{tag}.c"));
    let tmp = dir.join(format!("exports-{tag}.{ext}"));
    std::fs::write(&c, &src).map_err(|e| format!("{}: {e}", c.display()))?;
    let cc = std::env::var("EPITOME_SV_CC").or_else(|_| std::env::var("CC")).unwrap_or_else(|_| "cc".into());
    let mut words = cc.split_whitespace();
    let mut cmd = std::process::Command::new(words.next().unwrap_or("cc"));
    cmd.args(words).args(["-shared", "-fPIC", "-O1", "-o"]).arg(&tmp).arg(&c);
    let out = cmd.output().map_err(|e| format!("DPI exports need a C compiler to build their stubs; running `{cc}`: {e}"))?;
    let _ = std::fs::remove_file(&c);
    if !out.status.success() {
        let _ = std::fs::remove_file(&tmp);
        return Err(format!("building the DPI export stubs with `{cc}` failed:\n{}", String::from_utf8_lossy(&out.stderr)));
    }
    std::fs::rename(&tmp, &lib).map_err(|e| format!("{}: {e}", lib.display()))?;
    Ok(lib)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_text() {
        let d = [
            Decl { name: "f".into(), params: vec![("int", false), ("void *", true)], ret: "double" },
            Decl { name: "t".into(), params: vec![], ret: "int" },
            Decl { name: "v".into(), params: vec![("const char *", false)], ret: "void" },
        ];
        let s = source(&d);
        assert!(s.contains("double f(int a0, void * a1) {"));
        assert!(s.contains("args[0] = (void *)&a0;"));
        assert!(s.contains("args[1] = a1;"));
        assert!(s.contains("epitome_dpi_dispatch(1, args, (void *)&r);"));
        assert!(s.contains("void v(const char * a0) {"));
        assert!(s.contains("epitome_dpi_dispatch(2, args, 0);"));
    }
}
