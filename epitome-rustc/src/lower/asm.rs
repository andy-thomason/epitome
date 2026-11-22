//! `asm!` on x86-64, as an IR inline-assembly call, the way rustc writes
//! it (checked against its IR):
//!
//! - The templates are joined by newlines. `{0}`, `{name}` and `{}` become
//!   `${k}`, where the IR numbers the outputs first, then the inputs; an
//!   operand's modifier becomes LLVM's (`e` is `k`, `r` is `q`, `x` is
//!   `w`, `l` is `b`), and a `reg` operand without one is `q`, its 64-bit
//!   name. `{{`, `}}` are braces, and `$` is `$$`. A `const` operand's value
//!   is written in; a `sym` operand is an input of constraint `s`, the
//!   function's address, as `${k:c}`.
//! - `out` is `=&` (written before the inputs are all read), `lateout` is
//!   `=`; `inout` adds an input tied to its output (`"0"`). A register
//!   class is `r` (`reg`), `Q` (`reg_abcd`), `q` (`reg_byte`), `x`
//!   (`xmm_reg`, `ymm_reg`) or `v` (`zmm_reg`); a register is `{ax}`.
//! - The flags are clobbered unless `preserves_flags`, memory unless
//!   `nomem` or `readonly`; it has side effects unless `pure`, aligns the
//!   stack unless `nostack`, and is Intel's syntax unless `att_syntax`.

use super::Repr;
use super::body::{Fx, Operand, Place, Pv, R, Val};
use crate::ast::NodeId;
use crate::expand::asm::{Dir, parse};
use crate::ty::TyKind;
use epitome_ir::module::{ConstKind, Constant, InlineAsm, Value};
use epitome_ir::types::TypeId;

/// An output of the call: its constraint, IR type, and where it goes.
struct Out {
    constraint: String,
    ty: TypeId,
    place: Option<Place>,
}

impl Fx<'_, '_, '_, '_> {
    pub fn inline_asm(&mut self, n: NodeId, kids: &[NodeId]) -> R<Pv> {
        let ast = self.body.ast;
        let tokens = ast.tokens_of(kids[1]);
        let inner = tokens.get(1..tokens.len().saturating_sub(1)).ok_or("`asm!` without arguments")?;
        let asm = parse(inner).ok_or("`asm!` arguments not understood")?;
        let nt = asm.templates.len();
        let extra = &kids[2..];
        let nexprs = asm.exprs().count();
        if extra.len() != nt + nexprs {
            return Err("`asm!` whose operands expansion did not parse".into());
        }
        let opt = |w: &str| asm.options.contains(&w);
        let mut template = Vec::new();
        for &t in &extra[..nt] {
            let bytes = super::body::parse_str(ast.token(t).0).ok_or("an `asm!` template that is not a string")?;
            template.push(String::from_utf8(bytes).map_err(|_| "an `asm!` template not UTF-8")?);
        }
        let template = template.join("\n");
        // Each operand's expression nodes.
        let mut node_of = std::collections::HashMap::new();
        for (k, (i, j)) in asm.exprs().enumerate() {
            node_of.insert((i, j), extra[nt + k]);
        }

        // Outputs first, in order, then inputs.
        let mut outs: Vec<Out> = Vec::new();
        let mut index: Vec<Option<usize>> = vec![None; asm.operands.len()];
        for (i, op) in asm.operands.iter().enumerate() {
            if !matches!(op.dir, Dir::Out | Dir::LateOut | Dir::InOut | Dir::InLateOut) {
                continue;
            }
            // The output's place: the last expression (`a => b` writes `b`).
            let out_node = match op.dir {
                Dir::InOut | Dir::InLateOut if op.exprs.len() == 2 => node_of.get(&(i, 1)).copied(),
                _ => node_of.get(&(i, 0)).copied(),
            };
            let (place, ty) = match out_node {
                Some(e) => {
                    let p = self.place(e)?;
                    let Repr::Imm(t) = self.repr(p.ty)? else { return Err("an `asm!` output not a scalar".into()) };
                    (Some(p), t)
                }
                // `_`: kept nowhere, of the input's type or a word.
                None => {
                    let t = match node_of.get(&(i, 0)) {
                        Some(&e) if matches!(op.dir, Dir::InOut | Dir::InLateOut) => {
                            let t = self.ty(e)?;
                            match self.repr(t)? {
                                Repr::Imm(t) => t,
                                _ => return Err("an `asm!` operand not a scalar".into()),
                            }
                        }
                        _ => self.b.module().types.int(64),
                    };
                    (None, t)
                }
            };
            let early = if matches!(op.dir, Dir::Out | Dir::InOut) { "=&" } else { "=" };
            let constraint = format!("{early}{}", class_constraint(op.spec)?);
            index[i] = Some(outs.len());
            outs.push(Out { constraint, ty, place });
        }
        let mut in_constraints: Vec<String> = Vec::new();
        let mut in_values: Vec<Value> = Vec::new();
        let mut text_of: Vec<Option<String>> = vec![None; asm.operands.len()];
        for (i, op) in asm.operands.iter().enumerate() {
            match op.dir {
                Dir::In => {
                    let e = node_of[&(i, 0)];
                    let o = self.operand(e)?;
                    let v = self.imm(o)?;
                    index[i] = Some(outs.len() + in_constraints.len());
                    in_constraints.push(class_constraint(op.spec)?);
                    in_values.push(v);
                }
                Dir::InOut | Dir::InLateOut => {
                    let e = node_of[&(i, 0)];
                    let o = self.operand(e)?;
                    let v = self.imm(o)?;
                    in_constraints.push(index[i].unwrap().to_string());
                    in_values.push(v);
                }
                Dir::Const => {
                    let e = node_of[&(i, 0)];
                    let o = self.operand(e)?;
                    let Val::Imm(Value::Const(c)) = o.v else { return Err("an `asm!` const operand not a constant".into()) };
                    let signed = self.signed(o.ty);
                    let ConstKind::Int(a) = &self.b.module().consts.get(c).kind else { return Err("an `asm!` const operand not an integer".into()) };
                    let text = if signed { a.to_i64().map(|v| v.to_string()) } else { Some(a.low_u64().to_string()) };
                    text_of[i] = Some(text.ok_or("an `asm!` const operand too wide")?);
                }
                // As rustc writes it: an input of constraint `s`, the
                // function's address, which keeps the function.
                Dir::Sym => {
                    let e = node_of[&(i, 0)];
                    let t = self.ty(e)?;
                    let TyKind::FnDef(d, l) = self.cx.tcx.kind(t) else { return Err("an `asm!` sym operand not a function".into()) };
                    let f = self.fn_address(d, self.cx.tcx.tys(l))?;
                    index[i] = Some(outs.len() + in_constraints.len());
                    in_constraints.push("s".into());
                    in_values.push(f);
                }
                _ => {}
            }
        }

        // The template, in LLVM's form.
        let mut asm_text = String::new();
        let mut chars = template.chars().peekable();
        let mut next_implicit = 0;
        while let Some(c) = chars.next() {
            match c {
                '{' if chars.peek() == Some(&'{') => {
                    chars.next();
                    asm_text.push('{');
                }
                '}' if chars.peek() == Some(&'}') => {
                    chars.next();
                    asm_text.push('}');
                }
                '$' => asm_text.push_str("$$"),
                '{' => {
                    let mut spec = String::new();
                    for d in chars.by_ref() {
                        if d == '}' {
                            break;
                        }
                        spec.push(d);
                    }
                    let (arg, modifier) = spec.split_once(':').unwrap_or((&spec, ""));
                    let i = if arg.is_empty() {
                        next_implicit += 1;
                        next_implicit - 1
                    } else if let Ok(k) = arg.parse::<usize>() {
                        k
                    } else {
                        asm.operands.iter().position(|o| o.name == Some(arg)).ok_or_else(|| format!("`asm!` operand `{arg}` not found"))?
                    };
                    let op = asm.operands.get(i).ok_or("an `asm!` operand index out of range")?;
                    if let Some(t) = &text_of[i] {
                        asm_text.push_str(t);
                        continue;
                    }
                    let k = index[i].ok_or("an `asm!` operand with no register")?;
                    let class = op.spec.first().map(|t| t.0).unwrap_or("");
                    let m = match (modifier, class) {
                        _ if op.dir == Dir::Sym => "c",
                        ("e", _) => "k",
                        ("r", _) => "q",
                        ("x", "xmm_reg" | "ymm_reg" | "zmm_reg") => "x",
                        ("x", _) => "w",
                        ("l", _) => "b",
                        ("h", _) => "h",
                        ("y", _) => "t",
                        ("z", _) => "g",
                        ("", "reg" | "reg_abcd") => "q",
                        ("", _) => "",
                        (m, _) => return Err(format!("`asm!` modifier `{m}` is not supported")),
                    };
                    if m.is_empty() {
                        asm_text.push_str(&format!("${{{k}}}"));
                    } else {
                        asm_text.push_str(&format!("${{{k}:{m}}}"));
                    }
                }
                c => asm_text.push(c),
            }
        }

        // Constraints and clobbers.
        let mut constraints: Vec<String> = outs.iter().map(|o| o.constraint.clone()).collect();
        constraints.extend(in_constraints);
        if !asm.clobber_abis.is_empty() {
            let used: Vec<String> = outs.iter().map(|o| o.constraint.trim_start_matches(['=', '&']).to_string()).collect();
            for r in ["ax", "cx", "dx", "si", "di", "r8", "r9", "r10", "r11"]
                .iter()
                .map(|r| r.to_string())
                .chain((0..16).map(|k| format!("xmm{k}")))
            {
                let c = format!("{{{r}}}");
                if !used.contains(&c) {
                    constraints.push(format!("~{c}"));
                }
            }
        }
        if !opt("preserves_flags") {
            constraints.extend(["~{dirflag}", "~{fpsr}", "~{flags}"].map(String::from));
        }
        if !opt("nomem") && !opt("readonly") {
            constraints.push("~{memory}".into());
        }

        let ret = match outs.len() {
            0 => self.b.module().types.void(),
            1 => outs[0].ty,
            _ => {
                let tys: Vec<TypeId> = outs.iter().map(|o| o.ty).collect();
                self.b.module().types.struct_of(&tys)
            }
        };
        let params: Vec<TypeId> = in_values.iter().map(|&v| self.b.type_of(v)).collect();
        let fn_ty = self.b.module().types.function(ret, &params, false);
        let a = InlineAsm {
            fn_ty,
            side_effect: !opt("pure"),
            align_stack: !opt("nostack"),
            intel_dialect: !opt("att_syntax"),
            unwind: false,
            asm: asm_text.into_bytes(),
            constraints: constraints.join(",").into_bytes(),
        };
        let ptr = self.ptr_t();
        let callee = Value::Const(self.b.module().consts.intern(Constant { ty: ptr, kind: ConstKind::InlineAsm(a) }));
        let r = self.b.call_indirect(fn_ty, callee, &in_values);
        for (k, o) in outs.iter().enumerate() {
            let Some(p) = o.place else { continue };
            let v = if outs.len() == 1 { r.ok_or("`asm!` without its result")? } else { self.b.extract_value(r.ok_or("`asm!` without its results")?, &[k as u32]) };
            self.store(Operand { ty: p.ty, v: Val::Imm(v) }, p)?;
        }
        let t = self.ty(n)?;
        if opt("noreturn") {
            self.b.unreachable();
            self.dead();
        }
        Ok(Pv::Val(Operand { ty: t, v: Val::Zst }))
    }
}

/// The constraint for an operand's register class or register.
fn class_constraint(spec: &[crate::lexer::Token]) -> R<String> {
    let [t] = spec else { return Err("an `asm!` register not understood".into()) };
    if t.is_string() {
        let reg = crate::cfg::string_value(t.0);
        return Ok(format!("{{{}}}", llvm_register(reg).ok_or_else(|| format!("`asm!` register `{reg}` is not supported"))?));
    }
    Ok(match t.0 {
        "reg" => "r",
        "reg_abcd" => "Q",
        "reg_byte" => "q",
        "xmm_reg" | "ymm_reg" => "x",
        "zmm_reg" => "v",
        "kreg" => "^Yk",
        c => return Err(format!("`asm!` register class `{c}` is not supported")),
    }
    .to_string())
}

/// LLVM's name for an x86-64 register written in `asm!` (`eax` is `ax`).
fn llvm_register(r: &str) -> Option<String> {
    let base = match r {
        "rax" | "eax" | "ax" | "al" | "ah" => "ax",
        "rbx" | "ebx" | "bx" | "bl" | "bh" => "bx",
        "rcx" | "ecx" | "cx" | "cl" | "ch" => "cx",
        "rdx" | "edx" | "dx" | "dl" | "dh" => "dx",
        "rsi" | "esi" | "si" | "sil" => "si",
        "rdi" | "edi" | "di" | "dil" => "di",
        "rbp" | "ebp" | "bp" | "bpl" => "bp",
        _ => {
            if let Some(n) = r.strip_prefix('r') {
                let n = n.trim_end_matches(['d', 'w', 'b']);
                if n.parse::<u32>().is_ok_and(|k| (8..16).contains(&k)) {
                    return Some(format!("r{n}"));
                }
            }
            if r.starts_with("xmm") || r.starts_with("ymm") || r.starts_with("zmm") {
                return Some(r.to_string());
            }
            return None;
        }
    };
    Some(base.to_string())
}
