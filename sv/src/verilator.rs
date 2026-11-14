//! Verilator's model of a design, as the reference to check epitome-sv's
//! netlist against and to time it beside.
//!
//! Verilator is run as a program (`verilator --cc --exe --build`) on the
//! same [`Sources`], with a C++ driver written here for the top's ports.
//! The driver reads stimulus for many instances from a file, in the order
//! [`Io`] gives (for each cycle, for each instance, each input's 32-bit
//! words), and writes each instance's outputs at the end of each cycle in
//! the same order. With a clock, each cycle sets the inputs with the
//! clock low, evaluates, raises the clock and evaluates again, as
//! [`crate::model::Model::cycle`] does. Nothing of Verilator's is linked
//! into this process.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use crate::model::{Io, Source, Sources, fingerprint, words};

/// A Verilator that runs here.
#[derive(Clone, Debug)]
pub struct Verilator {
    pub path: PathBuf,
    /// What `--version` says, as `Verilator 5.053 devel rev ...`.
    pub version: String,
}

/// The oldest Verilator the driver is written for (`VerilatedContext`, a
/// model's root in `V<top>___024root.h`).
pub const OLDEST: (u32, u32) = (5, 0);

impl Verilator {
    /// `$VERILATOR`; else `$VERILATOR_ROOT/bin/verilator`; else `verilator`
    /// on the path: the first that runs.
    pub fn find() -> Option<Verilator> {
        let mut tries: Vec<PathBuf> = Vec::new();
        if let Some(v) = std::env::var_os("VERILATOR") {
            tries.push(v.into());
        }
        if let Some(root) = std::env::var_os("VERILATOR_ROOT") {
            tries.push(Path::new(&root).join("bin").join("verilator"));
        }
        tries.push("verilator".into());
        tries.into_iter().find_map(|path| {
            let out = Command::new(&path).arg("--version").output().ok().filter(|o| o.status.success())?;
            Some(Verilator { path, version: String::from_utf8_lossy(&out.stdout).trim().to_string() })
        })
    }

    /// The version's major and minor numbers.
    pub fn number(&self) -> Option<(u32, u32)> {
        let v = self.version.split_whitespace().nth(1)?;
        let (major, minor) = v.split_once('.')?;
        Some((major.parse().ok()?, minor.trim_end_matches(|c: char| !c.is_ascii_digit()).parse().ok()?))
    }

    /// Whether it is at least [`OLDEST`].
    pub fn new_enough(&self) -> bool {
        self.number().is_some_and(|n| n >= OLDEST)
    }

    /// Builds the model of `top` (whose ports `io` describes) from `src`,
    /// in `dir`: the in-memory sources written to `dir/src`, Verilator's
    /// output in `dir/obj`. Built again only when the driver, the command
    /// or a source has changed since the last build in `dir`.
    pub fn build(&self, dir: &Path, src: &Sources, top: &str, io: &Io) -> Result<Verilated, String> {
        let io_err = |e: std::io::Error| format!("{}: {e}", dir.display());
        std::fs::create_dir_all(dir.join("src")).map_err(io_err)?;
        let driver = driver(top, io);
        let cpp = dir.join("driver.cpp");
        let mut files = Vec::new();
        for item in &src.items {
            files.push(match item {
                Source::File(p) => p.clone(),
                Source::Text { name, text } => {
                    let p = dir.join("src").join(Path::new(name).file_name().unwrap_or(name.as_ref()));
                    if std::fs::read_to_string(&p).ok().as_deref() != Some(text.as_str()) {
                        std::fs::write(&p, text).map_err(io_err)?;
                    }
                    p
                }
            });
        }
        let mut cmd: Vec<String> = ["--cc", "--exe", "--build", "-j", "8", "-O3", "--x-assign", "fast", "--x-initial", "fast", "--noassert"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        cmd.extend(["-Wno-fatal", "-Wno-lint", "-Wno-style", "-CFLAGS", "-O3", "--top-module", top, "-o", "vmodel"].iter().map(|s| s.to_string()));
        // No ccache, so that the build time is the compiler's.
        cmd.extend(["-MAKEFLAGS", "OBJCACHE="].iter().map(|s| s.to_string()));
        cmd.push("-Mdir".into());
        cmd.push(dir.join("obj").display().to_string());
        cmd.extend(src.include_dirs.iter().map(|i| format!("-I{}", i.display())));
        cmd.extend(src.defines.iter().map(|(n, t)| if t.is_empty() { format!("-D{n}") } else { format!("-D{n}={t}") }));
        cmd.extend(files.iter().map(|f| f.display().to_string()));
        cmd.push(cpp.display().to_string());
        let stamp = format!("{}\n{driver}\n{}\n{}", self.version, cmd.join(" "), fingerprint(src));
        let bin = dir.join("obj").join("vmodel");
        let stamp_file = dir.join("stamp");
        let mut built = None;
        if !bin.exists() || std::fs::read_to_string(&stamp_file).ok().as_deref() != Some(stamp.as_str()) {
            std::fs::write(&cpp, &driver).map_err(io_err)?;
            let t0 = Instant::now();
            let out = Command::new(&self.path).args(&cmd).output().map_err(|e| format!("{}: {e}", self.path.display()))?;
            if !out.status.success() {
                return Err(format!("verilator for {top}:\n{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr)));
            }
            built = Some(t0.elapsed());
            std::fs::write(dir.join("build_time"), format!("{}", t0.elapsed().as_secs_f64())).map_err(io_err)?;
            std::fs::write(&stamp_file, &stamp).map_err(io_err)?;
        }
        let build_time = built.or_else(|| {
            let s = std::fs::read_to_string(dir.join("build_time")).ok()?;
            Some(Duration::from_secs_f64(s.trim().parse().ok()?))
        });
        let model_code = dir.join("obj").join(format!("V{top}__ALL.a"));
        Ok(Verilated {
            bin,
            in_words: io.in_words(),
            out_words: io.out_words(),
            built: built.is_some(),
            build_time,
            code_bytes: text_bytes(&model_code),
        })
    }
}

/// Bytes of code and read-only data in an object file or an archive of
/// them, as the system's `size` program counts them (its first column,
/// summed over an archive's members: Berkeley `text`, or Mach-O `__TEXT`).
pub fn text_bytes(object: &Path) -> Option<u64> {
    let out = Command::new("size").arg(object).output().ok().filter(|o| o.status.success())?;
    let text = String::from_utf8_lossy(&out.stdout);
    let sizes: Vec<u64> = text.lines().skip(1).filter_map(|l| l.split_whitespace().next()?.parse().ok()).collect();
    (!sizes.is_empty()).then(|| sizes.iter().sum())
}

/// The C++ driver of `top`'s model for the ports of `io`.
fn driver(top: &str, io: &Io) -> String {
    let mut put = String::new();
    let mut at = 0;
    for p in &io.ins {
        let (n, w) = (&p.name, p.width);
        let _ = match w {
            0..=32 => writeln!(put, "    t->{n} = v[{at}];"),
            33..=64 => writeln!(put, "    t->{n} = (uint64_t)v[{at}] | (uint64_t)v[{}] << 32;", at + 1),
            _ => writeln!(put, "    for (int j = 0; j < {}; j++) t->{n}[j] = v[{at} + j];", words(w)),
        };
        at += words(w);
    }
    let in_words = at;
    let mut get = String::new();
    at = 0;
    for p in &io.outs {
        let (n, w) = (&p.name, p.width);
        let _ = match w {
            0..=32 => writeln!(get, "    v[{at}] = t->{n};"),
            33..=64 => writeln!(get, "    v[{at}] = (uint32_t)t->{n}; v[{}] = (uint32_t)(t->{n} >> 32);", at + 1),
            _ => writeln!(get, "    for (int j = 0; j < {}; j++) v[{at} + j] = t->{n}[j];", words(w)),
        };
        at += words(w);
    }
    let out_words = at;
    let step = match &io.clock {
        Some(c) => format!("t->{0} = 0; t->eval(); t->{0} = 1; t->eval();", c.name),
        None => "t->eval();".into(),
    };
    format!(
        r#"// Written by epitome-sv (sv/src/verilator.rs): Verilator's model of {top},
// run on stimulus for many instances.
#include <chrono>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <vector>
#include "verilated.h"
#include "V{top}.h"
#include "V{top}___024root.h"
typedef V{top} Top;
static const int IN = {in_words}, OUT = {out_words};
static void put(Top *t, const uint32_t *v) {{
{put}}}
static void get(Top *t, uint32_t *v) {{
{get}}}
// driver run|bench CYCLES INSTANCES IN [OUT|REPS]
int main(int argc, char **argv) {{
    bool bench = argv[1][0] == 'b';
    int cycles = atoi(argv[2]), n = atoi(argv[3]);
    std::vector<uint32_t> in((size_t)cycles * n * IN), out((size_t)cycles * n * OUT);
    FILE *f = fopen(argv[4], "rb");
    if (!f || fread(in.data(), 4, in.size(), f) != in.size()) return 2;
    fclose(f);
    VerilatedContext ctx;
    std::vector<Top *> m(n);
    for (auto &t : m) t = new Top(&ctx);
    int reps = bench ? atoi(argv[5]) : 1;
    auto t0 = std::chrono::steady_clock::now();
    for (int r = 0; r < reps; r++)
        for (int c = 0; c < cycles; c++)
            for (int i = 0; i < n; i++) {{
                Top *t = m[i];
                put(t, &in[((size_t)c * n + i) * IN]);
                {step}
                get(t, &out[((size_t)c * n + i) * OUT]);
            }}
    double ns = std::chrono::duration<double, std::nano>(std::chrono::steady_clock::now() - t0).count();
    if (bench) {{
        printf("%.3f %zu\n", ns / ((double)reps * cycles * n), sizeof(V{top}___024root));
        return 0;
    }}
    f = fopen(argv[5], "wb");
    if (!f || fwrite(out.data(), 4, out.size(), f) != out.size()) return 3;
    fclose(f);
    return 0;
}}
"#
    )
}

/// Verilator's model of a top-level module, built as a program.
#[derive(Clone, Debug)]
pub struct Verilated {
    bin: PathBuf,
    in_words: usize,
    out_words: usize,
    /// Whether [`Verilator::build`] built it, rather than finding it built.
    pub built: bool,
    /// How long Verilator and the C++ compiler took, when it was built.
    pub build_time: Option<Duration>,
    /// Bytes of the model's code (`V<top>__ALL.a`, its runtime aside), if
    /// `size` can say.
    pub code_bytes: Option<u64>,
}

/// [`Verilated::bench`]'s result.
#[derive(Clone, Copy, Debug)]
pub struct Bench {
    /// Nanoseconds an instance a cycle.
    pub ns: f64,
    /// Bytes of the model's root, its state.
    pub state_bytes: usize,
}

impl Verilated {
    fn write_stimulus(&self, stim: &[u32]) -> Result<PathBuf, String> {
        let path = self.bin.with_file_name("stimulus.bin");
        let bytes: Vec<u8> = stim.iter().flat_map(|w| w.to_le_bytes()).collect();
        std::fs::write(&path, bytes).map_err(|e| format!("{}: {e}", path.display()))?;
        Ok(path)
    }

    /// What each instance's outputs are at the end of each cycle, for
    /// `stim`: for each cycle, for each instance, its inputs' words.
    pub fn run(&self, cycles: usize, n: usize, stim: &[u32]) -> Result<Vec<u32>, String> {
        assert_eq!(stim.len(), cycles * n * self.in_words, "stimulus for {cycles} cycles of {n} instances");
        let input = self.write_stimulus(stim)?;
        let output = self.bin.with_file_name("response.bin");
        let st = Command::new(&self.bin)
            .args(["run", &cycles.to_string(), &n.to_string()])
            .arg(&input)
            .arg(&output)
            .status()
            .map_err(|e| format!("{}: {e}", self.bin.display()))?;
        if !st.success() {
            return Err(format!("{} failed: {st}", self.bin.display()));
        }
        let bytes = std::fs::read(&output).map_err(|e| format!("{}: {e}", output.display()))?;
        let out: Vec<u32> = bytes.chunks_exact(4).map(|c| u32::from_le_bytes(c.try_into().unwrap())).collect();
        assert_eq!(out.len(), cycles * n * self.out_words);
        Ok(out)
    }

    /// The model's time an instance a cycle, on `stim` run `reps` times
    /// (from fresh models for the first only), and its state's size.
    pub fn bench(&self, cycles: usize, n: usize, stim: &[u32], reps: usize) -> Result<Bench, String> {
        let input = self.write_stimulus(stim)?;
        let out = Command::new(&self.bin)
            .args(["bench", &cycles.to_string(), &n.to_string()])
            .arg(&input)
            .arg(reps.to_string())
            .output()
            .map_err(|e| format!("{}: {e}", self.bin.display()))?;
        let text = String::from_utf8_lossy(&out.stdout);
        let mut it = text.split_whitespace();
        match (it.next().and_then(|x| x.parse().ok()), it.next().and_then(|x| x.parse().ok())) {
            (Some(ns), Some(state_bytes)) => Ok(Bench { ns, state_bytes }),
            _ => Err(format!("{}: no timing in {text:?}", self.bin.display())),
        }
    }
}

impl std::fmt::Display for Bench {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:.2} ns an instance a cycle; {} bytes of state a model", self.ns, self.state_bytes)
    }
}
