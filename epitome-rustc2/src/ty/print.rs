//! Printing types and signatures.

use super::{AdtKind, Const, CtorKind, Pred, TraitRef, Ty, TyCtxt, TyKind};
use super::{FloatTy, IntTy, Mutbl, UintTy};
use crate::resolve::{DefId, DefKind, Resolver};
use std::fmt::Write;

impl TyCtxt<'_> {
    /// A type as Rust source would write it (items by their own name).
    pub fn display(&self, r: &Resolver, t: Ty) -> String {
        let mut s = String::new();
        self.write_ty(r, t, &mut s);
        s
    }

    fn write_list(&self, r: &Resolver, tys: &[Ty], out: &mut String) {
        for (i, &t) in tys.iter().enumerate() {
            if i > 0 {
                out.push_str(", ");
            }
            self.write_ty(r, t, out);
        }
    }

    fn write_args(&self, r: &Resolver, tys: &[Ty], out: &mut String) {
        if !tys.is_empty() {
            out.push('<');
            self.write_list(r, tys, out);
            out.push('>');
        }
    }

    pub fn write_trait_ref(&self, r: &Resolver, t: &TraitRef, out: &mut String) {
        out.push_str(r.defs[t.def.0 as usize].name);
        let args = self.tys(t.args);
        if args.is_empty() && t.bindings.is_empty() {
            return;
        }
        out.push('<');
        self.write_list(r, &args, out);
        for (i, &(a, b)) in t.bindings.iter().enumerate() {
            if i > 0 || !args.is_empty() {
                out.push_str(", ");
            }
            let _ = write!(out, "{} = ", r.defs[a.0 as usize].name);
            self.write_ty(r, b, out);
        }
        out.push('>');
    }

    fn write_ty(&self, r: &Resolver, t: Ty, out: &mut String) {
        match self.kind(t) {
            TyKind::Bool => out.push_str("bool"),
            TyKind::Char => out.push_str("char"),
            TyKind::Str => out.push_str("str"),
            TyKind::Int(i) => out.push_str(match i {
                IntTy::I8 => "i8",
                IntTy::I16 => "i16",
                IntTy::I32 => "i32",
                IntTy::I64 => "i64",
                IntTy::I128 => "i128",
                IntTy::Isize => "isize",
            }),
            TyKind::Uint(u) => out.push_str(match u {
                UintTy::U8 => "u8",
                UintTy::U16 => "u16",
                UintTy::U32 => "u32",
                UintTy::U64 => "u64",
                UintTy::U128 => "u128",
                UintTy::Usize => "usize",
            }),
            TyKind::Float(f) => out.push_str(match f {
                FloatTy::F16 => "f16",
                FloatTy::F32 => "f32",
                FloatTy::F64 => "f64",
                FloatTy::F128 => "f128",
            }),
            TyKind::Never => out.push('!'),
            TyKind::Tuple(l) => {
                let tys = self.tys(l);
                out.push('(');
                self.write_list(r, &tys, out);
                if tys.len() == 1 {
                    out.push(',');
                }
                out.push(')');
            }
            TyKind::Ref(m, x) => {
                out.push_str(if m == Mutbl::Mut { "&mut " } else { "&" });
                self.write_ty(r, x, out);
            }
            TyKind::Ptr(m, x) => {
                out.push_str(if m == Mutbl::Mut { "*mut " } else { "*const " });
                self.write_ty(r, x, out);
            }
            TyKind::Array(x, c) => {
                out.push('[');
                self.write_ty(r, x, out);
                out.push_str("; ");
                self.write_const(c, out);
                out.push(']');
            }
            TyKind::Slice(x) => {
                out.push('[');
                self.write_ty(r, x, out);
                out.push(']');
            }
            TyKind::Adt(d, l) => {
                out.push_str(r.defs[d.0 as usize].name);
                self.write_args(r, &self.tys(l), out);
            }
            TyKind::FnDef(d, l) => {
                let _ = write!(out, "fn item {}", r.def_path(d));
                self.write_args(r, &self.tys(l), out);
            }
            TyKind::FnPtr(s) => {
                let s = self.sig_of(s);
                if s.unsafety {
                    out.push_str("unsafe ");
                }
                if s.abi != super::Abi::Rust {
                    out.push_str("extern \"C\" ");
                }
                out.push_str("fn(");
                self.write_list(r, &self.tys(s.inputs), out);
                if s.variadic {
                    out.push_str(", ...");
                }
                out.push(')');
                if self.kind(s.output) != TyKind::Tuple(self.list(Vec::new())) {
                    out.push_str(" -> ");
                    self.write_ty(r, s.output, out);
                }
            }
            TyKind::Dyn(d) => {
                out.push_str("dyn ");
                for (i, t) in self.dyn_of(d).traits.iter().enumerate() {
                    if i > 0 {
                        out.push_str(" + ");
                    }
                    self.write_trait_ref(r, t, out);
                }
            }
            TyKind::Param(p) => out.push_str(self.params[p.0 as usize].name),
            TyKind::Projection(a, l) => {
                let tys = self.tys(l);
                let assoc = &r.defs[a.0 as usize];
                // The trait is the associated type's parent.
                let tr = assoc.parent.and_then(|m| r.modules[m.0 as usize].def);
                out.push('<');
                self.write_ty(r, tys[0], out);
                if let Some(tr) = tr {
                    let _ = write!(out, " as {}", r.defs[tr.0 as usize].name);
                    self.write_args(r, &tys[1..], out);
                }
                let _ = write!(out, ">::{}", assoc.name);
            }
            TyKind::Opaque(o) => {
                out.push_str("impl ");
                for (i, t) in self.opaque(o).bounds.iter().enumerate() {
                    if i > 0 {
                        out.push_str(" + ");
                    }
                    self.write_trait_ref(r, t, out);
                }
            }
            TyKind::Foreign(d) => out.push_str(r.defs[d.0 as usize].name),
            TyKind::Pat(x, a, b) => {
                out.push_str("pattern_type!(");
                self.write_ty(r, x, out);
                let v = |c| match self.const_of(c) {
                    super::Const::Value(v) => v.to_string(),
                    _ => "?".into(),
                };
                out.push_str(&format!(" is {}..={})", v(a), v(b)));
            }
            TyKind::Closure(_, s) => {
                let s = self.sig_of(s);
                out.push_str("{closure}(");
                self.write_list(r, &self.tys(s.inputs), out);
                out.push_str(") -> ");
                self.write_ty(r, s.output, out);
            }
            TyKind::Infer(v) => {
                let _ = write!(out, "?{v}");
            }
            TyKind::ConstArg(c) => self.write_const(c, out),
            TyKind::Error => out.push_str("{error}"),
        }
    }

    fn write_const(&self, c: super::ConstId, out: &mut String) {
        match self.const_of(c) {
            Const::Value(v) => {
                let _ = write!(out, "{v}");
            }
            Const::Param(p) => out.push_str(self.params[p.0 as usize].name),
            Const::Expr(..) => out.push_str("{expr}"),
            Const::Error => out.push_str("{error}"),
        }
    }

    /// `<T: Bound, const N: usize>` for an item's own parameters (bounds are
    /// in its predicates).
    fn write_generics(&self, d: DefId, out: &mut String) {
        let g = &self.generics[d.0 as usize];
        let own: Vec<_> = g.own.iter().filter(|p| self.params[p.0 as usize].name != "Self").collect();
        if own.is_empty() {
            return;
        }
        out.push('<');
        for (i, p) in own.iter().enumerate() {
            if i > 0 {
                out.push_str(", ");
            }
            let p = &self.params[p.0 as usize];
            if p.is_const {
                out.push_str("const ");
            }
            out.push_str(p.name);
        }
        out.push('>');
    }

    fn write_preds(&self, r: &Resolver, d: DefId, out: &mut String) {
        let preds = &self.predicates[d.0 as usize];
        if preds.is_empty() {
            return;
        }
        out.push_str(" where ");
        for (i, p) in preds.iter().enumerate() {
            if i > 0 {
                out.push_str(", ");
            }
            match p {
                Pred::Trait(t, tr) => {
                    self.write_ty(r, *t, out);
                    out.push_str(": ");
                    self.write_trait_ref(r, tr, out);
                }
                Pred::Projection(_, a, l, ty) => {
                    let p = self.mk(TyKind::Projection(*a, *l));
                    self.write_ty(r, p, out);
                    out.push_str(" == ");
                    self.write_ty(r, *ty, out);
                }
            }
        }
    }

    /// One item's signature on one line, or `None` for items with none.
    pub fn signature_line(&self, r: &Resolver, d: DefId) -> Option<String> {
        let def = &r.defs[d.0 as usize];
        let mut s = String::new();
        let path = r.def_path(d);
        match def.kind {
            DefKind::Struct | DefKind::Union | DefKind::Enum => {
                let adt = self.adts.get(&d)?;
                let word = match adt.kind {
                    AdtKind::Struct => "struct",
                    AdtKind::Union => "union",
                    AdtKind::Enum => "enum",
                };
                let _ = write!(s, "{word} {path}");
                self.write_generics(d, &mut s);
                let fields = |s: &mut String, v: &super::VariantDef| match v.kind {
                    CtorKind::Unit => {}
                    CtorKind::Tuple => {
                        s.push('(');
                        let tys: Vec<Ty> = v.fields.iter().map(|f| f.ty).collect();
                        self.write_list(r, &tys, s);
                        s.push(')');
                    }
                    CtorKind::Named => {
                        s.push_str(" { ");
                        for (i, f) in v.fields.iter().enumerate() {
                            if i > 0 {
                                s.push_str(", ");
                            }
                            let _ = write!(s, "{}: ", f.name);
                            self.write_ty(r, f.ty, s);
                        }
                        s.push_str(" }");
                    }
                };
                if adt.kind == AdtKind::Enum {
                    s.push_str(" { ");
                    for (i, v) in adt.variants.iter().enumerate() {
                        if i > 0 {
                            s.push_str(", ");
                        }
                        s.push_str(v.name);
                        fields(&mut s, v);
                    }
                    s.push_str(" }");
                } else {
                    fields(&mut s, &adt.variants[0]);
                }
                self.write_preds(r, d, &mut s);
            }
            DefKind::Fn | DefKind::AssocFn | DefKind::ForeignFn => {
                let sig = self.sig_of(self.fn_sig[d.0 as usize]?);
                let _ = write!(s, "fn {path}");
                self.write_generics(d, &mut s);
                s.push('(');
                self.write_list(r, &self.tys(sig.inputs), &mut s);
                if sig.variadic {
                    s.push_str(", ...");
                }
                s.push_str(") -> ");
                self.write_ty(r, sig.output, &mut s);
                self.write_preds(r, d, &mut s);
            }
            DefKind::Impl => {
                let imp = &self.impls[*self.impl_index.get(&d)?];
                s.push_str("impl");
                self.write_generics(d, &mut s);
                s.push(' ');
                if let Some(t) = &imp.trait_ref {
                    if imp.negative {
                        s.push('!');
                    }
                    self.write_trait_ref(r, t, &mut s);
                    s.push_str(" for ");
                }
                self.write_ty(r, imp.self_ty, &mut s);
                self.write_preds(r, d, &mut s);
            }
            DefKind::Trait | DefKind::TraitAlias => {
                let _ = write!(s, "trait {path}");
                self.write_generics(d, &mut s);
                self.write_preds(r, d, &mut s);
            }
            DefKind::TypeAlias | DefKind::AssocType => {
                let _ = write!(s, "type {path}");
                self.write_generics(d, &mut s);
                s.push_str(" = ");
                self.write_ty(r, self.type_of(d)?, &mut s);
            }
            DefKind::Const | DefKind::AssocConst | DefKind::Static | DefKind::ForeignStatic => {
                let word =
                    if matches!(def.kind, DefKind::Static | DefKind::ForeignStatic) { "static" } else { "const" };
                let _ = write!(s, "{word} {path}: ");
                self.write_ty(r, self.type_of(d)?, &mut s);
            }
            _ => return None,
        }
        Some(s)
    }
}

/// Every item's signature, one per line, in definition order.
pub fn dump_signatures(tcx: &TyCtxt, r: &Resolver) -> String {
    let mut out = String::new();
    for d in 0..r.defs.len() as u32 {
        if let Some(line) = tcx.signature_line(r, DefId(d)) {
            out.push_str(&line);
            out.push('\n');
        }
    }
    out
}
