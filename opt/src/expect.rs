//! `llvm.expect` as branch weights (`epitome_ir::prof`), for the
//! optimiser's tables. It runs on each function as its task starts, after
//! its locals are promoted (so the branch testing an expected value is in
//! sight) and before anything that would fold through the call.

use epitome_ir::float::{FloatKind, to_f64};
use epitome_ir::module::*;
use epitome_ir::prof;
use epitome_ir::types::{Type, TypeId};

use crate::ctx::Shared;

pub struct Env<'a, 'm>(pub &'a Shared<'m>);

impl prof::Tables for Env<'_, '_> {
    fn callee_name(&self, call: &Call) -> Option<String> {
        self.0.direct_callee(call).map(|g| self.0.name(g))
    }

    fn int_const(&self, c: ConstId) -> Option<u64> {
        match &self.0.consts().get(c).kind {
            ConstKind::Int(v) => Some(v.low_u64()),
            ConstKind::Zero => Some(0),
            _ => None,
        }
    }

    fn double_const(&self, c: ConstId) -> Option<f64> {
        match self.0.consts().get(c).kind {
            ConstKind::Float(bits) => to_f64(FloatKind::Double, bits),
            _ => None,
        }
    }

    fn int_bits(&self, ty: TypeId) -> Option<u32> {
        match self.0.types().get(ty) {
            Type::Int(n) => Some(*n),
            _ => None,
        }
    }

    fn md(&self, id: MdId) -> Metadata {
        self.0.metadata()[id.index()].data.clone()
    }
}

impl prof::Env for Env<'_, '_> {
    fn add_md(&mut self, data: Metadata) -> MdId {
        self.0.add_md(data)
    }

    fn i32(&mut self, v: u32) -> Value {
        self.0.int(32, v as i64)
    }
}

/// Lowers `f`'s `llvm.expect` calls; returns how many went.
pub fn lower(sh: &Shared, f: &mut Function) -> usize {
    prof::lower_expect(&mut Env(sh), f)
}

/// Keeps instruction `i`'s weights right after its op `old` was replaced.
pub fn retarget(sh: &Shared, f: &mut Function, i: InstId, old: &Op) {
    if prof::prof_of(&f.insts[i.index()]).is_some() {
        prof::retarget(&mut Env(sh), &mut f.insts[i.index()], old);
    }
}
