//! SME's streaming mode and ZA, as functions' attributes ask for them (the
//! ACLE's `__arm_locally_streaming`, `__arm_new("za")`), made calls of the
//! description's entries (`llvm.epitome.sme.*`: `smstart sm`, `smstop
//! za`) at the function's ends and around its calls.
//!
//! - `aarch64_pstate_sm_body`: streaming mode on after the prologue, off
//!   before each return. `aarch64_pstate_sm_enabled`: the body is in
//!   streaming mode already (its callers switch).
//! - `aarch64_new_za`: ZA of its own: a caller's lazily saved ZA committed
//!   first (`__arm_tpidr2_save`, if TPIDR2_EL0 says there is one), then ZA
//!   on and zeroed, and off before each return; `aarch64_new_zt0`, ZT0
//!   zeroed too.
//! - A call from a streaming body to a function that is not streaming has
//!   streaming mode off around it, and from one that is not to one that is
//!   (`aarch64_pstate_sm_enabled`), on.
//!
//! The switches clear the vector and predicate registers, and their
//! entries say so ("clobbers"), so that no value stays in one across them.
//! Not yet: calls from a function with ZA live to one without (which need
//! ZA's lazy save, a TPIDR2 block), streaming-compatible callers of
//! functions that are not (which switch by a test of PSTATE.SM), and
//! scalable vectors into or out of a function that switches (whose length
//! the switch changes).

use epitome_ir::module::*;
use epitome_ir::types::Type;

use crate::prepare::{Ctx, Seq, call_intrinsic};

/// Whether attribute `key` (`"aarch64_new_za"`) is among `attrs` or the
/// groups they name.
fn has(m: &Module, attrs: &[Attr], key: &str) -> bool {
    attrs.iter().any(|a| match a {
        Attr::Str { key: k, .. } => k == key.as_bytes(),
        Attr::Group(g) => m.attr_groups.iter().find(|x| x.id == *g).is_some_and(|x| has(m, &x.attrs, key)),
        _ => false,
    })
}

/// A function's (or a call's) mode: streaming, not, or either.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Normal,
    Streaming,
    Compatible,
    /// Streaming within, by a switch of its own.
    Body,
}

fn mode(m: &Module, attrs: &[Attr]) -> Mode {
    if has(m, attrs, "aarch64_pstate_sm_body") {
        Mode::Body
    } else if has(m, attrs, "aarch64_pstate_sm_enabled") {
        Mode::Streaming
    } else if has(m, attrs, "aarch64_pstate_sm_compatible") {
        Mode::Compatible
    } else {
        Mode::Normal
    }
}

/// Whether a function with these attributes shares its caller's ZA or
/// ZT0, or keeps it (so that a call of it needs no lazy save).
fn shares_za(m: &Module, attrs: &[Attr]) -> bool {
    ["aarch64_in_za", "aarch64_out_za", "aarch64_inout_za", "aarch64_preserves_za", "aarch64_za_state_agnostic"]
        .iter()
        .any(|k| has(m, attrs, k))
}

fn has_za(m: &Module, attrs: &[Attr]) -> bool {
    shares_za(m, attrs) || has(m, attrs, "aarch64_new_za")
}

/// The attributes of a call's callee, and the call's own.
fn callee_attrs(m: &Module, call: &Call) -> Vec<Attr> {
    let mut out = call.fn_attrs.clone();
    if let Value::Const(c) = call.callee
        && let ConstKind::Global(g) = m.constant(c).kind
        && let GlobalKind::Function(f) = &m.global(g).kind
    {
        out.extend(f.fn_attrs.iter().cloned());
    }
    out
}

fn switch(cx: &mut Ctx, s: &mut Seq, what: &str) {
    let void = cx.m.types.intern(Type::Void);
    call_intrinsic(cx, s, &format!("llvm.epitome.sme.{what}"), vec![], void);
}

pub fn lower(cx: &mut Ctx, f: &mut Function, fname: &str) -> Result<(), String> {
    let m: &Module = cx.m;
    let own = mode(m, &f.fn_attrs);
    let new_za = has(m, &f.fn_attrs, "aarch64_new_za");
    let new_zt0 = has(m, &f.fn_attrs, "aarch64_new_zt0");
    let za_live = has_za(m, &f.fn_attrs) || new_zt0;
    if own == Mode::Normal && !za_live && !calls_streaming(cx, f) {
        return Ok(());
    }
    let switches = own == Mode::Body || new_za;
    if own == Mode::Body {
        let scalable = |t| crate::target::is_scalable_type(cx.m, t);
        let ret_scalable = matches!(cx.m.types.get(f.ty), Type::Function { ret, .. } if scalable(*ret));
        if f.params.iter().any(|p| scalable(f.locals[p.local.index()].ty)) || ret_scalable {
            return Err(format!(
                "@{fname}: scalable vectors into or out of a locally streaming function are not supported yet"
            ));
        }
    }
    // Around calls: streaming mode as the callee wants it.
    let body = match own {
        Mode::Body | Mode::Streaming => Mode::Streaming,
        o => o,
    };
    for b in f.layout.clone() {
        let mut k = 0;
        while k < f.blocks[b.index()].insts.len() {
            let i = f.blocks[b.index()].insts[k];
            let Op::Call(call) = &f.insts[i.index()].op else {
                k += 1;
                continue;
            };
            let name = crate::prepare::callee_name(cx.m, call).unwrap_or_default();
            if name.starts_with("llvm.") || name == "__arm_tpidr2_save" {
                k += 1;
                continue;
            }
            let attrs = callee_attrs(cx.m, call);
            let callee = mode(cx.m, &attrs);
            if za_live && !has_za(cx.m, &attrs) {
                return Err(format!(
                    "@{fname}: a call of @{name}, which has no ZA, with ZA live (its lazy save) is not supported yet"
                ));
            }
            let (before, after) = match (body, callee) {
                (Mode::Streaming, Mode::Normal) => ("stop.sm", "start.sm"),
                (Mode::Normal, Mode::Streaming) => ("start.sm", "stop.sm"),
                (Mode::Compatible, Mode::Normal | Mode::Streaming) => {
                    return Err(format!(
                        "@{fname}: a streaming-compatible function's call of @{name}, which is not, is not supported yet"
                    ));
                }
                _ => {
                    k += 1;
                    continue;
                }
            };
            let mut s = Seq { f, new: Vec::new() };
            switch(cx, &mut s, before);
            let pre = std::mem::take(&mut s.new);
            switch(cx, &mut s, after);
            let post = std::mem::take(&mut s.new);
            let n = pre.len();
            f.blocks[b.index()].insts.splice(k + 1..k + 1, post);
            f.blocks[b.index()].insts.splice(k..k, pre);
            k += n + 2;
        }
    }
    if !switches && !new_zt0 {
        return Ok(());
    }
    // Before each return: streaming mode off, then ZA.
    for b in f.layout.clone() {
        let Some(t) = f.terminator(b) else { continue };
        if !matches!(f.insts[t.index()].op, Op::Ret(_)) {
            continue;
        }
        let mut s = Seq { f, new: Vec::new() };
        if own == Mode::Body {
            switch(cx, &mut s, "stop.sm");
        }
        if new_za || new_zt0 {
            switch(cx, &mut s, "stop.za");
        }
        let new = std::mem::take(&mut s.new);
        let n = f.blocks[b.index()].insts.len();
        f.blocks[b.index()].insts.splice(n - 1..n - 1, new);
    }
    // The prologue, after the entry block's allocas.
    let entry = f.entry();
    let at = f.blocks[entry.index()]
        .insts
        .iter()
        .position(|&i| !matches!(f.insts[i.index()].op, Op::Alloca { .. }))
        .unwrap_or(0);
    let i64t = cx.m.types.int(64);
    let i32t = cx.m.types.int(32);
    let i1 = cx.m.types.int(1);
    if new_za || new_zt0 {
        // A caller's ZA, lazily saved: committed (TPIDR2_EL0 says where),
        // then TPIDR2_EL0 cleared.
        let on = f.split_block(entry, at, None);
        let save = f.add_block(None);
        let pos = f.layout.iter().position(|&x| x == on).unwrap();
        f.layout.insert(pos, save);
        let mut s = Seq { f, new: Vec::new() };
        let t = call_intrinsic(cx, &mut s, "llvm.epitome.sme.tpidr2", vec![], i64t);
        let zero = cx.int(i64t, 0);
        let none = s.op(Op::ICmp { pred: ICmpPred::Eq, samesign: false, lhs: t, rhs: zero }, i1);
        let pre = std::mem::take(&mut s.new);
        let void = cx.m.types.intern(Type::Void);
        let callee = crate::legalize::declare(cx, "__arm_tpidr2_save", void, &[]);
        let fn_ty = cx.m.types.intern(Type::Function { ret: void, params: vec![], varargs: false });
        s.op_void(Op::Call(Box::new(Call {
            tail: TailKind::None,
            fmf: FastMath::default(),
            cconv: CallConv::default(),
            ret_attrs: vec![],
            addrspace: None,
            fn_ty,
            callee,
            args: vec![],
            fn_attrs: vec![],
            bundles: vec![],
        })));
        switch(cx, &mut s, "tpidr2.clear");
        let saving = std::mem::take(&mut s.new);
        // Then ZA on, zeroed (and ZT0), and streaming mode.
        switch(cx, &mut s, "start.za");
        if own == Mode::Body {
            switch(cx, &mut s, "start.sm");
        }
        if new_za {
            let all = cx.int(i32t, 255);
            call_intrinsic(cx, &mut s, "llvm.aarch64.sme.zero", vec![all], void);
        }
        if new_zt0 {
            let zt = cx.int(i32t, 0);
            call_intrinsic(cx, &mut s, "llvm.aarch64.sme.zero.zt", vec![zt], void);
        }
        let start = std::mem::take(&mut s.new);
        // (The entry block now ends at its allocas; the split retargeted
        // the phis of its old successors to `on`.)
        f.blocks[entry.index()].insts.extend(pre);
        f.push_inst(entry, Op::CondBr { cond: none, then_dest: on, else_dest: save }, None);
        f.blocks[save.index()].insts.extend(saving);
        f.push_inst(save, Op::Br(on), None);
        f.blocks[on.index()].insts.splice(0..0, start);
    } else if own == Mode::Body {
        let mut s = Seq { f, new: Vec::new() };
        switch(cx, &mut s, "start.sm");
        let start = std::mem::take(&mut s.new);
        f.blocks[entry.index()].insts.splice(at..at, start);
    }
    Ok(())
}

/// Whether `f` calls a streaming function (whose call switches).
fn calls_streaming(cx: &Ctx, f: &Function) -> bool {
    f.insts.iter().any(|i| match &i.op {
        Op::Call(call) => mode(cx.m, &callee_attrs(cx.m, call)) == Mode::Streaming,
        _ => false,
    })
}
