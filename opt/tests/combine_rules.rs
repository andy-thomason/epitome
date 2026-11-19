//! Checks that every combine rule is sound, by brute force.
//!
//! Each integer rule is evaluated on every input at widths 1 to 4, on every
//! input at width 8 when it has at most two variables (a sample otherwise),
//! and on a sample at width 64. Wherever the pattern gives a value that is
//! neither poison nor undefined behaviour, the replacement must give the
//! same value. Semantics come from `combine::eval`, the same code the
//! combiner folds with, which follows the Language Reference.
//!
//! Float rules are checked the same way at `float` and `double` precision
//! on a set of special values: signed zeros, infinities, NaN, denormals.
//!
//! Rules with casts or `freeze` change widths or are nondeterministic, so
//! this checker cannot evaluate them; they are listed in `UNCHECKED` and
//! covered by the example and program tests instead.

use epitome_ir::module::{BinOp, IntFlags};
use epitome_opt::combine::dsl::{FloatLit, Head, Lit, Pat, Repl, Rule, Subject, parse_rules};
use epitome_opt::combine::eval::{self, Env, Folded, IntConst, mask};

const UNCHECKED: &[&str] = &[
    "zext_zext",
    "icmp_ne_zext_zero",
    "icmp_eq_zext_one",
    "icmp_eq_zext_zero",
    "icmp_ne_zext_one",
    "select_zext",
    "sext_sext",
    "sext_zext",
    "trunc_trunc",
    "trunc_zext_same",
    "trunc_zext_wide",
    "trunc_zext_narrow",
    "trunc_sext_same",
    "trunc_sext_wide",
    "trunc_sext_narrow",
    "bitcast_same",
    "bitcast_bitcast",
    "freeze_freeze",
];

fn rules() -> Vec<Rule> {
    let text = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/rules/combine.rules")).unwrap();
    parse_rules(&text).unwrap()
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum V {
    Int(u64),
    Poison,
    /// The block a branch goes to: a block variable's number.
    Block(usize),
}

/// An integer evaluation: `None` is undefined behaviour.
struct IntEval<'a> {
    vals: &'a [u64],
    widths: &'a [u32],
}

impl IntEval<'_> {
    fn pat(&self, p: &Pat, w: u32) -> Option<V> {
        match p {
            Pat::Var(v) | Pat::Const(v) => Some(V::Int(self.vals[*v] & mask(w))),
            Pat::Lit(Lit::Int(n)) => Some(V::Int(*n as u64 & mask(w))),
            Pat::Lit(Lit::Poison) => Some(V::Poison),
            Pat::Lit(Lit::Float(_)) => unreachable!(),
            Pat::Block(v) => Some(V::Block(*v)),
            Pat::Op(h, args) => {
                let vals: Vec<Option<V>> =
                    args.iter().zip(operand_widths(*h, w, self.inner(args))).map(|(a, aw)| self.pat(a, aw)).collect();
                apply(*h, &vals, w, self.inner(args))
            }
        }
    }

    fn repl(&self, r: &Repl, w: u32, env: &Env) -> Option<V> {
        match r {
            Repl::Var(v) => Some(V::Int(self.vals[*v] & mask(w))),
            Repl::Lit(Lit::Int(n)) => Some(V::Int(*n as u64 & mask(w))),
            Repl::Lit(Lit::Poison) => Some(V::Poison),
            Repl::Lit(Lit::Float(_)) => unreachable!(),
            Repl::Block(v) => Some(V::Block(*v)),
            Repl::Const(e) => Some(V::Int(eval::eval(e, env)? as u64 & mask(w))),
            Repl::Op(h, args) => {
                let inner = self.inner_repl(args);
                let vals: Vec<Option<V>> =
                    args.iter().zip(operand_widths(*h, w, inner)).map(|(a, aw)| self.repl(a, aw, env)).collect();
                apply(*h, &vals, w, inner)
            }
        }
    }

    /// The width of a comparison's operands: that of its first variable.
    fn inner(&self, args: &[Pat]) -> u32 {
        args.iter()
            .find_map(|a| match a {
                Pat::Var(v) | Pat::Const(v) => Some(self.widths[*v]),
                _ => None,
            })
            .unwrap_or(self.widths[0])
    }

    fn inner_repl(&self, args: &[Repl]) -> u32 {
        args.iter()
            .find_map(|a| match a {
                Repl::Var(v) => Some(self.widths[*v]),
                _ => None,
            })
            .unwrap_or(self.widths[0])
    }
}

/// The widths of an operation's operands, given its result width `w` and,
/// for a comparison, the width it compares.
fn operand_widths(h: Head, w: u32, inner: u32) -> Vec<u32> {
    match h {
        Head::ICmp(_) => vec![inner, inner],
        Head::Select | Head::CondBr => vec![1, w, w],
        _ => vec![w; h.arity()],
    }
}

fn apply(h: Head, vals: &[Option<V>], w: u32, inner: u32) -> Option<V> {
    // Undefined behaviour anywhere below makes the whole thing undefined.
    let vals: Vec<V> = vals.iter().copied().collect::<Option<_>>()?;
    match h {
        Head::Br => Some(vals[0]),
        // Branching on poison is undefined behaviour.
        Head::CondBr => match vals[0] {
            V::Poison => None,
            V::Int(c) => Some(if c & 1 == 1 { vals[1] } else { vals[2] }),
            V::Block(_) => unreachable!(),
        },
        Head::Select => match vals[0] {
            V::Poison => Some(V::Poison),
            V::Int(c) => Some(if c & 1 == 1 { vals[1] } else { vals[2] }),
            V::Block(_) => unreachable!(),
        },
        Head::Bin(op) => {
            // Dividing by poison is undefined behaviour.
            let divides = matches!(op, BinOp::UDiv | BinOp::SDiv | BinOp::URem | BinOp::SRem);
            match (vals[0], vals[1]) {
                (_, V::Poison) if divides => None,
                (V::Poison, _) | (_, V::Poison) => Some(V::Poison),
                (V::Int(a), V::Int(b)) => match eval::fold_binop(op, IntFlags::NONE, a, b, w)? {
                    Folded::Int(v) => Some(V::Int(v)),
                    Folded::Poison => Some(V::Poison),
                },
                _ => unreachable!("a block in arithmetic"),
            }
        }
        Head::ICmp(p) => match (vals[0], vals[1]) {
            (V::Int(a), V::Int(b)) => Some(V::Int(eval::fold_icmp(p, a, b, inner) as u64)),
            _ => Some(V::Poison),
        },
        _ => unreachable!("not an integer operation"),
    }
}

fn is_float(r: &Rule) -> bool {
    fn pat(p: &Pat) -> bool {
        match p {
            Pat::Op(Head::FNeg, _) => true,
            Pat::Op(Head::Bin(op), args) => op.is_float() || args.iter().any(pat),
            Pat::Op(_, args) => args.iter().any(pat),
            Pat::Lit(Lit::Float(_)) => true,
            _ => false,
        }
    }
    pat(&r.pat)
}

/// Widths of each variable, from where it first appears.
fn var_widths(r: &Rule, w: u32) -> Vec<u32> {
    fn walk(p: &Pat, w: u32, out: &mut [Option<u32>], inner: u32) {
        match p {
            Pat::Var(v) | Pat::Const(v) | Pat::Block(v) => {
                out[*v].get_or_insert(w);
            }
            Pat::Op(h, args) => {
                for (a, aw) in args.iter().zip(operand_widths(*h, w, inner)) {
                    walk(a, aw, out, inner);
                }
            }
            Pat::Lit(_) => {}
        }
    }
    let mut out = vec![None; r.vars.len()];
    // The root of a comparison has width 1 and compares `w`-wide values.
    let root_w = if matches!(r.pat, Pat::Op(Head::ICmp(_), _)) { 1 } else { w };
    walk(&r.pat, root_w, &mut out, w);
    out.into_iter().map(|x| x.unwrap_or(w)).collect()
}

/// Checks one integer rule at width `w` on the given inputs. Returns the
/// first counterexample.
fn check_int(r: &Rule, w: u32, inputs: &mut dyn Iterator<Item = Vec<u64>>) -> Result<usize, String> {
    let widths = var_widths(r, w);
    let root_w = if matches!(r.pat, Pat::Op(Head::ICmp(_), _)) { 1 } else { w };
    let mut tried = 0;
    for vals in inputs {
        let consts: Vec<Option<IntConst>> =
            vals.iter().zip(&widths).map(|(&v, &bits)| Some(IntConst { bits, value: v & mask(bits) })).collect();
        let wopt: Vec<Option<u32>> = widths.iter().map(|&x| Some(x)).collect();
        let env = Env { consts: &consts, widths: &wopt, bw: Some(w) };
        let same = |_: Subject, _: Subject| true;
        if !r.conds.iter().all(|c| eval::check(c, &env, &same)) {
            continue;
        }
        let e = IntEval { vals: &vals, widths: &widths };
        let Some(before) = e.pat(&r.pat, root_w) else { continue };
        tried += 1;
        if before == V::Poison {
            continue;
        }
        let after = e.repl(&r.repl, root_w, &env);
        if after != Some(before) {
            let named: Vec<String> = r.vars.iter().zip(&vals).map(|(n, v)| format!("{n} = {v}")).collect();
            return Err(format!(
                "rule {} (line {}) at i{w}: with {} the pattern gives {before:?} but the replacement {after:?}",
                r.name,
                r.line,
                named.join(", ")
            ));
        }
    }
    Ok(tried)
}

struct Lcg(u64);
impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        // Mix in small and extreme values, where bugs live.
        match self.0 >> 61 {
            0 => self.0 >> 60 & 7,
            1 => u64::MAX - (self.0 >> 60 & 7),
            2 => 1 << (self.0 >> 58 & 63),
            _ => self.0 ^ (self.0 >> 29),
        }
    }
}

fn all_inputs(n: usize, w: u32) -> Box<dyn Iterator<Item = Vec<u64>>> {
    let per = 1u64 << w;
    let total = per.pow(n as u32);
    Box::new((0..total).map(move |mut k| {
        (0..n)
            .map(|_| {
                let v = k % per;
                k /= per;
                v
            })
            .collect()
    }))
}

fn sampled(n: usize, count: usize, seed: u64) -> Box<dyn Iterator<Item = Vec<u64>>> {
    let mut rng = Lcg(seed);
    Box::new((0..count).map(move |_| (0..n).map(|_| rng.next()).collect()))
}

#[test]
fn integer_rules_are_sound() {
    let mut failures = Vec::new();
    let mut checked = 0;
    for r in rules() {
        if UNCHECKED.contains(&r.name.as_str()) || is_float(&r) || r.clobber.is_some() {
            continue;
        }
        checked += 1;
        let n = r.vars.len();
        let mut tried = 0;
        for w in [1, 2, 3, 4, 8, 64] {
            let mut inputs = if w <= 4 || (w == 8 && n <= 2) { all_inputs(n, w) } else { sampled(n, 30_000, w as u64) };
            match check_int(&r, w, &mut inputs) {
                Ok(t) => tried += t,
                Err(e) => {
                    failures.push(e);
                    break;
                }
            }
        }
        // A rule whose condition never holds, or whose pattern is always
        // poison, is checking nothing.
        if tried == 0 && failures.is_empty() {
            failures.push(format!("rule {} was never exercised", r.name));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    assert!(checked > 80, "only {checked} integer rules checked");
}

/// Float values worth trying: signed zeros, infinities, NaN, denormals.
const SPECIAL: &[f64] =
    &[0.0, -0.0, 1.0, -1.0, 0.5, -2.75, 1e-310, -1e-310, 1e308, f64::INFINITY, f64::NEG_INFINITY, f64::NAN, 3.0e-39];

fn float_eval(p: &Pat, vals: &[f64], single: bool) -> f64 {
    let round = |x: f64| if single { x as f32 as f64 } else { x };
    match p {
        Pat::Var(v) => round(vals[*v]),
        Pat::Lit(Lit::Float(FloatLit::PosZero)) => 0.0,
        Pat::Lit(Lit::Float(FloatLit::NegZero)) => -0.0,
        Pat::Lit(Lit::Float(FloatLit::One)) => 1.0,
        Pat::Op(Head::FNeg, a) => -float_eval(&a[0], vals, single),
        Pat::Op(Head::Bin(op), a) => {
            let (x, y) = (float_eval(&a[0], vals, single), float_eval(&a[1], vals, single));
            if single {
                let (x, y) = (x as f32, y as f32);
                (match op {
                    BinOp::FAdd => x + y,
                    BinOp::FSub => x - y,
                    BinOp::FMul => x * y,
                    BinOp::FDiv => x / y,
                    _ => unreachable!(),
                }) as f64
            } else {
                match op {
                    BinOp::FAdd => x + y,
                    BinOp::FSub => x - y,
                    BinOp::FMul => x * y,
                    BinOp::FDiv => x / y,
                    _ => unreachable!(),
                }
            }
        }
        p => panic!("unsupported in a float rule: {p:?}"),
    }
}

fn repl_as_pat(r: &Repl) -> Pat {
    match r {
        Repl::Var(v) => Pat::Var(*v),
        Repl::Block(v) => Pat::Block(*v),
        Repl::Lit(l) => Pat::Lit(*l),
        Repl::Op(h, a) => Pat::Op(*h, a.iter().map(repl_as_pat).collect()),
        Repl::Const(_) => panic!("integer constant in a float rule"),
    }
}

#[test]
fn float_rules_are_sound() {
    let mut failures = Vec::new();
    for r in rules().into_iter().filter(is_float) {
        let n = r.vars.len();
        let after = repl_as_pat(&r.repl);
        for single in [false, true] {
            for vals in
                all_inputs(n, 4).map(|ks| ks.iter().map(|&k| SPECIAL[k as usize % SPECIAL.len()]).collect::<Vec<f64>>())
            {
                let (a, b) = (float_eval(&r.pat, &vals, single), float_eval(&after, &vals, single));
                let same = (a.is_nan() && b.is_nan()) || a.to_bits() == b.to_bits();
                if !same {
                    failures.push(format!("rule {} with {vals:?}: {a} before, {b} after", r.name));
                    break;
                }
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn every_rule_is_checked_or_listed() {
    for name in UNCHECKED {
        assert!(rules().iter().any(|r| r.name == *name), "UNCHECKED names a rule that does not exist: {name}");
    }
    for r in rules() {
        let has_cast = format!("{:?}", r.pat).contains("Cast") || format!("{:?}", r.repl).contains("Cast");
        let has_freeze = format!("{:?}", r.pat).contains("Freeze");
        if (has_cast || has_freeze) && !UNCHECKED.contains(&r.name.as_str()) {
            panic!("rule {} cannot be checked here; add it to UNCHECKED and to an example test", r.name);
        }
    }
}

/// A memory rule, `load(p) @ store(v, q) -> r`, against a model of memory:
/// the load gives the stored value exactly when the addresses are the same
/// and the types are (a load of a different width, or from elsewhere,
/// gives something else). So the rule must require `mustalias(p, q)` and
/// `sametype(v, @)`, and its replacement must be `v` on every input.
fn check_memory(r: &Rule) -> Result<(), String> {
    use epitome_opt::combine::dsl::Cond;
    let Some(Pat::Op(Head::Store, args)) = &r.clobber else { return Err("not a memory rule".into()) };
    let Pat::Op(Head::Load, load_args) = &r.pat else { return Err(format!("{}: the root must be a load", r.name)) };
    let (Pat::Var(p), [Pat::Var(v), Pat::Var(q)]) = (&load_args[0], &args[..]) else {
        return Err(format!("{}: the model reads load(p) @ store(v, q) only", r.name));
    };
    let same = |a: usize, b: usize| {
        r.conds.iter().any(|c| match c {
            Cond::MustAlias(Subject::Var(x), Subject::Var(y)) => (*x, *y) == (a, b) || (*x, *y) == (b, a),
            _ => false,
        })
    };
    let typed = r.conds.iter().any(|c| {
        matches!(c, Cond::SameType(Subject::Var(x), Subject::Root) | Cond::SameType(Subject::Root, Subject::Var(x)) if x == v)
    });
    if !same(*p, *q) {
        return Err(format!("{}: the load sees the stored value only where the addresses are the same", r.name));
    }
    if !typed {
        return Err(format!("{}: the load sees the stored value only at the same type", r.name));
    }
    // The replacement, on every input at small widths, must be the value.
    for w in [1, 2, 4, 8] {
        for vals in all_inputs(r.vars.len(), w) {
            let widths = vec![w; r.vars.len()];
            let consts: Vec<Option<IntConst>> =
                vals.iter().map(|&x| Some(IntConst { bits: w, value: x & mask(w) })).collect();
            let wopt: Vec<Option<u32>> = widths.iter().map(|&x| Some(x)).collect();
            let env = Env { consts: &consts, widths: &wopt, bw: Some(w) };
            let e = IntEval { vals: &vals, widths: &widths };
            if e.repl(&r.repl, w, &env) != Some(V::Int(vals[*v] & mask(w))) {
                return Err(format!("{}: the replacement is not the stored value", r.name));
            }
        }
    }
    Ok(())
}

#[test]
fn memory_rules_are_sound() {
    let memory: Vec<Rule> = rules().into_iter().filter(|r| r.clobber.is_some()).collect();
    assert!(!memory.is_empty());
    for r in &memory {
        check_memory(r).unwrap();
    }
    // And it catches wrong ones.
    let wrong = parse_rules(
        "anywhere: load(p) @ store(v, q) -> v if sametype(v, @)
         any_type: load(p) @ store(v, q) -> v if mustalias(p, q)
         other: load(p) @ store(v, q) -> add(v, 1) if mustalias(p, q) && sametype(v, @)",
    )
    .unwrap();
    for r in &wrong {
        assert!(check_memory(r).is_err(), "the checker accepted the wrong memory rule {}", r.name);
    }
}

/// The checker itself must catch wrong rules.
#[test]
fn checker_rejects_wrong_rules() {
    let wrong = parse_rules(
        "wrap: icmp.ult(add(x, 1), x) -> false\n\
         off_by_one: add(add(x, C1), C2) -> add(x, C1 + C2 + 1)\n\
         shift: shl(shl(x, C1), C2) -> shl(x, C1 + C2)\n\
         sign: sdiv(x, 2) -> ashr(x, 1)\n\
         branch: condbr(c, ^a, ^b) -> br(^a)\n\
         swapped: condbr(c, ^a, ^b) -> condbr(c, ^b, ^a)",
    )
    .unwrap();
    for r in &wrong {
        let caught = [1, 2, 3, 4, 8].iter().any(|&w| check_int(r, w, &mut all_inputs(r.vars.len(), w)).is_err());
        assert!(caught, "the checker accepted the wrong rule {}", r.name);
    }
    let r = &parse_rules("plus_zero: fadd(x, 0.0) -> x").unwrap()[0];
    let after = repl_as_pat(&r.repl);
    let caught =
        SPECIAL.iter().any(|&x| float_eval(&r.pat, &[x], false).to_bits() != float_eval(&after, &[x], false).to_bits());
    assert!(caught, "the checker accepted x + 0.0 -> x");
}
