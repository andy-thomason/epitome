//! x86 assembly in Intel's syntax, as Rust's `asm!` writes it unless told
//! otherwise (LLVM's `inteldialect`), turned into AT&T's, which the
//! assembler reads: operands in the other order, registers with `%` and
//! immediates with `$`, memory (`qword ptr [rbx + 8*rcx + 16]`) as
//! `16(%rbx,%rcx,8)`, the size a `ptr` gives as the mnemonic's suffix where
//! no register says it, the indirect branches with `*`, and the mnemonics
//! the two write differently (`movzx`, `cdqe`).
//!
//! Lines that are directives or labels pass as they are; a statement may
//! follow a label on its line, and statements may be separated by `;`.

/// `text` (Intel's syntax, operands written in) in AT&T's.
pub fn to_att(text: &str) -> String {
    let mut out = Vec::new();
    for line in text.split('\n') {
        let mut parts = Vec::new();
        for stmt in line.split(';') {
            parts.push(statement(stmt));
        }
        out.push(parts.join(";"));
    }
    out.join("\n")
}

/// One statement: a label, a directive, or an instruction.
fn statement(s: &str) -> String {
    let lead: String = s.chars().take_while(|c| c.is_whitespace()).collect();
    let body = s.trim();
    // (A comment to the line's end.)
    let (body, comment) = match body.find('#') {
        Some(k) => (body[..k].trim_end(), &body[k..]),
        None => (body, ""),
    };
    if body.is_empty() || body.starts_with('.') {
        return s.to_string();
    }
    // A label, perhaps with a statement after it.
    if let Some(k) = body.find(':')
        && body[..k].chars().all(|c| c.is_ascii_alphanumeric() || "_.$".contains(c))
        && !body[..k].is_empty()
        && !is_reg(&body[..k])
    {
        let rest = statement(&body[k + 1..]);
        return format!("{lead}{}:{rest}{}", &body[..k], spaced(comment));
    }
    format!("{lead}{}{}", instruction(body), spaced(comment))
}

fn spaced(comment: &str) -> String {
    if comment.is_empty() { String::new() } else { format!(" {comment}") }
}

const PREFIXES: [&str; 6] = ["lock", "rep", "repe", "repz", "repne", "repnz"];

fn instruction(s: &str) -> String {
    let mut words = s.splitn(2, char::is_whitespace);
    let mut mn = words.next().unwrap_or("").to_ascii_lowercase();
    let mut rest = words.next().unwrap_or("").trim().to_string();
    let mut prefix = String::new();
    while PREFIXES.contains(&mn.as_str()) {
        prefix.push_str(&mn);
        prefix.push(' ');
        let mut w = rest.splitn(2, char::is_whitespace);
        mn = w.next().unwrap_or("").to_ascii_lowercase();
        rest = w.next().unwrap_or("").trim().to_string();
    }
    let ops: Vec<Operand> = split_operands(&rest).iter().map(|o| operand(o)).collect();
    let any_reg = ops.iter().any(|o| matches!(o, Operand::Reg(..)));
    // The mnemonics AT&T names otherwise.
    let renamed = match mn.as_str() {
        "cbw" => Some("cbtw"),
        "cwde" => Some("cwtl"),
        "cdqe" => Some("cltq"),
        "cwd" => Some("cwtd"),
        "cdq" => Some("cltd"),
        "cqo" => Some("cqto"),
        _ => None,
    };
    if let Some(r) = renamed {
        return format!("{prefix}{r}");
    }
    // Extensions: the source's size and the destination's in the name.
    if matches!(mn.as_str(), "movzx" | "movsx" | "movsxd")
        && let [Operand::Reg(d, dsize), src] = &ops[..]
    {
        let from = match src {
            Operand::Reg(_, s) | Operand::Mem(_, s) => *s,
            _ => 0,
        };
        let base = if mn == "movzx" { "movz" } else { "movs" };
        return format!("{prefix}{base}{}{} {}, %{d}", suffix(from), suffix(*dsize), src.att());
    }
    let branch = matches!(mn.as_str(), "call" | "jmp") || mn.starts_with('j') && mn.len() <= 4;
    let mut text: Vec<String> = ops
        .iter()
        .map(|o| match o {
            // (An indirect branch's target.)
            Operand::Reg(..) | Operand::Mem(..) if branch => format!("*{}", o.att()),
            Operand::Imm(v) if branch => v.clone(),
            _ => o.att(),
        })
        .collect();
    text.reverse();
    // Memory whose size only its `ptr` says, the mnemonic's suffix.
    if !any_reg && !branch && !mn.starts_with('v') && !mn.starts_with('f') {
        if let Some(Operand::Mem(_, size)) = ops.iter().find(|o| matches!(o, Operand::Mem(_, s) if *s > 0)) {
            mn.push_str(suffix(*size));
        }
    }
    if text.is_empty() { format!("{prefix}{mn}") } else { format!("{prefix}{mn} {}", text.join(", ")) }
}

/// AT&T's suffix for a size in bytes.
fn suffix(size: u32) -> &'static str {
    match size {
        1 => "b",
        2 => "w",
        4 => "l",
        8 => "q",
        _ => "",
    }
}

/// Splits at commas outside brackets and braces.
fn split_operands(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut depth = 0;
    let mut cur = String::new();
    for c in s.chars() {
        match c {
            '[' | '{' | '(' => depth += 1,
            ']' | '}' | ')' => depth -= 1,
            ',' if depth == 0 => {
                out.push(cur.trim().to_string());
                cur.clear();
                continue;
            }
            _ => {}
        }
        cur.push(c);
    }
    if !cur.trim().is_empty() {
        out.push(cur.trim().to_string());
    }
    out
}

#[derive(Debug, PartialEq)]
enum Operand {
    /// A register and its size in bytes (0 for vectors and the rest), with
    /// any AVX-512 decorations written after it.
    Reg(String, u32),
    /// Memory, in AT&T's form, and the size its `ptr` says (0 if none).
    Mem(String, u32),
    Imm(String),
    /// A decoration alone (`{rn-sae}`), or what is not understood, as it is.
    Other(String),
}

impl Operand {
    fn att(&self) -> String {
        match self {
            Operand::Reg(r, _) => format!("%{r}"),
            Operand::Mem(m, _) => m.clone(),
            Operand::Imm(v) => format!("${v}"),
            Operand::Other(s) => s.clone(),
        }
    }
}

/// An operand's decorations (`{k1}{z}`, `{1to16}`) split off its end, in
/// AT&T's form.
fn decorations(s: &str) -> (&str, String) {
    let Some(k) = s.find('{') else { return (s, String::new()) };
    let deco: String = s[k..]
        .split('}')
        .map(str::trim)
        .filter(|d| !d.is_empty())
        .map(|d| {
            let d = d.trim_start_matches('{').trim();
            if is_reg(d) { format!("{{%{d}}}") } else { format!("{{{d}}}") }
        })
        .collect::<Vec<_>>()
        .join(" ");
    (s[..k].trim(), deco)
}

fn operand(s: &str) -> Operand {
    let (core, deco) = decorations(s);
    let with = |x: String| if deco.is_empty() { x } else { format!("{x} {deco}") };
    if core.is_empty() {
        return Operand::Other(with(String::new()).trim().to_string());
    }
    let lower = core.to_ascii_lowercase();
    if is_reg(&lower) {
        let size = reg_size(&lower);
        return Operand::Reg(if deco.is_empty() { lower } else { format!("{lower} {deco}") }, size);
    }
    // Memory: a size, perhaps a segment, then the address in brackets.
    if let Some(open) = core.find('[') {
        let head = core[..open].trim().to_ascii_lowercase();
        let close = core.rfind(']').unwrap_or(core.len());
        let inner = &core[open + 1..close];
        let mut size = 0;
        let mut seg = String::new();
        for w in head.split_whitespace() {
            match w {
                "byte" => size = 1,
                "word" => size = 2,
                "dword" => size = 4,
                "qword" => size = 8,
                "ptr" => {}
                w if w.ends_with(':') => seg = format!("%{w}"),
                _ => {}
            }
        }
        let mem = address(inner, &seg);
        // (Decorations of memory: a broadcast, written after it.)
        let mem = if deco.is_empty() { mem } else { format!("{mem}{}", deco.replace(' ', "")) };
        return Operand::Mem(mem, size);
    }
    // An immediate: a number, or `offset sym`.
    if let Some(sym) = lower.strip_prefix("offset ") {
        return Operand::Imm(core[core.len() - sym.len()..].trim().to_string());
    }
    if let Some(v) = number(core) {
        return Operand::Imm(v);
    }
    Operand::Other(with(core.to_string()))
}

/// An Intel number (`16`, `-8`, `0x1f`, `1fh`) as AT&T writes it.
fn number(s: &str) -> Option<String> {
    let s = s.trim();
    let (neg, body) = match s.strip_prefix('-') {
        Some(b) => ("-", b.trim()),
        None => ("", s),
    };
    if body.chars().all(|c| c.is_ascii_digit()) && !body.is_empty() {
        return Some(format!("{neg}{body}"));
    }
    if let Some(h) = body.strip_prefix("0x").or_else(|| body.strip_prefix("0X"))
        && !h.is_empty()
        && h.chars().all(|c| c.is_ascii_hexdigit())
    {
        return Some(format!("{neg}0x{h}"));
    }
    if let Some(h) = body.strip_suffix(['h', 'H'])
        && h.starts_with(|c: char| c.is_ascii_digit())
        && h.chars().all(|c| c.is_ascii_hexdigit())
    {
        return Some(format!("{neg}0x{h}"));
    }
    None
}

/// The inside of `[...]` (`rbx + 8*rcx + 16`) as AT&T's address.
fn address(inner: &str, seg: &str) -> String {
    let mut seg = seg.to_string();
    let mut base: Option<String> = None;
    let mut index: Option<(String, String)> = None;
    let mut disp: Vec<String> = Vec::new();
    // Terms with their signs.
    let mut terms: Vec<(bool, String)> = Vec::new();
    let mut cur = String::new();
    let mut neg = false;
    for c in inner.chars() {
        if c == '+' || c == '-' {
            if !cur.trim().is_empty() {
                terms.push((neg, cur.trim().to_string()));
            }
            cur.clear();
            neg = c == '-';
        } else {
            cur.push(c);
        }
    }
    if !cur.trim().is_empty() {
        terms.push((neg, cur.trim().to_string()));
    }
    for (neg, t) in terms {
        let t = match t.split_once(':') {
            Some((s, rest)) if is_reg(&s.trim().to_ascii_lowercase()) => {
                seg = format!("%{}:", s.trim().to_ascii_lowercase());
                rest.trim().to_string()
            }
            _ => t,
        };
        let lower = t.to_ascii_lowercase();
        if let Some((a, b)) = lower.split_once('*') {
            let (a, b) = (a.trim(), b.trim());
            let (r, k) = if is_reg(a) { (a, b) } else { (b, a) };
            index = Some((r.to_string(), k.to_string()));
        } else if is_reg(&lower) && !neg {
            if base.is_none() {
                base = Some(lower);
            } else {
                index = Some((lower, "1".into()));
            }
        } else {
            let v = number(&t).unwrap_or(t);
            disp.push(if neg { format!("-{v}") } else { v });
        }
    }
    let mut d = String::new();
    for (k, x) in disp.iter().enumerate() {
        if k > 0 && !x.starts_with('-') {
            d.push('+');
        }
        d.push_str(x);
    }
    match (base, index) {
        (None, None) => format!("{seg}{d}"),
        (b, i) => {
            let b = b.map_or(String::new(), |b| format!("%{b}"));
            let i = i.map_or(String::new(), |(r, k)| format!(",%{r},{k}"));
            format!("{seg}{d}({b}{i})")
        }
    }
}

/// Whether `s` (lower case) names a register.
fn is_reg(s: &str) -> bool {
    reg_size(s) > 0 || is_other_reg(s)
}

/// An integer register's size in bytes (0 if it is not one).
fn reg_size(s: &str) -> u32 {
    const R64: [&str; 9] = ["rax", "rbx", "rcx", "rdx", "rsi", "rdi", "rsp", "rbp", "rip"];
    const R32: [&str; 9] = ["eax", "ebx", "ecx", "edx", "esi", "edi", "esp", "ebp", "eip"];
    const R16: [&str; 8] = ["ax", "bx", "cx", "dx", "si", "di", "sp", "bp"];
    const R8: [&str; 12] = ["al", "bl", "cl", "dl", "ah", "bh", "ch", "dh", "sil", "dil", "spl", "bpl"];
    if R64.contains(&s) {
        return 8;
    }
    if R32.contains(&s) {
        return 4;
    }
    if R16.contains(&s) {
        return 2;
    }
    if R8.contains(&s) {
        return 1;
    }
    if let Some(n) = s.strip_prefix('r') {
        let digits: String = n.chars().take_while(|c| c.is_ascii_digit()).collect();
        if let Ok(k) = digits.parse::<u32>()
            && (8..=15).contains(&k)
        {
            return match &n[digits.len()..] {
                "" => 8,
                "d" => 4,
                "w" => 2,
                "b" => 1,
                _ => 0,
            };
        }
    }
    0
}

/// The registers other than the integer ones: vectors, masks, x87, MMX,
/// segments, control and debug registers.
fn is_other_reg(s: &str) -> bool {
    let numbered = |p: &str, max: u32| {
        s.strip_prefix(p).is_some_and(|n| !n.is_empty() && n.parse::<u32>().is_ok_and(|k| k <= max))
    };
    numbered("xmm", 31)
        || numbered("ymm", 31)
        || numbered("zmm", 31)
        || numbered("k", 7)
        || numbered("mm", 7)
        || numbered("cr", 15)
        || numbered("dr", 15)
        || numbered("tmm", 7)
        || s == "st"
        || s.starts_with("st(")
        || ["cs", "ds", "es", "fs", "gs", "ss"].contains(&s)
}

#[cfg(test)]
mod tests {
    use super::to_att;

    #[test]
    fn instructions() {
        for (intel, att) in [
            // core::arch's cpuid, around rbx.
            ("mov rsi, rbx", "mov %rbx, %rsi"),
            ("cpuid", "cpuid"),
            ("xchg rsi, rbx", "xchg %rbx, %rsi"),
            ("mov eax, 1", "mov $1, %eax"),
            ("add rax, qword ptr [rbx + 8*rcx + 16]", "add 16(%rbx,%rcx,8), %rax"),
            ("mov qword ptr [rsp - 8], 0", "movq $0, -8(%rsp)"),
            ("lea rax, [rip + sym]", "lea sym(%rip), %rax"),
            ("movzx eax, byte ptr [rdi]", "movzbl (%rdi), %eax"),
            ("movsx rax, cx", "movswq %cx, %rax"),
            ("cdqe", "cltq"),
            ("call rax", "call *%rax"),
            ("jmp qword ptr [rax]", "jmp *(%rax)"),
            ("jne 2f", "jne 2f"),
            ("lock xadd qword ptr [rdi], rax", "lock xadd %rax, (%rdi)"),
            ("vaddps zmm0 {k1} {z}, zmm1, zmm2", "vaddps %zmm2, %zmm1, %zmm0 {%k1} {z}"),
            ("vbroadcastss zmm0, dword ptr [rax]", "vbroadcastss (%rax), %zmm0"),
            ("mov rax, qword ptr fs:[0]", "mov %fs:0, %rax"),
            ("mov rax, 0x1f", "mov $0x1f, %rax"),
            ("2: dec ecx", "2: dec %ecx"),
            (".p2align 4", ".p2align 4"),
        ] {
            assert_eq!(to_att(intel), att, "{intel}");
        }
    }
}
