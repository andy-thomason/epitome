//! Compiler intrinsics (`#[rustc_intrinsic]` functions of `core`),
//! lowered to IR instructions at the call.

use super::Repr;
use super::body::{Fx, Operand, Place, Pv, R, Val};
use crate::resolve::DefId;
use crate::ty::{Ty, TyKind};
use epitome_ir::module::{BinOp, Ordering, Value};

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
            "unchecked_funnel_shl" | "unchecked_funnel_shr" => {
                let x = self.imm(args[0])?;
                let y = self.imm(args[1])?;
                let s = self.imm(args[2])?;
                let it = self.b.type_of(x);
                let s = self.int_to(s, it, false);
                let f = format!("llvm.{}.i{}", if name.ends_with("shl") { "fshl" } else { "fshr" }, self.bits(it));
                let g = self.b.module().declare(&f, it, &[it, it, it], false);
                let r = self.b.call(g, &[x, y, s]).unwrap();
                imm(r)
            }
            // No bit is set in both: an `or`.
            "disjoint_bitor" => {
                let x = self.imm(args[0])?;
                let y = self.imm(args[1])?;
                let r = self.b.or(x, y);
                imm(r)
            }
            "carrying_mul_add" => {
                // `a × b + c + d` at twice the width, which cannot
                // overflow: its low half (unsigned) and its high half.
                let t = t0.ok_or("no type argument")?;
                let signed = self.signed(t);
                let vals = args.iter().map(|&a| self.imm(a)).collect::<R<Vec<Value>>>()?;
                let it = self.b.type_of(vals[0]);
                let bits = self.bits(it);
                let (lo, hi) = if bits <= 64 {
                    let wt = self.b.module().types.int(bits * 2);
                    let w: Vec<Value> = vals.iter().map(|&v| self.int_to(v, wt, signed)).collect();
                    let p = self.b.mul(w[0], w[1]);
                    let p = self.b.add(p, w[2]);
                    let p = self.b.add(p, w[3]);
                    let lo = self.int_to(p, it, false);
                    let sh = self.b.const_int(wt, bits as i64);
                    let hi = if signed { self.b.ashr(p, sh) } else { self.b.lshr(p, sh) };
                    (lo, self.int_to(hi, it, false))
                } else {
                    self.mul_add_128(&vals, signed)
                };
                let pt = self.temp(output)?;
                for (i, v) in [lo, hi].into_iter().enumerate() {
                    let f = self.field(pt, 0, i)?;
                    let ft = self.field_ty(output, 0, i)?;
                    self.store(Operand { ty: ft, v: Val::Imm(v) }, f)?;
                }
                Ok(Pv::Place(pt))
            }
            "carryless_mul" => {
                // `a` shifted left by each bit set in `b`, all xored: a
                // loop over the bits.
                let x = self.imm(args[0])?;
                let y = self.imm(args[1])?;
                let it = self.b.type_of(x);
                let bits = self.bits(it);
                let from = self.b.current_block().ok_or("no block")?;
                let body = self.new_block("clmul");
                let exit = self.new_block("clmul.end");
                self.b.br(body);
                self.b.at(body);
                let zero = self.b.const_int(it, 0);
                let one = self.b.const_int(it, 1);
                let i = self.b.phi(it, &[(zero, from)]);
                let acc = self.b.phi(it, &[(zero, from)]);
                let bit = self.b.lshr(y, i);
                let bit = self.b.and(bit, one);
                let mask = self.b.sub(zero, bit);
                let part = self.b.shl(x, i);
                let part = self.b.and(part, mask);
                let next = self.b.xor(acc, part);
                let i1 = self.b.add(i, one);
                let n = self.b.const_int(it, bits as i64);
                let more = self.b.icmp(epitome_ir::module::ICmpPred::Ult, i1, n);
                self.b.add_incoming(i, i1, body);
                self.b.add_incoming(acc, next, body);
                self.b.cond_br(more, body, exit);
                self.b.at(exit);
                imm(next)
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
            "raw_eq" => {
                // The bytes of `*a` and `*b` alike.
                let t = t0.ok_or("no type argument")?;
                let size = self.layout(t)?.size;
                let r = if size == 0 {
                    let i1 = self.b.module().types.int(1);
                    self.b.const_int(i1, 1)
                } else {
                    let a = self.imm(args[0])?;
                    let b = self.imm(args[1])?;
                    let ptr = self.ptr_t();
                    let i32t = self.b.module().types.int(32);
                    let g = self.b.module().declare("memcmp", i32t, &[ptr, ptr, isz], false);
                    let n = self.b.const_int(isz, size as i64);
                    let c = self.b.call(g, &[a, b, n]).unwrap();
                    let zero = self.b.const_int(i32t, 0);
                    self.b.icmp(epitome_ir::module::ICmpPred::Eq, c, zero)
                };
                let r = self.bool_of(r);
                imm(r)
            }
            "typed_swap_nonoverlapping" => {
                let t = t0.ok_or("no type argument")?;
                let size = self.layout(t)?.size;
                let x = self.imm(args[0])?;
                let y = self.imm(args[1])?;
                let tmp = self.temp(t)?;
                self.memcpy(tmp.addr, x, size);
                self.memcpy(x, y, size);
                self.memcpy(y, tmp.addr, size);
                Ok(Pv::Val(self.unit()))
            }
            "breakpoint" => {
                // x86-64's `int3`, as inline assembly.
                use epitome_ir::module::{ConstKind, Constant, InlineAsm};
                let void = self.b.module().types.void();
                let fn_ty = self.b.module().types.function(void, &[], false);
                let a = InlineAsm {
                    fn_ty,
                    side_effect: true,
                    align_stack: false,
                    intel_dialect: false,
                    unwind: false,
                    asm: b"int3".to_vec(),
                    constraints: Vec::new(),
                };
                let ptr = self.ptr_t();
                let callee = Value::Const(self.b.module().consts.intern(Constant { ty: ptr, kind: ConstKind::InlineAsm(a) }));
                self.b.call_indirect(fn_ty, callee, &[]);
                Ok(Pv::Val(self.unit()))
            }
            "type_id" => {
                // A 128-bit hash of the type: of its key, which names
                // every item in it by path and place, so two types have
                // the same one only if they are the same.
                let t = t0.ok_or("no type argument")?;
                let mut key = String::new();
                self.cx.type_key(t, &mut key);
                let h = fnv128(key.as_bytes());
                let i128t = self.b.module().types.int(128);
                let v = self.int_const(i128t, h);
                let p = self.temp(output)?;
                self.store_imm(v, p.addr, 8);
                Ok(Pv::Place(p))
            }
            "type_id_eq" => {
                let i128t = self.b.module().types.int(128);
                let a = self.mem(args[0])?;
                let b = self.mem(args[1])?;
                let a = self.load_imm(i128t, a, 8);
                let b = self.load_imm(i128t, b, 8);
                let c = self.b.icmp(epitome_ir::module::ICmpPred::Eq, a, b);
                let r = self.bool_of(c);
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
            _ if name.starts_with("atomic_") => self.atomic(name, &targs, args, output),
            "float_to_int_unchecked" => {
                // Not saturating: the value is known to fit.
                let v = self.imm(args[0])?;
                let Repr::Imm(it) = self.repr(output)? else { return Err("float_to_int_unchecked to a type in memory".into()) };
                let op = if self.signed(output) { epitome_ir::module::CastOp::FPToSI } else { epitome_ir::module::CastOp::FPToUI };
                let r = self.b.cast(op, v, it);
                imm(r)
            }
            // Algebraic: free to reassociate, which the plain operation
            // does not need to.
            "fadd_algebraic" | "fsub_algebraic" | "fmul_algebraic" | "fdiv_algebraic" | "frem_algebraic" => {
                let x = self.imm(args[0])?;
                let y = self.imm(args[1])?;
                let op = match &name[..4] {
                    "fadd" => BinOp::FAdd,
                    "fsub" => BinOp::FSub,
                    "fmul" => BinOp::FMul,
                    "fdiv" => BinOp::FDiv,
                    _ => BinOp::FRem,
                };
                let r = self.b.binary(op, x, y);
                imm(r)
            }
            _ if float_math(name).is_some() => {
                let (op, _) = float_math(name).unwrap();
                let vals = args.iter().map(|&a| self.imm(a)).collect::<R<Vec<Value>>>()?;
                let ft = self.b.type_of(vals[0]);
                let fbits = match self.b.module().types.get(ft) {
                    epitome_ir::types::Type::Half => 16,
                    epitome_ir::types::Type::Float => 32,
                    epitome_ir::types::Type::Double => 64,
                    epitome_ir::types::Type::Fp128 => 128,
                    _ => return Err(format!("`{name}` of a type not supported")),
                };
                let tys: Vec<epitome_ir::types::TypeId> = vals.iter().map(|&v| self.b.type_of(v)).collect();
                let f = if op == "powi" { format!("llvm.powi.f{fbits}.i32") } else { format!("llvm.{op}.f{fbits}") };
                let g = self.b.module().declare(&f, ft, &tys, false);
                let r = self.b.call(g, &vals).unwrap();
                imm(r)
            }
            _ => Err(format!("intrinsic `{name}` is not supported yet")),
        }
    }

    /// The atomic intrinsics: loads, stores, read-modify-writes,
    /// compare-exchanges and fences, with the orderings their const
    /// arguments name. A pointer is worked on as the integer of its size.
    fn atomic(&mut self, name: &str, targs: &[Ty], args: Vec<Operand>, output: Ty) -> R<Pv> {
        use epitome_ir::module::{Atomic, Op, RmwOp};
        let tcx = self.cx.tcx;
        // The orderings are the last arguments.
        let orderings: Vec<Ordering> = targs.iter().filter_map(|&t| match tcx.kind(t) {
            TyKind::ConstArg(c) => Some(c),
            _ => None,
        }).map(|c| self.atomic_ordering(c)).collect::<R<_>>()?;
        let ord = |i: usize| orderings.get(i).copied().ok_or_else(|| format!("`{name}` without its ordering"));
        if name == "atomic_fence" || name == "atomic_singlethreadfence" {
            let scope = (name == "atomic_singlethreadfence").then(|| b"singlethread".to_vec().into_boxed_slice());
            self.b.push(Op::Fence { scope, ordering: ord(0)? }, None);
            return Ok(Pv::Val(self.unit()));
        }
        let t = *targs.first().ok_or("atomic without a type")?;
        let size = self.layout(t)?.size;
        let align = epitome_ir::module::Align::new(size.max(1));
        let it = self.b.module().types.int(size as u32 * 8);
        let is_ptr = matches!(tcx.kind(t), TyKind::Ptr(..) | TyKind::Ref(..) | TyKind::FnPtr(_));
        let dst = self.imm(args[0])?;
        // A value as the integer the operation works on, and back.
        let to_int = |fx: &mut Self, o: Operand| -> R<Value> {
            let v = fx.imm(o)?;
            let vt = fx.b.type_of(v);
            let ptr = fx.ptr_t();
            Ok(if vt == ptr { fx.b.cast(epitome_ir::module::CastOp::PtrToInt, v, it) } else { fx.int_to(v, it, false) })
        };
        let from_int = |fx: &mut Self, v: Value| -> Value {
            if is_ptr {
                let ptr = fx.ptr_t();
                fx.b.cast(epitome_ir::module::CastOp::IntToPtr, v, ptr)
            } else {
                v
            }
        };
        let atomic = |o: Ordering| Some(Atomic { elementwise: false, scope: None, ordering: o });
        match name {
            "atomic_load" => {
                let v = self.b.push(Op::Load { volatile: false, atomic: atomic(ord(0)?), ptr: dst, align }, Some(it)).unwrap();
                let v = from_int(self, v);
                Ok(Pv::Val(Operand { ty: output, v: Val::Imm(v) }))
            }
            "atomic_store" => {
                let v = to_int(self, args[1])?;
                self.b.push(Op::Store { volatile: false, atomic: atomic(ord(0)?), value: v, ptr: dst, align }, None);
                Ok(Pv::Val(self.unit()))
            }
            "atomic_cxchg" | "atomic_cxchgweak" => {
                let cmp = to_int(self, args[1])?;
                let new = to_int(self, args[2])?;
                let i1 = self.b.module().types.int(1);
                let pair = self.b.module().types.struct_of(&[it, i1]);
                let op = Op::CmpXchg {
                    weak: name == "atomic_cxchgweak",
                    volatile: false,
                    ptr: dst,
                    cmp,
                    new,
                    scope: None,
                    success: ord(0)?,
                    failure: ord(1)?,
                    align,
                };
                let r = self.b.push(op, Some(pair)).unwrap();
                let old = self.b.extract_value(r, &[0]);
                let old = from_int(self, old);
                let ok = self.b.extract_value(r, &[1]);
                let ok = self.bool_of(ok);
                // `(T, bool)`.
                let p = self.temp(output)?;
                let f0 = self.field(p, 0, 0)?;
                self.store(Operand { ty: t, v: Val::Imm(old) }, f0)?;
                let f1 = self.field(p, 0, 1)?;
                self.store_imm(ok, f1.addr, 1);
                Ok(Pv::Place(p))
            }
            _ => {
                let op = match name {
                    "atomic_xchg" => RmwOp::Xchg,
                    "atomic_xadd" => RmwOp::Add,
                    "atomic_xsub" => RmwOp::Sub,
                    "atomic_and" => RmwOp::And,
                    "atomic_nand" => RmwOp::Nand,
                    "atomic_or" => RmwOp::Or,
                    "atomic_xor" => RmwOp::Xor,
                    "atomic_max" => RmwOp::Max,
                    "atomic_min" => RmwOp::Min,
                    "atomic_umax" => RmwOp::UMax,
                    "atomic_umin" => RmwOp::UMin,
                    _ => return Err(format!("intrinsic `{name}` is not supported yet")),
                };
                let v = to_int(self, args[1])?;
                let op = Op::AtomicRmw { op, volatile: false, elementwise: false, ptr: dst, value: v, scope: None, ordering: ord(0)?, align };
                let r = self.b.push(op, Some(it)).unwrap();
                let r = from_int(self, r);
                Ok(Pv::Val(Operand { ty: output, v: Val::Imm(r) }))
            }
        }
    }

    /// The ordering an `AtomicOrdering` const argument names (`{
    /// AO::Relaxed }`, or `Relaxed` through a const parameter), as IR's:
    /// Rust's `Relaxed` is `monotonic`.
    fn atomic_ordering(&mut self, c: crate::ty::ConstId) -> R<Ordering> {
        let name = match self.cx.tcx.const_of(c) {
            crate::ty::Const::Expr(k, f, n) => {
                let ast = &self.cx.crates[k as usize].files[f as usize].ast;
                // The last name in it: `{ AO::Relaxed }` names `Relaxed`.
                let text = ast.text(n);
                text.rsplit(|c: char| !c.is_alphanumeric() && c != '_').find(|w| !w.is_empty()).unwrap_or("").to_string()
            }
            crate::ty::Const::Value(v) => ["Relaxed", "Release", "Acquire", "AcqRel", "SeqCst"].get(v as usize).copied().unwrap_or("").to_string(),
            c => return Err(format!("an atomic ordering not known: {c:?}")),
        };
        Ok(match name.as_str() {
            "Relaxed" => Ordering::Monotonic,
            "Release" => Ordering::Release,
            "Acquire" => Ordering::Acquire,
            "AcqRel" => Ordering::AcqRel,
            "SeqCst" => Ordering::SeqCst,
            _ => return Err(format!("an atomic ordering not known: `{name}`")),
        })
    }

    /// `a × b + c + d` of 128-bit integers, as its low and high halves,
    /// from products of 64-bit halves: the back end has no arithmetic on
    /// integers wider than 128 bits. Signed, the high half is corrected by
    /// what the signs of the four take away.
    fn mul_add_128(&mut self, v: &[Value], signed: bool) -> (Value, Value) {
        use epitome_ir::module::ICmpPred;
        let t = self.b.type_of(v[0]);
        let m = self.int_const(t, u64::MAX as u128);
        let s64 = self.b.const_int(t, 64);
        let halves = |fx: &mut Self, x: Value| (fx.b.and(x, m), fx.b.lshr(x, s64));
        let (a0, a1) = halves(self, v[0]);
        let (b0, b1) = halves(self, v[1]);
        let p00 = self.b.mul(a0, b0);
        let p01 = self.b.mul(a0, b1);
        let p10 = self.b.mul(a1, b0);
        let p11 = self.b.mul(a1, b1);
        let (p00l, p00h) = halves(self, p00);
        let (p01l, p01h) = halves(self, p01);
        let (p10l, p10h) = halves(self, p10);
        let mid = self.b.add(p00h, p01l);
        let mid = self.b.add(mid, p10l);
        let (midl, midh) = halves(self, mid);
        let lo = self.b.shl(midl, s64);
        let mut lo = self.b.or(lo, p00l);
        let hi = self.b.add(p11, p01h);
        let hi = self.b.add(hi, p10h);
        let mut hi = self.b.add(hi, midh);
        let s127 = self.b.const_int(t, 127);
        if signed {
            let sa = self.b.ashr(v[0], s127);
            let sb = self.b.ashr(v[1], s127);
            let x = self.b.mul(sa, v[1]);
            let y = self.b.mul(v[0], sb);
            hi = self.b.add(hi, x);
            hi = self.b.add(hi, y);
        }
        for &c in &v[2..4] {
            let sum = self.b.add(lo, c);
            let carry = self.b.icmp(ICmpPred::Ult, sum, lo);
            let carry = self.int_to(carry, t, false);
            hi = self.b.add(hi, carry);
            if signed {
                let sc = self.b.ashr(c, s127);
                hi = self.b.add(hi, sc);
            }
            lo = sum;
        }
        (lo, hi)
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

/// A float maths intrinsic of `core` (`sqrtf32`, `fabs`,
/// `round_ties_even_f64`, `fmaf16`): the LLVM intrinsic it is, and its
/// type's width if the name says it.
fn float_math(name: &str) -> Option<(&'static str, Option<u32>)> {
    let (base, bits) = ["f16", "f32", "f64", "f128"]
        .iter()
        .find_map(|w| name.strip_suffix(w).map(|b| (b.trim_end_matches('_'), w[1..].parse().ok())))
        .unwrap_or((name, None));
    let op = match base {
        "sqrt" => "sqrt",
        "floor" => "floor",
        "ceil" => "ceil",
        "trunc" => "trunc",
        "round" => "round",
        "round_ties_even" | "rint" | "nearbyint" => "roundeven",
        "fabs" => "fabs",
        "copysign" => "copysign",
        "fma" => "fma",
        "fmuladd" => "fmuladd",
        // `maximum_number_nsz`: NaN loses, and either zero may win.
        "minnum" | "minimum_number_nsz" => "minnum",
        "maxnum" | "maximum_number_nsz" => "maxnum",
        "minimum" => "minimum",
        "maximum" => "maximum",
        "powi" => "powi",
        // The C library's, through the back end.
        "sin" => "sin",
        "cos" => "cos",
        "tan" => "tan",
        "exp" => "exp",
        "exp2" => "exp2",
        "log" => "log",
        "log2" => "log2",
        "log10" => "log10",
        "pow" => "pow",
        _ => return None,
    };
    Some((op, bits))
}

/// The 128-bit FNV-1a hash of `bytes`.
fn fnv128(bytes: &[u8]) -> u128 {
    let mut h: u128 = 0x6c62_272e_07bb_0142_62b8_2175_6295_c58d;
    for &b in bytes {
        h ^= b as u128;
        h = h.wrapping_mul((1 << 88) + 0x13b);
    }
    h
}
