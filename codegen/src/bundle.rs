//! Bundles: instructions issued together, one to each unit, in a word
//! with a header ([vliw.md](../../docs/design/vliw.md), section 1).
//!
//! `!target.bundles` lists the formats, each as property pairs:
//!
//! ```llvm
//! !1300 = !{!"name", !"b", !"section", !".text.b", !"bytes", !"16",
//!           !"field", !"len", !"0:4", !"field", !"port", !"4:4 8:4",
//!           !"slot", !"16:16", !"A", !"slot", !"32:16", !"B,C",
//!           !"prefix", !"rep {len}", !"rep=1", !"end", !"stop=1",
//!           !"lanes", !"::"}
//! ```
//!
//! | Property | Meaning |
//! | --- | --- |
//! | `section` | the sections whose code is in bundles of this format, by name (`.text.b .b.*`: `*` ends a prefix) |
//! | `bytes` | a bundle's size, header and slots and the zeros after them |
//! | `field` | a header field, `lsb:width`; several (`4:4 8:4`) make a family, `port[0]` and `port[1]` |
//! | `slot` | a slot, `lsb:width`, then the units whose entries go in it |
//! | `prefix` | a way of writing header fields before a bundle's `{`: a template of fields, and fields it sets |
//! | `end` | fields the last bundle of a section sets |
//! | `lanes` | the text between an item's lanes: `a :: b` is `a` in lane 0 and `b` in lane 1 |
//!
//! Bits are numbered from bit 0 of byte 0. An entry goes in a bundle when
//! it has `"unit"` (a list); `"lane"` says which lane it is written in (any
//! when absent); and `"header"` sets header fields from its operands:
//! `port[s>>4]=s&15; rot[s>>4]=1 if s>>8`, each assignment to a field
//! having to agree with the others in the bundle. Fields nothing sets are
//! zero.

use std::collections::HashMap;

use crate::enc::{self, Env, Expr};

/// A header field.
#[derive(Clone, Debug)]
pub struct Field {
    pub name: String,
    pub lsb: u32,
    pub width: u32,
}

#[derive(Clone, Debug)]
pub struct SlotDef {
    pub lsb: u32,
    pub width: u32,
    pub units: Vec<String>,
}

/// A way of writing a bundle's header fields before it: literal text and
/// fields' values, and the fields it sets besides.
#[derive(Clone, Debug)]
pub struct Prefix {
    pub parts: Vec<PrefixPart>,
    pub sets: Vec<(String, i64)>,
}

#[derive(Clone, Debug)]
pub enum PrefixPart {
    Lit(String),
    Field(String),
}

#[derive(Clone, Debug)]
pub struct Format {
    pub name: String,
    pub section: String,
    pub bytes: u32,
    pub fields: Vec<Field>,
    pub slots: Vec<SlotDef>,
    pub prefixes: Vec<Prefix>,
    pub end: Vec<(String, i64)>,
    pub lanes: Option<String>,
}

/// Where an entry goes in bundles (`"unit"`, `"lane"`, `"header"`).
#[derive(Clone, Debug)]
pub struct SlotUse {
    pub units: Vec<String>,
    pub lane: Option<u8>,
    pub header: Vec<Assign>,
}

impl SlotUse {
    /// The operands its header fields are worked out from: what the slot
    /// alone does not hold all of, and the bundle's header does.
    pub fn header_names(&self) -> Vec<String> {
        let mut names = Vec::new();
        for a in &self.header {
            for e in a.index.iter().chain([&a.value]).chain(a.guard.iter()) {
                enc::names_in(e, &mut names);
            }
        }
        names
    }
}

/// `field[index]=value if guard`.
#[derive(Clone, Debug)]
pub struct Assign {
    pub field: String,
    pub index: Option<Expr>,
    pub value: Expr,
    pub guard: Option<Expr>,
}

impl Assign {
    /// The field and its value, `Some(None)` if the guard does not hold,
    /// or `None` if the operands do not say.
    pub fn eval(&self, env: &Env) -> Option<Option<(String, i64)>> {
        if let Some(g) = &self.guard
            && enc::eval(g, env)? == 0
        {
            return Some(None);
        }
        let name = match &self.index {
            Some(e) => format!("{}[{}]", self.field, enc::eval(e, env)?),
            None => self.field.clone(),
        };
        Some(Some((name, enc::eval(&self.value, env)?)))
    }
}

fn lsb_width(s: &str) -> Result<(u32, u32), String> {
    let (a, b) = s.split_once(':').ok_or_else(|| format!("{s}: not lsb:width"))?;
    Ok((
        a.trim().parse().map_err(|_| format!("{s}: not lsb:width"))?,
        b.trim().parse().map_err(|_| format!("{s}: not lsb:width"))?,
    ))
}

/// `a=1 b=2`.
fn sets(s: &str) -> Result<Vec<(String, i64)>, String> {
    s.split_whitespace()
        .map(|kv| {
            let (k, v) = kv.split_once('=').ok_or_else(|| format!("{kv}: not field=value"))?;
            Ok((k.to_string(), v.parse().map_err(|_| format!("{kv}: not field=value"))?))
        })
        .collect()
}

fn prefix_parts(t: &str) -> Vec<PrefixPart> {
    let mut out = Vec::new();
    let mut rest = t;
    while let Some(open) = rest.find('{') {
        if open > 0 {
            out.push(PrefixPart::Lit(rest[..open].to_string()));
        }
        let close = rest[open..].find('}').map_or(rest.len(), |c| open + c);
        out.push(PrefixPart::Field(rest[open + 1..close].to_string()));
        rest = &rest[(close + 1).min(rest.len())..];
    }
    if !rest.is_empty() {
        out.push(PrefixPart::Lit(rest.to_string()));
    }
    out
}

impl Format {
    /// The formats of `!target.bundles`.
    pub fn read(m: &epitome_ir::module::Module) -> Result<Vec<Format>, String> {
        let mut out = Vec::new();
        for t in crate::machine::tuples(m, "target.bundles") {
            let mut f = Format {
                name: String::new(),
                section: String::new(),
                bytes: 0,
                fields: Vec::new(),
                slots: Vec::new(),
                prefixes: Vec::new(),
                end: Vec::new(),
                lanes: None,
            };
            let mut k = 0;
            let arg =
                |k: usize| t.get(k).cloned().ok_or_else(|| format!("!target.bundles: {} wants a value", t[k - 1]));
            while k < t.len() {
                match t[k].as_str() {
                    "name" => f.name = arg(k + 1)?,
                    "section" => f.section = arg(k + 1)?,
                    "bytes" => f.bytes = arg(k + 1)?.parse().map_err(|_| "!target.bundles: bytes: a number")?,
                    "lanes" => f.lanes = Some(arg(k + 1)?),
                    "end" => f.end = sets(&arg(k + 1)?)?,
                    "field" => {
                        let name = arg(k + 1)?;
                        let places: Vec<&str> =
                            t.get(k + 2).map(|s| s.split_whitespace().collect()).unwrap_or_default();
                        let many = places.len() > 1;
                        for (i, p) in places.iter().enumerate() {
                            let (lsb, width) = lsb_width(p)?;
                            let name = if many { format!("{name}[{i}]") } else { name.clone() };
                            f.fields.push(Field { name, lsb, width });
                        }
                        k += 3;
                        continue;
                    }
                    "slot" => {
                        let (lsb, width) = lsb_width(&arg(k + 1)?)?;
                        let units = arg(k + 2)?.split(',').map(|u| u.trim().to_string()).collect();
                        f.slots.push(SlotDef { lsb, width, units });
                        k += 3;
                        continue;
                    }
                    "prefix" => {
                        f.prefixes.push(Prefix { parts: prefix_parts(&arg(k + 1)?), sets: sets(&arg(k + 2)?)? });
                        k += 3;
                        continue;
                    }
                    o => return Err(format!("!target.bundles: unknown property {o}")),
                }
                k += 2;
            }
            if f.name.is_empty() || f.bytes == 0 {
                return Err("!target.bundles: a format needs a name and bytes".into());
            }
            for s in &f.slots {
                if s.lsb + s.width > f.bytes * 8 || s.width % 8 != 0 {
                    return Err(format!("bundle {}: a slot at {} is not whole bytes inside the bundle", f.name, s.lsb));
                }
            }
            for (name, _) in f.prefixes.iter().flat_map(|p| &p.sets).chain(&f.end) {
                if f.field(name).is_none() {
                    return Err(format!("bundle {}: no header field {name}", f.name));
                }
            }
            out.push(f);
        }
        Ok(out)
    }

    /// Whether section `name` holds bundles of this format: `section` is
    /// a list of names, each perhaps ending `*` for any name it starts.
    pub fn holds(&self, name: &str) -> bool {
        self.section.split_whitespace().any(|p| match p.strip_suffix('*') {
            Some(pre) => name.starts_with(pre),
            None => name == p,
        })
    }

    pub fn field(&self, name: &str) -> Option<&Field> {
        self.fields.iter().find(|f| f.name == name)
    }

    /// The header fields `text` (before a bundle's opening brace) gives,
    /// by the first prefix it is written as.
    pub fn read_prefix(&self, text: &str) -> Option<Vec<(String, i64)>> {
        'prefix: for p in &self.prefixes {
            let mut out = p.sets.clone();
            let mut rest = text;
            for (i, part) in p.parts.iter().enumerate() {
                match part {
                    PrefixPart::Lit(l) => match rest.strip_prefix(l.as_str()) {
                        Some(r) => rest = r,
                        None => continue 'prefix,
                    },
                    PrefixPart::Field(name) => {
                        // To the next literal text, or the end.
                        let end = match p.parts.get(i + 1) {
                            Some(PrefixPart::Lit(l)) => match rest.find(l.as_str()) {
                                Some(e) => e,
                                None => continue 'prefix,
                            },
                            _ => rest.len(),
                        };
                        let Ok(v) = rest[..end].trim().parse::<i64>() else { continue 'prefix };
                        out.push((name.clone(), v));
                        rest = &rest[end..];
                    }
                }
            }
            if rest.trim().is_empty() {
                return Some(out);
            }
        }
        None
    }

    /// The prefix that writes `header`'s fields (other than those `taken`
    /// explains), as text, or "" when none is needed. `None` when no
    /// prefix explains the fields that need one.
    pub fn write_prefix(&self, header: &[u8], taken: &HashMap<String, i64>) -> Option<String> {
        let need: Vec<&Field> =
            self.fields.iter().filter(|f| get(header, f.lsb, f.width) != 0 && !taken.contains_key(&f.name)).collect();
        if need.is_empty() {
            return Some(String::new());
        }
        'prefix: for p in &self.prefixes {
            let mut text = String::new();
            let mut covered: Vec<&str> = p.sets.iter().map(|(n, _)| n.as_str()).collect();
            for (n, v) in &p.sets {
                let f = self.field(n)?;
                if get(header, f.lsb, f.width) as i64 != *v {
                    continue 'prefix;
                }
            }
            for part in &p.parts {
                match part {
                    PrefixPart::Lit(l) => text.push_str(l),
                    PrefixPart::Field(n) => {
                        let f = self.field(n)?;
                        text.push_str(&get(header, f.lsb, f.width).to_string());
                        covered.push(n);
                    }
                }
            }
            if need.iter().all(|f| covered.contains(&f.name.as_str())) {
                return Some(text);
            }
        }
        None
    }
}

/// Parses `"header"`: assignments separated by `;`.
pub fn parse_assigns(text: &str) -> Result<Vec<Assign>, String> {
    let mut out = Vec::new();
    for a in text.split(';').map(str::trim).filter(|a| !a.is_empty()) {
        let (lhs, rhs) = a.split_once('=').ok_or_else(|| format!("\"header\": {a}: not field=value"))?;
        let (value, guard) = match rhs.split_once(" if ") {
            Some((v, g)) => (v, Some(enc::parse_expr(g.trim())?)),
            None => (rhs, None),
        };
        let lhs = lhs.trim();
        let (field, index) = match lhs.split_once('[') {
            Some((f, i)) => (f.to_string(), Some(enc::parse_expr(i.trim_end_matches(']'))?)),
            None => (lhs.to_string(), None),
        };
        out.push(Assign { field, index, value: enc::parse_expr(value.trim())?, guard });
    }
    Ok(out)
}

/// Bits `lsb` up, `width` of them, of `bytes`.
pub fn get(bytes: &[u8], lsb: u32, width: u32) -> u64 {
    let mut v = 0u64;
    for i in 0..width.min(64) {
        let b = lsb + i;
        if (bytes[(b / 8) as usize] >> (b % 8)) & 1 != 0 {
            v |= 1 << i;
        }
    }
    v
}

/// Puts `v` in bits `lsb` up, `width` of them, of `bytes`.
pub fn put(bytes: &mut [u8], lsb: u32, width: u32, v: u64) {
    for i in 0..width.min(64) {
        let b = lsb + i;
        let m = 1u8 << (b % 8);
        if (v >> i) & 1 != 0 {
            bytes[(b / 8) as usize] |= m;
        } else {
            bytes[(b / 8) as usize] &= !m;
        }
    }
}

/// A bundle being put together: its bytes, and the header fields set so far.
pub struct Builder<'f> {
    pub format: &'f Format,
    pub bytes: Vec<u8>,
    pub header: HashMap<String, i64>,
    pub used: Vec<bool>,
}

impl<'f> Builder<'f> {
    pub fn new(format: &'f Format) -> Self {
        Builder {
            format,
            bytes: vec![0; format.bytes as usize],
            header: HashMap::new(),
            used: vec![false; format.slots.len()],
        }
    }

    /// Sets a header field, which must agree with what set it before.
    pub fn set(&mut self, name: &str, v: i64) -> Result<(), String> {
        let f = self.format.field(name).ok_or_else(|| format!("no header field {name}"))?;
        if f.width < 64 && (v < 0 || v >= 1 << f.width) {
            return Err(format!("{v} does not fit header field {name}"));
        }
        match self.header.insert(name.to_string(), v) {
            Some(old) if old != v => Err(format!("header field {name} is both {old} and {v}")),
            _ => Ok(()),
        }
    }

    /// The first free slot taking one of `units`, of `len` bytes.
    pub fn free_slot(&self, units: &[String], len: usize) -> Option<usize> {
        (0..self.format.slots.len()).find(|&k| {
            let s = &self.format.slots[k];
            !self.used[k] && s.width as usize == len * 8 && s.units.iter().any(|u| units.contains(u))
        })
    }

    /// Puts an entry's bytes in slot `k`, and sets the header fields its
    /// operands set.
    pub fn place(&mut self, k: usize, word: &[u8], header: &[Assign], env: &Env) -> Result<(), String> {
        let mut sets = Vec::new();
        for a in header {
            match a.eval(env) {
                Some(Some(kv)) => sets.push(kv),
                Some(None) => {}
                None => return Err("its header fields cannot be worked out from its operands".into()),
            }
        }
        // All or nothing: a conflict leaves the bundle as it was.
        let saved = self.header.clone();
        for (n, v) in sets {
            if let Err(e) = self.set(&n, v) {
                self.header = saved;
                return Err(e);
            }
        }
        let s = &self.format.slots[k];
        put(&mut self.bytes, s.lsb, s.width, enc::word(word));
        self.used[k] = true;
        Ok(())
    }

    /// The bundle's bytes, its header written.
    pub fn finish(mut self) -> Vec<u8> {
        for (n, v) in &self.header {
            let f = self.format.field(n).expect("set checks the field");
            put(&mut self.bytes, f.lsb, f.width, *v as u64);
        }
        self.bytes
    }
}
