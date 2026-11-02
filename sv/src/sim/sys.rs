//! System tasks and functions (clauses 20, 21), formatted output, and the
//! built-in methods of strings, enums, arrays and events.

use super::*;
use crate::design::ops;

impl Sim<'_, '_, '_> {
    pub(super) fn syscall(&mut self, cx: &Cx, s: Sym, args: &[ExprId], ty: TypeId, x: ExprId) -> Const {
        let name = self.d.name(s);
        let int = |v: i64| Const::Bits(Bits::from_i64(32, true, v));
        match name {
            "$display" | "$displayb" | "$displayo" | "$displayh" | "$write" | "$writeb" | "$writeo" | "$writeh" => {
                let radix = radix_of(name);
                let mut text = self.format_args(cx, args, radix);
                if name.starts_with("$display") {
                    text.push('\n');
                }
                let _ = self.out.write_all(text.as_bytes());
                Const::Null
            }
            "$fdisplay" | "$fdisplayb" | "$fdisplayh" | "$fdisplayo" | "$fwrite" | "$fwriteb" | "$fwriteh" | "$fwriteo" => {
                let fd = args.first().map_or(1, |&a| self.eval(cx, a).to_i64().unwrap_or(1)) as u32;
                let mut text = self.format_args(cx, &args[1.min(args.len())..], radix_of(name));
                if name.starts_with("$fdisplay") {
                    text.push('\n');
                }
                self.fwrite(fd, &text);
                Const::Null
            }
            "$strobe" | "$strobeb" | "$strobeo" | "$strobeh" => {
                self.strobes.push((cx.clone(), args.to_vec(), radix_of(name)));
                Const::Null
            }
            "$monitor" | "$monitorb" | "$monitoro" | "$monitorh" => {
                self.monitors.clear();
                self.monitors.push(Monitor { cx: cx.clone(), args: args.to_vec(), last: None, radix: radix_of(name) });
                Const::Null
            }
            "$monitoron" => {
                self.monitor_on = true;
                Const::Null
            }
            "$monitoroff" => {
                self.monitor_on = false;
                Const::Null
            }
            "$finish" | "$stop" | "$exit" => {
                self.finishing = true;
                self.outcome.finished = true;
                Const::Null
            }
            "$fatal" | "$error" | "$warning" | "$info" => {
                // `$fatal`'s first argument is the finish number.
                let msg_args = if name == "$fatal" && !args.is_empty() && !self.is_string_literal(args[0]) { &args[1..] } else { args };
                let msg = self.format_args(cx, msg_args, b'd');
                let label = match name {
                    "$fatal" => "Fatal",
                    "$error" => "Error",
                    "$warning" => "Warning",
                    _ => "Info",
                };
                let _ = writeln!(self.out, "{label}: {msg}");
                match name {
                    "$fatal" => {
                        self.outcome.failed = true;
                        self.outcome.finished = true;
                        self.finishing = true;
                    }
                    "$error" => self.outcome.errors += 1,
                    _ => {}
                }
                Const::Null
            }
            "$time" => Const::Bits(Bits::from_u64(64, false, self.time_in(cx.inst).round() as u64)),
            "$stime" => Const::Bits(Bits::from_u64(32, false, self.time_in(cx.inst).round() as u64)),
            "$realtime" => Const::Real(self.time_in(cx.inst)),
            "$random" => {
                let v = self.rand32();
                Const::Bits(Bits::from_i64(32, true, v as i32 as i64))
            }
            "$urandom" => Const::Bits(Bits::from_u64(32, false, self.rand32() as u64)),
            "$urandom_range" => {
                let a = args.first().map_or(0, |&a| self.eval(cx, a).to_i64().unwrap_or(0));
                let b = args.get(1).map_or(0, |&a| self.eval(cx, a).to_i64().unwrap_or(0));
                let (lo, hi) = (a.min(b), a.max(b));
                let r = self.rand32() as i64;
                Const::Bits(Bits::from_u64(32, false, (lo + r % (hi - lo + 1)) as u64))
            }
            "$signed" | "$unsigned" => match self.eval(cx, args[0]) {
                Const::Bits(b) => Const::Bits(b.with_signed(name == "$signed")),
                v => v,
            },
            "$sformatf" | "$psprintf" => Const::Str(self.format_args(cx, args, b'd').into_bytes()),
            "$sformat" | "$swrite" | "$swriteb" | "$swriteo" | "$swriteh" => {
                let text = self.format_args(cx, &args[1.min(args.len())..], radix_of(name));
                if let Some(&dst) = args.first() {
                    let t = self.target(cx, dst);
                    self.write_cx(cx, &t, Const::Str(text.into_bytes()), false);
                }
                Const::Null
            }
            "$cast" => {
                let ok = self.cast(cx, args);
                Const::Bits(Bits::from_i64(32, true, ok as i64))
            }
            "$typename" => {
                let t = args.first().map_or(Types::UNKNOWN, |&a| match self.d.expr(a).kind {
                    ExprKind::TypeRef(t) => t,
                    _ => self.d.expr(a).ty,
                });
                Const::Str(self.d.types.describe(t, &self.d.syms).into_bytes())
            }
            "$test$plusargs" => {
                let v = self.eval(cx, args[0]);
                let key = String::from_utf8_lossy(&str_of(&v)).to_string();
                let hit = self.opts.plusargs.iter().any(|p| p.trim_start_matches('+').starts_with(&key));
                int(hit as i64)
            }
            "$value$plusargs" => {
                let v = self.eval(cx, args[0]);
                let spec = String::from_utf8_lossy(&str_of(&v)).to_string();
                let (key, conv) = spec.split_once('%').map_or((spec.as_str(), "d"), |(k, c)| (k, c));
                let found = self.opts.plusargs.iter().find_map(|p| p.trim_start_matches('+').strip_prefix(key).map(|r| r.to_string()));
                match (found, args.get(1)) {
                    (Some(val), Some(&dst)) => {
                        let parsed = match conv.chars().next() {
                            Some('s') => Const::Str(val.into_bytes()),
                            Some('h' | 'x') => Const::Bits(Bits::from_u64(64, false, u64::from_str_radix(&val, 16).unwrap_or(0))),
                            Some('b') => Const::Bits(Bits::from_u64(64, false, u64::from_str_radix(&val, 2).unwrap_or(0))),
                            Some('f' | 'e' | 'g') => Const::Real(val.parse().unwrap_or(0.0)),
                            _ => Const::Bits(Bits::from_i64(64, true, val.parse().unwrap_or(0))),
                        };
                        let t = self.target(cx, dst);
                        self.write_cx(cx, &t, parsed, false);
                        int(1)
                    }
                    _ => int(0),
                }
            }
            "$fopen" => {
                let v = self.eval(cx, args[0]);
                let path = String::from_utf8_lossy(&str_of(&v)).to_string();
                let mode = args.get(1).map(|&a| String::from_utf8_lossy(&str_of(&self.eval(cx, a))).to_string());
                let fd = self.next_fd;
                self.next_fd += 1;
                let reading = mode.as_deref().is_some_and(|m| m.starts_with('r'));
                if reading {
                    match std::fs::read(&path) {
                        Ok(b) => {
                            self.file_text.insert(fd, b);
                        }
                        Err(_) => return int(0),
                    }
                } else {
                    let append = mode.as_deref().is_some_and(|m| m.starts_with('a'));
                    if !append {
                        let _ = std::fs::write(&path, b"");
                    }
                }
                self.files.insert(fd, path);
                // A descriptor without a mode is a multichannel one (21.3.1).
                int(if mode.is_none() { 1 << (fd % 30 + 1) } else { (fd | 0x8000_0000) as i32 as i64 })
            }
            "$fclose" => Const::Null,
            "$fflush" => {
                let _ = self.out.flush();
                Const::Null
            }
            "$feof" => {
                let fd = args.first().map_or(0, |&a| self.eval(cx, a).to_i64().unwrap_or(0)) as u32 & 0x7fff_ffff;
                int(self.file_text.get(&fd).is_none_or(|t| t.is_empty()) as i64)
            }
            "$fgets" => {
                let fd = args.get(1).map_or(0, |&a| self.eval(cx, a).to_i64().unwrap_or(0)) as u32 & 0x7fff_ffff;
                let line = self.file_text.get_mut(&fd).map(|t| {
                    let end = t.iter().position(|&c| c == b'\n').map_or(t.len(), |p| p + 1);
                    t.drain(..end).collect::<Vec<u8>>()
                });
                match (line, args.first()) {
                    (Some(l), Some(&dst)) if !l.is_empty() => {
                        let n = l.len();
                        let t = self.target(cx, dst);
                        self.write_cx(cx, &t, Const::Str(l), false);
                        int(n as i64)
                    }
                    _ => int(0),
                }
            }
            "$readmemh" | "$readmemb" => {
                self.readmem(cx, args, name == "$readmemh");
                Const::Null
            }
            "$timeformat" => {
                let v: Vec<Const> = args.iter().map(|&a| self.eval(cx, a)).collect();
                if let Some(u) = v.first().and_then(|c| c.to_i64()) {
                    self.timeformat.0 = u as i8;
                }
                if let Some(p) = v.get(1).and_then(|c| c.to_i64()) {
                    self.timeformat.1 = p.max(0) as usize;
                }
                if let Some(s) = v.get(2) {
                    self.timeformat.2 = String::from_utf8_lossy(&str_of(s)).to_string();
                }
                if let Some(w) = v.get(3).and_then(|c| c.to_i64()) {
                    self.timeformat.3 = w.max(0) as usize;
                }
                Const::Null
            }
            "$printtimescale" => {
                let inst = match args.first().and_then(|&a| match &self.d.expr(a).kind {
                    ExprKind::Hier { .. } => self.hops(cx.inst, a),
                    _ => None,
                }) {
                    Some(i) => i,
                    None => cx.inst,
                };
                let (u, p) = if inst == u32::MAX {
                    self.d.unit_time
                } else {
                    let sp = self.d.spec(self.insts[inst as usize].spec);
                    (sp.time_unit, sp.time_precision)
                };
                let _ = writeln!(self.out, "Time scale of ({}) is {} / {}", self.inst_path(inst), unit_text(u), unit_text(p));
                Const::Null
            }
            "$timeunit" | "$timeprecision" => {
                let inst = match args.first().and_then(|&a| self.hops(cx.inst, a)) {
                    Some(i) => i,
                    None => cx.inst,
                };
                let (u, p) = if inst == u32::MAX {
                    self.d.unit_time
                } else {
                    let sp = self.d.spec(self.insts[inst as usize].spec);
                    (sp.time_unit, sp.time_precision)
                };
                int(if name == "$timeunit" { u as i64 } else { p as i64 })
            }
            "$dumpfile" | "$dumpvars" | "$dumpon" | "$dumpoff" | "$dumpall" | "$dumpflush" | "$dumplimit" | "$dumpports"
            | "$asserton" | "$assertoff" | "$assertkill" | "$assertcontrol" | "$assertpasson" | "$assertpassoff"
            | "$assertfailon" | "$assertfailoff" | "$assertnonvacuouson" | "$assertvacuousoff" | "$srandom" => Const::Null,
            "$sampled" | "$past" => args.first().map_or(Const::Null, |&a| self.eval(cx, a)),
            "$system" => int(0),
            "$bits" => {
                // A dynamic value's size is known only now.
                match args.first().map(|&a| (a, self.d.expr(a).kind.clone())) {
                    Some((_, ExprKind::TypeRef(t))) => int(self.d.types.width(t) as i64),
                    Some((a, _)) => {
                        let at = self.d.expr(a).ty;
                        let w = self.d.types.width(at);
                        if w > 0 {
                            int(w as i64)
                        } else {
                            let v = self.eval(cx, a);
                            int(bits_of_value(&self.d.types, &v, at) as i64)
                        }
                    }
                    None => int(0),
                }
            }
            "$size" | "$left" | "$right" | "$low" | "$high" | "$increment" | "$dimensions" | "$unpacked_dimensions" => {
                let Some(&a) = args.first() else { return int(0) };
                let at = match self.d.expr(a).kind {
                    ExprKind::TypeRef(t) => t,
                    _ => self.d.expr(a).ty,
                };
                let dim = args.get(1).map_or(1, |&b| self.eval(cx, b).to_i64().unwrap_or(1));
                // A dynamic outer dimension: its current size.
                if dim == 1
                    && matches!(self.d.types.get(at), Type::Unpacked { dim: UDim::Dynamic | UDim::Queue(_) | UDim::Assoc(_) | UDim::Wildcard, .. })
                {
                    let v = self.eval(cx, a);
                    let n = match &v {
                        Const::Aggregate(i) => {
                            if matches!(self.d.types.get(at), Type::Unpacked { dim: UDim::Assoc(_) | UDim::Wildcard, .. }) {
                                i.len() / 2
                            } else {
                                i.len()
                            }
                        }
                        _ => 0,
                    } as i64;
                    return int(match name {
                        "$size" => n,
                        "$left" | "$low" => 0,
                        "$right" | "$high" => n - 1,
                        "$increment" => -1,
                        _ => 1,
                    });
                }
                match name {
                    "$dimensions" | "$unpacked_dimensions" => {
                        let (p, u) = ops::count_dims(&self.d.types, at);
                        int(if name == "$dimensions" { (p + u) as i64 } else { u as i64 })
                    }
                    _ => match ops::nth_range(&self.d.types, at, dim) {
                        Some((l, r)) => int(match name {
                            "$size" => (l - r).abs() + 1,
                            "$left" => l,
                            "$right" => r,
                            "$low" => l.min(r),
                            "$high" => l.max(r),
                            _ => {
                                if l >= r {
                                    1
                                } else {
                                    -1
                                }
                            }
                        }),
                        None => int(0),
                    },
                }
            }
            "$clog2" | "$countones" | "$onehot" | "$onehot0" | "$isunknown" => {
                let v = self.eval(cx, args[0]);
                let Const::Bits(b) = v else { return int(0) };
                match name {
                    "$clog2" => {
                        if !b.is_known() {
                            return Const::Bits(Bits::x(32));
                        }
                        let n = b.to_u64().unwrap_or(u64::MAX);
                        int(if n <= 1 { 0 } else { 64 - (n - 1).leading_zeros() as i64 })
                    }
                    "$countones" => int((0..b.width()).filter(|&i| b.bit(i) == Logic::One).count() as i64),
                    "$isunknown" => Const::Bits(Bits::from_bool(!b.is_known())),
                    _ => {
                        let c = (0..b.width()).filter(|&i| b.bit(i) == Logic::One).count();
                        Const::Bits(Bits::from_bool(if name == "$onehot" { c == 1 } else { c <= 1 }))
                    }
                }
            }
            "$countbits" => {
                let v = self.eval(cx, args[0]);
                let Const::Bits(b) = v else { return int(0) };
                let want: Vec<Logic> = args[1..]
                    .iter()
                    .filter_map(|&a| match self.eval(cx, a) {
                        Const::Bits(c) => Some(c.bit(0)),
                        _ => None,
                    })
                    .collect();
                int((0..b.width()).filter(|&i| want.contains(&b.bit(i))).count() as i64)
            }
            "$itor" => match self.eval(cx, args[0]) {
                Const::Bits(b) => Const::Real(b.to_f64()),
                v => v,
            },
            "$rtoi" => match self.eval(cx, args[0]) {
                Const::Real(r) => int(r.trunc() as i64),
                v => v,
            },
            "$realtobits" => match self.eval(cx, args[0]) {
                Const::Real(r) => Const::Bits(Bits::from_u64(64, false, r.to_bits())),
                v => v,
            },
            "$bitstoreal" => match self.eval(cx, args[0]) {
                Const::Bits(b) => Const::Real(f64::from_bits(b.to_u64().unwrap_or(0))),
                v => v,
            },
            "$shortrealtobits" => match self.eval(cx, args[0]) {
                Const::Real(r) => Const::Bits(Bits::from_u64(32, false, (r as f32).to_bits() as u64)),
                v => v,
            },
            "$bitstoshortreal" => match self.eval(cx, args[0]) {
                Const::Bits(b) => Const::Real(f32::from_bits(b.to_u64().unwrap_or(0) as u32) as f64),
                v => v,
            },
            "$sqrt" | "$ln" | "$log10" | "$exp" | "$floor" | "$ceil" | "$sin" | "$cos" | "$tan" | "$asin" | "$acos" | "$atan"
            | "$sinh" | "$cosh" | "$tanh" | "$asinh" | "$acosh" | "$atanh" | "$pow" | "$atan2" | "$hypot" => {
                let f = |c: Const| match c {
                    Const::Real(r) => r,
                    Const::Bits(b) => b.to_f64(),
                    _ => 0.0,
                };
                let a = f(self.eval(cx, args[0]));
                let b = args.get(1).map_or(0.0, |&y| f(self.eval(cx, y)));
                Const::Real(match name {
                    "$sqrt" => a.sqrt(),
                    "$ln" => a.ln(),
                    "$log10" => a.log10(),
                    "$exp" => a.exp(),
                    "$floor" => a.floor(),
                    "$ceil" => a.ceil(),
                    "$sin" => a.sin(),
                    "$cos" => a.cos(),
                    "$tan" => a.tan(),
                    "$asin" => a.asin(),
                    "$acos" => a.acos(),
                    "$atan" => a.atan(),
                    "$sinh" => a.sinh(),
                    "$cosh" => a.cosh(),
                    "$tanh" => a.tanh(),
                    "$asinh" => a.asinh(),
                    "$acosh" => a.acosh(),
                    "$atanh" => a.atanh(),
                    "$pow" => a.powf(b),
                    "$atan2" => a.atan2(b),
                    _ => a.hypot(b),
                })
            }
            _ => {
                let _ = x;
                self.unsupported(format!("system task or function `{name}`"));
                ops::default_value(self.types(), ty)
            }
        }
    }

    fn is_string_literal(&self, x: ExprId) -> bool {
        matches!(self.d.expr(x).kind, ExprKind::Const(c) if matches!(self.d.konst(c), Const::Str(_)))
    }

    fn fwrite(&mut self, fd: u32, text: &str) {
        // Bit 0 of a multichannel descriptor, or descriptor 1, is the
        // output; 2 is stderr (21.3.1).
        if fd == 1 || fd == 0x8000_0001 || (fd & 0x8000_0000 == 0 && fd & 1 != 0) || fd == 2 || fd == 0x8000_0002 {
            let _ = self.out.write_all(text.as_bytes());
        }
        let key = if fd & 0x8000_0000 != 0 { fd & 0x7fff_ffff } else { self.files.keys().copied().find(|&k| 1 << (k % 30 + 1) == fd).unwrap_or(0) };
        if let Some(path) = self.files.get(&key) {
            use std::io::Write as _;
            if let Ok(mut f) = std::fs::OpenOptions::new().append(true).create(true).open(path) {
                let _ = f.write_all(text.as_bytes());
            }
        }
    }

    fn rand32(&mut self) -> u32 {
        // xorshift64*: deterministic from run to run.
        let mut x = self.rng;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.rng = x;
        (x.wrapping_mul(0x2545_f491_4f6c_dd1d) >> 32) as u32
    }

    /// `$cast(dst, src)`: assigns if the value fits the destination's type
    /// (6.24.2).
    fn cast(&mut self, cx: &Cx, args: &[ExprId]) -> bool {
        let (Some(&dst), Some(&src)) = (args.first(), args.get(1)) else { return false };
        let dt = self.d.expr(dst).ty;
        let v = self.eval(cx, src);
        let ok = match (self.d.types.get(dt).clone(), &v) {
            (Type::Class(c), Const::Handle(h)) => {
                let mut k = self.heap.get(*h as usize).map(|o| o.class);
                let mut found = false;
                while let Some(x) = k {
                    if x == c {
                        found = true;
                        break;
                    }
                    k = self.d.class(x).base;
                }
                found
            }
            (Type::Class(_), Const::Null) => true,
            (Type::Enum { items, .. }, Const::Bits(b)) => items.iter().any(|(_, x)| x.case_eq(&b.resize(x.width(), b.signed()))),
            _ => true,
        };
        if ok {
            let t = self.target(cx, dst);
            self.write_cx(cx, &t, v, false);
        }
        ok
    }

    fn readmem(&mut self, cx: &Cx, args: &[ExprId], hex: bool) {
        let (Some(&f), Some(&mem)) = (args.first(), args.get(1)) else { return };
        let path = String::from_utf8_lossy(&str_of(&self.eval(cx, f))).to_string();
        let Ok(text) = std::fs::read_to_string(&path) else {
            let _ = writeln!(self.out, "Warning: $readmem cannot open `{path}`");
            return;
        };
        let mt = self.d.expr(mem).ty;
        let Type::Unpacked { elem, dim: UDim::Range(l, r) } = self.d.types.get(mt).clone() else { return };
        let w = self.d.types.width(elem);
        let start = args.get(2).map_or(l.min(r), |&a| self.eval(cx, a).to_i64().unwrap_or(l.min(r)));
        let mut addr = start;
        let mut cur = self.eval(cx, mem);
        // Words, `@addr` jumps; comments are skipped.
        let cleaned: String = text.lines().map(|l| l.split("//").next().unwrap_or("")).collect::<Vec<_>>().join("\n");
        for word in cleaned.split_whitespace() {
            if let Some(a) = word.strip_prefix('@') {
                addr = i64::from_str_radix(a, 16).unwrap_or(addr);
                continue;
            }
            let b = crate::design::value::parse_digits(word, if hex { 16 } else { 2 }, Some(w), false).unwrap_or(Bits::x(w));
            let off = if l <= r { addr - l } else { l - addr };
            if let Const::Aggregate(items) = &mut cur
                && off >= 0
                && (off as usize) < items.len()
            {
                items[off as usize] = Const::Bits(b);
            }
            addr += 1;
        }
        let t = self.target(cx, mem);
        self.write_cx(cx, &t, cur, false);
    }

    // ---- formatting (21.2.1) ----

    /// The text of a `$display`-style argument list: string literals are
    /// format strings that take the arguments after them; other values are
    /// printed in the default radix.
    pub(super) fn format_args(&mut self, cx: &Cx, args: &[ExprId], radix: u8) -> String {
        let mut out = String::new();
        let mut i = 0;
        while i < args.len() {
            let a = args[i];
            i += 1;
            if self.is_string_literal(a) {
                let ExprKind::Const(c) = self.d.expr(a).kind else { unreachable!() };
                let Const::Str(f) = self.d.konst(c).clone() else { unreachable!() };
                let f = String::from_utf8_lossy(&f).to_string();
                i = self.format_one(cx, &f, args, i, &mut out);
            } else if matches!(self.d.expr(a).kind, ExprKind::Opaque) && self.d.expr(a).args.is_empty() {
                out.push(' ');
            } else {
                let v = self.eval(cx, a);
                let t = self.d.expr(a).ty;
                let spec = match radix {
                    b'b' => 'b',
                    b'o' => 'o',
                    b'h' => 'h',
                    _ => 'd',
                };
                let s = if matches!(v, Const::Str(_)) || t == Types::STRING { fmt_value(self, &v, t, 's', None, false, None, cx) } else { fmt_value(self, &v, t, spec, None, false, None, cx) };
                out.push_str(&s);
            }
        }
        out
    }

    /// Formats `f`, taking values from `args` from `next`; returns where it
    /// stopped taking them.
    fn format_one(&mut self, cx: &Cx, f: &str, args: &[ExprId], mut next: usize, out: &mut String) -> usize {
        let b = f.as_bytes();
        let mut i = 0;
        while i < b.len() {
            if b[i] != b'%' {
                let start = i;
                while i < b.len() && b[i] != b'%' {
                    i += 1;
                }
                out.push_str(&f[start..i]);
                continue;
            }
            i += 1;
            let mut left = false;
            let mut zero = false;
            if i < b.len() && b[i] == b'-' {
                left = true;
                i += 1;
            }
            let ws = i;
            while i < b.len() && b[i].is_ascii_digit() {
                i += 1;
            }
            let wtext = &f[ws..i];
            if wtext.starts_with('0') && wtext.len() == 1 {
                zero = true;
            }
            let width: Option<usize> = if wtext.is_empty() || zero { None } else { wtext.parse().ok() };
            let mut prec = None;
            if i < b.len() && b[i] == b'.' {
                i += 1;
                let ps = i;
                while i < b.len() && b[i].is_ascii_digit() {
                    i += 1;
                }
                prec = f[ps..i].parse().ok();
            }
            let Some(&conv) = b.get(i) else { break };
            i += 1;
            let conv = conv.to_ascii_lowercase() as char;
            let text = match conv {
                '%' => "%".to_string(),
                'm' => self.inst_path(cx.inst),
                'l' => "work".into(),
                _ => {
                    let Some(&a) = args.get(next) else {
                        out.push('%');
                        continue;
                    };
                    next += 1;
                    let v = self.eval(cx, a);
                    let t = self.d.expr(a).ty;
                    let caller_unit = self.unit_of(cx.inst);
                    let _ = caller_unit;
                    fmt_value(self, &v, t, conv, width, zero, prec, cx)
                }
            };
            let text = match width {
                Some(w) if text.chars().count() < w => {
                    if left {
                        format!("{text:<w$}")
                    } else {
                        format!("{text:>w$}")
                    }
                }
                _ => text,
            };
            out.push_str(&text);
        }
        next
    }

    // ---- built-in methods ----

    pub(super) fn method(&mut self, cx: &Cx, m: Sym, args: &[ExprId], ty: TypeId, _x: ExprId) -> Const {
        let name = self.d.name(m);
        let int = |v: i64| Const::Bits(Bits::from_i64(32, true, v));
        let Some(&base) = args.first() else { return Const::Null };
        // An array method's iterator index.
        if (name == "index" || self.iter_index.contains_key(&base_obj(self.d, base)))
            && let Some(&i) = self.iter_index.get(&base_obj(self.d, base)) {
                return int(i);
            }
        let bt = self.d.expr(base).ty;
        let v = self.eval(cx, base);
        let rest: Vec<ExprId> = args[1..].to_vec();
        match self.d.types.get(bt).clone() {
            Type::String => {
                let s = str_of(&v);
                let arg = |me: &mut Self, i: usize| rest.get(i).map(|&a| me.eval(cx, a));
                match name {
                    "len" => int(s.len() as i64),
                    "toupper" => Const::Str(s.to_ascii_uppercase()),
                    "tolower" => Const::Str(s.to_ascii_lowercase()),
                    "getc" => {
                        let i = arg(self, 0).and_then(|c| c.to_i64()).unwrap_or(-1);
                        int(s.get(i.max(0) as usize).copied().filter(|_| i >= 0).unwrap_or(0) as i64)
                    }
                    "substr" => {
                        let i = arg(self, 0).and_then(|c| c.to_i64()).unwrap_or(0);
                        let j = arg(self, 1).and_then(|c| c.to_i64()).unwrap_or(-1);
                        if i < 0 || j < i || j as usize >= s.len() {
                            Const::Str(vec![])
                        } else {
                            Const::Str(s[i as usize..=j as usize].to_vec())
                        }
                    }
                    "compare" | "icompare" => {
                        let o = str_of(&arg(self, 0).unwrap_or(Const::Null));
                        let (a, b) = if name == "icompare" { (s.to_ascii_lowercase(), o.to_ascii_lowercase()) } else { (s, o) };
                        int(match a.cmp(&b) {
                            std::cmp::Ordering::Less => -1,
                            std::cmp::Ordering::Equal => 0,
                            std::cmp::Ordering::Greater => 1,
                        })
                    }
                    "atoi" | "atohex" | "atooct" | "atobin" => {
                        let r = match name {
                            "atohex" => 16,
                            "atooct" => 8,
                            "atobin" => 2,
                            _ => 10,
                        };
                        let txt: String = String::from_utf8_lossy(&s).chars().filter(|&c| c != '_').collect();
                        let txt = txt.trim();
                        let digits: String = txt.chars().enumerate().take_while(|&(k, c)| c.is_digit(r) || (k == 0 && c == '-' && r == 10)).map(|(_, c)| c).collect();
                        int(i64::from_str_radix(&digits, r).unwrap_or(0))
                    }
                    "atoreal" => Const::Real(String::from_utf8_lossy(&s).trim().parse().unwrap_or(0.0)),
                    "putc" | "itoa" | "hextoa" | "octtoa" | "bintoa" | "realtoa" => {
                        let new = match name {
                            "putc" => {
                                let i = arg(self, 0).and_then(|c| c.to_i64()).unwrap_or(-1);
                                let c = arg(self, 1).and_then(|c| c.to_i64()).unwrap_or(0) as u8;
                                let mut s2 = s.clone();
                                if i >= 0 && (i as usize) < s2.len() && c != 0 {
                                    s2[i as usize] = c;
                                }
                                s2
                            }
                            "realtoa" => match arg(self, 0) {
                                Some(Const::Real(r)) => format!("{r}").into_bytes(),
                                _ => vec![],
                            },
                            _ => {
                                let val = arg(self, 0).unwrap_or(Const::Null);
                                let b = match val {
                                    Const::Bits(b) => b,
                                    _ => Bits::zero(32, true),
                                };
                                match name {
                                    "itoa" => b.to_decimal(),
                                    "hextoa" => trim_zeros(&b.to_radix(4)),
                                    "octtoa" => trim_zeros(&b.to_radix(3)),
                                    _ => trim_zeros(&b.to_radix(1)),
                                }
                                .into_bytes()
                            }
                        };
                        let t = self.target(cx, base);
                        self.write_cx(cx, &t, Const::Str(new), false);
                        Const::Null
                    }
                    _ => {
                        self.unsupported(format!("string method `{name}`"));
                        Const::Null
                    }
                }
            }
            Type::Enum { items, .. } => {
                let cur = match &v {
                    Const::Bits(b) => items.iter().position(|(_, x)| x.case_eq(&b.resize(x.width(), b.signed()))),
                    _ => None,
                };
                let n = items.len() as i64;
                let k = rest.first().map_or(1, |&a| self.eval(cx, a).to_i64().unwrap_or(1));
                match name {
                    "first" => items.first().map_or(Const::Null, |(_, x)| Const::Bits(x.clone())),
                    "last" => items.last().map_or(Const::Null, |(_, x)| Const::Bits(x.clone())),
                    "num" => int(n),
                    "name" => Const::Str(cur.map_or(vec![], |i| self.d.name(items[i].0).as_bytes().to_vec())),
                    "next" | "prev" => match cur {
                        Some(i) => {
                            let j = if name == "next" { (i as i64 + k).rem_euclid(n) } else { (i as i64 - k).rem_euclid(n) };
                            Const::Bits(items[j as usize].1.clone())
                        }
                        None => items.first().map_or(Const::Null, |(_, x)| Const::Bits(x.clone())),
                    },
                    _ => Const::Null,
                }
            }
            Type::Event => match name {
                "triggered" => {
                    let mut slots = Vec::new();
                    self.reads(cx, base, &mut slots);
                    let hit = slots.iter().any(|&s| self.triggered.get(&self.find(s)) == Some(&self.time));
                    Const::Bits(Bits::from_bool(hit))
                }
                _ => Const::Null,
            },
            Type::Unpacked { elem, dim } => self.array_method(cx, base, v, elem, dim, name, &rest, ty),
            Type::Class(_) => match name {
                "randomize" => {
                    if let Const::Handle(h) = v {
                        self.randomize(h);
                    }
                    int(1)
                }
                "srandom" | "set_randstate" | "rand_mode" | "constraint_mode" => Const::Null,
                _ => {
                    self.unsupported(format!("method `{name}` of a built-in class"));
                    ops::default_value(self.types(), ty)
                }
            },
            _ => {
                if name == "randomize" {
                    return int(1);
                }
                self.unsupported(format!("method `{name}`"));
                ops::default_value(self.types(), ty)
            }
        }
    }

    /// Methods of dynamic arrays, queues, fixed arrays and associative
    /// arrays (7.5–7.12).
    #[allow(clippy::too_many_arguments)]
    fn array_method(&mut self, cx: &Cx, base: ExprId, v: Const, elem: TypeId, dim: UDim, name: &str, rest: &[ExprId], ty: TypeId) -> Const {
        let int = |v: i64| Const::Bits(Bits::from_i64(32, true, v));
        let assoc = matches!(dim, UDim::Assoc(_) | UDim::Wildcard);
        let mut items = match v {
            Const::Aggregate(i) => i,
            _ => vec![],
        };
        let set = |me: &mut Self, items: Vec<Const>| {
            let t = me.target(cx, base);
            me.write_cx(cx, &t, Const::Aggregate(items), false);
        };
        let arg = |me: &mut Self, i: usize| rest.get(i).map(|&a| me.eval(cx, a));
        // A `with` clause: the clause, then the iterator.
        let with = if rest.len() == 2 && matches!(self.d.expr(rest[1]).kind, ExprKind::Object(_)) { Some((rest[0], rest[1])) } else { None };
        let values: Vec<Const> = if assoc { items.chunks(2).map(|c| c[1].clone()).collect() } else { items.clone() };
        match name {
            "size" | "num" => int(if assoc { items.len() / 2 } else { items.len() } as i64),
            "exists" => {
                let k = arg(self, 0).unwrap_or(Const::Null);
                int(items.chunks(2).any(|c| ops::const_eq(&c[0], &k)) as i64)
            }
            "delete" => {
                match (arg(self, 0), assoc) {
                    (None, _) => items.clear(),
                    (Some(k), true) => {
                        if let Some(p) = items.chunks(2).position(|c| ops::const_eq(&c[0], &k)) {
                            items.drain(p * 2..p * 2 + 2);
                        }
                    }
                    (Some(k), false) => {
                        let i = k.to_i64().unwrap_or(-1);
                        if i >= 0 && (i as usize) < items.len() {
                            items.remove(i as usize);
                        }
                    }
                }
                set(self, items);
                Const::Null
            }
            "push_back" | "push_front" => {
                let x = arg(self, 0).unwrap_or(Const::Null);
                let x = ops::convert(self.types(), &x, elem);
                if name == "push_back" {
                    items.push(x);
                } else {
                    items.insert(0, x);
                }
                set(self, items);
                Const::Null
            }
            "pop_back" | "pop_front" => {
                let x = if name == "pop_back" { items.pop() } else if items.is_empty() { None } else { Some(items.remove(0)) };
                set(self, items);
                x.unwrap_or_else(|| ops::default_value(self.types(), elem))
            }
            "insert" => {
                let i = arg(self, 0).and_then(|c| c.to_i64()).unwrap_or(0).clamp(0, items.len() as i64) as usize;
                let x = arg(self, 1).unwrap_or(Const::Null);
                let x = ops::convert(self.types(), &x, elem);
                items.insert(i, x);
                set(self, items);
                Const::Null
            }
            "first" | "last" | "next" | "prev" if assoc => {
                let mut keys: Vec<Const> = items.chunks(2).map(|c| c[0].clone()).collect();
                keys.sort_by(cmp_const);
                let Some(&kx) = rest.first() else { return int(0) };
                let cur = self.eval(cx, kx);
                let pick = match name {
                    "first" => keys.first().cloned(),
                    "last" => keys.last().cloned(),
                    "next" => keys.iter().find(|k| cmp_const(k, &cur) == std::cmp::Ordering::Greater).cloned(),
                    _ => keys.iter().rev().find(|k| cmp_const(k, &cur) == std::cmp::Ordering::Less).cloned(),
                };
                match pick {
                    Some(k) => {
                        let t = self.target(cx, kx);
                        self.write_cx(cx, &t, k, false);
                        int(1)
                    }
                    None => int(0),
                }
            }
            "reverse" => {
                items.reverse();
                set(self, items);
                Const::Null
            }
            "sort" | "rsort" => {
                match with {
                    Some((clause, it)) => {
                        let mut keyed: Vec<(Const, Const)> = Vec::new();
                        for x in items {
                            let k = self.with_value(cx, clause, it, &x, 0);
                            keyed.push((k, x));
                        }
                        keyed.sort_by(|a, b| cmp_const(&a.0, &b.0));
                        items = keyed.into_iter().map(|(_, x)| x).collect();
                    }
                    None => items.sort_by(cmp_const),
                }
                if name == "rsort" {
                    items.reverse();
                }
                set(self, items);
                Const::Null
            }
            "shuffle" => {
                for i in (1..items.len()).rev() {
                    let j = self.rand32() as usize % (i + 1);
                    items.swap(i, j);
                }
                set(self, items);
                Const::Null
            }
            "sum" | "product" | "and" | "or" | "xor" => {
                let op = match name {
                    "sum" => BinOp::Add,
                    "product" => BinOp::Mul,
                    "and" => BinOp::And,
                    "or" => BinOp::Or,
                    _ => BinOp::Xor,
                };
                let mut acc: Option<Const> = None;
                for (k, x) in values.iter().enumerate() {
                    let x = match with {
                        Some((clause, it)) => self.with_value(cx, clause, it, x, k as i64),
                        None => x.clone(),
                    };
                    acc = Some(match acc {
                        None => ops::convert(self.types(), &x, ty),
                        Some(a) => ops::binary(self.types(), op, a, x, ty).unwrap_or(Const::Null),
                    });
                }
                acc.unwrap_or_else(|| ops::default_value(self.types(), ty))
            }
            "min" | "max" | "unique" | "unique_index" | "find" | "find_index" | "find_first" | "find_first_index" | "find_last"
            | "find_last_index" => {
                let mut picked: Vec<(i64, Const)> = Vec::new();
                for (k, x) in values.iter().enumerate() {
                    let key = if assoc { items[k * 2].clone() } else { int(k as i64) };
                    let ok = match (name.starts_with("find"), with) {
                        (true, Some((clause, it))) => truth(&self.with_value(cx, clause, it, x, k as i64)) == Some(true),
                        _ => true,
                    };
                    if ok {
                        picked.push((key.to_i64().unwrap_or(k as i64), x.clone()));
                    }
                }
                let key_of = |me: &mut Self, k: usize, x: &Const| match with {
                    Some((clause, it)) if !name.starts_with("find") => me.with_value(cx, clause, it, x, k as i64),
                    _ => x.clone(),
                };
                let result: Vec<(i64, Const)> = match name {
                    "min" | "max" => {
                        let mut best: Option<(i64, Const, Const)> = None;
                        for (k, (i, x)) in picked.iter().enumerate() {
                            let key = key_of(self, k, x);
                            let better = match &best {
                                None => true,
                                Some((_, _, bk)) => {
                                    let o = cmp_const(&key, bk);
                                    if name == "min" { o == std::cmp::Ordering::Less } else { o == std::cmp::Ordering::Greater }
                                }
                            };
                            if better {
                                best = Some((*i, x.clone(), key));
                            }
                        }
                        best.map(|(i, x, _)| vec![(i, x)]).unwrap_or_default()
                    }
                    "unique" | "unique_index" => {
                        let mut seen: Vec<Const> = Vec::new();
                        let mut out = Vec::new();
                        for (k, (i, x)) in picked.iter().enumerate() {
                            let key = key_of(self, k, x);
                            if !seen.iter().any(|s| ops::const_eq(s, &key)) {
                                seen.push(key);
                                out.push((*i, x.clone()));
                            }
                        }
                        out
                    }
                    "find_first" | "find_first_index" => picked.first().cloned().into_iter().collect(),
                    "find_last" | "find_last_index" => picked.last().cloned().into_iter().collect(),
                    _ => picked,
                };
                if name.ends_with("index") {
                    Const::Aggregate(result.into_iter().map(|(i, _)| int(i)).collect())
                } else {
                    Const::Aggregate(result.into_iter().map(|(_, x)| x).collect())
                }
            }
            _ => {
                self.unsupported(format!("array method `{name}`"));
                ops::default_value(self.types(), ty)
            }
        }
    }

    /// The value of a `with` clause for one element (7.12).
    fn with_value(&mut self, cx: &Cx, clause: ExprId, it: ExprId, x: &Const, index: i64) -> Const {
        let ExprKind::Object(o) = self.d.expr(it).kind else { return Const::Null };
        let locals: Locals = Rc::new(RefCell::new(cx.locals.borrow().clone()));
        locals.borrow_mut().push((o, x.clone()));
        self.iter_index.insert(o, index);
        let c2 = Cx { inst: cx.inst, locals, this: cx.this };
        self.eval(&c2, clause)
    }

    /// `randomize()` without constraints: each `rand` and `randc` property
    /// gets a random value (18.5 is not modelled yet).
    fn randomize(&mut self, h: u32) {
        let Some(obj) = self.heap.get(h as usize) else { return };
        let fields = self.class_fields.get(&obj.class).cloned().unwrap_or_default();
        let mut any_constraint = false;
        let mut c = Some(obj.class);
        while let Some(k) = c {
            if self.d.scope(self.d.class(k).scope).items.iter().any(|i| matches!(i, Item::Opaque(_))) {
                any_constraint = true;
            }
            c = self.d.class(k).base;
        }
        if any_constraint {
            self.unsupported("constraints (randomize ignores them)".into());
        }
        for (i, f) in fields.iter().enumerate() {
            let o = self.d.object(*f);
            if o.qualifiers.iter().any(|&q| matches!(self.d.name(q), "rand" | "randc")) {
                let w = self.d.types.width(o.ty);
                if w > 0 {
                    let mut b = Bits::zero(w, self.d.types.is_signed(o.ty));
                    let mut k = 0;
                    while k < w {
                        let r = self.rand32() as u64;
                        b.set_slice(k, &Bits::from_u64(32.min(w - k), false, r));
                        k += 32;
                    }
                    self.heap[h as usize].fields[i] = Const::Bits(b);
                }
            }
        }
    }
}

fn base_obj(d: &Design, x: ExprId) -> ObjId {
    match d.expr(x).kind {
        ExprKind::Object(o) => o,
        _ => ObjId(u32::MAX),
    }
}

fn radix_of(name: &str) -> u8 {
    match name.as_bytes().last() {
        Some(b'b') if !name.ends_with("$readmemb") => b'b',
        Some(b'o') => b'o',
        Some(b'h') => b'h',
        _ => b'd',
    }
}

fn trim_zeros(s: &str) -> String {
    let t = s.trim_start_matches('0');
    if t.is_empty() { "0".into() } else { t.into() }
}

fn unit_text(e: i8) -> String {
    let (n, u) = match e {
        0 => (1, "s"),
        -1 => (100, "ms"),
        -2 => (10, "ms"),
        -3 => (1, "ms"),
        -4 => (100, "us"),
        -5 => (10, "us"),
        -6 => (1, "us"),
        -7 => (100, "ns"),
        -8 => (10, "ns"),
        -9 => (1, "ns"),
        -10 => (100, "ps"),
        -11 => (10, "ps"),
        -12 => (1, "ps"),
        -13 => (100, "fs"),
        -14 => (10, "fs"),
        _ => (1, "fs"),
    };
    format!("{n}{u}")
}

pub(super) fn str_of(v: &Const) -> Vec<u8> {
    match v {
        Const::Str(s) => s.clone(),
        Const::Bits(b) => b.to_str_bytes(),
        _ => vec![],
    }
}

fn cmp_const(a: &Const, b: &Const) -> std::cmp::Ordering {
    match (a, b) {
        (Const::Bits(x), Const::Bits(y)) => {
            if x.cmp_op(y, |o| o.is_lt()) == Logic::One {
                std::cmp::Ordering::Less
            } else if x.cmp_op(y, |o| o.is_gt()) == Logic::One {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Equal
            }
        }
        (Const::Real(x), Const::Real(y)) => x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal),
        (Const::Str(x), Const::Str(y)) => x.cmp(y),
        _ => std::cmp::Ordering::Equal,
    }
}

/// The bits a value holds, for `$bits` of a dynamic value.
fn bits_of_value(tys: &Types, v: &Const, t: TypeId) -> u64 {
    match v {
        Const::Bits(b) => b.width() as u64,
        Const::Str(s) => s.len() as u64 * 8,
        Const::Aggregate(items) => {
            let et = match tys.get(t) {
                Type::Unpacked { elem, .. } => *elem,
                _ => t,
            };
            items.iter().map(|x| bits_of_value(tys, x, et)).sum()
        }
        _ => 0,
    }
}

/// One value as a format specification says (21.2.1.2–21.2.1.6).
#[allow(clippy::too_many_arguments)]
fn fmt_value(sim: &Sim, v: &Const, t: TypeId, conv: char, width: Option<usize>, zero: bool, prec: Option<usize>, cx: &Cx) -> String {
    let tys = &sim.d.types;
    let w = match v {
        Const::Bits(b) => b.width(),
        _ => tys.width(t),
    };
    match conv {
        'd' | 'i' => match v {
            Const::Bits(b) => {
                let s = b.to_decimal();
                if zero || width.is_some() {
                    s
                } else {
                    // The width of the largest value the type holds.
                    let max = if b.signed() {
                        Bits::from_u64(w.max(1), false, 0).not().lshr(1).to_decimal().len() + 1
                    } else {
                        Bits::from_u64(w.max(1), false, 0).not().to_decimal().len()
                    };
                    format!("{s:>max$}")
                }
            }
            Const::Real(r) => format!("{}", r.round() as i64),
            Const::Str(s) => Bits::from_str_bytes(s).to_decimal(),
            Const::Handle(h) => format!("{h}"),
            _ => "0".into(),
        },
        'h' | 'x' | 'b' | 'o' => {
            let per = match conv {
                'b' => 1,
                'o' => 3,
                _ => 4,
            };
            let b = match v {
                Const::Bits(b) => b.clone(),
                Const::Str(s) => Bits::from_str_bytes(s),
                Const::Real(r) => Bits::from_f64(64, true, *r),
                _ => Bits::zero(1, false),
            };
            let s = b.to_radix(per);
            if zero { trim_zeros(&s) } else { s }
        }
        's' => {
            let bytes = match v {
                Const::Str(s) => s.clone(),
                Const::Bits(b) => {
                    // Leading zero bytes print as spaces unless `%0s`.
                    let n = b.width().div_ceil(8);
                    let mut out = Vec::new();
                    for k in (0..n).rev() {
                        let c = b.slice(k * 8, 8).to_u64().unwrap_or(0) as u8;
                        if c == 0 {
                            if !zero && !out.is_empty() {
                                out.push(b' ');
                            } else if !zero {
                                out.push(b' ');
                            }
                        } else {
                            out.push(c);
                        }
                    }
                    if zero {
                        out.retain(|&c| c != b' ' || true);
                    }
                    let start = out.iter().position(|&c| c != b' ').unwrap_or(out.len());
                    if zero || width.is_none() { out[start..].to_vec() } else { out }
                }
                Const::Real(r) => format!("{r}").into_bytes(),
                _ => vec![],
            };
            String::from_utf8_lossy(&bytes).to_string()
        }
        'c' => match v {
            Const::Bits(b) => ((b.to_u64().unwrap_or(0) & 0xff) as u8 as char).to_string(),
            _ => String::new(),
        },
        'f' | 'e' | 'g' => {
            let r = match v {
                Const::Real(r) => *r,
                Const::Bits(b) => b.to_f64(),
                _ => 0.0,
            };
            let p = prec.unwrap_or(6);
            match conv {
                'f' => format!("{r:.p$}"),
                'e' => {
                    let s = format!("{r:.p$e}");
                    // C's form: two-digit exponents with a sign.
                    match s.split_once('e') {
                        Some((m, e)) => {
                            let n: i32 = e.parse().unwrap_or(0);
                            format!("{m}e{}{:02}", if n < 0 { '-' } else { '+' }, n.abs())
                        }
                        None => s,
                    }
                }
                _ => {
                    let s = format!("{r}");
                    if prec.is_some() { format!("{r:.p$}") } else { s }
                }
            }
        }
        't' => {
            // A time in the caller's unit, shown in `$timeformat`'s.
            let x = match v {
                Const::Real(r) => *r,
                Const::Bits(b) => b.to_f64(),
                _ => 0.0,
            };
            let (unit, p, suffix, minw) = sim.timeformat.clone();
            let caller = sim.unit_of(cx.inst);
            let scaled = x * 10f64.powi((caller - unit) as i32);
            let s = format!("{scaled:.p$}{suffix}");
            if zero || width.is_some() { s } else { format!("{s:>minw$}") }
        }
        'v' => match v {
            Const::Bits(b) => match b.bit(0) {
                Logic::Zero => "St0".into(),
                Logic::One => "St1".into(),
                Logic::X => "StX".into(),
                Logic::Z => "HiZ".into(),
            },
            _ => String::new(),
        },
        'p' => pattern_text(sim, v),
        'u' | 'z' => match v {
            Const::Bits(b) => b.to_radix(1),
            _ => String::new(),
        },
        _ => String::new(),
    }
}

/// `%p`: an aggregate as an assignment pattern (21.2.1.7).
fn pattern_text(sim: &Sim, v: &Const) -> String {
    match v {
        Const::Aggregate(items) => {
            let parts: Vec<String> = items.iter().map(|x| pattern_text(sim, x)).collect();
            format!("'{{{}}}", parts.join(", "))
        }
        Const::Bits(b) => b.to_decimal(),
        Const::Str(s) => format!("\"{}\"", String::from_utf8_lossy(s)),
        Const::Real(r) => format!("{r}"),
        Const::Null => "null".into(),
        Const::Handle(h) => format!("{{handle {h}}}"),
        _ => String::new(),
    }
}
