//! Features: which optional parts of the instruction set a function may use.
//!
//! A description's entries may need features (`"feature"="m"`, or several,
//! comma-separated), and it names CPUs and the features each has
//! (`!target.cpus`). The features of a function are its CPU's: the one
//! `--mcpu` names, else the one its IR's `"target-cpu"` names (if the
//! description knows it), else the description's default (`cpu`); then
//! changed by its IR's `"target-features"` (`+neon,-lse`) and by `--mattr`,
//! in that order. Features the description does not know are ignored. A
//! description that names no CPUs has every feature on, unless turned off.
//!
//! A feature brings those it implies (`!target.implies`), as LLVM's do:
//! `+avx512f` turns on AVX2, AVX and the SSEs, and `-avx2` turns off the
//! AVX-512 features with it.
//!
//! With a feature off, the entries that need it are not chosen by
//! instruction selection, and if it is the vector registers' feature,
//! vectors are done lane by lane.

use std::collections::BTreeSet;

use epitome_ir::module::{Attr, Module};

use crate::target::Target;

pub type Features = BTreeSet<String>;

/// Every feature the description mentions.
pub fn known(t: &Target) -> Features {
    let mut all: Features = t.instrs.iter().flat_map(|i| i.features.iter().cloned()).collect();
    for (_, f) in &t.machine.cpus {
        all.extend(f.iter().cloned());
    }
    all.extend(t.machine.vector_widths.iter().filter_map(|(_, f)| f.clone()));
    for (f, implied) in &t.machine.implies {
        all.insert(f.clone());
        all.extend(implied.iter().cloned());
    }
    all
}

/// `f` and the features it implies, directly or through others.
fn with_implied(t: &Target, f: &str) -> Features {
    let mut out = Features::new();
    let mut todo = vec![f.to_string()];
    while let Some(g) = todo.pop() {
        if out.insert(g.clone()) {
            for (h, implied) in &t.machine.implies {
                if *h == g {
                    todo.extend(implied.iter().cloned());
                }
            }
        }
    }
    out
}

fn string_attr(m: &Module, attrs: &[Attr], key: &str) -> Option<String> {
    for a in attrs {
        match a {
            Attr::Str { key: k, value: Some(v) } if k == key.as_bytes() => {
                return Some(String::from_utf8_lossy(v).into_owned());
            }
            Attr::Group(g) => {
                if let Some(group) = m.attr_groups.iter().find(|x| x.id == *g)
                    && let Some(v) = string_attr(m, &group.attrs, key)
                {
                    return Some(v);
                }
            }
            _ => {}
        }
    }
    None
}

/// Applies `+f` and `-f` changes, for the features in `known`: `+f` turns
/// on what `f` implies too, and `-f` turns off what implies `f`.
fn apply(t: &Target, set: &mut Features, known: &Features, changes: &str) {
    for c in changes.split(',').map(str::trim).filter(|c| !c.is_empty()) {
        let (on, name) = match c.split_at(1) {
            ("+", n) => (true, n),
            ("-", n) => (false, n),
            _ => (true, c),
        };
        if known.contains(name) {
            if on {
                set.extend(with_implied(t, name));
            } else {
                set.retain(|g| !with_implied(t, g).contains(name));
            }
        }
    }
}

/// The features of a function with attributes `attrs`, given `--mcpu` and
/// `--mattr`.
pub fn of_function(
    t: &Target,
    m: &Module,
    attrs: &[Attr],
    mcpu: Option<&str>,
    mattr: &[String],
) -> Result<Features, String> {
    let known = known(t);
    let cpu_named = |name: &str| t.machine.cpus.iter().find(|(n, _)| n == name);
    let cpu = match mcpu {
        Some(name) => Some(cpu_named(name).ok_or_else(|| format!("the description has no CPU {name}"))?),
        None => string_attr(m, attrs, "target-cpu")
            .and_then(|c| cpu_named(&c))
            .or_else(|| t.machine.default_cpu.as_deref().and_then(cpu_named)),
    };
    let mut set = match cpu {
        Some((_, f)) => f.iter().flat_map(|g| with_implied(t, g)).collect(),
        None if t.machine.cpus.is_empty() => known.clone(),
        None => Features::new(),
    };
    if let Some(f) = string_attr(m, attrs, "target-features") {
        apply(t, &mut set, &known, &f);
    }
    for a in mattr {
        apply(t, &mut set, &known, a);
    }
    Ok(set)
}
