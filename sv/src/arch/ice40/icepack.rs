//! Running IceStorm's `icepack` as a black box: `.asc` text in on stdin,
//! a bitstream out on stdout (and `-u` the other way), with no files.

use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

/// The IceStorm commit `sv/tools/ice40_asc_check.py` builds.
pub const ICESTORM_COMMIT: &str = "00da9c3e6ceff056248e3451dd6fd3210d6ceec9";

#[derive(Clone, Debug)]
pub struct Icepack {
    pub path: PathBuf,
    /// The IceStorm commit, if it is the pinned build.
    pub commit: String,
}

impl Icepack {
    /// `$ICEPACK`; else the pinned build `ice40_asc_check.py
    /// --install-only` makes (under `$XDG_CACHE_HOME` or `~/.cache`); else
    /// `icepack` on the PATH.
    pub fn find() -> Option<Icepack> {
        if let Some(p) = std::env::var_os("ICEPACK") {
            return Some(Icepack { path: p.into(), commit: "unknown".into() });
        }
        let cache = std::env::var_os("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".cache")))?;
        let pinned = cache.join("epitome").join(format!("icestorm-{}", &ICESTORM_COMMIT[..12])).join("icepack/icepack");
        if pinned.is_file() {
            return Some(Icepack { path: pinned, commit: ICESTORM_COMMIT[..12].into() });
        }
        let path = std::env::var_os("PATH")?;
        std::env::split_paths(&path)
            .map(|d| d.join("icepack"))
            .find(|p| p.is_file())
            .map(|path| Icepack { path, commit: "unknown".into() })
    }

    fn run(&self, args: &[String], input: &[u8]) -> Result<Vec<u8>, String> {
        let mut child = Command::new(&self.path)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("{}: {e}", self.path.display()))?;
        let mut stdin = child.stdin.take().unwrap();
        let mut stdout = child.stdout.take().unwrap();
        let mut out = Vec::new();
        // Feed stdin from another thread: both pipes can fill.
        std::thread::scope(|s| {
            s.spawn(move || {
                let _ = stdin.write_all(input);
            });
            let _ = stdout.read_to_end(&mut out);
        });
        let res = child.wait_with_output().map_err(|e| e.to_string())?;
        if !res.status.success() {
            let msg = String::from_utf8_lossy(&res.stderr);
            return Err(format!("icepack {}: {}", args.join(" "), msg.lines().next().unwrap_or("failed")));
        }
        Ok(out)
    }

    /// Packs `.asc` text with `icepack`'s options `args`.
    pub fn pack(&self, asc: &str, args: &[String]) -> Result<Vec<u8>, String> {
        self.run(args, asc.as_bytes())
    }

    /// Unpacks a bitstream into `.asc` text.
    pub fn unpack(&self, bin: &[u8]) -> Result<String, String> {
        Ok(String::from_utf8_lossy(&self.run(&["-u".into()], bin)?).into_owned())
    }
}
