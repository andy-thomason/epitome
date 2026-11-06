//! 32-bit pointers (`tests/pointers32/mem32.ll`, the toy control core of
//! `tests/partial/tiny.ll` with memory and a data layout of `p:32`): the
//! pointer width comes from the data layout, so pointers are held in its
//! 32-bit registers, addresses are 32-bit sums, a pointer in memory or in
//! data is 4 bytes, and a global's address is a relocation. Programs run
//! on an interpreter of the toy with a byte memory.

use std::path::PathBuf;

use epitome_codegen::assembler::Assembler;
use epitome_codegen::target::Target;
use epitome_codegen::{Options, compile, mir};

fn mem32() -> Target {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/pointers32/mem32.ll");
    Target::load_path(p.to_str().unwrap()).unwrap()
}

/// Each function of `ir` compiled, by name: its code. The object must be
/// what the assembler makes of the assembly.
fn compiled(t: &Target, ir: &str) -> Vec<(String, Vec<u8>)> {
    let mut m = epitome_ir::parse_module(ir).unwrap();
    let c = compile(t, &mut m, &Options::default()).unwrap();
    let bytes = c.object_file(t).unwrap();
    let assembled = Assembler::new(t).assemble(&c.asm, None).unwrap().file(t).unwrap();
    assert_eq!(bytes, assembled, "the object assembled from the assembly differs:\n{}", c.asm);
    c.funcs.iter().map(|f| (f.name.clone(), mir::encode_function(t, f).unwrap().bytes)).collect()
}

fn code_of<'a>(funcs: &'a [(String, Vec<u8>)], name: &str) -> &'a [u8] {
    &funcs.iter().find(|(n, _)| n == name).unwrap_or_else(|| panic!("no {name}")).1
}

/// Runs code from its start with `args` in r1..., to `halt`: r1. The
/// stack pointer (r15, for `stack32.ll`) starts at the top of `mem`.
fn run(code: &[u8], args: &[u32], mem: &mut [u8]) -> u32 {
    let mut r = [0u32; 16];
    r[1..=args.len()].copy_from_slice(args);
    r[15] = mem.len() as u32;
    let mut pc = 0usize;
    for _ in 0..1_000_000 {
        let w = u32::from_le_bytes(code[pc..pc + 4].try_into().unwrap());
        let op = w >> 26;
        let (d, a, b) = ((w >> 22 & 15) as usize, (w >> 18 & 15) as usize, (w >> 14 & 15) as usize);
        let imm = ((w << 14) as i32 >> 14) as u32;
        let (x, y) = (r[a], r[b]);
        let at = x.wrapping_add(imm) as usize;
        let mut next = pc + 4;
        let mut set = |v: u32| r[d] = v;
        match op {
            0x01 => set(x.wrapping_add(y)),
            0x02 => set(x.wrapping_sub(y)),
            0x03 => set(x & y),
            0x04 => set(x | y),
            0x05 => set(x ^ y),
            0x06 => set(x.wrapping_shl(y)),
            0x07 => set(x.wrapping_shr(y)),
            0x08 => set((x as i32).wrapping_shr(y) as u32),
            0x09 => set(x.wrapping_mul(y)),
            0x0a => set(((x as i32) < (y as i32)) as u32),
            0x0b => set((x < y) as u32),
            0x0c => set((x == y) as u32),
            0x0d => set((x as i32).wrapping_div(y as i32) as u32),
            0x0e => set(x / y),
            0x0f => set((x as i32).wrapping_rem(y as i32) as u32),
            0x17 => set(x % y),
            0x10 => set(x.wrapping_add(imm)),
            0x11 => set(x & imm),
            0x12 => set(x | imm),
            0x13 => set(x ^ imm),
            0x14 => set(x << (w & 31)),
            0x15 => set(x >> (w & 31)),
            0x16 => set(((x as i32) >> (w & 31)) as u32),
            0x18 => set(imm),
            0x19 => set(imm << 14),
            0x1f => set(x),
            0x1a => set(u32::from_le_bytes(mem[at..at + 4].try_into().unwrap())),
            0x1b => mem[at..at + 4].copy_from_slice(&r[d].to_le_bytes()),
            0x1c => set(u32::from(mem[at])),
            0x1d => mem[at] = r[d] as u8,
            0x20..=0x28 => {
                let (p, q) = (r[d], r[a]);
                let taken = match op {
                    0x20 => p == q,
                    0x21 => p != q,
                    0x22 => (p as i32) < (q as i32),
                    0x23 => (p as i32) >= (q as i32),
                    0x24 => p < q,
                    0x25 => p >= q,
                    0x26 => p == 0,
                    0x27 => p != 0,
                    _ => true,
                };
                if taken {
                    next = (pc as i64 + imm as i32 as i64 * 4) as usize;
                }
            }
            0x3f => return r[1],
            _ => panic!("no instruction {w:08x} at {pc}"),
        }
        pc = next;
    }
    panic!("did not halt")
}

const IR: &str = r#"
target datalayout = "e-p:32:32-i32:32-i64:64-n32-S32"

; The sum of n words from a.
define i32 @sum(ptr %a, i32 %n) {
entry:
  %none = icmp eq i32 %n, 0
  br i1 %none, label %out, label %loop
loop:
  %i = phi i32 [ 0, %entry ], [ %i1, %loop ]
  %s = phi i32 [ 0, %entry ], [ %s1, %loop ]
  %p = getelementptr inbounds i32, ptr %a, i32 %i
  %v = load i32, ptr %p
  %s1 = add i32 %s, %v
  %i1 = add i32 %i, 1
  %more = icmp ult i32 %i1, %n
  br i1 %more, label %loop, label %out
out:
  %r = phi i32 [ 0, %entry ], [ %s1, %loop ]
  ret i32 %r
}

; A list of { ptr next, i32 value }, 8 bytes a node: the sum of the values.
%node = type { ptr, i32 }
define i32 @walk(ptr %head) {
entry:
  %empty = icmp eq ptr %head, null
  br i1 %empty, label %out, label %loop
loop:
  %n = phi ptr [ %head, %entry ], [ %next, %loop ]
  %s = phi i32 [ 0, %entry ], [ %s1, %loop ]
  %vp = getelementptr inbounds %node, ptr %n, i32 0, i32 1
  %v = load i32, ptr %vp
  %s1 = add i32 %s, %v
  %next = load ptr, ptr %n
  %end = icmp eq ptr %next, null
  br i1 %end, label %out, label %loop
out:
  %r = phi i32 [ 0, %entry ], [ %s1, %loop ]
  ret i32 %r
}

; Links nodes at a, a+8, ... into a list of n, values 10, 20, ...: the head.
define ptr @link(ptr %a, i32 %n) {
entry:
  br label %loop
loop:
  %i = phi i32 [ 0, %entry ], [ %i1, %loop ]
  %p = getelementptr inbounds %node, ptr %a, i32 %i
  %i1 = add i32 %i, 1
  %q = getelementptr inbounds %node, ptr %a, i32 %i1
  %last = icmp eq i32 %i1, %n
  %next = select i1 %last, ptr null, ptr %q
  store ptr %next, ptr %p
  %vp = getelementptr inbounds %node, ptr %p, i32 0, i32 1
  %v = mul i32 %i1, 10
  store i32 %v, ptr %vp
  br i1 %last, label %out, label %loop
out:
  ret ptr %a
}

; Pointers as integers, and their distance.
define i32 @distance(ptr %a, ptr %b) {
  %x = ptrtoint ptr %a to i32
  %y = ptrtoint ptr %b to i32
  %d = sub i32 %y, %x
  %q = inttoptr i32 %d to ptr
  %e = getelementptr i8, ptr %q, i32 1
  %r = ptrtoint ptr %e to i32
  ret i32 %r
}

; The larger of two pointers' bytes.
define i32 @bigger(ptr %a, ptr %b) {
  %lt = icmp ult ptr %a, %b
  %p = select i1 %lt, ptr %b, ptr %a
  %v = load i8, ptr %p
  %r = zext i8 %v to i32
  ret i32 %r
}
"#;

#[test]
fn check_target_passes() {
    let report = epitome_codegen::check::check(&mem32());
    assert_eq!(report.errors(), 0, "{}", report.render(true));
    assert!(!report.render(true).contains("data8"), "{}", report.render(true));
}

#[test]
fn programs_with_pointers_run() {
    let t = mem32();
    let funcs = compiled(&t, IR);
    let mut mem = vec![0u8; 1024];
    for (k, v) in [3u32, 5, 7, 11, 13].iter().enumerate() {
        mem[64 + 4 * k..68 + 4 * k].copy_from_slice(&v.to_le_bytes());
    }
    assert_eq!(run(code_of(&funcs, "sum"), &[64, 5], &mut mem), 39);
    assert_eq!(run(code_of(&funcs, "sum"), &[64, 0], &mut mem), 0);

    let head = run(code_of(&funcs, "link"), &[256, 4], &mut mem);
    assert_eq!(head, 256);
    // Each node's next is a 4-byte pointer, then its 4-byte value.
    assert_eq!(&mem[256..264], &[8, 1, 0, 0, 10, 0, 0, 0]);
    assert_eq!(&mem[280..288], &[0, 0, 0, 0, 40, 0, 0, 0]);
    assert_eq!(run(code_of(&funcs, "walk"), &[256], &mut mem), 100);
    assert_eq!(run(code_of(&funcs, "walk"), &[0], &mut mem), 0);

    assert_eq!(run(code_of(&funcs, "distance"), &[100, 160], &mut mem), 61);
    mem[10] = 0x42;
    mem[20] = 0x77;
    assert_eq!(run(code_of(&funcs, "bigger"), &[10, 20], &mut mem), 0x77);
    assert_eq!(run(code_of(&funcs, "bigger"), &[20, 10], &mut mem), 0x77);
}

/// A global holding a pointer to another is 4 bytes with a 4-byte data
/// relocation, and a global's address in code is the addr relocation.
#[test]
fn globals_hold_4_byte_pointers() {
    let ir = r#"
target datalayout = "e-p:32:32-i32:32-i64:64-n32-S32"
@table = global [2 x i32] [i32 7, i32 9]
@link = global ptr getelementptr (i8, ptr @table, i32 4)
define i32 @second() {
  %p = load ptr, ptr @link
  %v = load i32, ptr %p
  ret i32 %v
}
"#;
    let t = mem32();
    let mut m = epitome_ir::parse_module(ir).unwrap();
    let c = compile(&t, &mut m, &Options::default()).unwrap();
    assert!(c.asm.contains(".long table+4"), "{}", c.asm);
    let link = c.globals.iter().find(|g| g.symbol == "link").unwrap();
    assert_eq!(link.size(), 4);
    let obj = c.object_file(&t).unwrap();
    let assembled = Assembler::new(&t).assemble(&c.asm, None).unwrap().file(&t).unwrap();
    assert_eq!(obj, assembled, "{}", c.asm);
    // R_TINY_32 for the pointer in data, R_TINY_ABS18 for `la` in code.
    let image = epitome_codegen::objdump::Image::read(&obj, &t.machine).unwrap();
    let relocs = epitome_codegen::objdump::relocations(&image);
    assert!(relocs.contains("R_TINY_32") && relocs.contains("table+0x4"), "{relocs}");
    assert!(relocs.contains("R_TINY_ABS18") && relocs.contains("link"), "{relocs}");
}

/// A module whose pointers are another size is not for the machine.
#[test]
fn other_pointer_sizes_are_refused() {
    let t = mem32();
    let mut m =
        epitome_ir::parse_module("target datalayout = \"e-p:64:64\"\ndefine void @f() {\n  ret void\n}\n").unwrap();
    let e = compile(&t, &mut m, &Options::default()).err().unwrap();
    assert!(e.contains("pointers are 64 bits, and the machine's are 32"), "{e}");
}

fn stack32() -> Target {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/pointers32/stack32.ll");
    Target::load_path(p.to_str().unwrap()).unwrap()
}

#[test]
fn stack32_check_target_passes() {
    let report = epitome_codegen::check::check(&stack32());
    assert_eq!(report.errors(), 0, "{}", report.render(true));
}

/// With a stack: a local array, its address passed around as a 32-bit
/// pointer, and more live values than registers, which are spilled to
/// 4-byte slots.
#[test]
fn stack_frames_with_32_bit_pointers() {
    let mut ir = String::from(
        r#"
target datalayout = "e-p:32:32-i32:32-i64:64-n32-S32"

; A local array of n squares (n <= 16), then their sum, read back through
; a pointer kept in a local.
define i32 @squares(i32 %n) {
entry:
  %a = alloca [16 x i32], align 4
  %pp = alloca ptr, align 4
  store volatile ptr %a, ptr %pp
  br label %fill
fill:
  %i = phi i32 [ 0, %entry ], [ %i1, %fill ]
  %p = getelementptr inbounds [16 x i32], ptr %a, i32 0, i32 %i
  %sq = mul i32 %i, %i
  store volatile i32 %sq, ptr %p
  %i1 = add i32 %i, 1
  %more = icmp ult i32 %i1, %n
  br i1 %more, label %fill, label %sum
sum:
  %b = load volatile ptr, ptr %pp
  br label %add
add:
  %k = phi i32 [ 0, %sum ], [ %k1, %add ]
  %s = phi i32 [ 0, %sum ], [ %s1, %add ]
  %q = getelementptr inbounds i32, ptr %b, i32 %k
  %v = load volatile i32, ptr %q
  %s1 = add i32 %s, %v
  %k1 = add i32 %k, 1
  %again = icmp ult i32 %k1, %n
  br i1 %again, label %add, label %out
out:
  ret i32 %s1
}
"#,
    );
    // 24 values live at once, from volatile loads, then combined.
    ir.push_str("\ndefine i32 @pressure(ptr %m) {\n");
    for k in 0..24 {
        ir.push_str(&format!("  %p{k} = getelementptr inbounds i32, ptr %m, i32 {k}\n"));
        ir.push_str(&format!("  %v{k} = load volatile i32, ptr %p{k}\n"));
    }
    ir.push_str("  %s0 = mul i32 %v0, 1\n");
    for k in 1..24 {
        ir.push_str(&format!("  %t{k} = mul i32 %v{k}, {}\n  %s{k} = add i32 %s{}, %t{k}\n", k + 1, k - 1));
    }
    ir.push_str("  ret i32 %s23\n}\n");

    let t = stack32();
    let funcs = compiled(&t, &ir);
    let mut mem = vec![0u8; 4096];
    for n in [1u32, 5, 16] {
        assert_eq!(run(code_of(&funcs, "squares"), &[n], &mut mem), (0..n).map(|i| i * i).sum::<u32>(), "squares({n})");
    }
    for k in 0..24u32 {
        mem[4 * k as usize..4 * k as usize + 4].copy_from_slice(&(100 + k).to_le_bytes());
    }
    let want: u32 = (0..24u32).map(|k| (100 + k) * (k + 1)).sum();
    assert_eq!(run(code_of(&funcs, "pressure"), &[0], &mut mem), want);
}
