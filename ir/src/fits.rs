//! Does a function fit a partial machine? For a machine that lacks
//! something (a description's `!target.lacks`; Epitome's
//! `docs/design/partial-machines.md`), each use in a function of what the
//! machine does not have, with the block and the instruction, so that a
//! program that cannot be compiled is told why rather than failing
//! somewhere in the code generator.
//!
//! `epitome-llc` runs it on each function before lowering it and reports
//! what it finds as errors; `epitome-opt --target` runs it after
//! optimising and reports warnings, where the optimiser can still be asked
//! to do something about it (inline, unroll, promote to registers).

use crate::module::*;
use crate::printer::{inst_name, local_name};
use crate::types::Type;

/// What `!target.lacks` may list.
pub const LACKABLE: &[&str] = &["memory", "stack", "calls", "globals", "float", "i64", "trap", "indirect"];

/// What a machine lacks, as `check` needs to know it.
#[derive(Clone, Debug, Default)]
pub struct Machine {
    /// `!target.lacks`'s names.
    pub lacks: Vec<String>,
    /// The widest integer the machine holds in a register, in bits.
    pub widest: u32,
    /// How many arguments its convention passes in registers.
    pub arg_regs: usize,
    /// How many registers its convention returns a result in.
    pub result_regs: usize,
}

impl Machine {
    /// Whether the machine lacks `what`: `memory` implies `stack` and
    /// `globals`.
    pub fn lacks(&self, what: &str) -> bool {
        self.lacks.iter().any(|l| l == what || (l == "memory" && (what == "stack" || what == "globals")))
    }

    /// From a description (`!target.lacks`, and `!target.classes` that
    /// hold integers), for a tool that does not read the rest of it.
    /// Arguments are not counted.
    pub fn read(d: &Module) -> Result<Machine, String> {
        let mut mc = Machine { arg_regs: usize::MAX, result_regs: usize::MAX, ..Default::default() };
        for t in tuples(d, "target.lacks") {
            for what in t {
                if !LACKABLE.contains(&what.as_str()) {
                    return Err(format!("!target.lacks: {what} is not one of {}", LACKABLE.join(", ")));
                }
                mc.lacks.push(what);
            }
        }
        for t in tuples(d, "target.classes") {
            let prop = |k: &str| t.iter().position(|x| x == k).and_then(|i| t.get(i + 1));
            if prop("holds").is_some_and(|h| h == "int") && prop("sp").is_none_or(|s| s != "yes") {
                let bits = prop("bits").and_then(|b| b.parse().ok()).unwrap_or(0);
                mc.widest = mc.widest.max(bits);
            }
        }
        Ok(mc)
    }
}

/// The tuples of strings in named metadata `name`.
fn tuples(m: &Module, name: &str) -> Vec<Vec<String>> {
    let strings = |id: MdId| -> Vec<String> {
        match &m.metadata[id.index()].data {
            Metadata::Tuple { ops, .. } => ops
                .iter()
                .filter_map(|o| match o.map(|x| &m.metadata[x.index()].data) {
                    Some(Metadata::String(s)) => Some(String::from_utf8_lossy(s).into_owned()),
                    _ => None,
                })
                .collect(),
            _ => vec![],
        }
    };
    m.named_metadata.iter().filter(|nm| nm.name == name).flat_map(|nm| nm.ops.iter().map(|&op| strings(op))).collect()
}

/// Every way function `f` (named `name`) of module `m` does not fit
/// machine `mc`: one message each, `@name: does not fit: ...`. Empty for
/// a whole machine.
pub fn check(mc: &Machine, m: &Module, f: &Function, name: &str) -> Vec<String> {
    if mc.lacks.is_empty() {
        return vec![];
    }
    let arg_regs = mc.arg_regs;
    let mut c = Fits { mc, m, f, widest: mc.widest, out: Vec::new() };
    let fname = format!("@{name}");
    let say = |out: &mut Vec<String>, place: String, why: &str| {
        let line = format!("{fname}: does not fit: {place}: {why}");
        if !out.contains(&line) {
            out.push(line);
        }
    };

    // Arguments and the result.
    if let Type::Function { ret, params, .. } = m.types.get(f.ty) {
        if let Some(why) = c.type_problem(*ret) {
            say(&mut c.out, "the result".into(), &why);
        }
        let parts = c.scalars(*ret);
        if mc.lacks("stack") && parts > mc.result_regs {
            say(
                &mut c.out,
                "the result".into(),
                &format!(
                    "{parts} values, the convention returns {} in registers, and the machine has no stack",
                    mc.result_regs
                ),
            );
        }
        if mc.lacks("stack") && params.len() > arg_regs {
            say(
                &mut c.out,
                format!("{} arguments", params.len()),
                &format!("only {arg_regs} are passed in registers, and the machine has no stack"),
            );
        }
    }

    for &b in &f.layout {
        let blk = &f.blocks[b.index()];
        let bname = local_name(f, blk.local);
        for &i in &blk.insts {
            let inst = &f.insts[i.index()];
            let place = match inst.result {
                Some(_) => format!("block {bname}, {} = {}", inst_name(f, i), op_name(&inst.op)),
                None => format!("block {bname}, {}", op_name(&inst.op)),
            };
            for why in c.inst_problems(inst) {
                say(&mut c.out, place.clone(), &why);
            }
        }
    }
    // The first few say enough.
    const SHOWN: usize = 8;
    if c.out.len() > SHOWN {
        let more = c.out.len() - SHOWN;
        c.out.truncate(SHOWN);
        c.out.push(format!("{fname}: does not fit: and {more} more"));
    }
    c.out
}

struct Fits<'a> {
    mc: &'a Machine,
    m: &'a Module,
    f: &'a Function,
    widest: u32,
    out: Vec<String>,
}

impl Fits<'_> {
    fn ty(&self, v: Value) -> Option<crate::types::TypeId> {
        match v {
            Value::Local(l) => Some(self.f.locals[l.index()].ty),
            Value::Const(c) => Some(self.m.constant(c).ty),
            Value::Metadata(_) => None,
        }
    }

    /// How many scalars a value of type `ty` is made of (a vector is one).
    fn scalars(&self, ty: crate::types::TypeId) -> usize {
        match self.m.types.get(ty) {
            Type::Void => 0,
            Type::Struct { fields, .. } => fields.iter().map(|&t| self.scalars(t)).sum(),
            Type::Array { len, elem } => *len as usize * self.scalars(*elem),
            _ => 1,
        }
    }

    /// Why a value of type `ty` cannot be held, if it cannot.
    fn type_problem(&self, ty: crate::types::TypeId) -> Option<String> {
        let types = &self.m.types;
        let scalar = match types.get(ty) {
            Type::Vector { elem, .. } => *elem,
            _ => ty,
        };
        match types.get(scalar) {
            Type::Half | Type::BFloat | Type::Float | Type::Double | Type::X86Fp80 | Type::Fp128 | Type::PpcFp128
                if self.mc.lacks("float") =>
            {
                Some("a floating-point value, and the machine has no floating point".into())
            }
            Type::Int(bits) if self.mc.lacks("i64") && *bits > self.widest => {
                Some(format!("an i{bits}, and the machine's integers are at most {} bits (it lacks i64)", self.widest))
            }
            _ => None,
        }
    }

    /// The globals a constant refers to, and whether it takes a block's
    /// address.
    fn const_refs(&self, c: ConstId, globals: &mut Vec<GlobalId>, blocks: &mut bool) {
        match &self.m.constant(c).kind {
            ConstKind::Global(g) | ConstKind::DsoLocalEquivalent(g) | ConstKind::NoCfi(g) => globals.push(*g),
            ConstKind::BlockAddress { .. } => *blocks = true,
            ConstKind::Array(v) | ConstKind::Struct(v) | ConstKind::Vector(v) | ConstKind::PtrAuth(v) => {
                for &k in v {
                    self.const_refs(k, globals, blocks);
                }
            }
            ConstKind::Splat(k) => self.const_refs(*k, globals, blocks),
            ConstKind::Expr(e) => {
                let mut subs = Vec::new();
                expr_consts(e, &mut subs);
                for k in subs {
                    self.const_refs(k, globals, blocks);
                }
            }
            _ => {}
        }
    }

    fn global_name(&self, g: GlobalId) -> String {
        format!("@{}", self.m.global(g).name.clone().unwrap_or_else(|| g.0.to_string()))
    }

    fn inst_problems(&self, inst: &Inst) -> Vec<String> {
        let mc = self.mc;
        let mut out = Vec::new();
        let callee = inst.op.call().map(|c| c.callee);
        match &inst.op {
            Op::Alloca { .. } if mc.lacks("stack") => {
                out.push("an alloca, and the machine has no stack (promote it to registers: epitome-opt)".into())
            }
            Op::Load { .. }
            | Op::Store { .. }
            | Op::Fence { .. }
            | Op::CmpXchg { .. }
            | Op::AtomicRmw { .. }
            | Op::VaArg { .. }
                if mc.lacks("memory") =>
            {
                out.push("the machine has no memory".into())
            }
            Op::IndirectBr { .. } if mc.lacks("indirect") => {
                out.push("the machine has no branch through a register".into())
            }
            Op::LandingPad { .. }
            | Op::Resume(_)
            | Op::CatchPad { .. }
            | Op::CleanupPad { .. }
            | Op::CatchSwitch { .. }
            | Op::CatchRet { .. }
            | Op::CleanupRet { .. }
                if mc.lacks("calls") =>
            {
                out.push("unwinding, and the machine has no calls".into())
            }
            _ => {}
        }
        if let Some(call) = inst.op.call() {
            let target = match call.callee {
                Value::Const(k) => match &self.m.constant(k).kind {
                    ConstKind::Global(g) => Some(Some(*g)),
                    ConstKind::InlineAsm(_) => None,
                    _ => Some(None),
                },
                _ => Some(None),
            };
            match target {
                // Inline assembly names the machine's own instructions.
                None => {}
                Some(Some(g)) => {
                    let n = self.m.global(g).name.clone().unwrap_or_default();
                    if let Some(intrinsic) = n.strip_prefix("llvm.") {
                        if mc.lacks("memory")
                            && ["memcpy", "memmove", "memset"].iter().any(|p| intrinsic.starts_with(p))
                        {
                            out.push(format!("@{n}, and the machine has no memory"));
                        }
                        if mc.lacks("trap") && ["trap", "debugtrap", "ubsantrap"].contains(&intrinsic) {
                            out.push(format!("@{n}, and the machine has no trap"));
                        }
                        if mc.lacks("stack") && (intrinsic.starts_with("stacksave") || intrinsic.starts_with("va_")) {
                            out.push(format!("@{n}, and the machine has no stack"));
                        }
                    } else if mc.lacks("calls") {
                        out.push(format!("a call to @{n}, and the machine has no calls (it must be inlined)"));
                    }
                }
                Some(None) if mc.lacks("calls") || mc.lacks("indirect") => {
                    out.push("a call through a pointer, and the machine has no calls through a register".into())
                }
                Some(None) => {}
            }
        }
        // The values it makes and uses.
        if let Some(r) = inst.result
            && let Some(why) = self.type_problem(self.f.locals[r.index()].ty)
        {
            out.push(why);
        }
        let mut globals = Vec::new();
        let mut blocks = false;
        inst.op.for_each_value(|v| {
            if let Some(why) = self.ty(*v).and_then(|t| self.type_problem(t)) {
                out.push(why);
            }
            if let Value::Const(k) = v
                && Some(*v) != callee
            {
                self.const_refs(*k, &mut globals, &mut blocks);
            }
        });
        if mc.lacks("globals") {
            for g in globals {
                out.push(format!("the address of {}, and the machine has no globals", self.global_name(g)));
            }
        }
        if blocks && mc.lacks("indirect") {
            out.push("a block's address, and the machine has no branch through a register".into());
        }
        out.dedup();
        out
    }
}

/// The constants a constant expression is made of.
fn expr_consts(e: &ConstExpr, out: &mut Vec<ConstId>) {
    match e {
        ConstExpr::Cast { value, .. } => out.push(*value),
        ConstExpr::Gep { ptr, indices, .. } => {
            out.push(*ptr);
            out.extend(indices);
        }
        ConstExpr::Binary { lhs, rhs, .. } => out.extend([*lhs, *rhs]),
        ConstExpr::ExtractElement { vector, index } => out.extend([*vector, *index]),
        // (The rest are rare in code.)
        _ => {}
    }
}

/// An instruction's opcode, as the IR writes it, for messages.
fn op_name(op: &Op) -> String {
    match op {
        Op::Binary { op, .. } => return format!("{op:?}").to_lowercase(),
        Op::Cast { op, .. } => return format!("{op:?}").to_lowercase(),
        _ => {}
    }
    match op {
        Op::Ret(_) => "ret",
        Op::Br(_) | Op::CondBr { .. } => "br",
        Op::Switch { .. } => "switch",
        Op::IndirectBr { .. } => "indirectbr",
        Op::Invoke { .. } => "invoke",
        Op::CallBr { .. } => "callbr",
        Op::Resume(_) => "resume",
        Op::Unreachable => "unreachable",
        Op::FNeg { .. } => "fneg",
        Op::Binary { .. } => "arithmetic",
        Op::Alloca { .. } => "alloca",
        Op::Load { .. } => "load",
        Op::Store { .. } => "store",
        Op::Fence { .. } => "fence",
        Op::CmpXchg { .. } => "cmpxchg",
        Op::AtomicRmw { .. } => "atomicrmw",
        Op::Gep { .. } => "getelementptr",
        Op::Cast { .. } => "cast",
        Op::ICmp { .. } => "icmp",
        Op::FCmp { .. } => "fcmp",
        Op::Phi { .. } => "phi",
        Op::Select { .. } => "select",
        Op::Call(_) => "call",
        Op::VaArg { .. } => "va_arg",
        Op::LandingPad { .. } => "landingpad",
        _ => "instruction",
    }
    .to_string()
}
