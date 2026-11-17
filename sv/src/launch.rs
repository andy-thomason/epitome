//! Running a design the way test runners run Icarus Verilog: a compile
//! step that writes a design file, and a run step that loads VPI
//! libraries and simulates it. `epitome-iverilog` and `epitome-vvp` take
//! the command lines that runners such as cocotb's give `iverilog` and
//! `vvp`, so that a directory with links of those names to them makes a
//! runner use epitome-sv.
//!
//! The design file is ours: a line per setting, a keyword and its value,
//! with every path absolute.
//!
//! ```text
//! epitome-sv design 1
//! top counter
//! param counter.WIDTH=12
//! define FOO=1
//! incdir /work/rtl
//! timescale 1ns/1ps
//! file /work/rtl/counter.sv
//! svlib /work/c/model
//! ```
//!
//! DPI libraries (clause 35) are named as Annex J suggests, to either
//! program: `-sv_lib name` (a path without the platform's extension),
//! `-sv_liblist file` (a bootstrap file of such names) and `-sv_root dir`
//! (where relative names are found from). A runner that cannot add to a
//! command line can list paths in `EPITOME_SV_LIBS`, separated by `:`.

use crate::design::Design;
use crate::preprocess::{Options, Timescale};
use crate::source::SourceMap;
use std::path::{Path, PathBuf};

/// The first line of a design file.
pub const MAGIC: &str = "epitome-sv design 1";

/// What a design is built from.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DesignFile {
    pub tops: Vec<String>,
    /// Top-level parameters: `name` or `top.name`, and the value as text.
    pub params: Vec<(String, String)>,
    pub defines: Vec<(String, String)>,
    pub incdirs: Vec<PathBuf>,
    pub timescale: Option<String>,
    pub files: Vec<PathBuf>,
    /// DPI libraries, as `-sv_lib` names them.
    pub sv_libs: Vec<PathBuf>,
}

impl DesignFile {
    pub fn write(&self) -> String {
        let mut s = format!("{MAGIC}\n");
        for t in &self.tops {
            s += &format!("top {t}\n");
        }
        for (n, v) in &self.params {
            s += &format!("param {n}={v}\n");
        }
        for (n, v) in &self.defines {
            s += &format!("define {n}={v}\n");
        }
        for d in &self.incdirs {
            s += &format!("incdir {}\n", d.display());
        }
        if let Some(t) = &self.timescale {
            s += &format!("timescale {t}\n");
        }
        for f in &self.files {
            s += &format!("file {}\n", f.display());
        }
        for l in &self.sv_libs {
            s += &format!("svlib {}\n", l.display());
        }
        s
    }

    pub fn parse(text: &str) -> Result<DesignFile, String> {
        let mut lines = text.lines();
        if lines.next().map(str::trim) != Some(MAGIC) {
            return Err("not an epitome-sv design file".into());
        }
        let mut d = DesignFile::default();
        for line in lines {
            let line = line.trim_end();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let (key, value) = line.split_once(' ').unwrap_or((line, ""));
            let pair = |v: &str| {
                let (n, t) = v.split_once('=').unwrap_or((v, ""));
                (n.to_string(), t.to_string())
            };
            match key {
                "top" => d.tops.push(value.into()),
                "param" => d.params.push(pair(value)),
                "define" => d.defines.push(pair(value)),
                "incdir" => d.incdirs.push(value.into()),
                "timescale" => d.timescale = Some(value.into()),
                "file" => d.files.push(value.into()),
                "svlib" => d.sv_libs.push(value.into()),
                _ => return Err(format!("unknown setting `{key}` in the design file")),
            }
        }
        Ok(d)
    }

    /// Reads `iverilog`'s command line: the design and where to write it
    /// (`-o`, by default `a.out`). Options that do not change the design
    /// are accepted and ignored.
    pub fn from_iverilog_args(args: &[String]) -> Result<(DesignFile, PathBuf), String> {
        let mut d = DesignFile::default();
        let mut out = PathBuf::from("a.out");
        let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
        let mut queue: std::collections::VecDeque<(String, PathBuf)> = args.iter().map(|a| (a.clone(), cwd.clone())).collect();
        // Options whose value is the next argument.
        let takes = |a: &str| matches!(a, "-o" | "-s" | "-f" | "-c" | "-y" | "-Y" | "-l" | "-M" | "-m" | "-T" | "-t" | "-p" | "-B" | "-D" | "-I" | "-P");
        let mut libs = SvLibs::default();
        while let Some((a, base)) = queue.pop_front() {
            if libs.option(&a, &mut || queue.pop_front().map(|v| v.0))? {
                continue;
            }
            let (opt, val) = if a.len() == 2 && takes(&a) {
                let v = queue.pop_front().map(|v| v.0).ok_or_else(|| format!("{a} needs a value"))?;
                (a.clone(), v)
            } else if a.starts_with('-') && a.len() > 2 && takes(&a[..2]) {
                (a[..2].to_string(), a[2..].to_string())
            } else {
                (a.clone(), String::new())
            };
            let abs = |p: &str| -> PathBuf {
                let p = Path::new(p);
                if p.is_absolute() { p.to_path_buf() } else { base.join(p) }
            };
            match opt.as_str() {
                "-o" => out = abs(&val),
                "-s" => d.tops.push(val),
                "-D" => {
                    let (n, t) = val.split_once('=').unwrap_or((&val, "1"));
                    d.defines.push((n.into(), t.into()));
                }
                "-I" => d.incdirs.push(abs(&val)),
                "-P" => {
                    let (n, t) = val.split_once('=').ok_or_else(|| format!("-P{val}: expected name=value"))?;
                    d.params.push((n.into(), t.into()));
                }
                "-f" | "-c" => {
                    let path = abs(&val);
                    let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
                    let mut words = Vec::new();
                    for line in text.lines() {
                        let line = line.split("//").next().unwrap_or("");
                        let line = if line.trim_start().starts_with('#') { "" } else { line };
                        words.extend(line.split_whitespace().map(|w| (w.to_string(), cwd.clone())));
                    }
                    for w in words.into_iter().rev() {
                        queue.push_front(w);
                    }
                }
                "-l" => d.files.push(abs(&val)),
                "-y" | "-Y" | "-M" | "-m" | "-T" | "-t" | "-p" | "-B" => {}
                _ if a.starts_with("+incdir+") => d.incdirs.extend(a["+incdir+".len()..].split('+').filter(|s| !s.is_empty()).map(abs)),
                _ if a.starts_with("+define+") => {
                    for def in a["+define+".len()..].split('+').filter(|s| !s.is_empty()) {
                        let (n, t) = def.split_once('=').unwrap_or((def, "1"));
                        d.defines.push((n.into(), t.into()));
                    }
                }
                _ if a.starts_with("+timescale+") => d.timescale = Some(a["+timescale+".len()..].to_string()),
                _ if a.starts_with('-') || a.starts_with('+') => {}
                _ => d.files.push(abs(&a)),
            }
        }
        if d.files.is_empty() {
            return Err("no source files".into());
        }
        d.sv_libs = libs.names();
        Ok((d, out))
    }

    /// The preprocessor's options.
    pub fn options(&self) -> Result<Options, String> {
        let timescale = match &self.timescale {
            Some(t) => Some(Timescale::parse(t).ok_or_else(|| format!("bad timescale `{t}`"))?),
            None => None,
        };
        Ok(Options { include_dirs: self.incdirs.clone(), defines: self.defines.clone(), timescale, ..Default::default() })
    }

    /// Top-level parameter values, read as literals.
    pub fn param_values(&self) -> Result<Vec<(String, crate::design::Const)>, String> {
        self.params
            .iter()
            .map(|(n, v)| match crate::design::elab::parse_param_value(v) {
                Some(c) => Ok((n.clone(), c)),
                None => Err(format!("parameter {n}: cannot read the value `{v}`")),
            })
            .collect()
    }
}

/// Preprocesses, parses and elaborates the design, and hands it to `f`;
/// errors come back rendered.
pub fn with_design<R>(df: &DesignFile, f: impl FnOnce(&Design, &crate::ast::Ast) -> R) -> Result<R, String> {
    let map = SourceMap::new();
    let options = df.options()?;
    let params = df.param_values()?;
    let out = crate::preprocess::preprocess_files(&map, &df.files, &options)?;
    let tree = crate::parse::parse(&out).map_err(|e| map.render(&e))?;
    let d = crate::design::elab::elaborate_with(&tree, &out, &df.tops, &params).map_err(|e| map.render(&e))?;
    Ok(f(&d, &tree))
}

/// `vvp`'s command line: the VPI libraries to load (`-M dir`, `-m name`,
/// in order), the design file, and the arguments after it, which the
/// design and the libraries see.
#[derive(Debug, Default)]
pub struct VvpArgs {
    pub libs: Vec<PathBuf>,
    pub design: PathBuf,
    pub extra: Vec<String>,
    /// DPI libraries, as `-sv_lib` names them.
    pub sv_libs: Vec<PathBuf>,
}

impl VvpArgs {
    pub fn parse(args: &[String]) -> Result<VvpArgs, String> {
        let mut dirs: Vec<PathBuf> = Vec::new();
        let mut names = Vec::new();
        let mut it = args.iter();
        let mut design = None;
        let mut sv = SvLibs::default();
        while let Some(a) = it.next() {
            if sv.option(a, &mut || it.next().cloned())? {
                continue;
            }
            let mut value = |a: &str, flag: &str| -> Result<String, String> {
                if a.len() > flag.len() { Ok(a[flag.len()..].to_string()) } else { it.next().cloned().ok_or_else(|| format!("{flag} needs a value")) }
            };
            if a.starts_with("-M") {
                let v = value(a, "-M")?;
                if v == "-" {
                    dirs.clear();
                } else {
                    dirs.push(v.into());
                }
            } else if a.starts_with("-m") {
                names.push(value(a, "-m")?);
            } else if a == "-l" || a == "-d" {
                let _ = it.next();
            } else if a.starts_with('-') {
            } else {
                design = Some(PathBuf::from(a));
                break;
            }
        }
        let design = design.ok_or("no design file")?;
        let extra = it.cloned().collect();
        let mut libs = Vec::new();
        for n in names {
            libs.push(find_lib(&n, &dirs).ok_or_else(|| format!("cannot find VPI module `{n}`"))?);
        }
        Ok(VvpArgs { libs, design, extra, sv_libs: sv.names() })
    }
}

/// DPI libraries named by the options of Annex J.
#[derive(Clone, Debug, Default)]
pub struct SvLibs {
    /// `-sv_root`, for the names after it.
    root: Option<PathBuf>,
    /// From bootstrap files, which load first (J.4), and from `-sv_lib`.
    lists: Vec<PathBuf>,
    libs: Vec<PathBuf>,
}

impl SvLibs {
    /// Takes `arg` if it is `-sv_lib`, `-sv_liblist` or `-sv_root`, with
    /// its value after `=` or from `next`.
    pub fn option(&mut self, arg: &str, next: &mut dyn FnMut() -> Option<String>) -> Result<bool, String> {
        let (flag, inline) = match arg.split_once('=') {
            Some((f, v)) => (f, Some(v.to_string())),
            None => (arg, None),
        };
        if !matches!(flag, "-sv_lib" | "-sv_liblist" | "-sv_root") {
            return Ok(false);
        }
        let value = match inline {
            Some(v) => v,
            None => next().ok_or_else(|| format!("{flag} needs a value"))?,
        };
        let path = self.place(&value)?;
        match flag {
            "-sv_root" => self.root = Some(path),
            "-sv_lib" => self.libs.push(path),
            _ => {
                let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
                let mut lines = text.lines().filter(|l| !l.trim().is_empty());
                let first: String = lines.next().unwrap_or("").split_whitespace().collect();
                if first != "#!SV_LIBRARIES" {
                    return Err(format!("{}: a bootstrap file starts with `#!SV_LIBRARIES` (J.4.1)", path.display()));
                }
                for l in lines {
                    let l = l.trim();
                    if !l.starts_with('#') {
                        let p = self.place(l)?;
                        self.lists.push(p);
                    }
                }
            }
        }
        Ok(true)
    }

    /// A name made absolute: from `-sv_root`, else the working directory.
    fn place(&self, name: &str) -> Result<PathBuf, String> {
        let p = Path::new(name);
        if p.is_absolute() {
            return Ok(p.to_path_buf());
        }
        match &self.root {
            Some(r) => Ok(r.join(p)),
            None => Ok(std::env::current_dir().map_err(|e| e.to_string())?.join(p)),
        }
    }

    /// The names, bootstrap files' first.
    pub fn names(&self) -> Vec<PathBuf> {
        self.lists.iter().chain(&self.libs).cloned().collect()
    }
}

/// The DPI library files to load for these names, then those in
/// `EPITOME_SV_LIBS`, each once.
pub fn sv_lib_files(names: &[PathBuf]) -> Result<Vec<PathBuf>, String> {
    let mut all: Vec<PathBuf> = names.to_vec();
    if let Some(v) = std::env::var_os("EPITOME_SV_LIBS") {
        all.extend(std::env::split_paths(&v).filter(|p| !p.as_os_str().is_empty()));
    }
    let mut out: Vec<PathBuf> = Vec::new();
    for n in all {
        let f = find_sv_lib(&n).ok_or_else(|| format!("cannot find DPI library `{}`", n.display()))?;
        let key = f.canonicalize().unwrap_or_else(|_| f.clone());
        if !out.iter().any(|o| o.canonicalize().unwrap_or_else(|_| o.clone()) == key) {
            out.push(f);
        }
    }
    Ok(out)
}

/// A library's file: the name itself, or with the platform's extension
/// (J.4).
fn find_sv_lib(name: &Path) -> Option<PathBuf> {
    if name.is_file() {
        return Some(name.to_path_buf());
    }
    let exts: &[&str] = if cfg!(target_os = "macos") { &["dylib", "so"] } else if cfg!(windows) { &["dll"] } else { &["so"] };
    exts.iter().map(|e| PathBuf::from(format!("{}.{e}", name.display()))).find(|p| p.is_file())
}

/// A VPI module named on the command line: a path, or a name looked for
/// in the `-M` directories with the usual suffixes.
fn find_lib(name: &str, dirs: &[PathBuf]) -> Option<PathBuf> {
    let p = Path::new(name);
    if p.is_file() {
        return Some(p.to_path_buf());
    }
    let suffixes = ["", ".vpi", ".so", ".dylib", ".dll"];
    for d in dirs.iter().map(PathBuf::as_path).chain([Path::new(".")]) {
        for s in suffixes {
            let c = d.join(format!("{name}{s}"));
            if c.is_file() {
                return Some(c);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iverilog_command_line() {
        let dir = std::env::temp_dir().join(format!("epitome-launch-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let cmds = dir.join("cmds.f");
        std::fs::write(&cmds, "+timescale+1ns/1ps\n// a comment\n+define+A=2+B\n").unwrap();
        let args: Vec<String> =
            ["-o", "/o/sim.vvp", "-s", "top", "-g2012", "-DFOO=1", "-I/inc", "-Ptop.W=12", "-Wall", "-f", cmds.to_str().unwrap(), "/src/top.sv"]
                .iter()
                .map(|s| s.to_string())
                .collect();
        let (d, out) = DesignFile::from_iverilog_args(&args).unwrap();
        assert_eq!(out, PathBuf::from("/o/sim.vvp"));
        assert_eq!(d.tops, ["top"]);
        assert_eq!(d.params, [("top.W".to_string(), "12".to_string())]);
        assert_eq!(d.defines.len(), 3);
        assert_eq!(d.timescale.as_deref(), Some("1ns/1ps"));
        assert_eq!(d.files, [PathBuf::from("/src/top.sv")]);
        assert_eq!(DesignFile::parse(&d.write()).unwrap(), d);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn vvp_command_line() {
        let dir = std::env::temp_dir().join(format!("epitome-vvp-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("app.vpi"), "").unwrap();
        let a: Vec<String> = ["-M", dir.to_str().unwrap(), "-m", "app", "sim.vvp", "+x=1", "-none"].iter().map(|s| s.to_string()).collect();
        let v = VvpArgs::parse(&a).unwrap();
        assert_eq!(v.libs, [dir.join("app.vpi")]);
        assert_eq!(v.design, PathBuf::from("sim.vvp"));
        assert_eq!(v.extra, ["+x=1", "-none"]);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn dpi_libraries() {
        let dir = std::env::temp_dir().join(format!("epitome-svlib-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("sub")).unwrap();
        let ext = if cfg!(target_os = "macos") { "dylib" } else { "so" };
        for n in ["a", "sub/b", "sub/c"] {
            std::fs::write(dir.join(format!("{n}.{ext}")), "").unwrap();
        }
        std::fs::write(dir.join("boot"), "#! SV_LIBRARIES\n# a comment\n  sub/c\n").unwrap();
        let root = dir.to_str().unwrap();
        let a: Vec<String> = ["-sv_root", root, "-sv_lib", "a", "-sv_lib=sub/b", "-sv_liblist", "boot", "-M.", "sim.vvp", "+x"].iter().map(|s| s.to_string()).collect();
        let v = VvpArgs::parse(&a).unwrap();
        assert_eq!(v.design, PathBuf::from("sim.vvp"));
        assert_eq!(v.sv_libs, [dir.join("sub/c"), dir.join("a"), dir.join("sub/b")]);
        let files = sv_lib_files(&[v.sv_libs.clone(), v.sv_libs.clone()].concat()).unwrap();
        assert_eq!(files, [dir.join(format!("sub/c.{ext}")), dir.join(format!("a.{ext}")), dir.join(format!("sub/b.{ext}"))]);
        let args: Vec<String> = ["-o", "/o/x", "-sv_lib", &format!("{root}/a"), "/src/t.sv"].iter().map(|s| s.to_string()).collect();
        let (d, _) = DesignFile::from_iverilog_args(&args).unwrap();
        assert_eq!(d.sv_libs, [dir.join("a")]);
        assert!(d.tops.is_empty());
        assert_eq!(DesignFile::parse(&d.write()).unwrap(), d);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
