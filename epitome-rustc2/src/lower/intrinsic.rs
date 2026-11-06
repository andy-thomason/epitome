//! Compiler intrinsics (`#[rustc_intrinsic]` functions of `core`),
//! lowered to IR instructions at the call.

use super::Repr;
use super::body::{Fx, Operand, Place, Pv, R, Val};
use crate::resolve::DefId;
use crate::ty::{Ty, TyKind};
use epitome_ir::module::{BinOp, Value};

impl Fx<'_, '_, '_, '_> {
    /// A call of intrinsic `d` at `targs`.
    pub fn intrinsic(&mut self, d: DefId, targs: Vec<Ty>, args: Vec<Operand>) -> R<Pv> {
        let tcx = self.cx.tcx;
        let name = self.cx.r.defs[d.0 as usize].name;
        let (_, output, _, _) = self.cx.signature(&mut self.solver, d, &targs)?;
        let t0 = targs.first().copied();
        let isz = self.isize_t();
        let imm = |v: Value| Ok(Pv::Val(Operand { ty: output, v: Val::Imm(v) }));
        match name {
            "size_of" | "align_of" | "min_align_of" | "pref_align_of" => {
                let l = self.layout(t0.ok_or("no type argument")?)?;
                let v = if name == "size_of" { l.size } else { l.align };
                let c = self.b.const_int(isz, v as i64);
                imm(c)
            }
            "size_of_val" | "align_of_val" | "min_align_of_val" => {
                let t = t0.ok_or("no type argument")?;
                let l = self.layout(t)?;
                if l.is_sized() {
                    let v = if name == "size_of_val" { l.size } else { l.align };
                    let c = self.b.const_int(isz, v as i64);
                    return imm(c);
                }
                let p = self.deref(args[0])?;
                let meta = p.meta.ok_or("unsized value without metadata")?;
                let elem = match tcx.kind(t) {
                    TyKind::Slice(e) => self.layout(e)?,
                    TyKind::Str => self.layout(tcx.mk(TyKind::Uint(crate::ty::UintTy::U8)))?,
                    _ => return Err("size of an unsized type other than a slice".into()),
                };
                if name == "size_of_val" {
                    let s = self.b.const_int(isz, elem.size as i64);
                    let v = self.b.mul(meta, s);
                    imm(v)
                } else {
                    let c = self.b.const_int(isz, elem.align as i64);
                    imm(c)
                }
            }
            "transmute" | "transmute_unchecked" => {
                let src = args[0];
                let p = self.place_of(Pv::Val(src))?;
                let from = self.layout(src.ty)?.size;
                let to = self.layout(output)?.size;
                if from != to {
                    return Err("transmute between types of different sizes".into());
                }
                let o = self.load(Place { addr: p.addr, meta: None, ty: output })?;
                if let Val::Mem(_) = o.v {
                    // A copy, so the result does not share the argument's
                    // slot.
                    let t = self.temp(output)?;
                    self.store(o, t)?;
                    return Ok(Pv::Place(t));
                }
                Ok(Pv::Val(o))
            }
            "abort" => {
                self.trap();
                Ok(Pv::Val(Operand { ty: output, v: Val::Zst }))
            }
            "unreachable" => {
                self.b.unreachable();
                self.dead();
                Ok(Pv::Val(Operand { ty: output, v: Val::Zst }))
            }
            "assume" | "cold_path" | "forget" | "assert_inhabited" | "assert_zero_valid" | "assert_mem_uninitialized_valid" => {
                Ok(Pv::Val(self.unit()))
            }
            "likely" | "unlikely" | "black_box" => Ok(Pv::Val(args[0])),
            // The checks of a build without debug assertions.
            "ub_checks" | "contract_checks" | "overflow_checks" | "is_val_statically_known" | "needs_drop" => {
                let i8t = self.i8t();
                let z = self.b.const_int(i8t, 0);
                imm(z)
            }
            "caller_location" => {
                // Only panics use it, and they trap.
                let ptr = self.ptr_t();
                let z = self.b.module().const_zero(ptr);
                imm(z)
            }
            "add_with_overflow" | "sub_with_overflow" | "mul_with_overflow" => {
                let t = t0.ok_or("no type argument")?;
                let signed = self.signed(t);
                let op = &name[..3];
                let x = self.imm(args[0])?;
                let y = self.imm(args[1])?;
                let it = self.b.type_of(x);
                let bits = match self.b.module().types.get(it) {
                    epitome_ir::types::Type::Int(n) => *n,
                    _ => 64,
                };
                let i1 = self.b.module().types.int(1);
                let pair = self.b.module().types.struct_of(&[it, i1]);
                let f = format!("llvm.{}{op}.with.overflow.i{bits}", if signed { 's' } else { 'u' });
                let g = self.b.module().declare(&f, pair, &[it, it], false);
                let r = self.b.call(g, &[x, y]).unwrap();
                let v = self.b.extract_value(r, &[0]);
                let ov = self.b.extract_value(r, &[1]);
                let ov = self.bool_of(ov);
                let p = self.temp(output)?;
                let f0 = self.field(p, 0, 0)?;
                self.store(Operand { ty: t, v: Val::Imm(v) }, f0)?;
                let f1 = self.field(p, 0, 1)?;
                self.store_imm(ov, f1.addr, 1);
                Ok(Pv::Place(p))
            }
            "unchecked_add" | "wrapping_add" => self.arith2("+", args),
            "unchecked_sub" | "wrapping_sub" => self.arith2("-", args),
            "unchecked_mul" | "wrapping_mul" => self.arith2("*", args),
            "unchecked_div" | "exact_div" => self.arith2("/", args),
            "unchecked_rem" => self.arith2("%", args),
            "unchecked_shl" => self.arith2("<<", args),
            "unchecked_shr" => self.arith2(">>", args),
            "saturating_add" | "saturating_sub" => {
                let t = t0.ok_or("no type argument")?;
                let signed = self.signed(t);
                let x = self.imm(args[0])?;
                let y = self.imm(args[1])?;
                let it = self.b.type_of(x);
                let bits = match self.b.module().types.get(it) {
                    epitome_ir::types::Type::Int(n) => *n,
                    _ => 64,
                };
                let f = format!("llvm.{}{}.sat.i{bits}", if signed { 's' } else { 'u' }, if name.ends_with("add") { "add" } else { "sub" });
                let g = self.b.module().declare(&f, it, &[it, it], false);
                let r = self.b.call(g, &[x, y]).unwrap();
                imm(r)
            }
            "rotate_left" | "rotate_right" => {
                let x = self.imm(args[0])?;
                let s = self.imm(args[1])?;
                let it = self.b.type_of(x);
                let s = self.int_to(s, it, false);
                let f = format!("llvm.{}.i{}", if name == "rotate_left" { "fshl" } else { "fshr" }, self.bits(it));
                let g = self.b.module().declare(&f, it, &[it, it, it], false);
                let r = self.b.call(g, &[x, x, s]).unwrap();
                imm(r)
            }
            "ctpop" | "ctlz" | "cttz" | "ctlz_nonzero" | "cttz_nonzero" | "bswap" | "bitreverse" => {
                let x = self.imm(args[0])?;
                let it = self.b.type_of(x);
                let base = name.trim_end_matches("_nonzero");
                let f = format!("llvm.{base}.i{}", self.bits(it));
                let r = if matches!(base, "ctlz" | "cttz") {
                    let i1 = self.b.module().types.int(1);
                    let g = self.b.module().declare(&f, it, &[it, i1], false);
                    let poison = self.b.const_int(i1, name.ends_with("_nonzero") as i64);
                    self.b.call(g, &[x, poison]).unwrap()
                } else {
                    let g = self.b.module().declare(&f, it, &[it], false);
                    self.b.call(g, &[x]).unwrap()
                };
                // The count is a `u32`.
                let r = match self.repr(output)? {
                    Repr::Imm(ot) => self.int_to(r, ot, false),
                    _ => r,
                };
                imm(r)
            }
            "three_way_compare" => {
                let t = t0.ok_or("no type argument")?;
                let lt = self.compare("<", args[0], args[1])?;
                let gt = self.compare(">", args[0], args[1])?;
                let i8t = self.i8t();
                let m1 = self.b.const_int(i8t, -1);
                let one = self.b.const_int(i8t, 1);
                let zero = self.b.const_int(i8t, 0);
                let a = self.b.select(gt, one, zero);
                let r = self.b.select(lt, m1, a);
                let _ = t;
                imm(r)
            }
            "offset" | "arith_offset" => {
                let p = self.imm(args[0])?;
                let n = self.imm(args[1])?;
                let pointee = self.pointee(args[0].ty).ok_or("offset of something not a pointer")?;
                let size = self.layout(pointee)?.size;
                let s = self.b.const_int(isz, size as i64);
                let n = self.int_to(n, isz, true);
                let bytes = self.b.mul(n, s);
                let i8t = self.i8t();
                let r = self.b.gep(i8t, p, &[bytes]);
                imm(r)
            }
            "ptr_offset_from" | "ptr_offset_from_unsigned" => {
                let a = self.imm(args[0])?;
                let b = self.imm(args[1])?;
                let pointee = self.pointee(args[0].ty).ok_or("not a pointer")?;
                let size = self.layout(pointee)?.size.max(1);
                let ai = self.b.cast(epitome_ir::module::CastOp::PtrToInt, a, isz);
                let bi = self.b.cast(epitome_ir::module::CastOp::PtrToInt, b, isz);
                let d = self.b.sub(ai, bi);
                let s = self.b.const_int(isz, size as i64);
                let r = if name.ends_with("unsigned") { self.b.udiv(d, s) } else { self.b.sdiv(d, s) };
                imm(r)
            }
            "slice_get_unchecked" => {
                // Element `i` of the slice a fat pointer points at.
                let elem = targs.get(2).copied().ok_or("no element type")?;
                let p = self.deref(args[0])?;
                let i = self.imm(args[1])?;
                let size = self.layout(elem)?.size;
                let s = self.b.const_int(isz, size as i64);
                let off = self.b.mul(i, s);
                let i8t = self.i8t();
                let r = self.b.gep(i8t, p.addr, &[off]);
                imm(r)
            }
            "ptr_metadata" => {
                let pointee = self.pointee(args[0].ty).ok_or("metadata of something not a pointer")?;
                if !self.is_unsized(pointee)? {
                    return Ok(Pv::Val(Operand { ty: output, v: Val::Zst }));
                }
                let p = self.deref(args[0])?;
                imm(p.meta.ok_or("no metadata")?)
            }
            "aggregate_raw_ptr" => {
                let addr = self.imm(args[0])?;
                let pointee = self.pointee(output).ok_or("not a pointer")?;
                if !self.is_unsized(pointee)? {
                    return imm(addr);
                }
                let meta = self.imm(args[1])?;
                Ok(Pv::Val(self.fat(output, addr, meta)?))
            }
            "copy_nonoverlapping" | "copy" | "write_bytes" => {
                let t = t0.ok_or("no type argument")?;
                let size = self.layout(t)?.size;
                let (dst, src_or_val) = if name == "write_bytes" { (args[0], args[1]) } else { (args[1], args[0]) };
                let d = self.imm(dst)?;
                let s = self.imm(src_or_val)?;
                let n = self.imm(args[2])?;
                let k = self.b.const_int(isz, size as i64);
                let bytes = self.b.mul(n, k);
                let ptr = self.ptr_t();
                let i1 = self.b.module().types.int(1);
                let void = self.b.module().types.void();
                let no = self.b.const_int(i1, 0);
                let (f, second) = match name {
                    "copy_nonoverlapping" => ("llvm.memcpy.p0.p0.i64", ptr),
                    "copy" => ("llvm.memmove.p0.p0.i64", ptr),
                    _ => ("llvm.memset.p0.i64", self.i8t()),
                };
                let g = self.b.module().declare(f, void, &[ptr, second, isz, i1], false);
                self.b.call(g, &[d, s, bytes, no]);
                Ok(Pv::Val(self.unit()))
            }
            "read_via_copy" | "volatile_load" | "unaligned_volatile_load" => {
                let p = self.deref(args[0])?;
                let o = self.load(p)?;
                let t = self.temp(o.ty)?;
                self.store(o, t)?;
                Ok(Pv::Place(t))
            }
            "write_via_move" | "volatile_store" | "unaligned_volatile_store" => {
                let p = self.deref(args[0])?;
                self.store(Operand { ty: p.ty, v: args[1].v }, p)?;
                Ok(Pv::Val(self.unit()))
            }
            "discriminant_value" => {
                let p = self.deref(args[0])?;
                let d = self.discriminant(p)?;
                let r = match self.repr(output)? {
                    Repr::Imm(ot) => self.int_to(d, ot, true),
                    _ => return Err("discriminant of a type in memory".into()),
                };
                imm(r)
            }
            "const_eval_select" => {
                // The run-time function, with the tuple's fields.
                let g = targs.get(2).copied().ok_or("no run-time function")?;
                let TyKind::FnDef(f, l) = tcx.kind(g) else { return Err("run-time function not a function item".into()) };
                let tuple = self.place_of(Pv::Val(args[0]))?;
                let n = match tcx.kind(tuple.ty) {
                    TyKind::Tuple(l) => tcx.tys(l).len(),
                    _ => return Err("arguments not a tuple".into()),
                };
                let mut inner = Vec::new();
                for i in 0..n {
                    let fp = self.field(tuple, 0, i)?;
                    inner.push(self.load(fp)?);
                }
                self.call_def(f, tcx.tys(l), inner, None)
            }
            "compare_bytes" => {
                // libc's `memcmp`.
                let a = self.imm(args[0])?;
                let b = self.imm(args[1])?;
                let n = self.imm(args[2])?;
                let ptr = self.ptr_t();
                let i32t = self.b.module().types.int(32);
                let g = self.b.module().declare("memcmp", i32t, &[ptr, ptr, isz], false);
                let r = self.b.call(g, &[a, b, n]).unwrap();
                imm(r)
            }
            "ptr_guaranteed_cmp" => {
                let c = self.compare("==", args[0], args[1])?;
                let r = self.bool_of(c);
                imm(r)
            }
            "select_unpredictable" => {
                let c = self.imm(args[0])?;
                let c = self.truthy(c);
                match self.repr(output)? {
                    Repr::Imm(_) => {
                        let a = self.imm(args[1])?;
                        let b = self.imm(args[2])?;
                        let r = self.b.select(c, a, b);
                        imm(r)
                    }
                    Repr::Zst => Ok(Pv::Val(Operand { ty: output, v: Val::Zst })),
                    Repr::Mem => {
                        let a = self.mem(args[1])?;
                        let b = self.mem(args[2])?;
                        let r = self.b.select(c, a, b);
                        let t = self.temp(output)?;
                        self.store(Operand { ty: output, v: Val::Mem(r) }, t)?;
                        Ok(Pv::Place(t))
                    }
                }
            }
            _ => Err(format!("intrinsic `{name}` is not supported yet")),
        }
    }

    fn bits(&mut self, t: epitome_ir::types::TypeId) -> u32 {
        match self.b.module().types.get(t) {
            epitome_ir::types::Type::Int(n) => *n,
            _ => 64,
        }
    }

    fn arith2(&mut self, op: &str, args: Vec<Operand>) -> R<Pv> {
        if op == "/" || op == "%" {
            // Unchecked: no test for zero.
            let x = self.imm(args[0])?;
            let y = self.imm(args[1])?;
            let signed = self.signed(args[0].ty);
            let bop = match (op, signed) {
                ("/", true) => BinOp::SDiv,
                ("/", false) => BinOp::UDiv,
                (_, true) => BinOp::SRem,
                _ => BinOp::URem,
            };
            let v = self.b.binary(bop, x, y);
            return Ok(Pv::Val(Operand { ty: args[0].ty, v: Val::Imm(v) }));
        }
        Ok(Pv::Val(self.arith(op, args[0], args[1])?))
    }
}
