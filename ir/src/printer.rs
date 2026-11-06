//! Writes a module as IR text, laid out the way LLVM's tools lay it out.

use std::collections::HashMap;
use std::fmt::{self, Write};

use crate::float::{self, FloatKind};
use crate::metadata;
use crate::module::*;
use crate::types::{Type, TypeId};

impl fmt::Display for Module {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&print_module(self))
    }
}

/// Prints a whole module.
pub fn print_module(m: &Module) -> String {
    let mut p = Printer::new(m);
    p.module();
    p.out
}

/// A value's name as the printer writes it: `%name`, or `%N` for an
/// unnamed one, numbered as the printer numbers them. It numbers the whole
/// function, so it is for messages, not for loops.
pub fn local_name(f: &Function, l: LocalId) -> String {
    match &f.locals[l.index()].name {
        Some(n) => format!("%{}", ident(n)),
        None => match Slots::new(f).locals[l.index()] {
            Some(n) => format!("%{n}"),
            None => format!("%<value {}>", l.0),
        },
    }
}

/// An instruction for messages: its result's name, or the instruction's
/// text for one without a result (a store, a branch).
pub fn inst_name(f: &Function, i: InstId) -> String {
    match f.insts[i.index()].result {
        Some(r) => local_name(f, r),
        None => format!(
            "the {} in %{}",
            match &f.insts[i.index()].op {
                Op::Store { .. } => "store",
                Op::Ret(_) => "ret",
                Op::Br(_) | Op::CondBr { .. } => "br",
                Op::Switch { .. } => "switch",
                Op::Call(_) => "call",
                Op::Invoke { .. } => "invoke",
                _ => "instruction",
            },
            {
                let b = f.layout.iter().find(|b| f.blocks[b.index()].insts.contains(&i));
                b.map_or("?".to_string(), |b| {
                    let l = f.blocks[b.index()].local;
                    local_name(f, l)[1..].to_string()
                })
            }
        ),
    }
}

/// A place in a function that an analysis can comment on.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Place {
    /// The start of a block, after its label.
    Block(BlockId),
    /// Just before an instruction.
    Inst(InstId),
}

/// Comments for places in functions, as an analysis's printer gives them:
/// each line becomes a `; ` comment.
pub type Notes<'a> = &'a dyn Fn(GlobalId, Place) -> Option<String>;

/// The module, with `notes` as comments where they say (for printing an
/// analysis's results next to the IR they are about).
pub fn print_module_with(m: &Module, notes: Notes) -> String {
    let mut p = Printer::new(m);
    p.notes = Some(notes);
    p.module();
    p.out
}

/// Local value numbers for one function: `None` for named values.
struct Slots {
    locals: Vec<Option<u32>>,
}

impl Slots {
    fn new(f: &Function) -> Self {
        let mut locals = vec![None; f.locals.len()];
        let mut next = 0;
        let mut number = |l: LocalId| {
            if f.locals[l.index()].name.is_none() {
                locals[l.index()] = Some(next);
                next += 1;
            }
        };
        for p in &f.params {
            number(p.local);
        }
        for &b in &f.layout {
            number(f.blocks[b.index()].local);
            for &i in &f.blocks[b.index()].insts {
                if let Some(r) = f.insts[i.index()].result {
                    number(r);
                }
            }
        }
        Slots { locals }
    }
}

struct Printer<'m> {
    m: &'m Module,
    out: String,
    global_slots: HashMap<GlobalId, u32>,
    struct_slots: HashMap<u32, u32>,
    /// Slots of functions other than the current one, for `blockaddress`.
    fn_slots: HashMap<GlobalId, Slots>,
    /// The function being printed.
    cur: Option<(&'m Function, Slots)>,
    /// Metadata numbers, indexed by `MdId`; `None` for inline metadata.
    md_slots: Vec<Option<u32>>,
    /// Numbered metadata in number order.
    md_order: Vec<MdId>,
    /// Comments to add, and the function they are for.
    notes: Option<Notes<'m>>,
    cur_global: Option<GlobalId>,
}

impl<'m> Printer<'m> {
    fn new(m: &'m Module) -> Self {
        let mut global_slots = HashMap::new();
        let mut next = 0;
        for &g in &print_order(m) {
            if m.global(g).name.is_none() {
                global_slots.insert(g, next);
                next += 1;
            }
        }
        let mut struct_slots = HashMap::new();
        let mut next = 0;
        for (i, s) in m.types.structs.iter().enumerate() {
            if s.name.is_none() {
                struct_slots.insert(i as u32, next);
                next += 1;
            }
        }
        let md_order = metadata::numbering(m, &print_order(m));
        let mut md_slots = vec![None; m.metadata.len()];
        for (slot, id) in md_order.iter().enumerate() {
            md_slots[id.index()] = Some(slot as u32);
        }
        Printer {
            m,
            out: String::new(),
            global_slots,
            struct_slots,
            fn_slots: HashMap::new(),
            cur: None,
            md_slots,
            md_order,
            notes: None,
            cur_global: None,
        }
    }

    fn module(&mut self) {
        let m = self.m;
        if let Some(s) = &m.source_filename {
            writeln!(self.out, "source_filename = {}", quote(s)).unwrap();
        }
        if let Some(s) = &m.data_layout {
            writeln!(self.out, "target datalayout = {}", quote(s)).unwrap();
        }
        if let Some(s) = &m.triple {
            writeln!(self.out, "target triple = {}", quote(s)).unwrap();
        }
        if !m.module_asm.is_empty() {
            self.out.push('\n');
            if m.module_asm_options.is_empty() {
                for s in &m.module_asm {
                    writeln!(self.out, "module asm {}", quote(s)).unwrap();
                }
            } else {
                let opts: Vec<String> =
                    m.module_asm_options.iter().map(|(k, v)| format!("{k}: {}", quote(v))).collect();
                writeln!(self.out, "module asm({})", opts.join(", ")).unwrap();
                for s in &m.module_asm {
                    writeln!(self.out, "    {}", quote(s)).unwrap();
                }
            }
        }

        let structs: Vec<u32> = (0..m.types.structs.len() as u32).collect();
        if !structs.is_empty() {
            self.out.push('\n');
            for i in structs {
                let name = self.struct_name(i);
                match &m.types.structs[i as usize].body {
                    Some(body) => {
                        let fields: Vec<String> = body.fields.iter().map(|&t| self.ty(t)).collect();
                        let text = struct_text(body.packed, &fields);
                        writeln!(self.out, "{name} = type {text}").unwrap();
                    }
                    None => writeln!(self.out, "{name} = type opaque").unwrap(),
                }
            }
        }

        if !m.comdats.is_empty() {
            self.out.push('\n');
            for c in &m.comdats {
                let kind = c.kind.unwrap_or(ComdatKind::Any);
                let kw = ComdatKind::ALL.iter().find(|(_, k)| *k == kind).unwrap().0;
                writeln!(self.out, "${} = comdat {kw}", ident(&c.name)).unwrap();
            }
        }

        // Variables, aliases and ifuncs, each group after a blank line.
        let order = print_order(m);
        for group in 0..3 {
            let members: Vec<GlobalId> = order
                .iter()
                .copied()
                .filter(|&g| match m.global(g).kind {
                    GlobalKind::Variable(_) => group == 0,
                    GlobalKind::Alias { .. } => group == 1,
                    GlobalKind::IFunc { .. } => group == 2,
                    _ => false,
                })
                .collect();
            if !members.is_empty() {
                self.out.push('\n');
                for g in members {
                    self.global(g);
                }
            }
        }
        for &g in &order {
            if let GlobalKind::Function(f) = &m.global(g).kind {
                self.out.push('\n');
                self.function(g, f);
            }
        }

        if !m.attr_groups.is_empty() {
            self.out.push('\n');
            for g in &m.attr_groups {
                let attrs = self.attrs(&g.attrs);
                writeln!(self.out, "attributes #{} = {{ {attrs} }}", g.id).unwrap();
            }
        }

        if !m.named_metadata.is_empty() {
            self.out.push('\n');
            for n in &m.named_metadata {
                let ops: Vec<String> = n.ops.iter().map(|&o| self.md(o)).collect();
                writeln!(self.out, "!{} = !{{{}}}", md_name(&n.name), ops.join(", ")).unwrap();
            }
        }

        if !self.md_order.is_empty() {
            self.out.push('\n');
            for (slot, &id) in self.md_order.iter().enumerate() {
                let body = self.md_body(id);
                writeln!(self.out, "!{slot} = {body}").unwrap();
            }
        }
    }

    // -----------------------------------------------------------------------
    // Names

    fn global_name(&self, g: GlobalId) -> String {
        match &self.m.global(g).name {
            Some(n) => format!("@{}", ident(n)),
            None => format!("@{}", self.global_slots.get(&g).copied().unwrap_or(0)),
        }
    }

    fn struct_name(&self, i: u32) -> String {
        match &self.m.types.structs[i as usize].name {
            Some(n) => format!("%{}", ident(n)),
            None => format!("%{}", self.struct_slots[&i]),
        }
    }

    fn local_name(&self, f: &Function, slots: &Slots, l: LocalId) -> String {
        match &f.locals[l.index()].name {
            Some(n) => format!("%{}", ident(n)),
            None => format!("%{}", slots.locals[l.index()].unwrap_or(u32::MAX)),
        }
    }

    fn cur_local(&self, l: LocalId) -> String {
        let (f, slots) = self.cur.as_ref().expect("local value outside a function");
        self.local_name(f, slots, l)
    }

    fn block_name(&self, b: BlockId) -> String {
        let (f, _) = self.cur.as_ref().unwrap();
        self.cur_local(f.blocks[b.index()].local)
    }

    // -----------------------------------------------------------------------
    // Types

    fn ty(&self, t: TypeId) -> String {
        match self.m.types.get(t) {
            Type::Void => "void".into(),
            Type::Label => "label".into(),
            Type::Token => "token".into(),
            Type::Metadata => "metadata".into(),
            Type::X86Amx => "x86_amx".into(),
            Type::Int(n) => format!("i{n}"),
            Type::Byte(n) => format!("b{n}"),
            Type::Half => "half".into(),
            Type::BFloat => "bfloat".into(),
            Type::Float => "float".into(),
            Type::Double => "double".into(),
            Type::X86Fp80 => "x86_fp80".into(),
            Type::Fp128 => "fp128".into(),
            Type::PpcFp128 => "ppc_fp128".into(),
            Type::Ptr(0) => "ptr".into(),
            Type::Ptr(n) => format!("ptr addrspace({n})"),
            Type::Vector { scalable, len, elem } => {
                let v = if *scalable { "vscale x " } else { "" };
                format!("<{v}{len} x {}>", self.ty(*elem))
            }
            Type::Array { len, elem } => format!("[{len} x {}]", self.ty(*elem)),
            Type::Struct { packed, fields } => {
                let fields: Vec<String> = fields.iter().map(|&t| self.ty(t)).collect();
                struct_text(*packed, &fields)
            }
            Type::Named(i) => self.struct_name(*i),
            Type::Function { ret, params, varargs } => {
                let mut ps: Vec<String> = params.iter().map(|&t| self.ty(t)).collect();
                if *varargs {
                    ps.push("...".into());
                }
                format!("{} ({})", self.ty(*ret), ps.join(", "))
            }
            Type::TargetExt { name, types, ints } => {
                let mut parts = vec![quote(name)];
                parts.extend(types.iter().map(|&t| self.ty(t)));
                parts.extend(ints.iter().map(|i| i.to_string()));
                format!("target({})", parts.join(", "))
            }
        }
    }

    // -----------------------------------------------------------------------
    // Values and constants

    fn value_ty(&self, v: Value) -> TypeId {
        match v {
            Value::Local(l) => self.cur.as_ref().unwrap().0.locals[l.index()].ty,
            Value::Const(c) => self.m.constant(c).ty,
            Value::Metadata(_) => self.metadata_ty(),
        }
    }

    fn metadata_ty(&self) -> TypeId {
        // The type table always has `metadata` once metadata is used as a value.
        (0..self.m.types.len() as u32)
            .map(TypeId)
            .find(|&t| matches!(self.m.types.get(t), Type::Metadata))
            .expect("metadata type")
    }

    /// A value without its type.
    fn value(&self, v: Value) -> String {
        match v {
            Value::Local(l) => self.cur_local(l),
            Value::Const(c) => self.constant(c),
            Value::Metadata(md) => self.md(md),
        }
    }

    /// A value with its type, as in `i32 %x`.
    fn typed(&self, v: Value) -> String {
        format!("{} {}", self.ty(self.value_ty(v)), self.value(v))
    }

    /// Data in memory order as a constant of type `ty`: a string for an
    /// array of `i8`, otherwise each element, as LLVM writes an array of
    /// numbers.
    fn bytes(&self, ty: TypeId, b: &[u8]) -> String {
        match self.m.types.get(ty) {
            Type::Array { elem, .. } if matches!(self.m.types.get(*elem), Type::Int(8)) => format!("c{}", quote(b)),
            Type::Array { len, elem } => {
                let size = if *len == 0 { 0 } else { b.len() / *len as usize };
                let es: Vec<String> = b
                    .chunks(size.max(1))
                    .take(*len as usize)
                    .map(|c| format!("{} {}", self.ty(*elem), self.bytes(*elem, c)))
                    .collect();
                format!("[{}]", es.join(", "))
            }
            Type::Int(_) => {
                let mut v = 0u64;
                for (k, &x) in b.iter().take(8).enumerate() {
                    v |= (x as u64) << (8 * k);
                }
                let bits = b.len() as u32 * 8;
                crate::apint::ApInt::from_u64(bits, v).to_string()
            }
            t => match FloatKind::of(t) {
                Some(kind) => {
                    let mut v = 0u128;
                    for (k, &x) in b.iter().enumerate() {
                        v |= (x as u128) << (8 * k);
                    }
                    float::print(kind, v)
                }
                None => format!("c{}", quote(b)),
            },
        }
    }

    fn typed_const(&self, c: ConstId) -> String {
        format!("{} {}", self.ty(self.m.constant(c).ty), self.constant(c))
    }

    fn constant(&self, c: ConstId) -> String {
        let k = self.m.constant(c);
        match &k.kind {
            ConstKind::Int(v) => {
                if matches!(self.m.types.get(k.ty), Type::Int(1)) {
                    if v.is_zero() { "false".into() } else { "true".into() }
                } else {
                    v.to_string()
                }
            }
            ConstKind::Float(bits) => {
                let kind = FloatKind::of(self.m.types.get(k.ty)).expect("float constant of non-float type");
                float::print(kind, *bits)
            }
            ConstKind::Null => "null".into(),
            ConstKind::NoneToken => "none".into(),
            ConstKind::Undef => "undef".into(),
            ConstKind::Poison => "poison".into(),
            ConstKind::Zero => "zeroinitializer".into(),
            ConstKind::Array(es) => {
                let es: Vec<String> = es.iter().map(|&e| self.typed_const(e)).collect();
                format!("[{}]", es.join(", "))
            }
            ConstKind::Struct(es) => {
                let es: Vec<String> = es.iter().map(|&e| self.typed_const(e)).collect();
                let packed = match self.m.types.get(k.ty) {
                    Type::Struct { packed, .. } => *packed,
                    Type::Named(i) => self.m.types.structs[*i as usize].body.as_ref().is_some_and(|b| b.packed),
                    _ => false,
                };
                struct_text(packed, &es)
            }
            ConstKind::Vector(es) => {
                let es: Vec<String> = es.iter().map(|&e| self.typed_const(e)).collect();
                format!("<{}>", es.join(", "))
            }
            ConstKind::Bytes(b) => self.bytes(k.ty, b),
            ConstKind::Splat(e) => format!("splat ({})", self.typed_const(*e)),
            ConstKind::Global(g) => self.global_name(*g),
            ConstKind::BlockAddress { func, block } => {
                let block = self.blockaddress_block(*func, *block);
                format!("blockaddress({}, {block})", self.global_name(*func))
            }
            ConstKind::DsoLocalEquivalent(g) => format!("dso_local_equivalent {}", self.global_name(*g)),
            ConstKind::NoCfi(g) => format!("no_cfi {}", self.global_name(*g)),
            ConstKind::PtrAuth(ops) => {
                let ops: Vec<String> = ops.iter().map(|&o| self.typed_const(o)).collect();
                format!("ptrauth ({})", ops.join(", "))
            }
            ConstKind::Expr(e) => self.const_expr(k.ty, e),
            ConstKind::InlineAsm(a) => inline_asm(a),
        }
    }

    fn blockaddress_block(&self, func: GlobalId, block: BlockId) -> String {
        let GlobalKind::Function(f) = &self.m.global(func).kind else { return "%?".into() };
        let local = f.blocks[block.index()].local;
        if let Some(slots) = self.fn_slots.get(&func) {
            return self.local_name(f, slots, local);
        }
        let slots = Slots::new(f);
        self.local_name(f, &slots, local)
    }

    fn const_expr(&self, ty: TypeId, e: &ConstExpr) -> String {
        match e {
            ConstExpr::Cast { op, value } => {
                format!("{} ({} to {})", op.keyword(), self.typed_const(*value), self.ty(ty))
            }
            ConstExpr::Gep { flags, inrange, source_ty, ptr, indices } => {
                let mut s = String::from("getelementptr");
                s.push_str(&gep_flags(*flags, *inrange));
                write!(s, " ({}, {}", self.ty(*source_ty), self.typed_const(*ptr)).unwrap();
                for &i in indices {
                    write!(s, ", {}", self.typed_const(i)).unwrap();
                }
                s.push(')');
                s
            }
            ConstExpr::Binary { op, flags, lhs, rhs } => {
                format!(
                    "{}{} ({}, {})",
                    op.keyword(),
                    int_flags(*flags),
                    self.typed_const(*lhs),
                    self.typed_const(*rhs)
                )
            }
            ConstExpr::ExtractElement { vector, index } => {
                format!("extractelement ({}, {})", self.typed_const(*vector), self.typed_const(*index))
            }
            ConstExpr::InsertElement { vector, element, index } => format!(
                "insertelement ({}, {}, {})",
                self.typed_const(*vector),
                self.typed_const(*element),
                self.typed_const(*index)
            ),
            ConstExpr::ShuffleVector { a, b, mask } => {
                format!(
                    "shufflevector ({}, {}, {})",
                    self.typed_const(*a),
                    self.typed_const(*b),
                    self.typed_const(*mask)
                )
            }
        }
    }

    // -----------------------------------------------------------------------
    // Metadata

    /// A metadata operand: `!N` for numbered nodes, otherwise inline.
    fn md(&self, id: MdId) -> String {
        match self.md_slots[id.index()] {
            Some(slot) => format!("!{slot}"),
            None => self.md_body(id),
        }
    }

    fn md_body(&self, id: MdId) -> String {
        match &self.m.metadata[id.index()].data {
            Metadata::Tuple { distinct, ops } => {
                let ops: Vec<String> = ops.iter().map(|o| o.map_or_else(|| "null".into(), |o| self.md(o))).collect();
                format!("{}!{{{}}}", if *distinct { "distinct " } else { "" }, ops.join(", "))
            }
            Metadata::Node { distinct, kind, fields } => {
                let fields: Vec<String> = fields
                    .iter()
                    .map(|f| {
                        let v = match &f.value {
                            MdFieldValue::Null => "null".into(),
                            MdFieldValue::Md(m) => self.md(*m),
                            MdFieldValue::Str(s) => quote(s),
                            MdFieldValue::Word(w) => w.clone(),
                            MdFieldValue::Value(v) => self.typed(*v),
                        };
                        match &f.name {
                            Some(n) => format!("{n}: {v}"),
                            None => v,
                        }
                    })
                    .collect();
                format!("{}!{kind}({})", if *distinct { "distinct " } else { "" }, fields.join(", "))
            }
            Metadata::String(s) => format!("!{}", quote(s)),
            Metadata::Value(v) => self.typed(*v),
            Metadata::Forward => "!<undefined>".into(),
        }
    }

    fn attachments(&self, list: &[Attachment], comma: bool) -> String {
        let mut s = String::new();
        for a in list {
            s.push_str(if comma { ", " } else { " " });
            write!(s, "!{} {}", md_name(&a.kind), self.md(a.node)).unwrap();
        }
        s
    }

    // -----------------------------------------------------------------------
    // Attributes

    fn attr(&self, a: &Attr) -> String {
        match a {
            Attr::Word(w) => w.clone(),
            Attr::Int { name, value, style: IntStyle::Space } => format!("{name} {value}"),
            Attr::Int { name, value, style: IntStyle::Paren } => format!("{name}({value})"),
            Attr::Int { name, value, style: IntStyle::Eq } => format!("{name}={value}"),
            Attr::Type { name, ty } => format!("{name}({})", self.ty(*ty)),
            Attr::Parens { name, text } => format!("{name}({text})"),
            Attr::Str { key, value: None } => quote(key),
            Attr::Str { key, value: Some(v) } => format!("{}={}", quote(key), quote(v)),
            Attr::Group(n) => format!("#{n}"),
        }
    }

    fn attrs(&self, list: &[Attr]) -> String {
        list.iter().map(|a| self.attr(a)).collect::<Vec<_>>().join(" ")
    }

    /// The attributes followed by a space, or nothing.
    fn attrs_sp(&self, list: &[Attr]) -> String {
        if list.is_empty() { String::new() } else { format!("{} ", self.attrs(list)) }
    }

    // -----------------------------------------------------------------------
    // Globals

    fn props(&self, p: &GlobalProps, is_declaration: bool, is_function: bool) -> String {
        let mut s = String::new();
        if p.linkage != Linkage::External || is_declaration {
            write!(s, "{} ", p.linkage.keyword()).unwrap();
        }
        if p.dso_local {
            s.push_str("dso_local ");
        }
        match p.visibility {
            Visibility::Default => {}
            Visibility::Hidden => s.push_str("hidden "),
            Visibility::Protected => s.push_str("protected "),
        }
        match p.dll_storage {
            DllStorage::None => {}
            DllStorage::Import => s.push_str("dllimport "),
            DllStorage::Export => s.push_str("dllexport "),
        }
        match p.thread_local {
            None => {}
            Some(ThreadLocal::GeneralDynamic) => s.push_str("thread_local "),
            Some(ThreadLocal::LocalDynamic) => s.push_str("thread_local(localdynamic) "),
            Some(ThreadLocal::InitialExec) => s.push_str("thread_local(initialexec) "),
            Some(ThreadLocal::LocalExec) => s.push_str("thread_local(localexec) "),
        }
        if !is_function {
            s.push_str(unnamed_addr(p.unnamed_addr));
        }
        s
    }

    fn comdat(&self, c: ComdatRef) -> String {
        match c {
            ComdatRef::Own => "comdat".into(),
            ComdatRef::Named(id) => format!("comdat(${})", ident(&self.m.comdats[id.index()].name)),
        }
    }

    fn global(&mut self, g: GlobalId) {
        let gl = self.m.global(g);
        let mut s = format!("{} = ", self.global_name(g));
        match &gl.kind {
            GlobalKind::Variable(v) => {
                s.push_str(&self.props(&gl.props, v.init.is_none() && gl.props.linkage == Linkage::External, false));
                if let Some(a) = gl.props.addrspace.filter(|&a| a != 0) {
                    write!(s, "addrspace({a}) ").unwrap();
                }
                if v.externally_initialized {
                    s.push_str("externally_initialized ");
                }
                s.push_str(if v.constant { "constant " } else { "global " });
                s.push_str(&self.ty(v.ty));
                if let Some(init) = v.init {
                    write!(s, " {}", self.constant(init)).unwrap();
                }
                if let Some(sec) = &v.section {
                    write!(s, ", section {}", quote(sec)).unwrap();
                }
                if let Some(p) = &gl.props.partition {
                    write!(s, ", partition {}", quote(p)).unwrap();
                }
                if let Some(c) = v.comdat {
                    write!(s, ", {}", self.comdat(c)).unwrap();
                }
                if let Some(a) = v.align {
                    write!(s, ", align {a}").unwrap();
                }
                if let Some(cm) = &v.code_model {
                    write!(s, ", code_model {}", quote(cm)).unwrap();
                }
                for san in &v.sanitizer {
                    write!(s, ", {san}").unwrap();
                }
                s.push_str(&self.attachments(&gl.attachments, true));
                if !v.attrs.is_empty() {
                    write!(s, " {}", self.attrs(&v.attrs)).unwrap();
                }
            }
            GlobalKind::Alias { value_ty, aliasee: target } | GlobalKind::IFunc { value_ty, resolver: target } => {
                s.push_str(&self.props(&gl.props, false, false));
                let kw = if matches!(gl.kind, GlobalKind::Alias { .. }) { "alias" } else { "ifunc" };
                write!(s, "{kw} {}, {}", self.ty(*value_ty), self.typed_const(*target)).unwrap();
                if let Some(p) = &gl.props.partition {
                    write!(s, ", partition {}", quote(p)).unwrap();
                }
                s.push_str(&self.attachments(&gl.attachments, true));
            }
            GlobalKind::Function(_) | GlobalKind::Forward => unreachable!(),
        }
        self.out.push_str(&s);
        self.out.push('\n');
    }

    // -----------------------------------------------------------------------
    // Functions

    fn function(&mut self, g: GlobalId, f: &'m Function) {
        let gl = self.m.global(g);
        let Type::Function { ret, varargs, .. } = self.m.types.get(f.ty).clone() else { unreachable!() };

        // A comment listing the attributes from the groups the function uses.
        let mut listed = Vec::new();
        for a in &f.fn_attrs {
            if let Attr::Group(n) = a
                && let Some(group) = self.m.attr_groups.iter().find(|grp| grp.id == *n)
            {
                listed.extend(group.attrs.iter().filter(|a| !matches!(a, Attr::Str { .. })).map(|a| self.attr(a)));
            }
        }
        if !listed.is_empty() {
            writeln!(self.out, "; Function Attrs: {}", listed.join(" ")).unwrap();
        }

        let slots = Slots::new(f);
        let mut s = String::from(if f.has_body { "define " } else { "declare " });
        if !f.has_body && !gl.attachments.is_empty() {
            s.push_str(self.attachments(&gl.attachments, false).trim_start());
            s.push(' ');
        }
        s.push_str(&self.props(&gl.props, false, true));
        s.push_str(&cconv(&f.cconv));
        s.push_str(&self.attrs_sp(&f.ret_attrs));
        write!(s, "{} {}(", self.ty(ret), self.global_name(g)).unwrap();
        let mut params: Vec<String> = f
            .params
            .iter()
            .map(|p| {
                let mut t = self.ty(f.locals[p.local.index()].ty);
                if !p.attrs.is_empty() {
                    write!(t, " {}", self.attrs(&p.attrs)).unwrap();
                }
                if f.has_body {
                    write!(t, " {}", self.local_name(f, &slots, p.local)).unwrap();
                }
                t
            })
            .collect();
        if varargs {
            params.push("...".into());
        }
        s.push_str(&params.join(", "));
        s.push(')');
        let ua = unnamed_addr(gl.props.unnamed_addr);
        if !ua.is_empty() {
            write!(s, " {}", ua.trim_end()).unwrap();
        }
        if let Some(a) = gl.props.addrspace.filter(|&a| a != 0) {
            write!(s, " addrspace({a})").unwrap();
        }
        if !f.fn_attrs.is_empty() {
            write!(s, " {}", self.attrs(&f.fn_attrs)).unwrap();
        }
        if let Some(sec) = &f.section {
            write!(s, " section {}", quote(sec)).unwrap();
        }
        if let Some(p) = &gl.props.partition {
            write!(s, " partition {}", quote(p)).unwrap();
        }
        if let Some(c) = f.comdat {
            write!(s, " {}", self.comdat(c)).unwrap();
        }
        if let Some(a) = f.align {
            write!(s, " align {a}").unwrap();
        }
        if let Some(a) = f.prefalign {
            write!(s, " prefalign({a})").unwrap();
        }
        if let Some(gc) = &f.gc {
            write!(s, " gc {}", quote(gc)).unwrap();
        }
        if let Some(c) = f.prefix {
            write!(s, " prefix {}", self.typed_const(c)).unwrap();
        }
        if let Some(c) = f.prologue {
            write!(s, " prologue {}", self.typed_const(c)).unwrap();
        }
        if let Some(c) = f.personality {
            write!(s, " personality {}", self.typed_const(c)).unwrap();
        }
        if f.has_body {
            s.push_str(&self.attachments(&gl.attachments, false));
        }
        self.out.push_str(&s);
        if !f.has_body {
            self.out.push('\n');
            return;
        }
        self.out.push_str(" {\n");
        self.cur = Some((f, slots));
        self.cur_global = Some(g);
        let preds = predecessors(f);
        for (i, &b) in f.layout.iter().enumerate() {
            let block = &f.blocks[b.index()];
            if i > 0 {
                self.out.push('\n');
            }
            let named = f.locals[block.local.index()].name.is_some();
            if i > 0 || named {
                let label = self.block_name(b);
                let label = format!("{}:", &label[1..]);
                let ps = &preds[b.index()];
                if ps.is_empty() {
                    if i > 0 {
                        writeln!(self.out, "{label:<50}; No predecessors!").unwrap();
                    } else {
                        writeln!(self.out, "{label}").unwrap();
                    }
                } else {
                    let names: Vec<String> = ps.iter().map(|&p| self.block_name(p)).collect();
                    writeln!(self.out, "{label:<50}; preds = {}", names.join(", ")).unwrap();
                }
            }
            self.note(Place::Block(b));
            for &inst in &block.insts {
                self.note(Place::Inst(inst));
                self.inst(&f.insts[inst.index()]);
            }
        }
        self.out.push_str("}\n");
        let (_, slots) = self.cur.take().unwrap();
        self.fn_slots.insert(g, slots);
    }

    fn note(&mut self, at: Place) {
        let (Some(notes), Some(g)) = (self.notes, self.cur_global) else { return };
        if let Some(text) = notes(g, at) {
            for line in text.lines() {
                writeln!(self.out, "  ; {line}").unwrap();
            }
        }
    }

    fn inst(&mut self, inst: &Inst) {
        for r in &inst.debug_records {
            let args: Vec<String> = r.args.iter().map(|&a| self.md(a)).collect();
            writeln!(self.out, "    #dbg_{}({})", r.kind, args.join(", ")).unwrap();
        }
        let mut s = String::from("  ");
        if let Some(r) = inst.result {
            write!(s, "{} = ", self.cur_local(r)).unwrap();
        }
        s.push_str(&self.op(inst));
        s.push_str(&self.attachments(&inst.attachments, true));
        self.out.push_str(&s);
        self.out.push('\n');
    }

    fn result_ty(&self, inst: &Inst) -> String {
        match inst.result {
            Some(r) => self.ty(self.cur.as_ref().unwrap().0.locals[r.index()].ty),
            None => "void".into(),
        }
    }

    fn label(&self, b: BlockId) -> String {
        format!("label {}", self.block_name(b))
    }

    fn op(&self, inst: &Inst) -> String {
        match &inst.op {
            Op::Ret(None) => "ret void".into(),
            Op::Ret(Some(v)) => format!("ret {}", self.typed(*v)),
            Op::Br(b) => format!("br {}", self.label(*b)),
            Op::CondBr { cond, then_dest, else_dest } => {
                format!("br {}, {}, {}", self.typed(*cond), self.label(*then_dest), self.label(*else_dest))
            }
            Op::Switch { value, default, cases } => {
                let mut s = format!("switch {}, {} [\n", self.typed(*value), self.label(*default));
                for (c, b) in cases {
                    writeln!(s, "    {}, {}", self.typed_const(*c), self.label(*b)).unwrap();
                }
                s.push_str("  ]");
                s
            }
            Op::IndirectBr { addr, dests } => {
                let ds: Vec<String> = dests.iter().map(|&d| self.label(d)).collect();
                format!("indirectbr {}, [{}]", self.typed(*addr), ds.join(", "))
            }
            Op::Invoke { call, normal, unwind } => format!(
                "invoke {}\n          to {} unwind {}",
                self.call(call, inst),
                self.label(*normal),
                self.label(*unwind)
            ),
            Op::CallBr { call, default, indirect } => {
                let ds: Vec<String> = indirect.iter().map(|&d| self.label(d)).collect();
                format!("callbr {}\n          to {} [{}]", self.call(call, inst), self.label(*default), ds.join(", "))
            }
            Op::Resume(v) => format!("resume {}", self.typed(*v)),
            Op::CatchSwitch { parent, handlers, unwind } => {
                let hs: Vec<String> = handlers.iter().map(|&h| self.label(h)).collect();
                format!(
                    "catchswitch within {} [{}] unwind {}",
                    parent.map_or_else(|| "none".into(), |p| self.value(p)),
                    hs.join(", "),
                    unwind.map_or_else(|| "to caller".into(), |u| self.label(u))
                )
            }
            Op::CatchRet { pad, dest } => format!("catchret from {} to {}", self.value(*pad), self.label(*dest)),
            Op::CleanupRet { pad, unwind } => format!(
                "cleanupret from {} unwind {}",
                self.value(*pad),
                unwind.map_or_else(|| "to caller".into(), |u| self.label(u))
            ),
            Op::Unreachable => "unreachable".into(),
            Op::FNeg { fmf, value } => format!("fneg{} {}", fast_math(*fmf), self.typed(*value)),
            Op::Binary { op, flags, fmf, lhs, rhs } => format!(
                "{}{}{} {}, {}",
                op.keyword(),
                int_flags(*flags),
                fast_math(*fmf),
                self.typed(*lhs),
                self.value(*rhs)
            ),
            Op::BitExtract { value, offset } => {
                format!("bitextract {}, {}, {}", self.result_ty(inst), self.typed(*value), self.typed(*offset))
            }
            Op::BitInsert { base, value, offset } => {
                format!("bitinsert {}, {}, {}", self.typed(*base), self.typed(*value), self.typed(*offset))
            }
            Op::ExtractElement { vector, index } => {
                format!("extractelement {}, {}", self.typed(*vector), self.typed(*index))
            }
            Op::InsertElement { vector, element, index } => {
                format!("insertelement {}, {}, {}", self.typed(*vector), self.typed(*element), self.typed(*index))
            }
            Op::ShuffleVector { a, b, mask } => {
                format!("shufflevector {}, {}, {}", self.typed(*a), self.typed(*b), self.typed_const(*mask))
            }
            Op::ExtractValue { aggregate, indices } => {
                format!("extractvalue {}{}", self.typed(*aggregate), index_list(indices))
            }
            Op::InsertValue { aggregate, element, indices } => {
                format!("insertvalue {}, {}{}", self.typed(*aggregate), self.typed(*element), index_list(indices))
            }
            Op::Alloca { inalloca, ty, count, align, addrspace } => {
                let mut s = format!("alloca {}{}", if *inalloca { "inalloca " } else { "" }, self.ty(*ty));
                if let Some(c) = count {
                    write!(s, ", {}", self.typed(*c)).unwrap();
                }
                if let Some(a) = align {
                    write!(s, ", align {a}").unwrap();
                }
                if let Some(a) = addrspace {
                    write!(s, ", addrspace({a})").unwrap();
                }
                s
            }
            Op::Load { volatile, atomic, ptr, align } => {
                let mut s = String::from("load ");
                if atomic.is_some() {
                    s.push_str("atomic ");
                }
                if *volatile {
                    s.push_str("volatile ");
                }
                if atomic.as_ref().is_some_and(|a| a.elementwise) {
                    s.push_str("elementwise ");
                }
                write!(s, "{}, {}", self.result_ty(inst), self.typed(*ptr)).unwrap();
                if let Some(a) = atomic {
                    s.push_str(&atomic_suffix(&a.scope, a.ordering));
                }
                if let Some(a) = align {
                    write!(s, ", align {a}").unwrap();
                }
                s
            }
            Op::Store { volatile, atomic, value, ptr, align } => {
                let mut s = String::from("store ");
                if atomic.is_some() {
                    s.push_str("atomic ");
                }
                if *volatile {
                    s.push_str("volatile ");
                }
                if atomic.as_ref().is_some_and(|a| a.elementwise) {
                    s.push_str("elementwise ");
                }
                write!(s, "{}, {}", self.typed(*value), self.typed(*ptr)).unwrap();
                if let Some(a) = atomic {
                    s.push_str(&atomic_suffix(&a.scope, a.ordering));
                }
                if let Some(a) = align {
                    write!(s, ", align {a}").unwrap();
                }
                s
            }
            Op::Fence { scope, ordering } => format!("fence{}", atomic_suffix(scope, *ordering)),
            Op::CmpXchg { weak, volatile, ptr, cmp, new, scope, success, failure, align } => {
                let mut s = String::from("cmpxchg ");
                if *weak {
                    s.push_str("weak ");
                }
                if *volatile {
                    s.push_str("volatile ");
                }
                write!(s, "{}, {}, {}", self.typed(*ptr), self.typed(*cmp), self.typed(*new)).unwrap();
                s.push_str(&atomic_suffix(scope, *success));
                write!(s, " {}", failure.keyword()).unwrap();
                if let Some(a) = align {
                    write!(s, ", align {a}").unwrap();
                }
                s
            }
            Op::AtomicRmw { op, volatile, elementwise, ptr, value, scope, ordering, align } => {
                let mut s = String::from("atomicrmw ");
                if *volatile {
                    s.push_str("volatile ");
                }
                if *elementwise {
                    s.push_str("elementwise ");
                }
                write!(s, "{} {}, {}", op.keyword(), self.typed(*ptr), self.typed(*value)).unwrap();
                s.push_str(&atomic_suffix(scope, *ordering));
                if let Some(a) = align {
                    write!(s, ", align {a}").unwrap();
                }
                s
            }
            Op::Gep { flags, source_ty, ptr, indices } => {
                let mut s =
                    format!("getelementptr{} {}, {}", gep_flags(*flags, None), self.ty(*source_ty), self.typed(*ptr));
                for &i in indices {
                    write!(s, ", {}", self.typed(i)).unwrap();
                }
                s
            }
            Op::Cast { op, flags, fmf, value } => format!(
                "{}{}{} {} to {}",
                op.keyword(),
                fast_math(*fmf),
                int_flags(*flags),
                self.typed(*value),
                self.result_ty(inst)
            ),
            Op::ICmp { pred, samesign, lhs, rhs } => format!(
                "icmp {}{} {}, {}",
                if *samesign { "samesign " } else { "" },
                pred.keyword(),
                self.typed(*lhs),
                self.value(*rhs)
            ),
            Op::FCmp { pred, fmf, lhs, rhs } => {
                format!("fcmp{} {} {}, {}", fast_math(*fmf), pred.keyword(), self.typed(*lhs), self.value(*rhs))
            }
            Op::Phi { fmf, incoming } => {
                let inc: Vec<String> =
                    incoming.iter().map(|(v, b)| format!("[ {}, {} ]", self.value(*v), self.block_name(*b))).collect();
                format!("phi{} {} {}", fast_math(*fmf), self.result_ty(inst), inc.join(", "))
            }
            Op::Select { fmf, cond, then_value, else_value } => format!(
                "select{} {}, {}, {}",
                fast_math(*fmf),
                self.typed(*cond),
                self.typed(*then_value),
                self.typed(*else_value)
            ),
            Op::Freeze(v) => format!("freeze {}", self.typed(*v)),
            Op::Call(call) => {
                let tail = match call.tail {
                    TailKind::None => "",
                    TailKind::Tail => "tail ",
                    TailKind::MustTail => "musttail ",
                    TailKind::NoTail => "notail ",
                };
                format!("{tail}call{}", self.call(call, inst))
            }
            Op::VaArg { list } => format!("va_arg {}, {}", self.typed(*list), self.result_ty(inst)),
            Op::LandingPad { cleanup, clauses } => {
                let mut s = format!("landingpad {}", self.result_ty(inst));
                if *cleanup {
                    s.push_str("\n          cleanup");
                }
                for c in clauses {
                    match c {
                        Clause::Catch(v) => write!(s, "\n          catch {}", self.typed(*v)).unwrap(),
                        Clause::Filter(v) => write!(s, "\n          filter {}", self.typed(*v)).unwrap(),
                    }
                }
                s
            }
            Op::CatchPad { within, args } => {
                let args: Vec<String> = args.iter().map(|&a| self.typed(a)).collect();
                format!("catchpad within {} [{}]", self.value(*within), args.join(", "))
            }
            Op::CleanupPad { within, args } => {
                let args: Vec<String> = args.iter().map(|&a| self.typed(a)).collect();
                format!(
                    "cleanuppad within {} [{}]",
                    within.map_or_else(|| "none".into(), |w| self.value(w)),
                    args.join(", ")
                )
            }
        }
    }

    /// The part of a call after `call`/`invoke`/`callbr`, starting with a
    /// space for `call`'s flags.
    fn call(&self, call: &Call, inst: &Inst) -> String {
        let _ = inst;
        let Type::Function { ret, varargs, .. } = self.m.types.get(call.fn_ty) else { unreachable!() };
        let mut s = String::new();
        s.push_str(&fast_math(call.fmf));
        s.push(' ');
        s.push_str(&cconv(&call.cconv));
        s.push_str(&self.attrs_sp(&call.ret_attrs));
        if let Some(a) = call.addrspace {
            write!(s, "addrspace({a}) ").unwrap();
        }
        let ty = if *varargs { self.ty(call.fn_ty) } else { self.ty(*ret) };
        write!(s, "{ty} {}(", self.value(call.callee)).unwrap();
        let args: Vec<String> = call
            .args
            .iter()
            .map(|a| {
                let ty = self.ty(self.value_ty(a.value));
                if a.attrs.is_empty() {
                    format!("{ty} {}", self.value(a.value))
                } else {
                    format!("{ty} {} {}", self.attrs(&a.attrs), self.value(a.value))
                }
            })
            .collect();
        s.push_str(&args.join(", "));
        s.push(')');
        if !call.fn_attrs.is_empty() {
            write!(s, " {}", self.attrs(&call.fn_attrs)).unwrap();
        }
        if !call.bundles.is_empty() {
            let bs: Vec<String> = call
                .bundles
                .iter()
                .map(|b| {
                    let ins: Vec<String> = b.inputs.iter().map(|&v| self.typed(v)).collect();
                    format!("{}({})", quote(&b.tag), ins.join(", "))
                })
                .collect();
            write!(s, " [ {} ]", bs.join(", ")).unwrap();
        }
        // `invoke` and `callbr` follow the keyword with a space already.
        if matches!(inst.op, Op::Invoke { .. } | Op::CallBr { .. }) {
            return s.trim_start().to_string();
        }
        s
    }
}

/// Globals in the order LLVM prints them: variables, then aliases, then
/// ifuncs, then functions, each in module order.
fn print_order(m: &Module) -> Vec<GlobalId> {
    let rank = |g: &GlobalId| match m.global(*g).kind {
        GlobalKind::Variable(_) => 0,
        GlobalKind::Alias { .. } => 1,
        GlobalKind::IFunc { .. } => 2,
        GlobalKind::Function(_) => 3,
        GlobalKind::Forward => 4,
    };
    let mut order = m.global_order.clone();
    order.sort_by_key(rank);
    order
}

/// For each block, the blocks that branch to it, most recent first.
fn predecessors(f: &Function) -> Vec<Vec<BlockId>> {
    let mut preds = vec![Vec::new(); f.blocks.len()];
    for &b in &f.layout {
        if let Some(&last) = f.blocks[b.index()].insts.last() {
            for s in f.insts[last.index()].op.successors() {
                if !preds[s.index()].contains(&b) {
                    preds[s.index()].push(b);
                }
            }
        }
    }
    for p in &mut preds {
        p.reverse();
    }
    preds
}

fn struct_text(packed: bool, fields: &[String]) -> String {
    let body = if fields.is_empty() { "{}".to_string() } else { format!("{{ {} }}", fields.join(", ")) };
    if packed { format!("<{body}>") } else { body }
}

fn unnamed_addr(u: UnnamedAddr) -> &'static str {
    match u {
        UnnamedAddr::None => "",
        UnnamedAddr::Local => "local_unnamed_addr ",
        UnnamedAddr::Global => "unnamed_addr ",
    }
}

fn cconv(c: &CallConv) -> String {
    match c {
        CallConv::C => String::new(),
        CallConv::Named(n) => format!("{n} "),
        CallConv::Numbered(n) => format!("cc {n} "),
    }
}

fn int_flags(f: IntFlags) -> String {
    IntFlags::NAMES.iter().filter(|(_, b)| f.contains(*b)).map(|(n, _)| format!(" {n}")).collect()
}

fn fast_math(f: FastMath) -> String {
    if f == FastMath::ALL {
        return " fast".into();
    }
    FastMath::NAMES.iter().filter(|(_, b)| f.0 & b != 0).map(|(n, _)| format!(" {n}")).collect()
}

fn gep_flags(f: GepFlags, inrange: Option<(i64, i64)>) -> String {
    let mut s = String::new();
    if f.0 & GepFlags::INBOUNDS != 0 {
        s.push_str(" inbounds");
    } else if f.0 & GepFlags::NUSW != 0 {
        s.push_str(" nusw");
    }
    if f.0 & GepFlags::NUW != 0 {
        s.push_str(" nuw");
    }
    if let Some((a, b)) = inrange {
        write!(s, " inrange({a}, {b})").unwrap();
    }
    s
}

fn atomic_suffix(scope: &Option<Box<[u8]>>, ordering: Ordering) -> String {
    match scope {
        Some(sc) => format!(" syncscope({}) {}", quote(sc), ordering.keyword()),
        None => format!(" {}", ordering.keyword()),
    }
}

fn index_list(indices: &[u32]) -> String {
    indices.iter().map(|i| format!(", {i}")).collect()
}

fn inline_asm(a: &InlineAsm) -> String {
    let mut s = String::from("asm ");
    if a.side_effect {
        s.push_str("sideeffect ");
    }
    if a.align_stack {
        s.push_str("alignstack ");
    }
    if a.intel_dialect {
        s.push_str("inteldialect ");
    }
    if a.unwind {
        s.push_str("unwind ");
    }
    write!(s, "{}, {}", quote(&a.asm), quote(&a.constraints)).unwrap();
    s
}

/// A quoted string with `"`, `\` and unprintable bytes escaped as `\XX`.
pub fn quote(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() + 2);
    s.push('"');
    for &b in bytes {
        if (0x20..0x7f).contains(&b) && b != b'"' && b != b'\\' {
            s.push(b as char);
        } else {
            write!(s, "\\{b:02X}").unwrap();
        }
    }
    s.push('"');
    s
}

/// A name after a sigil: bare if it only has identifier characters, else quoted.
pub fn ident(name: &str) -> String {
    let b = name.as_bytes();
    let plain = !b.is_empty()
        && !b[0].is_ascii_digit()
        && b.iter().all(|&c| c.is_ascii_alphanumeric() || matches!(c, b'-' | b'$' | b'.' | b'_'));
    if plain { name.to_string() } else { quote(b) }
}

/// A metadata name: special characters are escaped but never quoted.
fn md_name(name: &str) -> String {
    let mut s = String::new();
    for (i, &c) in name.as_bytes().iter().enumerate() {
        let ok = c.is_ascii_alphabetic() || matches!(c, b'-' | b'$' | b'.' | b'_') || (i > 0 && c.is_ascii_digit());
        if ok {
            s.push(c as char);
        } else {
            write!(s, "\\{c:02X}").unwrap();
        }
    }
    s
}
