//! Calling conventions and platforms, read from the description's named
//! metadata (`!abi.NAME`), with registers named as the description names
//! them (`machine.rs`): which registers carry arguments and
//! results, which a call clobbers and which survive it, how arguments past
//! the registers are laid out on the stack, where variadic arguments go,
//! and the assembler's dialect.

use epitome_ir::module::{MdId, Metadata, Module};

use crate::object::SectionKind;

/// How arguments that do not fit in registers are laid out on the stack.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StackArgs {
    /// Each at its own size and alignment, packed (Apple).
    Natural,
    /// Each in an 8-byte slot (AAPCS64).
    Slots,
}

/// Where the variable arguments of a call to a variadic function go.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Variadic {
    /// Always on the stack, in 8-byte slots (Apple).
    Stack,
    /// Where named arguments would go (AAPCS64).
    Named,
    /// Where named arguments would go, but floating-point ones in the
    /// integer registers (RISC-V).
    Integer,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Syntax {
    MachO,
    Elf,
}

#[derive(Clone, Debug)]
pub struct Abi {
    pub name: String,
    pub args: Vec<u8>,
    pub result: u8,
    pub clobbers: Vec<u8>,
    pub saved: Vec<u8>,
    pub reserved: Vec<u8>,
    pub scratch: Vec<u8>,
    /// A register for the addresses of stack slots too far for an
    /// instruction's offset (`far-scratch`), never allocated; for machines
    /// whose offsets reach only a little way (RISC-V's 2047 bytes).
    pub far_scratch: Option<u8>,
    /// Where a caller puts the address its callee writes its result at
    /// (`sret`: AArch64's x8); if none, it is the first argument.
    pub sret: Option<u8>,
    /// A saved register that addresses the frame where `sp` moves (a
    /// function with dynamic allocas).
    pub base: Option<u8>,
    /// An integer wider than a register starts in an even-numbered
    /// register, and at a 16-aligned stack slot (AAPCS64's `i128`).
    pub wide_pairs_even: bool,
    /// An integer wider than a register goes at a 16-aligned stack slot,
    /// in registers as any other (`"wide-pairs"="stack16"`: System V's
    /// `__int128`).
    pub wide_stack16: bool,
    /// The `va_list` a variadic function makes: a pointer to the next
    /// argument (false), or AAPCS64's record of register save areas and
    /// the stack (`va-list` = `aapcs64`).
    pub va_list_aapcs64: bool,
    /// Or System V's (`va-list` = `sysv`): `{ gp_offset, fp_offset,
    /// overflow_arg_area, reg_save_area }`, the registers saved in one
    /// area, the integer ones then the vector ones 16 bytes apart.
    pub va_list_sysv: bool,
    /// Homogeneous floating-point aggregates are passed and returned in
    /// floating-point registers (`hfa`; see `prepare::hfa_of`).
    pub hfa: bool,
    /// The registers a landing pad finds the exception and its selector in
    /// (`eh-regs`; by default the first two argument registers).
    pub eh_regs: Vec<u8>,
    /// Floating-point registers (32-63 for d0-d31): arguments, result,
    /// those a call may change, those preserved, and scratch for spills.
    pub fp_args: Vec<u8>,
    pub fp_result: u8,
    pub fp_clobbers: Vec<u8>,
    pub fp_saved: Vec<u8>,
    pub fp_scratch: Vec<u8>,
    pub fp: u8,
    pub lr: u8,
    /// The return address is on the stack, pushed by the call (x86's
    /// `"return-address"="stack"`), not in a link register: there is no
    /// frame record, and the frame starts 8 bytes below the CFA.
    pub ra_on_stack: bool,
    /// The registers results come back in (`"results"`), where they are
    /// not the result register and the argument registers after it.
    pub result_list: Option<Vec<u8>>,
    /// A register that tells a variadic callee how many vector registers
    /// its arguments take (`"variadic-fp-count"`: System V's al).
    pub variadic_fp_count: Option<u8>,
    pub stack_args: StackArgs,
    /// The stack pointer's alignment at calls, in bytes (`stack-align`).
    pub stack_align: i64,
    pub variadic: Variadic,
    pub syntax: Syntax,
    pub triples: Vec<String>,
}

/// One register's worth of an argument: its size in bytes, and whether it
/// is floating-point.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Part {
    pub size: u64,
    pub fp: bool,
    /// The address a result is returned at (`sret`): in the convention's
    /// register for it, if it has one, without taking an argument's.
    pub sret: bool,
    /// A part of an integer wider than a register (`i128`), which some
    /// conventions start in an even register.
    pub wide: bool,
    /// A member of a homogeneous floating-point aggregate (`[3 x float]`,
    /// see `prepare::hfa_of`): on the stack, the members are side by side.
    pub hfa: bool,
    /// A piece of a struct passed by value in memory (`byval`): its
    /// pieces side by side on the stack, 8 bytes each but the last.
    pub byval: bool,
}

/// Where one argument goes.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ArgLoc {
    Reg(u8),
    /// A byte offset from the stack pointer at the call.
    Stack(i64),
}

impl Abi {
    /// The registers to allocate values to that a call clobbers.
    pub fn caller_saved(&self) -> Vec<u8> {
        self.clobbers
            .iter()
            .copied()
            .filter(|r| !self.reserved.contains(r) && !self.scratch.contains(r) && *r != self.fp && *r != self.lr)
            .collect()
    }

    /// The registers to allocate values to that survive calls.
    pub fn callee_saved(&self) -> Vec<u8> {
        self.saved.iter().copied().filter(|r| !self.reserved.contains(r) && !self.scratch.contains(r)).collect()
    }

    pub fn fp_caller_saved(&self) -> Vec<u8> {
        self.fp_clobbers.iter().copied().filter(|r| !self.fp_scratch.contains(r)).collect()
    }

    pub fn fp_callee_saved(&self) -> Vec<u8> {
        self.fp_saved.clone()
    }

    /// The registers a result of `n` integer parts comes back in: the
    /// result register and the argument registers after it (x0 and x1), or
    /// the convention's list (SysV's rax, rdx).
    pub fn results(&self, n: usize) -> Vec<u8> {
        if let Some(list) = &self.result_list {
            return list.iter().take(n.max(1)).copied().collect();
        }
        let at = self.args.iter().position(|&r| r == self.result).unwrap_or(0);
        let mut out = vec![self.result];
        out.extend(self.args.iter().skip(at + 1).take(n.saturating_sub(1)).copied());
        out
    }

    /// The registers `n` floating-point results come in: the first, then
    /// the floating-point argument registers after it.
    pub fn fp_results(&self, n: usize) -> Vec<u8> {
        let at = self.fp_args.iter().position(|&r| r == self.fp_result).unwrap_or(0);
        let mut out = vec![self.fp_result];
        out.extend(self.fp_args.iter().skip(at + 1).take(n.saturating_sub(1)).copied());
        out
    }

    /// Every register a call may change.
    pub fn all_clobbers(&self) -> Vec<u8> {
        self.clobbers.iter().chain(&self.fp_clobbers).copied().collect()
    }

    /// Where each argument goes, given each one's size in bytes and how
    /// many are named (the rest are variable arguments), and the bytes of
    /// stack the call needs.
    pub fn assign(&self, sizes: &[u64], named: usize) -> (Vec<ArgLoc>, i64) {
        let parts: Vec<Vec<Part>> = sizes
            .iter()
            .map(|&s| vec![Part { size: s, fp: false, sret: false, wide: false, hfa: false, byval: false }])
            .collect();
        let (locs, stack) = self.assign_parts(&parts, named);
        (locs.into_iter().map(|l| l[0]).collect(), stack)
    }

    /// Like `assign`, for arguments that may have several parts: a small
    /// struct passed as `[2 x i64]` takes consecutive registers if enough
    /// are left, and otherwise goes on the stack whole, with no registers
    /// used for arguments after it.
    pub fn assign_parts(&self, args: &[Vec<Part>], named: usize) -> (Vec<Vec<ArgLoc>>, i64) {
        let mut locs = Vec::new();
        let mut next_reg = 0;
        let mut next_fp = 0;
        let mut stack: i64 = 0;
        for (k, parts) in args.iter().enumerate() {
            if let ([p], Some(r)) = (parts.as_slice(), self.sret)
                && p.sret
            {
                locs.push(vec![ArgLoc::Reg(r)]);
                continue;
            }
            let variadic = k >= named;
            if parts.first().is_some_and(|p| p.byval) {
                stack = (stack + 7) / 8 * 8;
                locs.push((0..parts.len() as i64).map(|q| ArgLoc::Stack(stack + 8 * q)).collect());
                stack += parts.iter().map(|p| p.size as i64).sum::<i64>();
                continue;
            }
            let fp = parts.iter().all(|p| p.fp) && !(variadic && self.variadic == Variadic::Integer);
            let n = parts.len();
            if variadic && self.variadic == Variadic::Stack {
                let mut here = Vec::new();
                for _ in parts.iter() {
                    stack = (stack + 7) / 8 * 8;
                    here.push(ArgLoc::Stack(stack));
                    stack += 8;
                }
                locs.push(here);
                continue;
            }
            // Floating-point values take floating-point registers, counted
            // separately; integers and pointers take x0-x7.
            let (regs, next) = if fp { (&self.fp_args, &mut next_fp) } else { (&self.args, &mut next_reg) };
            let even = self.wide_pairs_even && parts.first().is_some_and(|p| p.wide);
            if even && *next % 2 == 1 && *next + 1 + n <= regs.len() {
                *next += 1;
            }
            if *next + n <= regs.len() {
                locs.push(regs[*next..*next + n].iter().map(|&r| ArgLoc::Reg(r)).collect());
                *next += n;
                continue;
            }
            if n > 1 {
                *next = regs.len();
            }
            let mut here = Vec::new();
            if even || (self.wide_stack16 && parts.first().is_some_and(|p| p.wide)) {
                stack = (stack + 15) / 16 * 16;
            }
            // A homogeneous aggregate: its members side by side, aligned as
            // a member is (at least 8, its size rounded up to 8, but for
            // natural placement).
            if parts.first().is_some_and(|p| p.hfa) {
                let member = parts[0].size.max(1) as i64;
                let natural = self.stack_args == StackArgs::Natural;
                let align = if natural { member } else { member.max(8) };
                stack = (stack + align - 1) / align * align;
                for k in 0..n as i64 {
                    here.push(ArgLoc::Stack(stack + k * member));
                }
                stack += n as i64 * member;
                if !natural {
                    stack = (stack + 7) / 8 * 8;
                }
                locs.push(here);
                continue;
            }
            for &Part { size, .. } in parts {
                // (A 16-byte vector: aligned to 16 either way.)
                let (align, room) = match (self.stack_args, n) {
                    (StackArgs::Natural, 1) => (size.max(1) as i64, size.max(1) as i64),
                    _ if size > 8 => (16, 16),
                    _ => (8, 8),
                };
                stack = (stack + align - 1) / align * align;
                here.push(ArgLoc::Stack(stack));
                stack += room;
            }
            locs.push(here);
        }
        (locs, (stack + self.stack_align - 1) / self.stack_align * self.stack_align)
    }

    /// The symbol for a global's name. A name starting with byte 1 is
    /// used as it is, without the platform's prefix (LLVM's `@"\01name"`).
    pub fn symbol(&self, name: &str) -> String {
        if let Some(exact) = name.strip_prefix('\u{1}') {
            return exact.to_string();
        }
        match self.syntax {
            Syntax::MachO => format!("_{name}"),
            Syntax::Elf => name.to_string(),
        }
    }

    /// How unwind and exception tables reach symbol `name` (a personality
    /// routine, a type caught): Mach-O's through its GOT entry, so by its
    /// own name; ELF's through a word of ours holding its address (see
    /// `asm::eh_stubs`).
    pub fn eh_ref(&self, name: &str) -> String {
        match self.syntax {
            Syntax::MachO => self.symbol(name),
            Syntax::Elf => format!("{}.DW.stub", self.symbol(name)),
        }
    }

    /// The prefix of local labels, which the assembler keeps out of the
    /// symbol table.
    pub fn local_prefix(&self) -> &'static str {
        match self.syntax {
            Syntax::MachO => "L",
            Syntax::Elf => ".L",
        }
    }

    /// The directive that switches to a section of a name of its own
    /// (`Section::name`), holding data of this kind.
    pub fn named_section_directive(&self, name: &str, kind: SectionKind) -> String {
        match self.syntax {
            Syntax::MachO => format!(".section {name}"),
            Syntax::Elf => {
                let flags = match kind {
                    SectionKind::Text => "ax",
                    SectionKind::ReadOnly => "a",
                    _ => "aw",
                };
                let ty = ["init_array", "fini_array", "preinit_array"]
                    .into_iter()
                    .find(|t| name.strip_prefix('.').is_some_and(|n| n.starts_with(t)))
                    .unwrap_or("progbits");
                format!(".section {name},\"{flags}\",@{ty}")
            }
        }
    }

    /// The directive that switches to a section of this kind.
    pub fn section_directive(&self, kind: SectionKind) -> &'static str {
        match (self.syntax, kind) {
            (Syntax::MachO, SectionKind::Text) => ".section __TEXT,__text,regular,pure_instructions",
            (Syntax::MachO, SectionKind::Data | SectionKind::Bss) => ".section __DATA,__data",
            (Syntax::MachO, SectionKind::ReadOnly) => ".section __TEXT,__const",
            (Syntax::MachO, SectionKind::RelRo) => ".section __DATA,__const",
            (Syntax::Elf, SectionKind::Text) => ".text",
            (Syntax::Elf, SectionKind::Data) => ".data",
            (Syntax::Elf, SectionKind::Bss) => ".bss",
            (Syntax::Elf, SectionKind::ReadOnly) => ".section .rodata",
            (Syntax::Elf, SectionKind::RelRo) => ".section .data.rel.ro,\"aw\"",
            // Written from .cfi directives, never switched to.
            (Syntax::MachO, SectionKind::EhFrame) => ".section __TEXT,__eh_frame",
            (Syntax::Elf, SectionKind::EhFrame) => ".section .eh_frame,\"a\"",
            (Syntax::MachO, SectionKind::ExceptTable) => ".section __TEXT,__gcc_except_tab",
            (Syntax::Elf, SectionKind::ExceptTable) => ".section .gcc_except_table,\"a\"",
            (Syntax::MachO, SectionKind::ThreadVars) => ".section __DATA,__thread_vars,thread_local_variables",
            (Syntax::MachO, SectionKind::ThreadData) => ".section __DATA,__thread_data,thread_local_regular",
            (Syntax::Elf, SectionKind::ThreadVars | SectionKind::ThreadData) => ".section .tdata,\"awT\",@progbits",
        }
    }
}

/// Every convention the description names.
pub fn read(m: &Module, prefix: &str, machine: &crate::machine::Machine) -> Result<Vec<Abi>, String> {
    let regs = |names: &[String]| machine.regs(names);
    let strings = |id: MdId| -> Vec<String> {
        match &m.metadata[id.index()].data {
            Metadata::Tuple { ops, .. } => ops
                .iter()
                .filter_map(|o| match o.map(|x| &m.metadata[x.index()].data) {
                    Some(Metadata::String(s)) => Some(String::from_utf8_lossy(s).into_owned()),
                    _ => None,
                })
                .collect(),
            _ => vec![],
        }
    };
    let mut out = Vec::new();
    for nm in &m.named_metadata {
        let Some(name) = nm.name.strip_prefix(prefix) else { continue };
        let mut props: std::collections::HashMap<String, Vec<String>> = Default::default();
        for &op in &nm.ops {
            let list = strings(op);
            if let Some((k, v)) = list.split_first() {
                props.insert(k.clone(), v.to_vec());
            }
        }
        let get = |k: &str| props.get(k).cloned().ok_or_else(|| format!("{name}: no {k}"));
        let one = |k: &str| -> Result<String, String> {
            get(k)?.first().cloned().ok_or_else(|| format!("{name}: empty {k}"))
        };
        let first = |k: &str| -> Result<u8, String> {
            regs(&get(k)?)?.first().copied().ok_or_else(|| format!("{name}: empty {k}"))
        };
        let frame = regs(&get("frame")?)?;
        out.push(Abi {
            name: name.to_string(),
            args: regs(&get("args")?)?,
            result: first("result")?,
            clobbers: regs(&get("clobbers")?)?,
            saved: regs(&get("saved")?)?,
            reserved: {
                let mut r = regs(&get("reserved")?)?;
                r.extend(props.get("far-scratch").map(|v| regs(v)).transpose()?.unwrap_or_default());
                r
            },
            far_scratch: props.get("far-scratch").map(|v| regs(v)).transpose()?.and_then(|v| v.first().copied()),
            sret: props.get("sret").map(|v| regs(v)).transpose()?.and_then(|v| v.first().copied()),
            base: props.get("base").map(|v| regs(v)).transpose()?.and_then(|v| v.first().copied()),
            wide_pairs_even: props.get("wide-pairs").is_some_and(|v| v.first().is_some_and(|x| x == "even")),
            wide_stack16: props.get("wide-pairs").is_some_and(|v| v.first().is_some_and(|x| x == "stack16")),
            va_list_aapcs64: props.get("va-list").is_some_and(|v| v.first().is_some_and(|x| x == "aapcs64")),
            va_list_sysv: props.get("va-list").is_some_and(|v| v.first().is_some_and(|x| x == "sysv")),
            hfa: props.get("hfa").is_some_and(|v| v.first().is_some_and(|x| x == "yes")),
            eh_regs: match props.get("eh-regs") {
                Some(v) => regs(v)?,
                None => regs(&get("args")?)?.into_iter().take(2).collect(),
            },
            scratch: regs(&get("scratch")?)?,
            fp_args: regs(&get("fp-args")?)?,
            fp_result: first("fp-result")?,
            fp_clobbers: regs(&get("fp-clobbers")?)?,
            fp_saved: regs(&get("fp-saved")?)?,
            fp_scratch: regs(&get("fp-scratch")?)?,
            fp: frame[0],
            lr: *frame.get(1).unwrap_or(&frame[0]),
            ra_on_stack: props.get("return-address").is_some_and(|v| v.first().is_some_and(|x| x == "stack")),
            result_list: props.get("results").map(|v| regs(v)).transpose()?,
            variadic_fp_count: props
                .get("variadic-fp-count")
                .map(|v| regs(v))
                .transpose()?
                .and_then(|v| v.first().copied()),
            stack_align: {
                let a: i64 = one("stack-align")?.parse().map_err(|_| format!("{name}: stack-align"))?;
                if a <= 0 || a & (a - 1) != 0 {
                    return Err(format!("{name}: stack-align {a} is not a power of two"));
                }
                a
            },
            stack_args: match one("stack-args")?.as_str() {
                "natural" => StackArgs::Natural,
                "slots" => StackArgs::Slots,
                s => return Err(format!("{name}: stack-args {s}")),
            },
            variadic: match one("variadic")?.as_str() {
                "stack" => Variadic::Stack,
                "named" => Variadic::Named,
                "integer" => Variadic::Integer,
                s => return Err(format!("{name}: variadic {s}")),
            },
            syntax: match one("syntax")?.as_str() {
                "macho" => Syntax::MachO,
                "elf" => Syntax::Elf,
                s => return Err(format!("{name}: syntax {s}")),
            },
            triples: get("triples")?,
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::target::Target;

    #[test]
    fn stack_arguments() {
        let t = Target::aarch64();
        let darwin = t.abi("darwin").unwrap();
        let aapcs = t.abi("aapcs64").unwrap();
        // Eight in registers, then a char, a short, an int and a long.
        let sizes = [8, 8, 8, 8, 8, 8, 8, 8, 1, 2, 4, 8];
        let (d, dn) = darwin.assign(&sizes, sizes.len());
        assert_eq!(&d[8..], &[ArgLoc::Stack(0), ArgLoc::Stack(2), ArgLoc::Stack(4), ArgLoc::Stack(8)]);
        assert_eq!(dn, 16);
        let (a, an) = aapcs.assign(&sizes, sizes.len());
        assert_eq!(&a[8..], &[ArgLoc::Stack(0), ArgLoc::Stack(8), ArgLoc::Stack(16), ArgLoc::Stack(24)]);
        assert_eq!(an, 32);
        // printf("%d %d", a, b): one named argument.
        let (d, _) = darwin.assign(&[8, 4, 4], 1);
        assert_eq!(d, vec![ArgLoc::Reg(0), ArgLoc::Stack(0), ArgLoc::Stack(8)]);
        let (a, _) = aapcs.assign(&[8, 4, 4], 1);
        assert_eq!(a, vec![ArgLoc::Reg(0), ArgLoc::Reg(1), ArgLoc::Reg(2)]);
        assert!(!darwin.caller_saved().contains(&18) && aapcs.caller_saved().contains(&18));
    }
}
