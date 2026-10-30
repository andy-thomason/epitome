//! The parser for IR text.
//!
//! It works over the token list from the lexer with unlimited lookahead.
//! Names may be used before they are defined; such forward references are
//! resolved when the definition arrives, and any left over are reported at
//! the end.

use std::collections::HashMap;
use std::fmt;

use crate::apint::ApInt;
use crate::float::{self, FloatKind};
use crate::lexer::{Token, line_col, offset_of, tokenize};
use crate::metadata;
use crate::module::*;
use crate::types::{StructBody, Type, TypeId};

/// A parse error with its position in the source.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseError {
    /// Byte offset into the source.
    pub offset: usize,
    /// 1-based line.
    pub line: usize,
    /// 1-based column, in bytes.
    pub column: usize,
    pub message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}: {}", self.line, self.column, self.message)
    }
}

impl std::error::Error for ParseError {}

pub type Result<T> = std::result::Result<T, ParseError>;

/// Parses a module from IR text.
pub fn parse_module(src: &str) -> Result<Module> {
    let mut p = Parser::new(src);
    p.parse_top_level()?;
    p.finish()
}

/// A name: `%foo` / `@foo` or `%12` / `@12`.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
enum Key {
    Named(String),
    Num(u32),
}

/// Words that may begin a constant, so cannot be attributes.
const VALUE_KEYWORDS: &[&str] = &[
    "true",
    "false",
    "null",
    "none",
    "undef",
    "poison",
    "zeroinitializer",
    "splat",
    "blockaddress",
    "dso_local_equivalent",
    "no_cfi",
    "ptrauth",
    "asm",
    "getelementptr",
    "extractelement",
    "insertelement",
    "shufflevector",
    "add",
    "sub",
    "xor",
    "trunc",
    "ptrtoint",
    "ptrtoaddr",
    "inttoptr",
    "bitcast",
    "addrspacecast",
];

/// Words that begin a top-level entity. Attribute lists stop at these.
const TOP_LEVEL_KEYWORDS: &[&str] =
    &["define", "declare", "attributes", "source_filename", "target", "module", "uselistorder", "uselistorder_bb"];

const OPCODES: &[&str] = &[
    "ret",
    "br",
    "switch",
    "indirectbr",
    "invoke",
    "callbr",
    "resume",
    "catchswitch",
    "catchret",
    "cleanupret",
    "unreachable",
    "fneg",
    "add",
    "fadd",
    "sub",
    "fsub",
    "mul",
    "fmul",
    "udiv",
    "sdiv",
    "fdiv",
    "urem",
    "srem",
    "frem",
    "shl",
    "lshr",
    "ashr",
    "and",
    "or",
    "xor",
    "bitextract",
    "bitinsert",
    "extractelement",
    "insertelement",
    "shufflevector",
    "extractvalue",
    "insertvalue",
    "alloca",
    "load",
    "store",
    "fence",
    "cmpxchg",
    "atomicrmw",
    "getelementptr",
    "trunc",
    "zext",
    "sext",
    "fptrunc",
    "fpext",
    "fptoui",
    "fptosi",
    "uitofp",
    "sitofp",
    "ptrtoint",
    "ptrtoaddr",
    "inttoptr",
    "bitcast",
    "addrspacecast",
    "icmp",
    "fcmp",
    "phi",
    "select",
    "freeze",
    "call",
    "tail",
    "musttail",
    "notail",
    "va_arg",
    "landingpad",
    "catchpad",
    "cleanuppad",
    "uselistorder",
];

/// Attributes whose argument is a type.
const TYPE_ATTRS: &[&str] = &["byval", "byref", "preallocated", "inalloca", "sret", "elementtype"];

/// Parser state for the function being parsed.
struct FnState {
    func: Function,
    names: HashMap<Key, LocalId>,
    /// Where each forward reference was first used, for error messages.
    first_use: HashMap<LocalId, usize>,
    /// Blocks that have appeared in the body.
    placed: Vec<bool>,
    next_num: u32,
}

struct Parser<'a> {
    src: &'a str,
    toks: Vec<Token<'a>>,
    pos: usize,
    m: Module,
    global_names: HashMap<Key, GlobalId>,
    global_first_use: HashMap<GlobalId, usize>,
    next_global_num: u32,
    md_slots: HashMap<u32, MdId>,
    md_first_use: HashMap<MdId, usize>,
    comdat_names: HashMap<String, ComdatId>,
    comdat_first_use: HashMap<ComdatId, usize>,
    defined_structs: HashMap<TypeId, ()>,
    struct_first_use: HashMap<TypeId, usize>,
    /// Block names of functions already parsed, for `blockaddress`.
    func_blocks: HashMap<GlobalId, HashMap<Key, BlockId>>,
    /// `blockaddress` references into functions not yet parsed. The index
    /// in the list is the `BlockId` the block will get.
    pending_blocks: HashMap<GlobalId, Vec<(Key, usize)>>,
    f: Option<FnState>,
}

impl<'a> Parser<'a> {
    fn new(src: &'a str) -> Self {
        Parser {
            src,
            toks: tokenize(src),
            pos: 0,
            m: Module::default(),
            global_names: HashMap::new(),
            global_first_use: HashMap::new(),
            next_global_num: 0,
            md_slots: HashMap::new(),
            md_first_use: HashMap::new(),
            comdat_names: HashMap::new(),
            comdat_first_use: HashMap::new(),
            defined_structs: HashMap::new(),
            struct_first_use: HashMap::new(),
            func_blocks: HashMap::new(),
            pending_blocks: HashMap::new(),
            f: None,
        }
    }

    // -----------------------------------------------------------------------
    // Token helpers

    fn peek(&self) -> Token<'a> {
        self.toks[self.pos]
    }

    fn peek_at(&self, n: usize) -> Token<'a> {
        self.toks[(self.pos + n).min(self.toks.len() - 1)]
    }

    fn next(&mut self) -> Token<'a> {
        let t = self.toks[self.pos];
        if self.pos < self.toks.len() - 1 {
            self.pos += 1;
        }
        t
    }

    fn eat(&mut self, text: &str) -> bool {
        if self.peek().0 == text {
            self.next();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, text: &str) -> Result<Token<'a>> {
        if self.peek().0 == text { Ok(self.next()) } else { Err(self.error_here(format!("expected '{text}'"))) }
    }

    fn offset(&self, tok: Token<'a>) -> usize {
        offset_of(self.src, tok.0)
    }

    fn error_at_offset(&self, offset: usize, message: impl Into<String>) -> ParseError {
        let (line, column) = line_col(self.src, offset);
        ParseError { offset, line, column, message: message.into() }
    }

    fn error_at(&self, tok: Token<'a>, message: impl Into<String>) -> ParseError {
        self.error_at_offset(self.offset(tok), message)
    }

    fn error_here(&self, message: impl Into<String>) -> ParseError {
        let tok = self.peek();
        let found = if tok.is_eof() { "end of file".to_string() } else { format!("'{}'", tok.0) };
        self.error_at(tok, format!("{}, found {found}", message.into()))
    }

    fn expect_u64(&mut self) -> Result<u64> {
        let tok = self.peek();
        match tok.0.parse::<u64>() {
            Ok(v) => {
                self.next();
                Ok(v)
            }
            Err(_) => Err(self.error_here("expected an unsigned integer")),
        }
    }

    fn expect_u32(&mut self) -> Result<u32> {
        let tok = self.peek();
        let v = self.expect_u64()?;
        u32::try_from(v).map_err(|_| self.error_at(tok, "integer too large"))
    }

    fn expect_i64(&mut self) -> Result<i64> {
        let tok = self.peek();
        match tok.0.parse::<i64>() {
            Ok(v) => {
                self.next();
                Ok(v)
            }
            Err(_) => Err(self.error_here("expected an integer")),
        }
    }

    fn expect_string(&mut self) -> Result<Vec<u8>> {
        let tok = self.peek();
        if tok.is_string() && tok.0.len() >= 2 && tok.0.ends_with('"') {
            self.next();
            Ok(tok.string_bytes())
        } else if tok.is_string() {
            Err(self.error_at(tok, "unterminated string"))
        } else {
            Err(self.error_here("expected a string"))
        }
    }

    fn name_of(&self, tok: Token<'a>) -> Result<String> {
        String::from_utf8(tok.name_bytes()).map_err(|_| self.error_at(tok, "name is not valid UTF-8"))
    }

    /// The key for a `%`/`@` token: a number for `%12`, else a name.
    fn key_of(&self, tok: Token<'a>) -> Result<Key> {
        let body = &tok.0[1..];
        if !body.is_empty() && body.bytes().all(|b| b.is_ascii_digit()) {
            let n = body.parse().map_err(|_| self.error_at(tok, "number too large"))?;
            return Ok(Key::Num(n));
        }
        if body.is_empty() {
            return Err(self.error_at(tok, "expected a name"));
        }
        Ok(Key::Named(self.name_of(tok)?))
    }

    /// The key for a label definition token such as `entry:` or `7:`.
    fn label_key(&self, tok: Token<'a>) -> Result<Key> {
        let body = &tok.0[..tok.0.len() - 1];
        if body.bytes().all(|b| b.is_ascii_digit()) {
            let n = body.parse().map_err(|_| self.error_at(tok, "number too large"))?;
            return Ok(Key::Num(n));
        }
        let bytes = if body.starts_with('"') {
            crate::lexer::unescape(&body[1..body.len() - 1])
        } else {
            body.as_bytes().to_vec()
        };
        let name = String::from_utf8(bytes).map_err(|_| self.error_at(tok, "name is not valid UTF-8"))?;
        Ok(Key::Named(name))
    }

    /// Skips a parenthesised group starting at the current `(`, returning
    /// the source text between the parentheses.
    fn raw_parens(&mut self) -> Result<String> {
        let open = self.expect("(")?;
        let start = self.offset(open) + 1;
        let mut depth = 1;
        loop {
            let tok = self.next();
            match tok.0 {
                "(" => depth += 1,
                ")" => {
                    depth -= 1;
                    if depth == 0 {
                        return Ok(self.src[start..self.offset(tok)].to_string());
                    }
                }
                "" => return Err(self.error_at(open, "unclosed '('")),
                _ => {}
            }
        }
    }

    fn parse_align(&mut self) -> Result<u64> {
        let tok = self.peek();
        let v = self.expect_u64()?;
        if v != 0 && !v.is_power_of_two() {
            return Err(self.error_at(tok, "alignment must be a power of two"));
        }
        Ok(v)
    }

    // -----------------------------------------------------------------------
    // Top level

    fn parse_top_level(&mut self) -> Result<()> {
        loop {
            let tok = self.peek();
            match tok.0 {
                "" => return Ok(()),
                "source_filename" => {
                    self.next();
                    self.expect("=")?;
                    self.m.source_filename = Some(self.expect_string()?);
                }
                "target" => {
                    self.next();
                    let what = self.next();
                    self.expect("=")?;
                    let value = self.expect_string()?;
                    match what.0 {
                        "datalayout" => self.m.data_layout = Some(value),
                        "triple" => self.m.triple = Some(value),
                        _ => return Err(self.error_at(what, "expected 'datalayout' or 'triple'")),
                    }
                }
                // `module asm "line"`, or with options and several lines:
                // `module asm(target_cpu: "apple-m1") "line" "line"`.
                "module" => {
                    self.next();
                    self.expect("asm")?;
                    if self.peek().0 == "(" {
                        self.next();
                        while self.peek().0 != ")" {
                            // (`target_cpu:` is one token, as a label is.)
                            let key = self.next().0.to_string();
                            let key = match key.strip_suffix(':') {
                                Some(k) => k.to_string(),
                                None => {
                                    self.expect(":")?;
                                    key
                                }
                            };
                            let value = self.expect_string()?;
                            self.m.module_asm_options.push((key, value));
                            if self.peek().0 == "," {
                                self.next();
                            }
                        }
                        self.expect(")")?;
                    }
                    let s = self.expect_string()?;
                    self.m.module_asm.push(s);
                    while self.peek().is_string() {
                        let s = self.expect_string()?;
                        self.m.module_asm.push(s);
                    }
                }
                "attributes" => self.parse_attr_group()?,
                "define" | "declare" => self.parse_function()?,
                "uselistorder" | "uselistorder_bb" => self.skip_uselistorder()?,
                _ if tok.is_local() => self.parse_named_type()?,
                _ if tok.is_global() => self.parse_global()?,
                _ if tok.is_comdat() => self.parse_comdat()?,
                _ if tok.is_metadata_name() => self.parse_metadata_def()?,
                _ if tok.0.starts_with('^') => {
                    return Err(self.error_at(tok, "summary entries are not supported"));
                }
                _ => return Err(self.error_here("expected a top-level entity")),
            }
        }
    }

    fn skip_uselistorder(&mut self) -> Result<()> {
        // `uselistorder <ty> <value>, { 1, 0, ... }` or
        // `uselistorder_bb @f, %bb, { ... }`. The order is not modelled.
        while self.peek().0 != "{" {
            if self.peek().is_eof() {
                return Err(self.error_here("expected '{'"));
            }
            self.next();
        }
        while self.next().0 != "}" {
            if self.peek().is_eof() {
                return Err(self.error_here("expected '}'"));
            }
        }
        Ok(())
    }

    fn finish(mut self) -> Result<Module> {
        for (i, g) in self.m.globals.iter().enumerate() {
            if matches!(g.kind, GlobalKind::Forward) {
                let off = self.global_first_use[&GlobalId(i as u32)];
                let name = g.name.clone().unwrap_or_else(|| "?".into());
                return Err(self.error_at_offset(off, format!("use of undefined global '@{name}'")));
            }
        }
        for (i, node) in self.m.metadata.iter().enumerate() {
            if matches!(node.data, Metadata::Forward) {
                let off = self.md_first_use[&MdId(i as u32)];
                return Err(self.error_at_offset(off, format!("use of undefined metadata '!{}'", node.slot.unwrap())));
            }
        }
        for (i, c) in self.m.comdats.iter().enumerate() {
            if c.kind.is_none() {
                let off = self.comdat_first_use[&ComdatId(i as u32)];
                return Err(self.error_at_offset(off, format!("use of undefined comdat '${}'", c.name)));
            }
        }
        for (ty, off) in &self.struct_first_use {
            if !self.defined_structs.contains_key(ty) {
                return Err(self.error_at_offset(*off, "use of undefined type"));
            }
        }
        if let Some((_, refs)) = self.pending_blocks.iter().next() {
            return Err(self.error_at_offset(refs[0].1, "blockaddress refers to a function with no body"));
        }
        // Function bodies are checked as they finish; drop parser-only state.
        self.func_blocks.clear();
        // Declarations of upgraded debug intrinsics are no longer used.
        let globals = &self.m.globals;
        self.m.global_order.retain(|g| {
            let global = &globals[g.index()];
            let unused = matches!(&global.kind, GlobalKind::Function(f) if !f.has_body)
                && global.name.as_deref().is_some_and(|n| debug_intrinsic_kind(n).is_some());
            !unused
        });
        metadata::unique_metadata(&mut self.m);
        Ok(self.m)
    }

    fn parse_named_type(&mut self) -> Result<()> {
        let name_tok = self.next();
        let key = self.key_of(name_tok)?;
        self.expect("=")?;
        self.expect("type")?;
        let ty = match &key {
            Key::Named(n) => self.m.types.named_struct(n),
            Key::Num(n) => self.m.types.numbered_struct(*n),
        };
        if self.defined_structs.insert(ty, ()).is_some() {
            return Err(self.error_at(name_tok, "redefinition of type"));
        }
        let Type::Named(index) = *self.m.types.get(ty) else { unreachable!() };
        if self.eat("opaque") {
            return Ok(());
        }
        let body_tok = self.peek();
        let body = self.parse_type()?;
        match self.m.types.get(body).clone() {
            Type::Struct { packed, fields } => {
                self.m.types.structs[index as usize].body = Some(StructBody { packed, fields });
                Ok(())
            }
            _ => Err(self.error_at(body_tok, "a named type must be a struct")),
        }
    }

    fn parse_comdat(&mut self) -> Result<()> {
        let tok = self.next();
        let id = self.comdat_ref(tok)?;
        self.expect("=")?;
        self.expect("comdat")?;
        let kind_tok = self.next();
        let kind = ComdatKind::ALL
            .iter()
            .find(|(k, _)| *k == kind_tok.0)
            .map(|(_, k)| *k)
            .ok_or_else(|| self.error_at(kind_tok, "expected a comdat selection kind"))?;
        let c = &mut self.m.comdats[id.index()];
        if c.kind.is_some() {
            return Err(self.error_at(tok, "redefinition of comdat"));
        }
        c.kind = Some(kind);
        Ok(())
    }

    fn comdat_ref(&mut self, tok: Token<'a>) -> Result<ComdatId> {
        let name = self.name_of(tok)?;
        if let Some(&id) = self.comdat_names.get(&name) {
            return Ok(id);
        }
        let id = ComdatId(self.m.comdats.len() as u32);
        self.m.comdats.push(Comdat { name: name.clone(), kind: None });
        self.comdat_names.insert(name, id);
        let off = self.offset(tok);
        self.comdat_first_use.insert(id, off);
        Ok(id)
    }

    /// `comdat` or `comdat($name)`, after the `comdat` keyword.
    fn parse_comdat_ref(&mut self) -> Result<ComdatRef> {
        if self.eat("(") {
            let tok = self.next();
            if !tok.is_comdat() {
                return Err(self.error_at(tok, "expected a comdat name"));
            }
            let id = self.comdat_ref(tok)?;
            self.expect(")")?;
            Ok(ComdatRef::Named(id))
        } else {
            Ok(ComdatRef::Own)
        }
    }

    fn parse_attr_group(&mut self) -> Result<()> {
        self.expect("attributes")?;
        let tok = self.next();
        if !tok.is_attr_group() {
            return Err(self.error_at(tok, "expected an attribute group such as #0"));
        }
        let id: u32 = tok.0[1..].parse().map_err(|_| self.error_at(tok, "bad attribute group number"))?;
        self.expect("=")?;
        self.expect("{")?;
        let mut attrs = Vec::new();
        while !self.eat("}") {
            if self.peek().is_eof() {
                return Err(self.error_here("expected '}'"));
            }
            attrs.push(self.parse_attr()?);
        }
        if self.m.attr_groups.iter().any(|g| g.id == id) {
            return Err(self.error_at(tok, "redefinition of attribute group"));
        }
        self.m.attr_groups.push(AttrGroup { id, attrs });
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Attributes

    /// True if the token can start an attribute in a list that is followed
    /// by `stop` words.
    fn is_attr_start(&self, tok: Token<'a>) -> bool {
        tok.is_attr_group()
            || tok.is_string()
            || (tok.is_word()
                && !tok.is_number()
                && !TOP_LEVEL_KEYWORDS.contains(&tok.0)
                && !OPCODES.contains(&tok.0)
                && !VALUE_KEYWORDS.contains(&tok.0)
                && !self.is_type_start(tok))
    }

    fn parse_attr(&mut self) -> Result<Attr> {
        let tok = self.next();
        if tok.is_attr_group() {
            let n = tok.0[1..].parse().map_err(|_| self.error_at(tok, "bad attribute group number"))?;
            return Ok(Attr::Group(n));
        }
        if tok.is_string() {
            self.pos -= 1;
            let key = self.expect_string()?;
            let value = if self.eat("=") { Some(self.expect_string()?) } else { None };
            return Ok(Attr::Str { key, value });
        }
        if !tok.is_word() {
            return Err(self.error_at(tok, "expected an attribute"));
        }
        let name = tok.0.to_string();
        if self.peek().0 == "(" {
            if TYPE_ATTRS.contains(&tok.0) {
                self.next();
                let ty = self.parse_type()?;
                self.expect(")")?;
                return Ok(Attr::Type { name, ty });
            }
            let text = self.raw_parens()?;
            if let Ok(value) = text.trim().parse::<u64>() {
                return Ok(Attr::Int { name, value, style: IntStyle::Paren });
            }
            return Ok(Attr::Parens { name, text });
        }
        if name == "align" {
            let value = self.parse_align()?;
            return Ok(Attr::Int { name, value, style: IntStyle::Space });
        }
        if self.peek().0 == "=" {
            self.next();
            let value = self.expect_u64()?;
            return Ok(Attr::Int { name, value, style: IntStyle::Eq });
        }
        Ok(Attr::Word(name))
    }

    /// Return attributes: everything up to the type.
    fn parse_ret_attrs(&mut self) -> Result<Vec<Attr>> {
        let mut attrs = Vec::new();
        while !self.is_type_start(self.peek()) && self.is_attr_start(self.peek()) {
            attrs.push(self.parse_attr()?);
        }
        Ok(attrs)
    }

    /// Parameter or argument attributes: everything up to the value.
    fn parse_param_attrs(&mut self) -> Result<Vec<Attr>> {
        let mut attrs = Vec::new();
        while self.is_attr_start(self.peek()) && !self.peek().is_attr_group() {
            attrs.push(self.parse_attr()?);
        }
        Ok(attrs)
    }

    /// Function attributes at a call site, up to bundles, `to`, attachments
    /// or the next instruction.
    fn parse_call_fn_attrs(&mut self) -> Result<Vec<Attr>> {
        let mut attrs = Vec::new();
        while self.is_attr_start(self.peek()) && !matches!(self.peek().0, "to" | "unwind") {
            attrs.push(self.parse_attr()?);
        }
        Ok(attrs)
    }

    // -----------------------------------------------------------------------
    // Types

    fn is_type_start(&self, tok: Token<'a>) -> bool {
        let t = tok.0;
        matches!(
            t,
            "void"
                | "label"
                | "token"
                | "metadata"
                | "x86_amx"
                | "half"
                | "bfloat"
                | "float"
                | "double"
                | "x86_fp80"
                | "fp128"
                | "ppc_fp128"
                | "ptr"
                | "target"
                | "{"
                | "<"
                | "["
        ) || ((t.starts_with('i') || t.starts_with('b')) && t.len() > 1 && t[1..].bytes().all(|b| b.is_ascii_digit()))
            || tok.is_local()
    }

    fn parse_type(&mut self) -> Result<TypeId> {
        let mut ty = self.parse_base_type()?;
        loop {
            match self.peek().0 {
                "(" => ty = self.parse_function_type(ty)?,
                "*" => return Err(self.error_here("typed pointers are not supported; use 'ptr'")),
                _ => return Ok(ty),
            }
        }
    }

    fn parse_function_type(&mut self, ret: TypeId) -> Result<TypeId> {
        self.expect("(")?;
        let mut params = Vec::new();
        let mut varargs = false;
        if !self.eat(")") {
            loop {
                if self.eat("...") {
                    varargs = true;
                    self.expect(")")?;
                    break;
                }
                params.push(self.parse_type()?);
                if self.eat(")") {
                    break;
                }
                self.expect(",")?;
            }
        }
        Ok(self.m.types.intern(Type::Function { ret, params, varargs }))
    }

    fn parse_base_type(&mut self) -> Result<TypeId> {
        let tok = self.next();
        let t = tok.0;
        let ty = match t {
            "void" => Type::Void,
            "label" => Type::Label,
            "token" => Type::Token,
            "metadata" => Type::Metadata,
            "x86_amx" => Type::X86Amx,
            "half" => Type::Half,
            "bfloat" => Type::BFloat,
            "float" => Type::Float,
            "double" => Type::Double,
            "x86_fp80" => Type::X86Fp80,
            "fp128" => Type::Fp128,
            "ppc_fp128" => Type::PpcFp128,
            "ptr" => {
                let addrspace = if self.peek().0 == "addrspace" { self.parse_addrspace()? } else { 0 };
                Type::Ptr(addrspace)
            }
            "target" => {
                self.expect("(")?;
                let name = self.expect_string()?;
                let mut types = Vec::new();
                let mut ints = Vec::new();
                while self.eat(",") {
                    if self.peek().is_int() {
                        ints.push(self.expect_u32()?);
                    } else if ints.is_empty() {
                        types.push(self.parse_type()?);
                    } else {
                        return Err(self.error_here("type parameters must come before integer parameters"));
                    }
                }
                self.expect(")")?;
                Type::TargetExt { name, types, ints }
            }
            "{" => {
                let fields = self.parse_type_list("}")?;
                Type::Struct { packed: false, fields }
            }
            "[" => {
                let len = self.expect_u64()?;
                self.expect("x")?;
                let elem = self.parse_type()?;
                self.expect("]")?;
                Type::Array { len, elem }
            }
            "<" => {
                if self.eat("{") {
                    let fields = self.parse_type_list("}")?;
                    self.expect(">")?;
                    Type::Struct { packed: true, fields }
                } else {
                    let scalable = self.eat("vscale");
                    if scalable {
                        self.expect("x")?;
                    }
                    let len_tok = self.peek();
                    let len = self.expect_u32()?;
                    if len == 0 {
                        return Err(self.error_at(len_tok, "vectors must have at least one element"));
                    }
                    self.expect("x")?;
                    let elem = self.parse_type()?;
                    self.expect(">")?;
                    Type::Vector { scalable, len, elem }
                }
            }
            _ if tok.is_local() => {
                let ty = match self.key_of(tok)? {
                    Key::Named(n) => self.m.types.named_struct(&n),
                    Key::Num(n) => self.m.types.numbered_struct(n),
                };
                let off = self.offset(tok);
                self.struct_first_use.entry(ty).or_insert(off);
                return Ok(ty);
            }
            _ if (t.starts_with('i') || t.starts_with('b'))
                && t.len() > 1
                && t[1..].bytes().all(|b| b.is_ascii_digit()) =>
            {
                let bits: u32 = t[1..].parse().map_err(|_| self.error_at(tok, "bit width too large"))?;
                if bits == 0 || bits > 1 << 23 {
                    return Err(self.error_at(tok, "bit width must be between 1 and 2^23"));
                }
                if t.starts_with('i') { Type::Int(bits) } else { Type::Byte(bits) }
            }
            _ => {
                self.pos -= 1;
                return Err(self.error_here("expected a type"));
            }
        };
        Ok(self.m.types.intern(ty))
    }

    fn parse_type_list(&mut self, close: &str) -> Result<Vec<TypeId>> {
        let mut list = Vec::new();
        if self.eat(close) {
            return Ok(list);
        }
        loop {
            list.push(self.parse_type()?);
            if self.eat(close) {
                return Ok(list);
            }
            self.expect(",")?;
        }
    }

    /// `addrspace(N)` or `addrspace("A")`.
    fn parse_addrspace(&mut self) -> Result<u32> {
        self.expect("addrspace")?;
        self.expect("(")?;
        let v = if self.peek().is_string() {
            let tok = self.peek();
            let s = self.expect_string()?;
            let spec = match s.as_slice() {
                b"A" => 'A',
                b"G" => 'G',
                b"P" => 'P',
                _ => return Err(self.error_at(tok, "expected \"A\", \"G\" or \"P\"")),
            };
            self.layout_addrspace(spec)
        } else {
            self.expect_u32()?
        };
        self.expect(")")?;
        Ok(v)
    }

    /// The address space the data layout gives for `A`, `G` or `P`.
    fn layout_addrspace(&self, spec: char) -> u32 {
        let Some(layout) = &self.m.data_layout else { return 0 };
        let layout = String::from_utf8_lossy(layout);
        layout.split('-').find_map(|part| part.strip_prefix(spec).and_then(|n| n.parse().ok())).unwrap_or(0)
    }

    // -----------------------------------------------------------------------
    // Globals

    fn global_ref(&mut self, tok: Token<'a>) -> Result<GlobalId> {
        let key = self.key_of(tok)?;
        if let Some(&id) = self.global_names.get(&key) {
            return Ok(id);
        }
        let id = GlobalId(self.m.globals.len() as u32);
        let name = match &key {
            Key::Named(n) => Some(n.clone()),
            Key::Num(_) => None,
        };
        self.m.globals.push(Global {
            name,
            props: GlobalProps::default(),
            attachments: Vec::new(),
            kind: GlobalKind::Forward,
        });
        self.global_names.insert(key, id);
        let off = self.offset(tok);
        self.global_first_use.insert(id, off);
        Ok(id)
    }

    /// Looks up or creates the global named by `tok`, checking it is not
    /// already defined.
    fn global_def(&mut self, tok: Token<'a>) -> Result<GlobalId> {
        if let Key::Num(n) = self.key_of(tok)? {
            if n < self.next_global_num {
                return Err(self.error_at(
                    tok,
                    format!("global numbers must increase; expected @{} or higher", self.next_global_num),
                ));
            }
            self.next_global_num = n + 1;
        }
        let id = self.global_ref(tok)?;
        if !matches!(self.m.globals[id.index()].kind, GlobalKind::Forward) {
            return Err(self.error_at(tok, "redefinition of global"));
        }
        self.m.global_order.push(id);
        Ok(id)
    }

    /// Linkage, preemption, visibility, DLL storage, thread-local and
    /// unnamed_addr, in that order, as far as they are present.
    fn parse_global_props(&mut self) -> Result<GlobalProps> {
        let mut props = GlobalProps::default();
        if let Some((_, l)) = Linkage::ALL.iter().find(|(k, _)| *k == self.peek().0) {
            props.linkage = *l;
            props.explicit_linkage = true;
            self.next();
        }
        if self.eat("dso_local") {
            props.dso_local = true;
        } else {
            self.eat("dso_preemptable");
        }
        props.visibility = match self.peek().0 {
            "default" => Visibility::Default,
            "hidden" => Visibility::Hidden,
            "protected" => Visibility::Protected,
            _ => props.visibility,
        };
        if matches!(self.peek().0, "default" | "hidden" | "protected") {
            self.next();
        }
        props.dll_storage = match self.peek().0 {
            "dllimport" => DllStorage::Import,
            "dllexport" => DllStorage::Export,
            _ => DllStorage::None,
        };
        if props.dll_storage != DllStorage::None {
            self.next();
        }
        if self.eat("thread_local") {
            let mut model = ThreadLocal::GeneralDynamic;
            if self.eat("(") {
                let tok = self.next();
                model = match tok.0 {
                    "localdynamic" => ThreadLocal::LocalDynamic,
                    "initialexec" => ThreadLocal::InitialExec,
                    "localexec" => ThreadLocal::LocalExec,
                    _ => return Err(self.error_at(tok, "expected a thread-local model")),
                };
                self.expect(")")?;
            }
            props.thread_local = Some(model);
        }
        self.parse_unnamed_addr(&mut props);
        Ok(props)
    }

    fn parse_unnamed_addr(&mut self, props: &mut GlobalProps) {
        if self.eat("unnamed_addr") {
            props.unnamed_addr = UnnamedAddr::Global;
        } else if self.eat("local_unnamed_addr") {
            props.unnamed_addr = UnnamedAddr::Local;
        }
    }

    fn parse_global(&mut self) -> Result<()> {
        let name_tok = self.next();
        self.expect("=")?;
        let id = self.global_def(name_tok)?;
        let mut props = self.parse_global_props()?;
        if self.peek().0 == "addrspace" {
            props.addrspace = Some(self.parse_addrspace()?);
        }
        let externally_initialized = self.eat("externally_initialized");
        let kw = self.next();
        let mut attachments = Vec::new();
        let kind = match kw.0 {
            "alias" | "ifunc" => {
                let value_ty = self.parse_type()?;
                self.expect(",")?;
                let ptr_ty = self.parse_type()?;
                let target = self.parse_const(ptr_ty)?;
                while self.eat(",") {
                    if self.eat("partition") {
                        props.partition = Some(self.expect_string()?);
                    } else {
                        attachments.push(self.parse_attachment()?);
                    }
                }
                if kw.0 == "alias" {
                    GlobalKind::Alias { value_ty, aliasee: target }
                } else {
                    GlobalKind::IFunc { value_ty, resolver: target }
                }
            }
            "global" | "constant" => {
                let ty = self.parse_type()?;
                let is_decl =
                    props.explicit_linkage && matches!(props.linkage, Linkage::External | Linkage::ExternWeak);
                let init = if is_decl { None } else { Some(self.parse_const(ty)?) };
                let mut var = GlobalVar {
                    constant: kw.0 == "constant",
                    externally_initialized,
                    ty,
                    init,
                    section: None,
                    comdat: None,
                    align: None,
                    code_model: None,
                    sanitizer: Vec::new(),
                    attrs: Vec::new(),
                };
                while self.eat(",") {
                    let tok = self.peek();
                    match tok.0 {
                        "section" => {
                            self.next();
                            var.section = Some(self.expect_string()?);
                        }
                        "partition" => {
                            self.next();
                            props.partition = Some(self.expect_string()?);
                        }
                        "comdat" => {
                            self.next();
                            var.comdat = Some(self.parse_comdat_ref()?);
                        }
                        "align" => {
                            self.next();
                            var.align = Some(self.parse_align()?);
                        }
                        "code_model" => {
                            self.next();
                            var.code_model = Some(self.expect_string()?);
                        }
                        "no_sanitize_address"
                        | "no_sanitize_hwaddress"
                        | "sanitize_address_dyninit"
                        | "sanitize_memtag" => {
                            self.next();
                            var.sanitizer.push(tok.0.to_string());
                        }
                        _ if tok.is_metadata_name() => attachments.push(self.parse_attachment()?),
                        _ => return Err(self.error_here("expected a global variable property")),
                    }
                }
                while self.peek().is_attr_group() {
                    var.attrs.push(self.parse_attr()?);
                }
                GlobalKind::Variable(var)
            }
            _ => return Err(self.error_at(kw, "expected 'global', 'constant', 'alias' or 'ifunc'")),
        };
        self.m.sort_attachments(&mut attachments);
        let g = &mut self.m.globals[id.index()];
        g.props = props;
        g.attachments = attachments;
        g.kind = kind;
        Ok(())
    }

    fn parse_calling_conv(&mut self) -> Result<CallConv> {
        let tok = self.peek();
        if tok.0 == "cc" {
            self.next();
            return Ok(CallConv::Numbered(self.expect_u32()?));
        }
        if CALLING_CONVENTIONS.contains(&tok.0) {
            self.next();
            return Ok(if tok.0 == "ccc" { CallConv::Named("ccc".into()) } else { CallConv::Named(tok.0.into()) });
        }
        Ok(CallConv::C)
    }

    // -----------------------------------------------------------------------
    // Functions

    fn parse_function(&mut self) -> Result<()> {
        let is_define = self.next().0 == "define";
        // Declarations write their attachments straight after `declare`.
        let mut attachments = Vec::new();
        while !is_define && self.peek().is_metadata_name() {
            attachments.push(self.parse_attachment()?);
        }
        let mut props = self.parse_global_props()?;
        let cconv = self.parse_calling_conv()?;
        let ret_attrs = self.parse_ret_attrs()?;
        let ret_tok = self.peek();
        let ret = self.parse_type()?;
        if matches!(self.m.types.get(ret), Type::Function { .. } | Type::Label | Type::Metadata) {
            return Err(self.error_at(ret_tok, "invalid function return type"));
        }
        let name_tok = self.next();
        if !name_tok.is_global() {
            return Err(self.error_at(name_tok, "expected a function name"));
        }
        let id = self.global_def(name_tok)?;

        let mut state = FnState {
            func: Function { cconv, ret_attrs, has_body: is_define, ..Function::default() },
            names: HashMap::new(),
            first_use: HashMap::new(),
            placed: Vec::new(),
            next_num: 0,
        };

        // Parameters.
        self.expect("(")?;
        let mut param_tys = Vec::new();
        let mut varargs = false;
        if !self.eat(")") {
            loop {
                if self.eat("...") {
                    varargs = true;
                    self.expect(")")?;
                    break;
                }
                let ty = self.parse_type()?;
                let attrs = self.parse_param_attrs()?;
                let name = if self.peek().is_local() {
                    let tok = self.next();
                    Some((tok, self.key_of(tok)?))
                } else {
                    None
                };
                let local = LocalId(state.func.locals.len() as u32);
                let local_name = match name {
                    Some((tok, Key::Num(n))) => {
                        if n != state.next_num {
                            return Err(
                                self.error_at(tok, format!("argument expected to be numbered '%{}'", state.next_num))
                            );
                        }
                        state.next_num += 1;
                        state.names.insert(Key::Num(n), local);
                        None
                    }
                    Some((tok, Key::Named(n))) => {
                        if state.names.insert(Key::Named(n.clone()), local).is_some() {
                            return Err(self.error_at(tok, "redefinition of argument"));
                        }
                        Some(n)
                    }
                    None => {
                        state.names.insert(Key::Num(state.next_num), local);
                        state.next_num += 1;
                        None
                    }
                };
                let index = state.func.params.len() as u32;
                state.func.locals.push(Local { name: local_name, ty, def: LocalDef::Param(index) });
                state.func.params.push(Param { local, attrs });
                param_tys.push(ty);
                if self.eat(")") {
                    break;
                }
                self.expect(",")?;
            }
        }
        state.func.ty = self.m.types.intern(Type::Function { ret, params: param_tys, varargs });

        // Everything after the parameter list.
        loop {
            let tok = self.peek();
            match tok.0 {
                "unnamed_addr" | "local_unnamed_addr" => self.parse_unnamed_addr(&mut props),
                "addrspace" => props.addrspace = Some(self.parse_addrspace()?),
                "section" => {
                    self.next();
                    state.func.section = Some(self.expect_string()?);
                }
                "partition" => {
                    self.next();
                    props.partition = Some(self.expect_string()?);
                }
                "comdat" => {
                    self.next();
                    state.func.comdat = Some(self.parse_comdat_ref()?);
                }
                "align" => {
                    self.next();
                    state.func.align = Some(self.parse_align()?);
                }
                "prefalign" => {
                    self.next();
                    self.expect("(")?;
                    state.func.prefalign = Some(self.parse_align()?);
                    self.expect(")")?;
                }
                "gc" => {
                    self.next();
                    state.func.gc = Some(self.expect_string()?);
                }
                "prefix" | "prologue" | "personality" => {
                    self.next();
                    let ty = self.parse_type()?;
                    let c = self.parse_const(ty)?;
                    match tok.0 {
                        "prefix" => state.func.prefix = Some(c),
                        "prologue" => state.func.prologue = Some(c),
                        _ => state.func.personality = Some(c),
                    }
                }
                "{" if is_define => break,
                _ if tok.is_metadata_name() && self.peek_at(1).0 != "=" => {
                    attachments.push(self.parse_attachment()?);
                }
                _ if self.is_attr_start(tok) => {
                    let a = self.parse_attr()?;
                    state.func.fn_attrs.push(a);
                }
                _ => break,
            }
        }

        self.m.sort_attachments(&mut attachments);
        {
            let g = &mut self.m.globals[id.index()];
            g.props = props;
            g.attachments = attachments;
        }

        if is_define {
            // Blocks that earlier `blockaddress` constants refer to.
            if let Some(refs) = self.pending_blocks.remove(&id) {
                for (key, off) in refs {
                    let label = self.m.types.label();
                    let b = new_block(&mut state, key, label);
                    let l = state.func.blocks[b.index()].local;
                    state.first_use.insert(l, off);
                }
            }
            self.f = Some(state);
            self.parse_body()?;
            state = self.f.take().unwrap();
            self.check_body(&state)?;
            let blocks = state
                .names
                .iter()
                .filter_map(|(k, &l)| match state.func.locals[l.index()].def {
                    LocalDef::Block(b) => Some((k.clone(), b)),
                    _ => None,
                })
                .collect();
            self.func_blocks.insert(id, blocks);
        }
        self.m.globals[id.index()].kind = GlobalKind::Function(Box::new(state.func));
        Ok(())
    }

    fn fs(&mut self) -> &mut FnState {
        self.f.as_mut().expect("not in a function")
    }

    fn parse_body(&mut self) -> Result<()> {
        self.expect("{")?;
        let mut current: Option<BlockId> = None;
        let mut records: Vec<DebugRecord> = Vec::new();
        loop {
            let tok = self.peek();
            if tok.0 == "}" {
                if current.is_some() {
                    return Err(self.error_at(tok, "expected a terminator instruction before '}'"));
                }
                if self.fs().func.layout.is_empty() {
                    return Err(self.error_at(tok, "function body requires at least one basic block"));
                }
                self.next();
                break;
            }
            if tok.is_eof() {
                return Err(self.error_here("expected '}'"));
            }
            if tok.is_label() {
                if current.is_some() {
                    return Err(self.error_at(tok, "expected a terminator instruction before this label"));
                }
                self.next();
                let key = self.label_key(tok)?;
                current = Some(self.define_block(tok, Some(key))?);
                continue;
            }
            let block = match current {
                Some(b) => b,
                None => {
                    let b = self.define_block(tok, None)?;
                    current = Some(b);
                    b
                }
            };
            if tok.0.starts_with("#dbg_") {
                records.push(self.parse_debug_record()?);
                continue;
            }
            if tok.0 == "uselistorder" {
                self.skip_uselistorder()?;
                continue;
            }
            let inst = self.parse_inst(std::mem::take(&mut records))?;
            if let Some(record) = self.upgrade_debug_intrinsic(tok, inst)? {
                // The call is gone; its records and the new one go to the
                // next instruction.
                let call = self.fs().func.insts.pop().unwrap();
                records = call.debug_records;
                records.push(record);
                continue;
            }
            let state = self.fs();
            state.func.blocks[block.index()].insts.push(inst);
            if state.func.insts[inst.index()].op.is_terminator() {
                current = None;
            }
        }
        if !records.is_empty() {
            return Err(self.error_here("debug records must come before an instruction"));
        }
        Ok(())
    }

    /// Starts a block, with a label or implicitly numbered.
    fn define_block(&mut self, tok: Token<'a>, key: Option<Key>) -> Result<BlockId> {
        let label = self.m.types.label();
        let state = self.f.as_mut().unwrap();
        let key = match key {
            Some(Key::Num(n)) => {
                if n != state.next_num {
                    let expected = state.next_num;
                    return Err(self.error_at(tok, format!("label expected to be numbered '{expected}'")));
                }
                state.next_num += 1;
                Key::Num(n)
            }
            Some(k) => k,
            None => {
                state.next_num += 1;
                Key::Num(state.next_num - 1)
            }
        };
        let block = match state.names.get(&key) {
            Some(&l) => match state.func.locals[l.index()].def {
                LocalDef::Block(b) if !state.placed[b.index()] => b,
                LocalDef::Block(_) => return Err(self.error_at(tok, "redefinition of block")),
                _ => return Err(self.error_at(tok, "label name is already used by a value")),
            },
            None => new_block(state, key, label),
        };
        state.placed[block.index()] = true;
        state.func.layout.push(block);
        Ok(block)
    }

    /// A reference to a block, as in `label %bb`. The `label` keyword has
    /// already been consumed, if present.
    fn block_ref(&mut self) -> Result<BlockId> {
        let tok = self.next();
        if !tok.is_local() {
            return Err(self.error_at(tok, "expected a block label such as '%bb'"));
        }
        let key = self.key_of(tok)?;
        let label = self.m.types.label();
        let off = self.offset(tok);
        let state = self.f.as_mut().unwrap();
        match state.names.get(&key) {
            Some(&l) => match state.func.locals[l.index()].def {
                LocalDef::Block(b) => Ok(b),
                _ => Err(self.error_at(tok, "expected a block, found a value")),
            },
            None => {
                let b = new_block(state, key, label);
                let l = state.func.blocks[b.index()].local;
                state.first_use.insert(l, off);
                Ok(b)
            }
        }
    }

    fn label_ref(&mut self) -> Result<BlockId> {
        self.expect("label")?;
        self.block_ref()
    }

    fn check_body(&self, state: &FnState) -> Result<()> {
        for (i, local) in state.func.locals.iter().enumerate() {
            let id = LocalId(i as u32);
            let undefined = match local.def {
                LocalDef::Forward => true,
                LocalDef::Block(b) => !state.placed[b.index()],
                _ => false,
            };
            if undefined {
                let off = state.first_use.get(&id).copied().unwrap_or(0);
                let name = state.names.iter().find(|(_, l)| **l == id).map(|(k, _)| match k {
                    Key::Named(n) => format!("%{}", crate::printer::ident(n)),
                    Key::Num(n) => format!("%{n}"),
                });
                return Err(self.error_at_offset(off, format!("use of undefined value '{}'", name.unwrap_or_default())));
            }
        }
        Ok(())
    }

    /// A use of a local value with an expected type.
    fn local_ref(&mut self, tok: Token<'a>, ty: TypeId) -> Result<LocalId> {
        let key = self.key_of(tok)?;
        let off = self.offset(tok);
        let Some(state) = self.f.as_mut() else {
            return Err(self.error_at(tok, "local values can only be used inside a function"));
        };
        if let Some(&l) = state.names.get(&key) {
            let have = state.func.locals[l.index()].ty;
            if have != ty {
                return Err(self.error_at(tok, "value used with a different type from its definition"));
            }
            return Ok(l);
        }
        let l = LocalId(state.func.locals.len() as u32);
        let name = match &key {
            Key::Named(n) => Some(n.clone()),
            Key::Num(_) => None,
        };
        state.func.locals.push(Local { name, ty, def: LocalDef::Forward });
        state.names.insert(key, l);
        state.first_use.insert(l, off);
        Ok(l)
    }

    /// Defines the result of an instruction.
    fn define_result(&mut self, tok: Option<Token<'a>>, ty: TypeId, inst: InstId) -> Result<Option<LocalId>> {
        let void = matches!(self.m.types.get(ty), Type::Void);
        let state = self.f.as_mut().unwrap();
        let Some(tok) = tok else {
            if void {
                return Ok(None);
            }
            // An unnamed result takes the next number.
            let key = Key::Num(state.next_num);
            state.next_num += 1;
            if state.names.contains_key(&key) {
                return Err(self.error_here("implicit value number is already used"));
            }
            let l = LocalId(state.func.locals.len() as u32);
            state.func.locals.push(Local { name: None, ty, def: LocalDef::Inst(inst) });
            state.names.insert(key, l);
            return Ok(Some(l));
        };
        if void {
            return Err(self.error_at(tok, "instructions returning void cannot have a name"));
        }
        let key = self.key_of(tok)?;
        let state = self.f.as_mut().unwrap();
        if let Key::Num(n) = key {
            if n < state.next_num {
                let expected = state.next_num;
                return Err(self.error_at(tok, format!("instruction expected to be numbered '%{expected}' or higher")));
            }
            state.next_num = n + 1;
        }
        if let Some(&l) = state.names.get(&key) {
            let local = &mut state.func.locals[l.index()];
            if local.def != LocalDef::Forward {
                return Err(self.error_at(tok, "redefinition of value"));
            }
            if local.ty != ty {
                let use_off = state.first_use[&l];
                return Err(self.error_at_offset(use_off, "value used with a different type from its definition"));
            }
            local.def = LocalDef::Inst(inst);
            return Ok(Some(l));
        }
        let name = match &key {
            Key::Named(n) => Some(n.clone()),
            Key::Num(_) => None,
        };
        let l = LocalId(state.func.locals.len() as u32);
        state.func.locals.push(Local { name, ty, def: LocalDef::Inst(inst) });
        state.names.insert(key, l);
        Ok(Some(l))
    }

    // -----------------------------------------------------------------------
    // Values and constants

    fn value_type(&mut self, v: Value) -> TypeId {
        match v {
            Value::Local(l) => self.f.as_ref().unwrap().func.locals[l.index()].ty,
            Value::Const(c) => self.m.consts.get(c).ty,
            Value::Metadata(_) => self.m.types.metadata(),
        }
    }

    fn parse_typed_value(&mut self) -> Result<Value> {
        let ty = self.parse_type()?;
        self.parse_value(ty)
    }

    fn parse_value(&mut self, ty: TypeId) -> Result<Value> {
        let tok = self.peek();
        if tok.is_local() {
            self.next();
            return Ok(Value::Local(self.local_ref(tok, ty)?));
        }
        if matches!(self.m.types.get(ty), Type::Metadata) {
            return Ok(Value::Metadata(self.parse_md()?));
        }
        Ok(Value::Const(self.parse_const(ty)?))
    }

    /// A typed value of a given type: `<ty> <value>` where `ty` must match.
    fn parse_typed_value_of(&mut self, expected: TypeId, what: &str) -> Result<Value> {
        let tok = self.peek();
        let ty = self.parse_type()?;
        if ty != expected {
            return Err(self.error_at(tok, format!("{what} has the wrong type")));
        }
        self.parse_value(ty)
    }

    fn intern(&mut self, ty: TypeId, kind: ConstKind) -> ConstId {
        self.m.consts.intern(Constant { ty, kind })
    }

    fn is_zero_const(&self, c: ConstId) -> bool {
        match &self.m.consts.get(c).kind {
            ConstKind::Int(v) => v.is_zero(),
            ConstKind::Float(bits) => float::is_pos_zero(*bits),
            ConstKind::Null | ConstKind::Zero => true,
            _ => false,
        }
    }

    /// Builds an aggregate, using the canonical forms: all-zero aggregates
    /// become `zeroinitializer` and arrays of `i8` become `c"..."`.
    fn aggregate(&mut self, ty: TypeId, elems: Vec<ConstId>, kind: fn(Vec<ConstId>) -> ConstKind) -> ConstId {
        if elems.iter().all(|&e| self.is_zero_const(e)) {
            return self.intern(ty, ConstKind::Zero);
        }
        if let Type::Array { elem, .. } = *self.m.types.get(ty)
            && matches!(self.m.types.get(elem), Type::Int(8))
        {
            let bytes: Option<Vec<u8>> = elems
                .iter()
                .map(|&e| match &self.m.consts.get(e).kind {
                    ConstKind::Int(v) => Some(v.low_u64() as u8),
                    _ => None,
                })
                .collect();
            if let Some(bytes) = bytes {
                return self.intern(ty, ConstKind::Bytes(bytes));
            }
        }
        self.intern(ty, kind(elems))
    }

    fn parse_const(&mut self, ty: TypeId) -> Result<ConstId> {
        let tok = self.next();
        let t = tok.0;
        let tyk = self.m.types.get(ty).clone();
        let fkind = FloatKind::of(&tyk);
        let kind = match t {
            _ if tok.is_global() => {
                if !matches!(tyk, Type::Ptr(_)) {
                    return Err(self.error_at(tok, "global values have pointer type"));
                }
                ConstKind::Global(self.global_ref(tok)?)
            }
            "true" | "false" => {
                if tyk != Type::Int(1) {
                    return Err(self.error_at(tok, "'true' and 'false' have type i1"));
                }
                ConstKind::Int(ApInt::from_u64(1, (t == "true") as u64))
            }
            "null" => {
                if !matches!(tyk, Type::Ptr(_)) {
                    return Err(self.error_at(tok, "'null' must have pointer type"));
                }
                ConstKind::Null
            }
            "none" => {
                if tyk != Type::Token {
                    return Err(self.error_at(tok, "'none' must have token type"));
                }
                ConstKind::NoneToken
            }
            "undef" => ConstKind::Undef,
            "poison" => ConstKind::Poison,
            "zeroinitializer" => ConstKind::Zero,
            _ if tok.is_int() && matches!(tyk, Type::Int(_) | Type::Byte(_)) => {
                let (Type::Int(bits) | Type::Byte(bits)) = tyk else { unreachable!() };
                let v = ApInt::parse_decimal(bits, t)
                    .ok_or_else(|| self.error_at(tok, "integer constant does not fit its type"))?;
                ConstKind::Int(v)
            }
            _ if (t.starts_with("u0x") || t.starts_with("s0x")) && matches!(tyk, Type::Int(_) | Type::Byte(_)) => {
                let (Type::Int(bits) | Type::Byte(bits)) = tyk else { unreachable!() };
                let v = ApInt::parse_hex(bits, &t[3..], t.starts_with('s'))
                    .ok_or_else(|| self.error_at(tok, "integer constant does not fit its type"))?;
                ConstKind::Int(v)
            }
            _ if tok.is_number() && fkind.is_some() => {
                let bits = float::parse(fkind.unwrap(), t)
                    .ok_or_else(|| self.error_at(tok, "invalid floating-point constant"))?;
                ConstKind::Float(bits)
            }
            _ if tok.is_number() => {
                return Err(self.error_at(tok, "numeric constant does not match its type"));
            }
            _ if t.starts_with("c\"") => {
                if t.len() < 3 || !t.ends_with('"') {
                    return Err(self.error_at(tok, "unterminated string"));
                }
                let bytes = tok.string_bytes();
                match tyk {
                    Type::Array { len, elem }
                        if *self.m.types.get(elem) == Type::Int(8) && len == bytes.len() as u64 => {}
                    _ => return Err(self.error_at(tok, "string constant does not match its array type")),
                }
                if bytes.iter().all(|&b| b == 0) { ConstKind::Zero } else { ConstKind::Bytes(bytes) }
            }
            "[" => {
                let Type::Array { len, elem } = tyk else {
                    return Err(self.error_at(tok, "array constant must have array type"));
                };
                let elems = self.parse_const_list("]", |_| Some(elem))?;
                if elems.len() as u64 != len {
                    return Err(self.error_at(tok, "array constant has the wrong number of elements"));
                }
                return Ok(self.aggregate(ty, elems, ConstKind::Array));
            }
            "{" | "<" if t == "{" || self.peek().0 == "{" => {
                let packed = t == "<";
                if packed {
                    self.next();
                }
                let (is_packed, fields) = match &tyk {
                    Type::Struct { packed, fields } => (*packed, fields.clone()),
                    Type::Named(i) => match &self.m.types.structs[*i as usize].body {
                        Some(b) => (b.packed, b.fields.clone()),
                        None => return Err(self.error_at(tok, "constant of opaque struct type")),
                    },
                    _ => return Err(self.error_at(tok, "struct constant must have struct type")),
                };
                if packed != is_packed {
                    return Err(self.error_at(tok, "struct constant packing does not match its type"));
                }
                let elems = self.parse_const_list("}", |i| fields.get(i).copied())?;
                if packed {
                    self.expect(">")?;
                }
                if elems.len() != fields.len() {
                    return Err(self.error_at(tok, "struct constant has the wrong number of fields"));
                }
                return Ok(self.aggregate(ty, elems, ConstKind::Struct));
            }
            "<" => {
                let Type::Vector { scalable: false, len, elem } = tyk else {
                    return Err(self.error_at(tok, "vector constant must have fixed vector type"));
                };
                let elems = self.parse_const_list(">", |_| Some(elem))?;
                if elems.len() as u32 != len {
                    return Err(self.error_at(tok, "vector constant has the wrong number of elements"));
                }
                if elems.iter().all(|&e| self.is_zero_const(e)) { ConstKind::Zero } else { ConstKind::Vector(elems) }
            }
            "splat" => {
                let Type::Vector { elem, .. } = tyk else {
                    return Err(self.error_at(tok, "splat must have vector type"));
                };
                self.expect("(")?;
                let v = self.parse_typed_const_of(elem)?;
                self.expect(")")?;
                ConstKind::Splat(v)
            }
            "blockaddress" => {
                self.expect("(")?;
                let ftok = self.next();
                if !ftok.is_global() {
                    return Err(self.error_at(ftok, "expected a function"));
                }
                let func = self.global_ref(ftok)?;
                self.expect(",")?;
                let btok = self.next();
                if !btok.is_local() {
                    return Err(self.error_at(btok, "expected a block"));
                }
                let block = self.blockaddress_block(func, btok)?;
                self.expect(")")?;
                ConstKind::BlockAddress { func, block }
            }
            "dso_local_equivalent" | "no_cfi" => {
                let ftok = self.next();
                if !ftok.is_global() {
                    return Err(self.error_at(ftok, "expected a function"));
                }
                let g = self.global_ref(ftok)?;
                if t == "no_cfi" { ConstKind::NoCfi(g) } else { ConstKind::DsoLocalEquivalent(g) }
            }
            "ptrauth" => {
                self.expect("(")?;
                let mut ops = Vec::new();
                loop {
                    let ty = self.parse_type()?;
                    ops.push(self.parse_const(ty)?);
                    if self.eat(")") {
                        break;
                    }
                    self.expect(",")?;
                }
                ConstKind::PtrAuth(ops)
            }
            "getelementptr" => {
                let (flags, inrange) = self.parse_gep_flags()?;
                self.expect("(")?;
                let source_ty = self.parse_type()?;
                self.expect(",")?;
                let ptr = self.parse_typed_const()?;
                let mut indices = Vec::new();
                while self.eat(",") {
                    indices.push(self.parse_typed_const()?);
                }
                self.expect(")")?;
                ConstKind::Expr(ConstExpr::Gep { flags, inrange, source_ty, ptr, indices })
            }
            "extractelement" | "insertelement" | "shufflevector" => {
                self.expect("(")?;
                let a = self.parse_typed_const()?;
                self.expect(",")?;
                let b = self.parse_typed_const()?;
                let c = if t != "extractelement" {
                    self.expect(",")?;
                    Some(self.parse_typed_const()?)
                } else {
                    None
                };
                self.expect(")")?;
                ConstKind::Expr(match t {
                    "extractelement" => ConstExpr::ExtractElement { vector: a, index: b },
                    "insertelement" => ConstExpr::InsertElement { vector: a, element: b, index: c.unwrap() },
                    _ => ConstExpr::ShuffleVector { a, b, mask: c.unwrap() },
                })
            }
            "add" | "sub" | "xor" => {
                let op = BinOp::ALL.iter().find(|(k, _)| *k == t).unwrap().1;
                let flags = self.parse_int_flags(op.allowed_flags());
                self.expect("(")?;
                let lhs = self.parse_typed_const()?;
                self.expect(",")?;
                let rhs = self.parse_typed_const()?;
                self.expect(")")?;
                ConstKind::Expr(ConstExpr::Binary { op, flags, lhs, rhs })
            }
            "trunc" | "ptrtoint" | "ptrtoaddr" | "inttoptr" | "bitcast" | "addrspacecast" => {
                let op = CastOp::ALL.iter().find(|(k, _)| *k == t).unwrap().1;
                self.expect("(")?;
                let value = self.parse_typed_const()?;
                self.expect("to")?;
                let to_tok = self.peek();
                let to = self.parse_type()?;
                if to != ty {
                    return Err(self.error_at(to_tok, "cast result type does not match"));
                }
                self.expect(")")?;
                ConstKind::Expr(ConstExpr::Cast { op, value })
            }
            _ => {
                self.pos -= 1;
                return Err(self.error_here("expected a constant"));
            }
        };
        Ok(self.intern(ty, kind))
    }

    fn parse_typed_const(&mut self) -> Result<ConstId> {
        let ty = self.parse_type()?;
        self.parse_const(ty)
    }

    fn parse_typed_const_of(&mut self, expected: TypeId) -> Result<ConstId> {
        let tok = self.peek();
        let ty = self.parse_type()?;
        if ty != expected {
            return Err(self.error_at(tok, "constant has the wrong type"));
        }
        self.parse_const(ty)
    }

    /// Typed constants up to `close`. `elem_ty(i)` gives the required type
    /// of element `i`.
    fn parse_const_list(&mut self, close: &str, elem_ty: impl Fn(usize) -> Option<TypeId>) -> Result<Vec<ConstId>> {
        let mut elems = Vec::new();
        if self.eat(close) {
            return Ok(elems);
        }
        loop {
            let tok = self.peek();
            let ty = self.parse_type()?;
            if elem_ty(elems.len()) != Some(ty) {
                return Err(self.error_at(tok, "element has the wrong type"));
            }
            elems.push(self.parse_const(ty)?);
            if self.eat(close) {
                return Ok(elems);
            }
            self.expect(",")?;
        }
    }

    /// The block named by `tok` in function `func`, for `blockaddress`.
    fn blockaddress_block(&mut self, func: GlobalId, tok: Token<'a>) -> Result<BlockId> {
        let key = self.key_of(tok)?;
        let off = self.offset(tok);
        // Already parsed.
        if let Some(blocks) = self.func_blocks.get(&func) {
            return blocks.get(&key).copied().ok_or_else(|| self.error_at(tok, "no such block in that function"));
        }
        // The function being parsed now.
        let current = self.f.is_some() && self.m.global_order.last() == Some(&func);
        if current {
            let label = self.m.types.label();
            let state = self.f.as_mut().unwrap();
            return match state.names.get(&key) {
                Some(&l) => match state.func.locals[l.index()].def {
                    LocalDef::Block(b) => Ok(b),
                    _ => Err(self.error_at(tok, "expected a block")),
                },
                None => {
                    let b = new_block(state, key, label);
                    let l = state.func.blocks[b.index()].local;
                    state.first_use.insert(l, off);
                    Ok(b)
                }
            };
        }
        // A function still to come.
        let refs = self.pending_blocks.entry(func).or_default();
        if let Some(i) = refs.iter().position(|(k, _)| *k == key) {
            return Ok(BlockId(i as u32));
        }
        refs.push((key, off));
        Ok(BlockId(refs.len() as u32 - 1))
    }

    fn parse_int_flags(&mut self, allowed: IntFlags) -> IntFlags {
        let mut flags = IntFlags::NONE;
        loop {
            let tok = self.peek().0;
            match IntFlags::NAMES.iter().find(|(k, f)| *k == tok && allowed.contains(*f)) {
                Some((_, f)) => {
                    flags = flags | *f;
                    self.next();
                }
                None => return flags,
            }
        }
    }

    fn parse_fmf(&mut self) -> FastMath {
        let mut fmf = FastMath::default();
        loop {
            let tok = self.peek().0;
            if tok == "fast" {
                fmf = FastMath::ALL;
            } else if let Some((_, bit)) = FastMath::NAMES.iter().find(|(k, _)| *k == tok) {
                fmf.0 |= bit;
            } else {
                return fmf;
            }
            self.next();
        }
    }

    fn parse_gep_flags(&mut self) -> Result<(GepFlags, Option<(i64, i64)>)> {
        let mut flags = GepFlags::default();
        let mut inrange = None;
        loop {
            match self.peek().0 {
                "inbounds" => flags.0 |= GepFlags::INBOUNDS,
                "nusw" => flags.0 |= GepFlags::NUSW,
                "nuw" => flags.0 |= GepFlags::NUW,
                "inrange" => {
                    self.next();
                    self.expect("(")?;
                    let start = self.expect_i64()?;
                    self.expect(",")?;
                    let end = self.expect_i64()?;
                    self.expect(")")?;
                    inrange = Some((start, end));
                    continue;
                }
                _ => return Ok((flags, inrange)),
            }
            self.next();
        }
    }

    // -----------------------------------------------------------------------
    // Metadata

    fn md_slot(&mut self, tok: Token<'a>) -> Result<MdId> {
        let n: u32 = tok.0[1..].parse().map_err(|_| self.error_at(tok, "bad metadata number"))?;
        if let Some(&id) = self.md_slots.get(&n) {
            return Ok(id);
        }
        let id = MdId(self.m.metadata.len() as u32);
        self.m.metadata.push(MdNode { slot: Some(n), data: Metadata::Forward });
        self.md_slots.insert(n, id);
        let off = self.offset(tok);
        self.md_first_use.insert(id, off);
        Ok(id)
    }

    fn add_md(&mut self, data: Metadata) -> MdId {
        let id = MdId(self.m.metadata.len() as u32);
        self.m.metadata.push(MdNode { slot: None, data });
        id
    }

    fn parse_metadata_def(&mut self) -> Result<()> {
        let tok = self.next();
        self.expect("=")?;
        if tok.0[1..].bytes().all(|b| b.is_ascii_digit()) {
            let id = self.md_slot(tok)?;
            if !matches!(self.m.metadata[id.index()].data, Metadata::Forward) {
                return Err(self.error_at(tok, "redefinition of metadata"));
            }
            let distinct = self.eat("distinct");
            let data = self.parse_md_node(distinct)?;
            self.m.metadata[id.index()].data = data;
            return Ok(());
        }
        // Named metadata: a list of nodes.
        let name = String::from_utf8(tok.name_bytes()).map_err(|_| self.error_at(tok, "name is not valid UTF-8"))?;
        self.expect("!")?;
        self.expect("{")?;
        let mut ops = Vec::new();
        if !self.eat("}") {
            loop {
                let t = self.next();
                if !t.is_metadata_name() || !t.0[1..].bytes().all(|b| b.is_ascii_digit()) {
                    return Err(self.error_at(t, "named metadata operands must be numbered nodes such as !0"));
                }
                ops.push(self.md_slot(t)?);
                if self.eat("}") {
                    break;
                }
                self.expect(",")?;
            }
        }
        self.m.named_metadata.push(NamedMetadata { name, ops });
        Ok(())
    }

    /// A node body: `!{...}` or `!Name(...)`.
    fn parse_md_node(&mut self, distinct: bool) -> Result<Metadata> {
        let tok = self.next();
        if tok.0 == "!" && self.peek().0 == "{" {
            self.next();
            let mut ops = Vec::new();
            if !self.eat("}") {
                loop {
                    if self.eat("null") {
                        ops.push(None);
                    } else {
                        ops.push(Some(self.parse_md()?));
                    }
                    if self.eat("}") {
                        break;
                    }
                    self.expect(",")?;
                }
            }
            return Ok(Metadata::Tuple { distinct, ops });
        }
        if tok.is_metadata_name() && self.peek().0 == "(" {
            let kind = tok.0[1..].to_string();
            self.next();
            let mut fields = Vec::new();
            if !self.eat(")") {
                loop {
                    let ftok = self.peek();
                    let name = if ftok.is_label() {
                        self.next();
                        Some(ftok.0[..ftok.0.len() - 1].to_string())
                    } else {
                        None
                    };
                    let value = self.parse_md_field_value()?;
                    fields.push(MdField { name, value });
                    if self.eat(")") {
                        break;
                    }
                    self.expect(",")?;
                }
            }
            return Ok(Metadata::Node { distinct, kind, fields });
        }
        self.pos -= 1;
        Err(self.error_here("expected a metadata node"))
    }

    fn parse_md_field_value(&mut self) -> Result<MdFieldValue> {
        let tok = self.peek();
        if self.eat("null") {
            return Ok(MdFieldValue::Null);
        }
        if tok.0.starts_with('!') || tok.0 == "distinct" {
            return Ok(MdFieldValue::Md(self.parse_md()?));
        }
        if tok.is_string() {
            return Ok(MdFieldValue::Str(self.expect_string()?));
        }
        if self.is_type_start(tok) && !tok.is_number() {
            return Ok(MdFieldValue::Value(self.parse_typed_value()?));
        }
        if tok.is_word() || tok.is_number() {
            self.next();
            let mut text = tok.0.to_string();
            while self.eat("|") {
                let t = self.next();
                text.push_str(" | ");
                text.push_str(t.0);
            }
            return Ok(MdFieldValue::Word(text));
        }
        Err(self.error_here("expected a metadata field value"))
    }

    /// A metadata operand: a node, string, reference or typed value.
    fn parse_md(&mut self) -> Result<MdId> {
        let tok = self.peek();
        if tok.0 == "!" {
            if self.peek_at(1).is_string() {
                self.next();
                let s = self.expect_string()?;
                return Ok(self.add_md(Metadata::String(s)));
            }
            let data = self.parse_md_node(false)?;
            return Ok(self.add_md(data));
        }
        if tok.0 == "distinct" {
            self.next();
            let data = self.parse_md_node(true)?;
            return Ok(self.add_md(data));
        }
        if tok.is_metadata_name() {
            if tok.0[1..].bytes().all(|b| b.is_ascii_digit()) {
                self.next();
                return self.md_slot(tok);
            }
            let data = self.parse_md_node(false)?;
            return Ok(self.add_md(data));
        }
        let v = self.parse_typed_value()?;
        Ok(self.add_md(Metadata::Value(v)))
    }

    /// `!kind !node`
    fn parse_attachment(&mut self) -> Result<Attachment> {
        let tok = self.next();
        if !tok.is_metadata_name() || tok.0[1..].bytes().all(|b| b.is_ascii_digit()) {
            return Err(self.error_at(tok, "expected a metadata kind such as !dbg"));
        }
        let kind = String::from_utf8(tok.name_bytes()).map_err(|_| self.error_at(tok, "name is not valid UTF-8"))?;
        self.m.register_md_kind(&kind);
        let node = self.parse_md()?;
        Ok(Attachment { kind, node })
    }

    /// `, !kind !node` repeated, after an instruction.
    fn parse_inst_attachments(&mut self) -> Result<Vec<Attachment>> {
        let mut list = Vec::new();
        while self.peek().0 == "," && self.peek_at(1).is_metadata_name() {
            self.next();
            list.push(self.parse_attachment()?);
        }
        Ok(list)
    }

    /// Turns a call to one of the old debug intrinsics, such as
    /// `call void @llvm.dbg.value(metadata i32 %x, metadata !1, metadata !DIExpression()), !dbg !2`,
    /// into the record that replaces it: `#dbg_value(i32 %x, !1, !DIExpression(), !2)`.
    /// The arguments move across unchanged and the `!dbg` location becomes
    /// the last one.
    fn upgrade_debug_intrinsic(&mut self, tok: Token<'a>, inst: InstId) -> Result<Option<DebugRecord>> {
        let i = &self.f.as_ref().unwrap().func.insts[inst.index()];
        let Op::Call(call) = &i.op else { return Ok(None) };
        let Value::Const(c) = call.callee else { return Ok(None) };
        let ConstKind::Global(g) = self.m.consts.get(c).kind else { return Ok(None) };
        let Some((kind, arity)) = self.m.global(g).name.as_deref().and_then(debug_intrinsic_kind) else {
            return Ok(None);
        };
        let mut args = Vec::new();
        for a in &call.args {
            match a.value {
                Value::Metadata(md) => args.push(md),
                _ => return Err(self.error_at(tok, "debug intrinsic arguments must be metadata")),
            }
        }
        if args.len() != arity {
            return Err(self.error_at(tok, format!("llvm.dbg.{kind} takes {arity} arguments")));
        }
        let Some(loc) = i.attachments.iter().find(|a| a.kind == "dbg") else {
            return Err(self.error_at(tok, "debug intrinsic call has no !dbg location"));
        };
        args.push(loc.node);
        Ok(Some(DebugRecord { kind: kind.to_string(), args }))
    }

    fn parse_debug_record(&mut self) -> Result<DebugRecord> {
        let tok = self.next();
        let kind = tok.0["#dbg_".len()..].to_string();
        self.expect("(")?;
        let mut args = Vec::new();
        if !self.eat(")") {
            loop {
                args.push(self.parse_md()?);
                if self.eat(")") {
                    break;
                }
                self.expect(",")?;
            }
        }
        Ok(DebugRecord { kind, args })
    }

    // -----------------------------------------------------------------------
    // Instructions

    fn push_inst(&mut self, op: Op, records: Vec<DebugRecord>) -> InstId {
        let state = self.fs();
        let id = InstId(state.func.insts.len() as u32);
        state.func.insts.push(Inst { result: None, op, attachments: Vec::new(), debug_records: records });
        id
    }

    fn parse_inst(&mut self, records: Vec<DebugRecord>) -> Result<InstId> {
        let result_tok = if self.peek().is_local() && self.peek_at(1).0 == "=" {
            let t = self.next();
            self.next();
            Some(t)
        } else {
            None
        };
        let op_tok = self.peek();
        let (op, ty) = self.parse_op()?;
        let mut attachments = self.parse_inst_attachments()?;
        self.m.sort_attachments(&mut attachments);
        let id = self.push_inst(op, records);
        let result = self
            .define_result(result_tok, ty, id)
            .map_err(|e| if result_tok.is_none() { self.error_at(op_tok, e.message) } else { e })?;
        let inst = &mut self.fs().func.insts[id.index()];
        inst.result = result;
        inst.attachments = attachments;
        Ok(id)
    }

    fn parse_ordering(&mut self) -> Result<Ordering> {
        let tok = self.next();
        Ordering::ALL
            .iter()
            .find(|(k, _)| *k == tok.0)
            .map(|(_, o)| *o)
            .ok_or_else(|| self.error_at(tok, "expected an atomic ordering"))
    }

    fn parse_syncscope(&mut self) -> Result<Option<Box<[u8]>>> {
        if self.eat("syncscope") {
            self.expect("(")?;
            let s = self.expect_string()?;
            self.expect(")")?;
            Ok(Some(s.into()))
        } else {
            Ok(None)
        }
    }

    /// `, align N` if present.
    fn parse_opt_align(&mut self) -> Result<Option<Align>> {
        if self.peek().0 == "," && self.peek_at(1).0 == "align" {
            self.next();
            self.next();
            Ok(Some(self.parse_inst_align()?))
        } else {
            Ok(None)
        }
    }

    /// An instruction's `align N`: unlike a global's, it may not be 0.
    fn parse_inst_align(&mut self) -> Result<Align> {
        let tok = self.peek();
        let v = self.expect_u64()?;
        Align::new(v).ok_or_else(|| self.error_at(tok, "alignment must be a power of two no larger than 2^32"))
    }

    /// Parses an instruction after any `%name =`, returning it and its result type.
    fn parse_op(&mut self) -> Result<(Op, TypeId)> {
        let tok = self.next();
        let void = self.m.types.void();
        let op = match tok.0 {
            "ret" => {
                if self.eat("void") {
                    (Op::Ret(None), void)
                } else {
                    (Op::Ret(Some(self.parse_typed_value()?)), void)
                }
            }
            "br" => {
                if self.peek().0 == "label" {
                    (Op::Br(self.label_ref()?), void)
                } else {
                    let i1 = self.m.types.int(1);
                    let cond = self.parse_typed_value_of(i1, "branch condition")?;
                    self.expect(",")?;
                    let then_dest = self.label_ref()?;
                    self.expect(",")?;
                    let else_dest = self.label_ref()?;
                    (Op::CondBr { cond, then_dest, else_dest }, void)
                }
            }
            "switch" => {
                let ty = self.parse_type()?;
                let value = self.parse_value(ty)?;
                self.expect(",")?;
                let default = self.label_ref()?;
                self.expect("[")?;
                let mut cases = Vec::new();
                while !self.eat("]") {
                    let c = self.parse_typed_const_of(ty)?;
                    self.expect(",")?;
                    let dest = self.label_ref()?;
                    cases.push((c, dest));
                }
                (Op::Switch { value, default, cases: cases.into() }, void)
            }
            "indirectbr" => {
                let addr = self.parse_typed_value()?;
                self.expect(",")?;
                self.expect("[")?;
                let mut dests = Vec::new();
                if !self.eat("]") {
                    loop {
                        dests.push(self.label_ref()?);
                        if self.eat("]") {
                            break;
                        }
                        self.expect(",")?;
                    }
                }
                (Op::IndirectBr { addr, dests: dests.into() }, void)
            }
            "invoke" => {
                let call = self.parse_call(TailKind::None)?;
                let ret = self.call_ret(&call);
                self.expect("to")?;
                let normal = self.label_ref()?;
                self.expect("unwind")?;
                let unwind = self.label_ref()?;
                (Op::Invoke { call: Box::new(call), normal, unwind }, ret)
            }
            "callbr" => {
                let call = self.parse_call(TailKind::None)?;
                let ret = self.call_ret(&call);
                self.expect("to")?;
                let default = self.label_ref()?;
                self.expect("[")?;
                let mut indirect = Vec::new();
                if !self.eat("]") {
                    loop {
                        indirect.push(self.label_ref()?);
                        if self.eat("]") {
                            break;
                        }
                        self.expect(",")?;
                    }
                }
                (Op::CallBr { call: Box::new(call), default, indirect: indirect.into() }, ret)
            }
            "resume" => (Op::Resume(self.parse_typed_value()?), void),
            "catchswitch" => {
                self.expect("within")?;
                let token = self.m.types.token();
                let parent = if self.eat("none") { None } else { Some(self.parse_value(token)?) };
                self.expect("[")?;
                let mut handlers = Vec::new();
                loop {
                    handlers.push(self.label_ref()?);
                    if self.eat("]") {
                        break;
                    }
                    self.expect(",")?;
                }
                self.expect("unwind")?;
                let unwind = if self.eat("to") {
                    self.expect("caller")?;
                    None
                } else {
                    Some(self.label_ref()?)
                };
                (Op::CatchSwitch { parent, handlers: handlers.into(), unwind }, token)
            }
            "catchret" => {
                self.expect("from")?;
                let token = self.m.types.token();
                let pad = self.parse_value(token)?;
                self.expect("to")?;
                let dest = self.label_ref()?;
                (Op::CatchRet { pad, dest }, void)
            }
            "cleanupret" => {
                self.expect("from")?;
                let token = self.m.types.token();
                let pad = self.parse_value(token)?;
                self.expect("unwind")?;
                let unwind = if self.eat("to") {
                    self.expect("caller")?;
                    None
                } else {
                    Some(self.label_ref()?)
                };
                (Op::CleanupRet { pad, unwind }, void)
            }
            "unreachable" => (Op::Unreachable, void),
            "fneg" => {
                let fmf = self.parse_fmf();
                let value = self.parse_typed_value()?;
                let ty = self.value_type(value);
                (Op::FNeg { fmf, value }, ty)
            }
            "bitextract" => {
                let ty = self.parse_type()?;
                self.expect(",")?;
                let value = self.parse_typed_value()?;
                self.expect(",")?;
                let offset = self.parse_typed_value()?;
                (Op::BitExtract { value, offset }, ty)
            }
            "bitinsert" => {
                let base = self.parse_typed_value()?;
                self.expect(",")?;
                let value = self.parse_typed_value()?;
                self.expect(",")?;
                let offset = self.parse_typed_value()?;
                let ty = self.value_type(base);
                (Op::BitInsert { base, value, offset }, ty)
            }
            "extractelement" => {
                let vector = self.parse_typed_value()?;
                self.expect(",")?;
                let index = self.parse_typed_value()?;
                let vty = self.value_type(vector);
                let ty = self.m.types.scalar(vty);
                (Op::ExtractElement { vector, index }, ty)
            }
            "insertelement" => {
                let vector = self.parse_typed_value()?;
                self.expect(",")?;
                let element = self.parse_typed_value()?;
                self.expect(",")?;
                let index = self.parse_typed_value()?;
                let ty = self.value_type(vector);
                (Op::InsertElement { vector, element, index }, ty)
            }
            "shufflevector" => {
                let a = self.parse_typed_value()?;
                self.expect(",")?;
                let b = self.parse_typed_value()?;
                self.expect(",")?;
                let mask_ty = self.parse_type()?;
                let mask = self.parse_const(mask_ty)?;
                let aty = self.value_type(a);
                let elem = self.m.types.scalar(aty);
                let ty = self.m.types.with_elem(mask_ty, elem);
                (Op::ShuffleVector { a, b, mask }, ty)
            }
            "extractvalue" => {
                let aggregate = self.parse_typed_value()?;
                let mut ty = self.value_type(aggregate);
                let mut indices = Vec::new();
                while self.peek().0 == "," && self.peek_at(1).is_int() {
                    self.next();
                    let itok = self.peek();
                    let i = self.expect_u32()?;
                    ty = self
                        .m
                        .types
                        .member(ty, i as u64)
                        .ok_or_else(|| self.error_at(itok, "invalid extractvalue index"))?;
                    indices.push(i);
                }
                if indices.is_empty() {
                    return Err(self.error_here("expected an index"));
                }
                (Op::ExtractValue { aggregate, indices: indices.into() }, ty)
            }
            "insertvalue" => {
                let aggregate = self.parse_typed_value()?;
                self.expect(",")?;
                let element = self.parse_typed_value()?;
                let ty = self.value_type(aggregate);
                let mut indices = Vec::new();
                while self.peek().0 == "," && self.peek_at(1).is_int() {
                    self.next();
                    indices.push(self.expect_u32()?);
                }
                if indices.is_empty() {
                    return Err(self.error_here("expected an index"));
                }
                (Op::InsertValue { aggregate, element, indices: indices.into() }, ty)
            }
            "alloca" => {
                let inalloca = self.eat("inalloca");
                let ty = self.parse_type()?;
                let mut count = None;
                let mut align = None;
                let mut addrspace = None;
                while self.peek().0 == "," && !self.peek_at(1).is_metadata_name() {
                    self.next();
                    match self.peek().0 {
                        "align" => {
                            self.next();
                            align = Some(self.parse_inst_align()?);
                        }
                        "addrspace" => addrspace = Some(self.parse_addrspace()?),
                        _ => count = Some(self.parse_typed_value()?),
                    }
                }
                let ptr = self.m.types.ptr(addrspace.unwrap_or(0));
                (Op::Alloca { inalloca, ty, count, align, addrspace }, ptr)
            }
            "load" => {
                let is_atomic = self.eat("atomic");
                let volatile = self.eat("volatile");
                let elementwise = self.eat("elementwise");
                let ty = self.parse_type()?;
                self.expect(",")?;
                let ptr = self.parse_typed_value()?;
                let atomic = if is_atomic {
                    let scope = self.parse_syncscope()?;
                    let ordering = self.parse_ordering()?;
                    Some(Atomic { elementwise, scope, ordering })
                } else {
                    None
                };
                let align = self.parse_opt_align()?;
                (Op::Load { volatile, atomic, ptr, align }, ty)
            }
            "store" => {
                let is_atomic = self.eat("atomic");
                let volatile = self.eat("volatile");
                let elementwise = self.eat("elementwise");
                let value = self.parse_typed_value()?;
                self.expect(",")?;
                let ptr = self.parse_typed_value()?;
                let atomic = if is_atomic {
                    let scope = self.parse_syncscope()?;
                    let ordering = self.parse_ordering()?;
                    Some(Atomic { elementwise, scope, ordering })
                } else {
                    None
                };
                let align = self.parse_opt_align()?;
                (Op::Store { volatile, atomic, value, ptr, align }, void)
            }
            "fence" => {
                let scope = self.parse_syncscope()?;
                let ordering = self.parse_ordering()?;
                (Op::Fence { scope, ordering }, void)
            }
            "cmpxchg" => {
                let weak = self.eat("weak");
                let volatile = self.eat("volatile");
                let ptr = self.parse_typed_value()?;
                self.expect(",")?;
                let cmp = self.parse_typed_value()?;
                self.expect(",")?;
                let new = self.parse_typed_value()?;
                let scope = self.parse_syncscope()?;
                let success = self.parse_ordering()?;
                let failure = self.parse_ordering()?;
                let align = self.parse_opt_align()?;
                let cty = self.value_type(cmp);
                let i1 = self.m.types.int(1);
                let ty = self.m.types.intern(Type::Struct { packed: false, fields: vec![cty, i1] });
                (Op::CmpXchg { weak, volatile, ptr, cmp, new, scope, success, failure, align }, ty)
            }
            "atomicrmw" => {
                let volatile = self.eat("volatile");
                let elementwise = self.eat("elementwise");
                let op_tok = self.next();
                let op = RmwOp::ALL
                    .iter()
                    .find(|(k, _)| *k == op_tok.0)
                    .map(|(_, o)| *o)
                    .ok_or_else(|| self.error_at(op_tok, "expected an atomicrmw operation"))?;
                let ptr = self.parse_typed_value()?;
                self.expect(",")?;
                let value = self.parse_typed_value()?;
                let scope = self.parse_syncscope()?;
                let ordering = self.parse_ordering()?;
                let align = self.parse_opt_align()?;
                let ty = self.value_type(value);
                (Op::AtomicRmw { op, volatile, elementwise, ptr, value, scope, ordering, align }, ty)
            }
            "getelementptr" => {
                let flags_tok = self.peek();
                let (flags, inrange) = self.parse_gep_flags()?;
                if inrange.is_some() {
                    return Err(self.error_at(flags_tok, "inrange is allowed only on a constant getelementptr"));
                }
                let source_ty = self.parse_type()?;
                self.expect(",")?;
                let ptr = self.parse_typed_value()?;
                let mut indices = Vec::new();
                while self.peek().0 == "," && !self.peek_at(1).is_metadata_name() {
                    self.next();
                    indices.push(self.parse_typed_value()?);
                }
                // The result is a vector of pointers if any operand is a vector.
                let pty = self.value_type(ptr);
                let mut ty = pty;
                for &i in &indices {
                    let ity = self.value_type(i);
                    if let Type::Vector { .. } = self.m.types.get(ity) {
                        let scalar = self.m.types.scalar(pty);
                        ty = self.m.types.with_elem(ity, scalar);
                    }
                }
                (Op::Gep { flags, source_ty, ptr, indices: indices.into() }, ty)
            }
            "icmp" => {
                let samesign = self.eat("samesign");
                let ptok = self.next();
                let pred = ICmpPred::ALL
                    .iter()
                    .find(|(k, _)| *k == ptok.0)
                    .map(|(_, p)| *p)
                    .ok_or_else(|| self.error_at(ptok, "expected an icmp predicate"))?;
                let lhs = self.parse_typed_value()?;
                self.expect(",")?;
                let lty = self.value_type(lhs);
                let rhs = self.parse_value(lty)?;
                let i1 = self.m.types.int(1);
                let ty = self.m.types.with_elem(lty, i1);
                (Op::ICmp { pred, samesign, lhs, rhs }, ty)
            }
            "fcmp" => {
                let fmf = self.parse_fmf();
                let ptok = self.next();
                let pred = FCmpPred::ALL
                    .iter()
                    .find(|(k, _)| *k == ptok.0)
                    .map(|(_, p)| *p)
                    .ok_or_else(|| self.error_at(ptok, "expected an fcmp predicate"))?;
                let lhs = self.parse_typed_value()?;
                self.expect(",")?;
                let lty = self.value_type(lhs);
                let rhs = self.parse_value(lty)?;
                let i1 = self.m.types.int(1);
                let ty = self.m.types.with_elem(lty, i1);
                (Op::FCmp { pred, fmf, lhs, rhs }, ty)
            }
            "phi" => {
                let fmf = self.parse_fmf();
                let ty = self.parse_type()?;
                let mut incoming = Vec::new();
                loop {
                    self.expect("[")?;
                    let v = self.parse_value(ty)?;
                    self.expect(",")?;
                    let b = self.block_ref()?;
                    self.expect("]")?;
                    incoming.push((v, b));
                    if !(self.peek().0 == "," && self.peek_at(1).0 == "[") {
                        break;
                    }
                    self.next();
                }
                (Op::Phi { fmf, incoming }, ty)
            }
            "select" => {
                let fmf = self.parse_fmf();
                let cond = self.parse_typed_value()?;
                self.expect(",")?;
                let then_value = self.parse_typed_value()?;
                self.expect(",")?;
                let tty = self.value_type(then_value);
                let else_value = self.parse_typed_value_of(tty, "select operand")?;
                (Op::Select { fmf, cond, then_value, else_value }, tty)
            }
            "freeze" => {
                let v = self.parse_typed_value()?;
                let ty = self.value_type(v);
                (Op::Freeze(v), ty)
            }
            "tail" | "musttail" | "notail" | "call" => {
                let tail = match tok.0 {
                    "tail" => TailKind::Tail,
                    "musttail" => TailKind::MustTail,
                    "notail" => TailKind::NoTail,
                    _ => TailKind::None,
                };
                if tail != TailKind::None {
                    self.expect("call")?;
                }
                let call = self.parse_call(tail)?;
                let ret = self.call_ret(&call);
                (Op::Call(Box::new(call)), ret)
            }
            "va_arg" => {
                let list = self.parse_typed_value()?;
                self.expect(",")?;
                let ty = self.parse_type()?;
                (Op::VaArg { list }, ty)
            }
            "landingpad" => {
                let ty = self.parse_type()?;
                let cleanup = self.eat("cleanup");
                let mut clauses = Vec::new();
                loop {
                    if self.eat("catch") {
                        clauses.push(Clause::Catch(self.parse_typed_value()?));
                    } else if self.eat("filter") {
                        clauses.push(Clause::Filter(self.parse_typed_value()?));
                    } else {
                        break;
                    }
                }
                if !cleanup && clauses.is_empty() {
                    return Err(self.error_here("landingpad needs 'cleanup' or at least one clause"));
                }
                (Op::LandingPad { cleanup, clauses: clauses.into() }, ty)
            }
            "catchpad" | "cleanuppad" => {
                self.expect("within")?;
                let token = self.m.types.token();
                let within =
                    if tok.0 == "cleanuppad" && self.eat("none") { None } else { Some(self.parse_value(token)?) };
                self.expect("[")?;
                let mut args = Vec::new();
                if !self.eat("]") {
                    loop {
                        args.push(self.parse_typed_value()?);
                        if self.eat("]") {
                            break;
                        }
                        self.expect(",")?;
                    }
                }
                let op = match (tok.0, within) {
                    ("catchpad", Some(within)) => Op::CatchPad { within, args: args.into() },
                    (_, within) => Op::CleanupPad { within, args: args.into() },
                };
                (op, token)
            }
            t => {
                if let Some((_, op)) = BinOp::ALL.iter().find(|(k, _)| *k == t) {
                    let op = *op;
                    let flags = self.parse_int_flags(op.allowed_flags());
                    let fmf = if op.is_float() { self.parse_fmf() } else { FastMath::default() };
                    let ty = self.parse_type()?;
                    let lhs = self.parse_value(ty)?;
                    self.expect(",")?;
                    let rhs = self.parse_value(ty)?;
                    (Op::Binary { op, flags, fmf, lhs, rhs }, ty)
                } else if let Some((_, op)) = CastOp::ALL.iter().find(|(k, _)| *k == t) {
                    let op = *op;
                    let fmf = if op.allows_fmf() { self.parse_fmf() } else { FastMath::default() };
                    let flags = self.parse_int_flags(op.allowed_flags());
                    let value = self.parse_typed_value()?;
                    self.expect("to")?;
                    let ty = self.parse_type()?;
                    (Op::Cast { op, flags, fmf, value }, ty)
                } else {
                    self.pos -= 1;
                    return Err(self.error_here("expected an instruction"));
                }
            }
        };
        Ok(op)
    }

    fn call_ret(&self, call: &Call) -> TypeId {
        match self.m.types.get(call.fn_ty) {
            Type::Function { ret, .. } => *ret,
            _ => unreachable!(),
        }
    }

    /// The part of `call`/`invoke`/`callbr` after the keyword.
    fn parse_call(&mut self, tail: TailKind) -> Result<Call> {
        let fmf = self.parse_fmf();
        let cconv = self.parse_calling_conv()?;
        let ret_attrs = self.parse_ret_attrs()?;
        let addrspace = if self.peek().0 == "addrspace" { Some(self.parse_addrspace()?) } else { None };
        let ty = self.parse_type()?;
        let explicit_fn_ty = matches!(self.m.types.get(ty), Type::Function { .. });

        // The callee. Inline asm needs the function type, which may only be
        // known after the arguments, so it is built afterwards.
        let callee_tok = self.peek();
        let mut asm = None;
        let callee = if self.eat("asm") {
            let mut a = InlineAsm {
                fn_ty: ty,
                side_effect: self.eat("sideeffect"),
                align_stack: false,
                intel_dialect: false,
                unwind: false,
                asm: Vec::new(),
                constraints: Vec::new(),
            };
            a.align_stack = self.eat("alignstack");
            a.intel_dialect = self.eat("inteldialect");
            a.unwind = self.eat("unwind");
            a.asm = self.expect_string()?;
            self.expect(",")?;
            a.constraints = self.expect_string()?;
            asm = Some(a);
            None
        } else {
            let ptr = self.m.types.ptr(addrspace.unwrap_or(0));
            Some(self.parse_value(ptr)?)
        };

        // Arguments.
        self.expect("(")?;
        let mut args = Vec::new();
        if !self.eat(")") {
            loop {
                let aty = self.parse_type()?;
                let attrs = if matches!(self.m.types.get(aty), Type::Metadata) {
                    Vec::new()
                } else {
                    self.parse_param_attrs()?
                };
                let value = self.parse_value(aty)?;
                args.push(CallArg { value, attrs });
                if self.eat(")") {
                    break;
                }
                self.expect(",")?;
            }
        }

        let fn_ty = if explicit_fn_ty {
            ty
        } else {
            let params = args.iter().map(|a| self.value_type(a.value)).collect();
            self.m.types.intern(Type::Function { ret: ty, params, varargs: false })
        };
        if let Type::Function { params, varargs, .. } = self.m.types.get(fn_ty).clone() {
            let count_ok = if varargs { args.len() >= params.len() } else { args.len() == params.len() };
            if !count_ok {
                return Err(self.error_at(callee_tok, "wrong number of arguments for the function type"));
            }
            for (a, p) in args.iter().zip(&params) {
                if self.value_type(a.value) != *p {
                    return Err(self.error_at(callee_tok, "argument type does not match the function type"));
                }
            }
        }
        let callee = match (callee, asm) {
            (Some(c), _) => c,
            (None, Some(mut a)) => {
                a.fn_ty = fn_ty;
                let ptr = self.m.types.ptr(0);
                Value::Const(self.intern(ptr, ConstKind::InlineAsm(a)))
            }
            (None, None) => unreachable!(),
        };

        let fn_attrs = self.parse_call_fn_attrs()?;
        let mut bundles = Vec::new();
        if self.peek().0 == "[" && self.peek_at(1).is_string() {
            self.next();
            loop {
                let tag = self.expect_string()?;
                self.expect("(")?;
                let mut inputs = Vec::new();
                if !self.eat(")") {
                    loop {
                        inputs.push(self.parse_typed_value()?);
                        if self.eat(")") {
                            break;
                        }
                        self.expect(",")?;
                    }
                }
                bundles.push(Bundle { tag, inputs });
                if self.eat("]") {
                    break;
                }
                self.expect(",")?;
            }
        }
        Ok(Call { tail, fmf, cconv, ret_attrs, addrspace, fn_ty, callee, args, fn_attrs, bundles })
    }
}

/// Creates a block with a fresh local, not yet placed in the layout.
fn new_block(state: &mut FnState, key: Key, label: TypeId) -> BlockId {
    let b = BlockId(state.func.blocks.len() as u32);
    let l = LocalId(state.func.locals.len() as u32);
    let name = match &key {
        Key::Named(n) => Some(n.clone()),
        Key::Num(_) => None,
    };
    state.func.locals.push(Local { name, ty: label, def: LocalDef::Block(b) });
    state.func.blocks.push(Block { local: l, insts: Vec::new() });
    state.placed.push(false);
    state.names.insert(key, l);
    b
}

/// For the old debug intrinsics, the record kind that replaces them and
/// the number of arguments they take.
fn debug_intrinsic_kind(name: &str) -> Option<(&'static str, usize)> {
    match name {
        "llvm.dbg.value" => Some(("value", 3)),
        "llvm.dbg.declare" => Some(("declare", 3)),
        "llvm.dbg.assign" => Some(("assign", 6)),
        "llvm.dbg.label" => Some(("label", 1)),
        _ => None,
    }
}
