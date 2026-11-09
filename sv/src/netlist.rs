//! The compiled netlist (docs/design/sv.md sections 8.1 and 8.3): two-state,
//! values stored as narrow as they fit.
//!
//! A lane is one instance of a specialisation: in one of the independent
//! simulations the netlist is built for ([`Options::sims`]), and, below an
//! array of instances (a generate loop's), in one of its iterations. Each
//! specialisation becomes one function for each number of lanes it runs
//! with, `sv_eval_<n>_<lanes>(ptr %s, ptr %p...)`, almost always one. The
//! lanes set the code's [`Shape`]:
//!
//! - one lane is scalar code: plain integers, each value in 64-bit pieces
//!   (one for up to 64 bits), and costly arms of an `if` or `case` behind
//!   real branches (see [`Options::branch_min`]);
//! - up to 4, 8 or 16 lanes are vectors of that many 32-bit lanes, each value
//!   in 32-bit pieces, with `if` and `case` as selects;
//! - more lanes are a loop over registers of 16, so the code grows with the
//!   specialisation's source, not with the number of instances.
//!
//! For each variable or net, the state holds its pieces (low piece first),
//! each piece a register of lanes for each register the shape has. A child
//! instance's state is part of its parent's, and the parent calls the
//! child's function between two passes: one that works out the child's
//! inputs, and one that reads its outputs. A single child runs with its
//! parent's lanes, and its ports are where its parent keeps the values they
//! are connected to; an array of `k` children is one call with `k` times
//! the parent's lanes, iteration `i` in lanes `i * w..(i + 1) * w` for a
//! parent `w` lanes wide, and the parent copies each iteration's inputs to
//! its lanes and its outputs back.
//!
//! A specialisation with registers below it also has `_ff`, which runs its
//! clocked processes where an edge came (each lane's edge a mask) and
//! makes the registers' next values, and `_commit`; the top's `_step`
//! settles, samples, commits and settles again (see [`Phase`]).
//!
//! Functions are written out where they are called. Fixed-size unpacked
//! arrays are laid out as packed ones with the same bounds would be.
//!
//! The output is Epitome IR as text, for `epitome-opt` and
//! `epitome-codegen`.
//!
//! Not yet: four-state values, tasks, hierarchical references, gates,
//! latches, clocks made by the design's own registers (an edge is seen at
//! the next evaluation), declaration initialisers of registers, and loops
//! whose trip count is not a constant. Small generate loops, and longer
//! ones whose iterations are a select or two, are written out, others
//! rolled back into loops; procedural `for` loops are unrolled.
//! Immediate assertions and `initial` blocks that only assert are dropped.

use std::cell::{Cell, RefCell};
use std::collections::{BTreeSet, BinaryHeap, HashMap, HashSet};
use std::fmt::Write;

use crate::bdd::{Bdd, FALSE, Manager, TRUE, TooBig};
use crate::design::types::{Type, TypeId, UDim};
use crate::hash::FxMap;
use crate::design::value::{Bits, Logic};
use crate::design::{
    AssignKind, BinOp, Binding, CaseKind, Const, Design, Dir, DriverId, ExprId, ExprKind, InstId, Item, Lifetime, ObjId,
    ObjKind, PartKind, ProcId, ProcKind, ScopeId, ScopeKind, SpecId, StmtId, StmtKind, SubId, SubKind, UnOp, ops,
};

/// Most lanes in one register: 32-bit lanes of a 512-bit vector.
pub const LANES: usize = 16;
/// A group: 512 independent simulations, the lanes a regression of many
/// runs fills ([`Options::sims`]).
pub const GROUP: usize = 512;
/// Most bytes of state for many simulations at once: each function's loop
/// goes over all of its state, which then stays in a core's second-level
/// cache (1 MB on AMD's Zen 5; CVA6's issue stage for 512 simulations, 1.6
/// MB, ran a third as fast as for 128). Beyond it, a netlist for many
/// simulations holds fewer ([`Netlist::lanes`]), and a caller runs more
/// states.
pub const STATE_MAX: u64 = 512 << 10;

/// How a function runs its lanes: `n` lanes a register (1 is scalar code),
/// and `chunks` registers of them, a loop over them when more than one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Shape {
    pub n: u32,
    pub chunks: u32,
}

impl Shape {
    /// The shape for `lanes` lanes: scalar for one, 128-, 256- or 512-bit
    /// registers of 32-bit lanes for up to 4, 8 or 16, and more registers
    /// of 16 beyond that.
    pub fn of(lanes: u32) -> Shape {
        match lanes {
            0 | 1 => Shape { n: 1, chunks: 1 },
            2..=4 => Shape { n: 4, chunks: 1 },
            5..=8 => Shape { n: 8, chunks: 1 },
            _ => Shape { n: 16, chunks: lanes.div_ceil(16) },
        }
    }

    pub fn scalar(self) -> bool {
        self.n == 1
    }

    /// The lanes it has room for, padding included.
    pub fn width(self) -> u32 {
        self.n * self.chunks
    }

    /// Bits in a piece of a value: 64 in scalar code, else a lane's 32.
    fn pb(self) -> u32 {
        if self.scalar() { 64 } else { 32 }
    }
}

/// How a value is stored for each register of lanes: computed on in
/// pieces, it is kept as narrow as it fits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lane {
    /// One bit: a bit plane, 16 bits a register (registers of 16 lanes
    /// only).
    Bit,
    /// Up to 8 bits: a byte a lane.
    B8,
    /// Up to 16 bits.
    B16,
    /// 32-bit pieces, as many as the width needs (in scalar code, values of
    /// 17 to 32 bits).
    W32,
    /// 64-bit pieces: scalar code's values wider than 32 bits.
    W64,
}

impl Lane {
    fn of(w: u32, sh: Shape, bit_planes: bool) -> Lane {
        match w {
            1 if bit_planes && sh.n == 16 => Lane::Bit,
            0..=8 => Lane::B8,
            9..=16 => Lane::B16,
            17..=32 => Lane::W32,
            _ if sh.scalar() => Lane::W64,
            _ => Lane::W32,
        }
    }

    /// Bytes for one register of lanes (a piece of `W32` or `W64`).
    fn reg_bytes(self, sh: Shape) -> u64 {
        let lane = match self {
            Lane::Bit => return sh.n as u64 / 8,
            Lane::B8 => 1,
            Lane::B16 => 2,
            Lane::W32 => 4,
            Lane::W64 => 8,
        };
        lane * sh.n as u64
    }

    /// Bytes from one piece of a value to the next.
    fn piece_bytes(self, sh: Shape) -> u64 {
        self.reg_bytes(sh) * sh.chunks as u64
    }

    fn pieces(self, w: u32) -> u64 {
        match self {
            Lane::W32 => pieces(w) as u64,
            Lane::W64 => (w as u64).div_ceil(64).max(1),
            _ => 1,
        }
    }

    /// Bytes for a `w`-bit value.
    fn bytes(self, w: u32, sh: Shape) -> u64 {
        self.pieces(w) * self.piece_bytes(sh)
    }
}

/// The compiled netlist of one top-level instance.
pub struct Netlist {
    /// The functions, as Epitome IR text.
    pub ir: String,
    /// The top specialisation's function.
    pub entry: String,
    /// Bytes of state.
    pub size: usize,
    /// The independent simulations the state holds, lanes `0..lanes`.
    pub lanes: usize,
    /// The top's ports, where they are in the state.
    pub ports: Vec<Field>,
    /// Assertions and assertion-only `initial` blocks left out.
    pub dropped: usize,
    /// With `Options::profile`, what each counter of `sv_prof` times.
    pub profile: Vec<String>,
}

/// Where a value is in the state.
#[derive(Clone, Debug)]
pub struct Field {
    pub name: String,
    pub dir: Dir,
    pub offset: usize,
    pub width: u32,
    pub lane: Lane,
    /// The shape of the function whose state it is in.
    pub shape: Shape,
}

impl Field {
    /// The byte offset of 32-bit word `k` (bits `32 * k` up) of lane `n`,
    /// for a value in 32- or 64-bit pieces.
    pub fn at(&self, k: usize, n: usize) -> usize {
        match self.lane {
            Lane::W32 => self.offset + k * Lane::W32.piece_bytes(self.shape) as usize + n * 4,
            Lane::W64 => {
                assert_eq!(n, 0, "64-bit pieces are scalar code's");
                self.offset + k * 4
            }
            l => panic!("{l:?} is not in pieces"),
        }
    }

    /// Its 32-bit words.
    pub fn pieces(&self) -> usize {
        pieces(self.width)
    }

    /// Writes lane `n`'s value (its low 64 bits; the rest zero).
    ///
    /// # Safety
    /// `state` is a state for this netlist.
    pub unsafe fn put(&self, state: *mut u8, n: usize, v: u64) {
        let v = if self.width < 64 { v & ((1u64 << self.width) - 1) } else { v };
        let (r, l) = (n / 16, n % 16);
        unsafe {
            match self.lane {
                Lane::Bit => {
                    let p = state.add(self.offset + 2 * r) as *mut u16;
                    p.write(p.read() & !(1 << l) | ((v as u16 & 1) << l));
                }
                Lane::B8 => state.add(self.offset + n).write(v as u8),
                Lane::B16 => (state.add(self.offset + 2 * n) as *mut u16).write(v as u16),
                Lane::W32 | Lane::W64 => {
                    for k in 0..self.pieces() {
                        let piece = if k < 2 { (v >> (32 * k)) as u32 } else { 0 };
                        (state.add(self.at(k, n)) as *mut u32).write(piece);
                    }
                }
            }
        }
    }

    /// Lane `n`'s value (its low 64 bits).
    ///
    /// # Safety
    /// `state` is a state for this netlist.
    pub unsafe fn get(&self, state: *const u8, n: usize) -> u64 {
        let (r, l) = (n / 16, n % 16);
        unsafe {
            match self.lane {
                Lane::Bit => ((state.add(self.offset + 2 * r) as *const u16).read() >> l & 1) as u64,
                Lane::B8 => state.add(self.offset + n).read() as u64,
                Lane::B16 => (state.add(self.offset + 2 * n) as *const u16).read() as u64,
                Lane::W32 | Lane::W64 => {
                    (0..self.pieces().min(2)).map(|k| ((state.add(self.at(k, n)) as *const u32).read() as u64) << (32 * k)).sum()
                }
            }
        }
    }
}

impl Netlist {
    /// Instances a state holds: its lanes.
    pub fn group(&self) -> usize {
        self.lanes
    }

    pub fn port(&self, name: &str) -> Option<&Field> {
        self.ports.iter().find(|p| p.name == name)
    }
}

/// A type's width in the netlist: an integral type's, or a fixed-size
/// unpacked array's, whose elements are side by side as those of a packed
/// array with the same bounds would be; 0 for other types.
fn tw(tys: &crate::design::types::Types, t: TypeId) -> u32 {
    match tys.get(t) {
        Type::Unpacked { elem, dim: UDim::Range(l, r) } => tw(tys, *elem) * ((l - r).unsigned_abs() as u32 + 1),
        _ if tys.is_integral(t) => tys.width(t),
        _ => 0,
    }
}

/// The bounds of a type's first dimension, unpacked or packed.
fn range_of(tys: &crate::design::types::Types, t: TypeId) -> (i64, i64) {
    match tys.get(t) {
        Type::Unpacked { dim: UDim::Range(l, r), .. } => (*l, *r),
        _ => tys.packed_range(t).unwrap_or((tw(tys, t) as i64 - 1, 0)),
    }
}

/// The width of one element of a type's first dimension.
fn elem_bits(tys: &crate::design::types::Types, t: TypeId) -> u32 {
    match tys.get(t) {
        Type::Unpacked { elem, .. } => tw(tys, *elem),
        _ => tys.elem_width(t),
    }
}

/// Where elements `lo..=hi` of `t` start, in elements from the least
/// significant end, if they are in range.
fn lsb_off(tys: &crate::design::types::Types, t: TypeId, lo: i64, hi: i64) -> Option<i64> {
    let (l, r) = range_of(tys, t);
    if lo < l.min(r) || hi > l.max(r) {
        return None;
    }
    Some(if l >= r { lo - r } else { r - hi })
}

/// Where a select from `lo` to `hi` starts, in elements from the least
/// significant end, in range or out.
fn sel_start(tys: &crate::design::types::Types, t: TypeId, lo: i64, hi: i64) -> i64 {
    let (l, r) = range_of(tys, t);
    let (lo, hi) = (lo.min(hi), lo.max(hi));
    if l >= r { lo - r } else { r - hi }
}

/// A constant's bits in `w` bits: an unpacked one's elements side by side,
/// the first the most significant.
fn const_bits(c: &Const, w: u32) -> Option<Bits> {
    match c {
        Const::Aggregate(items) if !items.is_empty() && w % items.len() as u32 == 0 => {
            let ew = w / items.len() as u32;
            let parts: Option<Vec<Bits>> = items.iter().map(|i| const_bits(i, ew)).collect();
            Some(Bits::concat(&parts?))
        }
        _ => ops::bits_of(c).map(|b| b.resize(w, b.signed())),
    }
}

/// 32-bit words in a `w`-bit value.
fn pieces(w: u32) -> usize {
    (w as usize).div_ceil(32).max(1)
}

/// The low `pb` bits set.
fn ones_of(pb: u32) -> u64 {
    if pb >= 64 { !0 } else { (1u64 << pb) - 1 }
}

/// The bits of the top piece of a `w`-bit value, in pieces of `pb` bits.
fn top_mask(w: u32, pb: u32) -> u64 {
    if w % pb == 0 { ones_of(pb) } else { (1u64 << (w % pb)) - 1 }
}

/// Choices in how the netlist is laid out.
#[derive(Clone, Copy, Debug)]
pub struct Options {
    /// One-bit elements as bit planes, a 16-bit mask for each register of
    /// 16 lanes: for targets with mask registers (AVX-512, SVE). Otherwise
    /// they take a lane each.
    pub bit_planes: bool,
    /// Specialisations proved to count zeros of their input (`recognise`)
    /// become that count, `llvm.ctlz` on lanes, not their own logic.
    pub recognise: bool,
    /// Costly arms of vector code run only where some lane of a register
    /// takes them: arms of at least this many instructions a register of
    /// lanes (see [`GUARD_MIN`]); 0 for none, every arm run and selects
    /// merging them.
    pub guard_min: usize,
    /// Arms of scalar code of at least this many instructions are behind a
    /// branch (see [`BRANCH_MIN`]); smaller ones are run and selected
    /// between, as in vector code. 0 for none.
    pub branch_min: usize,
    /// Independent simulations of the top run at once, each a lane of every
    /// specialisation (more than 16 are rounded up to a multiple of 16, and
    /// fewer are held where their state would pass [`STATE_MAX`]).
    pub sims: usize,
    /// For profiling: the intrinsic that reads a cycle counter
    /// (`llvm.x86.rdtsc`), and each call of a child's function and each
    /// part of the top's step adds the cycles it took to an `i64` of the
    /// global `sv_prof`, and one to the next, named in [`Netlist::profile`].
    pub profile: Option<&'static str>,
}

impl Default for Options {
    fn default() -> Self {
        Options { bit_planes: true, recognise: true, guard_min: GUARD_MIN, branch_min: BRANCH_MIN, sims: 1, profile: None }
    }
}

/// Lowers the instance `top` and everything below it.
pub fn compile(d: &Design, top: InstId) -> Result<Netlist, String> {
    compile_with(d, top, &Options::default())
}

/// The functions of a specialisation with registers besides its settling
/// one.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    /// Its clocked processes on the settled values, where their edges came:
    /// the registers' next values.
    Ff,
    /// The next values become the registers'.
    Commit,
}

impl Phase {
    fn suffix(self) -> &'static str {
        match self {
            Phase::Ff => "_ff",
            Phase::Commit => "_commit",
        }
    }
}

/// A specialisation, lowered for so many lanes.
type Key = (SpecId, u32);

pub fn compile_with(d: &Design, top: InstId, opts: &Options) -> Result<Netlist, String> {
    let spec = d.instance(top).spec;
    let mut sims = if opts.sims > 16 { opts.sims.next_multiple_of(16) } else { opts.sims.max(1) };
    // For many simulations, as many registers of them as keep the state
    // within STATE_MAX.
    if sims > 16 {
        let mut n = Lowering::new(d, (spec, sims as u32), opts);
        n.layout((spec, sims as u32))?;
        let per = n.layouts[&(spec, sims as u32)].size.div_ceil(sims as u64 / 16);
        while sims > 16 && per * (sims as u64 / 16) > STATE_MAX {
            sims = (sims / 2).next_multiple_of(16);
        }
    }
    let key = (spec, sims as u32);
    let mut n = Lowering::new(d, key, opts);
    n.layout(key)?;
    // The inputs the top's settling reads, each with a copy of its value
    // at the last settling, and whether there has been one.
    if n.layouts[&key].clocked {
        for p in &d.spec(spec).ports {
            let Some(o) = p.object.filter(|_| p.dir == Dir::Input) else { continue };
            if !n.comb_reads(key, o) {
                continue;
            }
            let lay = n.layouts.get_mut(&key).unwrap();
            let (lane, w) = (lay.lane(o), tw(&d.types, d.object(o).ty));
            let copy = lay.room_for(w, lane);
            n.watch.push((lay.objs[&o], lane, w, copy));
        }
        let lay = n.layouts.get_mut(&key).unwrap();
        n.settled = lay.room(1, 1);
    }
    n.function(key)?;
    for decl in n.decls.borrow().iter() {
        let _ = writeln!(n.ir, "{decl}");
    }
    let lay = &n.layouts[&key];
    let mut ports = Vec::new();
    for p in &d.spec(spec).ports {
        let o = p.object.ok_or_else(|| format!("interface port {} is not supported yet", d.name(p.name)))?;
        ports.push(Field {
            name: d.name(p.name).to_string(),
            dir: p.dir,
            offset: lay.objs[&o] as usize,
            width: tw(&d.types, d.object(o).ty),
            lane: lay.lane(o),
            shape: lay.shape,
        });
    }
    let dropped = n.dropped.get() + n.asserts.borrow().len();
    let entry = if lay.clocked { format!("{}_step", fn_name(key)) } else { fn_name(key) };
    let size = lay.size as usize;
    let profile = n.profile.take();
    let mut ir = n.ir;
    if let Some(counter) = opts.profile {
        let _ = write!(ir, "\n@sv_prof = global [{} x i64] zeroinitializer\ndeclare i64 @{counter}()\n", 2 * profile.len().max(1));
    }
    Ok(Netlist { ir, entry, size, lanes: sims, ports, dropped, profile })
}

impl<'d, 'a> Lowering<'d, 'a> {
    fn new(d: &'d Design<'a>, top: Key, opts: &Options) -> Self {
        Lowering {
            top: Some(top),
            opts: *opts,
            d,
            layouts: HashMap::new(),
            done: HashSet::new(),
            ir: String::new(),
            folds: RefCell::default(),
            dropped: Cell::new(0),
            asserts: RefCell::default(),
            loops: Vec::new(),
            sym_orders: RefCell::default(),
            sub_locals: RefCell::default(),
            loop_vars: RefCell::default(),
            port_deps: RefCell::default(),
            parts: HashMap::new(),
            comb_reads: RefCell::default(),
            watch: Vec::new(),
            settled: 0,
            profile: RefCell::default(),
            codes: HashMap::new(),
            arrays: HashMap::new(),
            decls: RefCell::default(),
        }
    }
}

fn fn_name((s, lanes): Key) -> String {
    format!("sv_eval_{}_{lanes}", s.0)
}

/// Where a specialisation's values and children are in its state.
struct Layout {
    shape: Shape,
    size: u64,
    /// The specialisation's static values.
    statics: HashSet<ObjId>,
    /// Where those that need one have their slot in the state: not those
    /// worked out and used in one loop.
    objs: HashMap<ObjId, u64>,
    /// How narrow values are stored; the rest are `Lane::W32`.
    lanes: HashMap<ObjId, Lane>,
    /// Below the top: the ports, as the function's pointer parameters.
    port_ix: HashMap<ObjId, usize>,
    /// For each child (an array's first instance, for an array), what is
    /// passed for each of its ports.
    args: HashMap<InstId, Vec<Arg>>,
    /// The objects kept an element to a slot (docs/design/sv.md section
    /// 8.3); the rest are 32-bit pieces.
    elems: HashMap<ObjId, Elems>,
    /// Where each child's state is, and what the child is lowered as.
    kids: HashMap<InstId, u64>,
    kid_keys: HashMap<InstId, Key>,
    /// Its processes, drivers and children, in source order.
    code: Vec<Node>,
    /// What it is proved to compute, if that replaces its own logic.
    known: Option<Known>,
    /// Its clocked processes (`always_ff`, and `always` on edges).
    ff: Vec<ProcId>,
    /// What they assign with nonblocking assignments: its registers, each
    /// with a slot for its next value.
    regs: Vec<ObjId>,
    shadow: HashMap<ObjId, u64>,
    /// For each edge a clocked process waits on, the slot that holds the
    /// value its expression had at the last evaluation.
    prev: HashMap<ExprId, u64>,
    /// It or a specialisation below it has registers.
    clocked: bool,
    /// Where its `_ff` function notes whether an edge came.
    fired: u64,
    /// The values only continuous assignments write, all in one loop:
    /// each starts there from zero, not from its last value, as every bit
    /// it has is one they drive or zero.
    fresh: HashSet<ObjId>,
    /// The children called twice, as the parent's code loops through them
    /// though none of their outputs depends on itself.
    splits: HashMap<InstId, Split>,
    /// The arrays kept an element to a piece-aligned run (see [`Pad`]).
    pads: HashMap<ObjId, Pad>,
    /// Wide values that stay in the state, not in registers: padded
    /// arrays, other arrays indexed by a value, and registers that no
    /// clocked process reads. Each select loads the pieces it reads, and
    /// each assignment stores the pieces it changes in the lanes that run
    /// it (`Func::path`), so the arms of a branch need no selects of them
    /// where they meet, nor phis around a guarded arm.
    kept: HashSet<ObjId>,
    /// Of those, the registers: nonblocking assignments store their next
    /// values in place, as no clocked process reads them.
    in_place: HashSet<ObjId>,
    /// For a child its parent calls twice (see [`Split`]): its code in two
    /// functions, `_a` with what the first call's outputs need, and `_b`
    /// with the rest and what of `_a` reads inputs the first call does not
    /// have, run again.
    part: Option<(Vec<Node>, Vec<Node>)>,
}

/// A child called twice: first with the inputs that some of its outputs
/// depend on (the rest as they were), for those outputs, then again with
/// all of them, for the rest. Its code settles, so the second call gives
/// the first call's outputs again.
#[derive(Clone, Debug, Default)]
struct Split {
    /// The child's inputs that the first call needs.
    ins: HashSet<ObjId>,
    /// The outputs it gives.
    outs: HashSet<ObjId>,
}

/// An array of wide elements that is indexed by a value, kept with each
/// element starting a piece: a select of one of its elements, the
/// same bits of each, is then the same shift whichever it is, and a
/// select of whole pieces. It is kept in the state (`Layout::kept`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Pad {
    /// The element's width.
    ew: u32,
    n: u32,
    /// Bits in a piece.
    pb: u32,
}

impl Pad {
    /// Bits from one element to the next.
    fn stride(self) -> u32 {
        self.ew.div_ceil(self.pb) * self.pb
    }

    fn width(self) -> u32 {
        self.n * self.stride()
    }
}

/// Bits of an object: (object, from, to) with `to` exclusive.
type Span = (ObjId, u32, u32);

/// A piece of a specialisation's combinational code, statement by
/// statement in its processes: the bits it writes, from the bits it
/// reads (and those that decide whether it runs).
struct Dep {
    node: usize,
    w: Vec<Span>,
    r: Vec<Span>,
}

/// What a parent passes for a child's port.
#[derive(Clone, Copy, Debug)]
enum Arg {
    /// The parent's own value, connected whole: the child reads and writes
    /// it where it is, with nothing copied.
    Alias(ObjId),
    /// A slot in the parent's state, at this offset and laid out as the
    /// child's values are, that the parent fills before the call (an input)
    /// or reads after it (an output).
    Slot(u64),
}

/// Where a value is: at an offset in the function's state, or behind one
/// of its pointer parameters.
#[derive(Clone, Copy, Debug)]
enum Base {
    State(u64),
    Port(usize),
}

impl Layout {
    fn new(shape: Shape) -> Layout {
        Layout {
            shape,
            size: 0,
            statics: HashSet::new(),
            objs: HashMap::new(),
            lanes: HashMap::new(),
            port_ix: HashMap::new(),
            args: HashMap::new(),
            elems: HashMap::new(),
            kids: HashMap::new(),
            kid_keys: HashMap::new(),
            code: Vec::new(),
            known: None,
            ff: Vec::new(),
            regs: Vec::new(),
            shadow: HashMap::new(),
            prev: HashMap::new(),
            clocked: false,
            fired: 0,
            fresh: HashSet::new(),
            splits: HashMap::new(),
            pads: HashMap::new(),
            kept: HashSet::new(),
            in_place: HashSet::new(),
            part: None,
        }
    }

    fn lane(&self, o: ObjId) -> Lane {
        self.lanes.get(&o).copied().unwrap_or(Lane::W32)
    }

    /// Room for `bytes` at the end, aligned to `align`: where it is.
    fn room(&mut self, bytes: u64, align: u64) -> u64 {
        let at = self.size.next_multiple_of(align.max(1));
        self.size = at + bytes;
        at
    }

    /// Room for a `w`-bit value stored as `lane`.
    fn room_for(&mut self, w: u32, lane: Lane) -> u64 {
        let sh = self.shape;
        self.room(lane.bytes(w, sh), lane.reg_bytes(sh))
    }

    /// A slot for `o`, `w` bits wide, at the end.
    fn place(&mut self, o: ObjId, w: u32, lane: Lane) {
        let at = self.room_for(w, lane);
        self.objs.insert(o, at);
        if lane != Lane::W32 {
            self.lanes.insert(o, lane);
        }
    }
}

/// A specialisation proved, for every value of its one input, to give a
/// count of the input's trailing or leading zeros, or of its ones, on each
/// output (with a constant where the input is zero), or whether it is
/// zero, or a constant. It is then that count: `llvm.ctlz` on lanes, or
/// the ones counted a piece at a time.
#[derive(Clone, Debug)]
struct Known {
    input: ObjId,
    width: u32,
    count: Count,
    outs: Vec<(ObjId, Out)>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Count {
    Trailing,
    Leading,
    Ones,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Out {
    /// The count, or `at_zero` where the input is zero.
    Count { at_zero: u64 },
    /// Whether the input is zero (or not, if `inverted`).
    Zero { inverted: bool },
    Const(u64),
}

/// An object used only an element at a time, kept an element to a slot:
/// in vector code, one-bit elements as bit planes, a 16-bit mask for each
/// register of lanes (a k register with AVX-512), and wider ones (up to 32
/// bits) in a register of 32-bit lanes each; in scalar code, each element
/// as narrow as it fits (up to 64 bits). Selecting an element is then
/// choosing an address, with no shifts.
#[derive(Clone, Copy, Debug)]
struct Elems {
    /// The element's width.
    ew: u32,
    /// How many elements.
    n: u32,
    /// How each element is stored.
    lane: Lane,
    /// Bytes from one element's slot to the next.
    stride: u64,
}

impl Elems {
    fn planes(self) -> bool {
        self.lane == Lane::Bit
    }

    fn stride(self) -> u64 {
        self.stride
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Node {
    Proc(ProcId),
    Driver(DriverId),
    /// A child's inputs, worked out in the parent.
    In(InstId),
    /// The call of a child's function, then its outputs.
    Call(InstId),
    /// For a child called twice (`Layout::splits`): the inputs its first
    /// call needs, and that call with the outputs it gives.
    Pre(InstId),
    Early(InstId),
    /// A run of a generate loop's iterations, rolled back into a loop.
    Loop(usize),
}

/// Iterations of a generate loop with the same code, which differ only in
/// constants that step by the same amount each time (the genvar, and
/// what is worked out from it): run as a loop over `count` iterations.
struct Rolled {
    count: u32,
    /// The first iteration's continuous assignments, the loop's body.
    body: Vec<DriverId>,
    /// The first iteration's parameters that step: the parameter, its
    /// first value and its step.
    steps: Vec<(ObjId, i64, i64)>,
    /// Every iteration's continuous assignments, for what the loop reads
    /// and writes.
    all: Vec<DriverId>,
    /// The first iteration's element selects whose index is in range in
    /// every iteration: they need no check.
    in_range: HashSet<ExprId>,
}

/// Fewest iterations worth a loop.
const ROLL_MIN: usize = 4;
/// Fewest pieces of a value worth keeping in the state (`Layout::kept`).
const KEPT_MIN: usize = 8;
/// Fewest instructions, for one register of lanes, of an arm of an `if` or
/// `case` (or a clocked process's body) worth a branch around it, taken only
/// when some lane needs the arm. The test is a few instructions (the mask's
/// bits, a compare, a branch: kortestw and jz with AVX-512) and a branch
/// mispredicted now and then, some 15 to 20 cycles; an arm of 64 vector
/// instructions costs some 30 to 60, so below this the selects are cheaper
/// on average even where the arm is skipped half the time.
pub const GUARD_MIN: usize = 64;
/// Fewest instructions of an arm of an `if` or `case` in scalar code worth
/// a branch around it rather than running it and selecting its values: a
/// branch costs a compare and a jump, and now and then a misprediction,
/// where the selects cost both arms' work every time.
pub const BRANCH_MIN: usize = 8;
/// Most expression nodes, over all iterations, of a run of iterations
/// left unrolled: each iteration of a rolled loop is a pass over the
/// group's state, which costs more than a small body written out.
const UNROLL_MAX: usize = 1024;
/// Most expression nodes in one iteration for which [`UNROLL_TINY_MAX`]
/// holds instead. A body this small is a select or two (a loop that moves
/// one bit an iteration, as a bit reversal does): rolled, each iteration
/// is a pass over the state with its selects' offsets worked out at run
/// time, many times what the same selects cost written out with constant
/// offsets.
const TINY_BODY: usize = 32;
/// Most expression nodes, over all iterations, of a run of iterations with
/// a tiny body left unrolled: past this even they roll, so the code stays
/// bounded by the source however many iterations a parameter asks for.
const UNROLL_TINY_MAX: usize = 8192;

struct Lowering<'d, 'a> {
    opts: Options,
    /// The top specialisation: its ports are in its state; every other
    /// specialisation's are where its parent passes pointers to.
    top: Option<Key>,
    d: &'d Design<'a>,
    layouts: HashMap<Key, Layout>,
    done: HashSet<Key>,
    ir: String,
    folds: RefCell<HashMap<ExprId, Option<Const>>>,
    /// `initial` blocks and kept syntax left out.
    dropped: Cell<usize>,
    /// Immediate assertions left out.
    asserts: RefCell<HashSet<StmtId>>,
    loops: Vec<Rolled>,
    /// Specialisations' code in order, kept for proving what their
    /// parents compute after a layout of their own has dropped it.
    sym_orders: RefCell<HashMap<SpecId, Vec<Node>>>,
    sub_locals: RefCell<HashMap<SubId, Vec<ObjId>>>,
    /// The parameters that step in the loop being lowered: not constants
    /// there.
    loop_vars: RefCell<HashSet<ObjId>>,
    port_deps: RefCell<HashMap<Key, std::rc::Rc<HashMap<ObjId, HashSet<ObjId>>>>>,
    /// The specialisations called twice, each by its one parent, with
    /// what the first call has and gives.
    parts: HashMap<SpecId, Split>,
    comb_reads: RefCell<HashMap<(Key, ObjId), bool>>,
    /// For the top's step: each input its settling reads, where it is, how
    /// it is stored, its width, and where its copy is.
    watch: Vec<(u64, Lane, u32, u64)>,
    /// Where the top notes that it has settled.
    settled: u64,
    /// With `Options::profile`, what each counter times.
    profile: RefCell<Vec<String>>,
    /// Each specialisation's static values and code, as `collect` finds
    /// them.
    codes: HashMap<SpecId, (Vec<ObjId>, Vec<Node>)>,
    /// Arrays of instances (a generate loop's, one an iteration), by their
    /// first instance: called as one, with a lane block an iteration.
    arrays: HashMap<InstId, Vec<InstId>>,
    /// The intrinsics the functions call.
    decls: RefCell<BTreeSet<String>>,
}

impl<'d, 'a> Lowering<'d, 'a> {
    /// A specialisation's static values and code, found once.
    fn code_of(&mut self, spec: SpecId) -> Result<(Vec<ObjId>, Vec<Node>), String> {
        if let Some(c) = self.codes.get(&spec) {
            return Ok(c.clone());
        }
        let d = self.d;
        let mut objs = Vec::new();
        let mut code = Vec::new();
        self.collect(d.spec(spec).scope, &mut objs, &mut code)?;
        for &o in &objs {
            let ob = d.object(o);
            if tw(&d.types, ob.ty) == 0 {
                return Err(format!("{}: only integral values and fixed-size arrays of them are supported yet", d.name(ob.name)));
            }
            // The state starts at zero: a variable that starts at anything
            // else would silently not.
            if ob.kind == ObjKind::Variable
                && let Some(e) = ob.init
                && !self.fold(e).and_then(|c| const_bits(&c, tw(&d.types, ob.ty))).is_some_and(|b| b.known_or_zero().is_zero())
            {
                return Err(format!("{}: initial values of variables are not supported yet", d.name(ob.name)));
            }
        }
        self.codes.insert(spec, (objs.clone(), code.clone()));
        Ok((objs, code))
    }

    /// The instances an instance node stands for: an array's, or itself.
    fn members(&self, i: InstId) -> Vec<InstId> {
        self.arrays.get(&i).cloned().unwrap_or_else(|| vec![i])
    }

    /// The lanes child `i` runs with, below a function of shape `sh` with
    /// `lanes` lanes: its parent's, or, for an array, a block as wide as
    /// the parent's registers for each iteration.
    fn child_lanes(&self, sh: Shape, lanes: u32, i: InstId) -> u32 {
        match self.members(i).len() as u32 {
            1 => lanes,
            k => k * sh.width(),
        }
    }

    fn layout(&mut self, key: Key) -> Result<(), String> {
        if self.layouts.contains_key(&key) {
            return Ok(());
        }
        let d = self.d;
        let (spec, lanes) = key;
        let sh = Shape::of(lanes);
        let planes = self.opts.bit_planes;
        let (objs, code) = self.code_of(spec)?;
        let mut lay = Layout::new(sh);
        lay.statics = objs.iter().copied().collect();
        // Clocked processes are not in the order of the rest: they run
        // after it, on what it has worked out.
        let (ff, code): (Vec<Node>, Vec<Node>) =
            code.into_iter().partition(|n| matches!(*n, Node::Proc(p) if self.clocked(p).is_some()));
        lay.ff = ff.into_iter().map(|n| if let Node::Proc(p) = n { p } else { unreachable!() }).collect();
        let mut ff_reads = Vec::new();
        for &p in &lay.ff {
            let (terms, body) = self.clocked(p).unwrap();
            let mut w = Vec::new();
            self.nba_writes(body, &mut w);
            for (o, _, _) in w {
                if lay.statics.contains(&o) && !lay.regs.contains(&o) {
                    lay.regs.push(o);
                }
            }
            self.stmt_access(body, &mut ff_reads, &mut Vec::new(), &mut Vec::new());
            for t in terms {
                self.reads(t.expr, &mut ff_reads);
                if let Some(c) = t.iff {
                    self.reads(c, &mut ff_reads);
                }
            }
        }
        lay.regs.sort_unstable_by_key(|o| o.0);
        let ff_reads: HashSet<ObjId> = ff_reads.into_iter().map(|t| t.0).collect();
        lay.elems = self.element_only(&objs, &code, sh);
        lay.elems.retain(|o, _| !lay.regs.contains(o) && !ff_reads.contains(o));
        let indexed = self.indexed(&code, &lay.ff);
        lay.pads = self.padded(&objs, &indexed, sh);
        lay.pads.retain(|o, _| !lay.elems.contains_key(o));
        lay.kept = lay.pads.keys().copied().collect();
        for &o in &objs {
            let ob = d.object(o);
            let reg = lay.regs.contains(&o) && !ff_reads.contains(&o);
            if ob.dir.is_none() && !lay.elems.contains_key(&o) && tw(&d.types, ob.ty) as usize >= 32 * KEPT_MIN && (reg || indexed.contains(&o)) {
                lay.kept.insert(o);
            }
        }
        lay.in_place = lay.kept.iter().copied().filter(|o| lay.regs.contains(o) && !ff_reads.contains(o)).collect();
        for node in code {
            if let Node::In(i) = node {
                lay.code.push(Node::In(i));
                lay.code.push(Node::Call(i));
            } else {
                lay.code.push(node);
            }
        }
        // An array whose call would wait on itself (an iteration's input
        // made from another's output) is called an instance at a time.
        self.layouts.insert(key, lay);
        let arrays: Vec<InstId> = match self.order_of(key) {
            Ok(_) => Vec::new(),
            Err(stuck) => {
                let code = &self.layouts[&key].code;
                let mut v: Vec<InstId> = stuck
                    .iter()
                    .filter_map(|&n| if let Node::In(i) | Node::Call(i) = code[n] { Some(i) } else { None })
                    .filter(|i| self.arrays.contains_key(i))
                    .collect();
                v.sort_unstable_by_key(|i| i.0);
                v.dedup();
                v
            }
        };
        if !arrays.is_empty() {
            let split: HashMap<InstId, Vec<InstId>> = arrays.iter().map(|i| (*i, self.arrays.remove(i).unwrap())).collect();
            let entry = self.codes.get_mut(&spec).unwrap();
            entry.1 = entry
                .1
                .iter()
                .flat_map(|&n| match n {
                    Node::In(i) => split.get(&i).cloned().unwrap_or_else(|| vec![i]).into_iter().map(Node::In).collect(),
                    n => vec![n],
                })
                .collect();
            let (_, code) = self.code_of(spec)?;
            let mut nodes = Vec::new();
            for node in code {
                match node {
                    Node::In(i) => {
                        nodes.push(Node::In(i));
                        nodes.push(Node::Call(i));
                    }
                    Node::Proc(p) if self.clocked(p).is_some() => {}
                    n => nodes.push(n),
                }
            }
            self.layouts.get_mut(&key).unwrap().code = nodes;
        }
        let mut lay = self.layouts.remove(&key).unwrap();
        for node in lay.code.clone() {
            if let Node::In(i) = node {
                let ck = (d.instance(i).spec, self.child_lanes(sh, lanes, i));
                self.layout(ck)?;
                lay.kid_keys.insert(i, ck);
            }
        }
        lay.clocked = !lay.ff.is_empty() || lay.kid_keys.values().any(|k| self.layouts[k].clocked);
        self.layouts.insert(key, lay);
        self.split_calls(key)?;
        if let Some(split) = self.parts.get(&spec).cloned() {
            let part = self.part_of(key, &split);
            self.layouts.get_mut(&key).unwrap().part = part;
        }
        let below_top = self.top != Some(key);
        // Below the top, ports are the function's pointer parameters.
        let port_lanes = |lay: &mut Layout| {
            for (k, p) in d.spec(spec).ports.iter().enumerate() {
                if let Some(o) = p.object {
                    let lane = Lane::of(tw(&d.types, d.object(o).ty), sh, planes);
                    if lane != Lane::W32 {
                        lay.lanes.insert(o, lane);
                    }
                    if below_top {
                        lay.port_ix.insert(o, k);
                    }
                }
            }
        };
        // A specialisation proved to count zeros keeps only its ports.
        if self.opts.recognise
            && let Some(k) = self.recognise(key)
        {
            let mut lay = Layout::new(sh);
            port_lanes(&mut lay);
            for p in &d.spec(spec).ports {
                let o = p.object.expect("recognised specialisations have plain ports");
                lay.statics.insert(o);
                if !below_top {
                    let w = tw(&d.types, d.object(o).ty);
                    lay.place(o, w, Lane::of(w, sh, planes));
                }
            }
            lay.known = Some(k);
            self.layouts.insert(key, lay);
            return Ok(());
        }
        let mut lay = self.layouts.remove(&key).unwrap();
        port_lanes(&mut lay);
        // What each child's ports are connected to: the parent's own values
        // where a single child's are connected whole.
        let mut args: Vec<(InstId, Vec<Option<ObjId>>)> = Vec::new();
        for node in &lay.code {
            if let Node::In(i) = *node {
                let inst = d.instance(i);
                let single = self.members(i).len() == 1;
                let row = d
                    .spec(inst.spec)
                    .ports
                    .iter()
                    .map(|p| {
                        let po = p.object?;
                        let e = inst.connections.iter().find(|c| c.port == p.name)?.expr?;
                        if single { self.alias_target(&lay, e, po) } else { None }
                    })
                    .collect();
                args.push((i, row));
            }
        }
        let aliased: HashSet<ObjId> = args.iter().flat_map(|a| a.1.iter().flatten().copied()).collect();
        self.layouts.insert(key, lay);
        let mut local = self.loop_locals(key)?;
        local.retain(|o| !ff_reads.contains(o) && !self.layouts[&key].kept.contains(o));
        let fresh = self.fresh_values(key)?;
        let mut lay = self.layouts.remove(&key).unwrap();
        let piece = if sh.scalar() { Lane::W64 } else { Lane::W32 };
        for o in objs {
            if local.contains(&o) && !aliased.contains(&o) || lay.port_ix.contains_key(&o) {
                continue;
            }
            let w = tw(&d.types, d.object(o).ty);
            match (lay.elems.get(&o), lay.pads.get(&o)) {
                (Some(&e), _) => {
                    let at = lay.room(e.n as u64 * e.stride(), e.lane.reg_bytes(sh));
                    lay.objs.insert(o, at);
                }
                (None, Some(p)) => lay.place(o, p.width(), piece),
                // (Kept in the state: in whole pieces.)
                (None, None) if lay.kept.contains(&o) => lay.place(o, w, piece),
                (None, None) => lay.place(o, w, Lane::of(w, sh, planes)),
            }
        }
        lay.fresh = fresh;
        lay.fresh.retain(|o| !lay.kept.contains(o));
        // The registers' next values, and the edges' last values. Only a
        // port below the top needs a slot for its next value: its storage
        // may be its parent's, read by the parent's clocked processes,
        // which run after its own. The rest are written in place.
        for o in lay.regs.clone() {
            if !(below_top && d.object(o).dir.is_some()) {
                continue;
            }
            let w = tw(&d.types, d.object(o).ty);
            let lane = lay.lane(o);
            let at = lay.room_for(w, lane);
            lay.shadow.insert(o, at);
        }
        if !lay.ff.is_empty() {
            lay.fired = lay.room(1, 1);
        }
        for p in lay.ff.clone() {
            for t in self.clocked(p).unwrap().0 {
                if !lay.prev.contains_key(&t.expr) {
                    let at = lay.room_for(1, Lane::of(1, sh, planes));
                    lay.prev.insert(t.expr, at);
                }
            }
        }
        // Slots for the ports not connected to a value whole, laid out as
        // the child's values are.
        for (i, row) in args {
            let ck = lay.kid_keys[&i];
            let csh = Shape::of(ck.1);
            let mut out = Vec::new();
            for (p, alias) in d.spec(ck.0).ports.iter().zip(row) {
                out.push(match alias {
                    Some(o) => Arg::Alias(o),
                    None => {
                        let po = p.object.ok_or("interface ports are not supported yet")?;
                        let w = tw(&d.types, d.object(po).ty);
                        let lane = Lane::of(w, csh, planes);
                        Arg::Slot(lay.room(lane.bytes(w, csh), lane.reg_bytes(csh)))
                    }
                });
            }
            lay.args.insert(i, out);
        }
        for node in lay.code.clone() {
            if let Node::In(i) = node {
                let size = self.layouts[&lay.kid_keys[&i]].size;
                let at = lay.room(size, 64);
                lay.kids.insert(i, at);
            }
        }
        self.layouts.insert(key, lay);
        Ok(())
    }

    /// The parent's value a port connection is, whole, if the child can
    /// use it where it is: a variable, net or port of the same width (any
    /// conversion between them changing no bits), not kept an element to
    /// a slot.
    fn alias_target(&self, lay: &Layout, e: ExprId, port: ObjId) -> Option<ObjId> {
        let d = self.d;
        let mut x = d.expr(e);
        let w = tw(&d.types, d.object(port).ty);
        while matches!(x.kind, ExprKind::Convert) && tw(&d.types, x.ty) == w && d.types.is_integral(d.expr(x.args[0]).ty) {
            x = d.expr(x.args[0]);
        }
        match x.kind {
            ExprKind::Object(o)
                if matches!(d.object(o).kind, ObjKind::Variable | ObjKind::Net(_))
                    && lay.statics.contains(&o)
                    && !lay.elems.contains_key(&o)
                    && !lay.pads.contains_key(&o)
                    && tw(&d.types, d.object(o).ty) == w =>
            {
                Some(o)
            }
            _ => None,
        }
    }

    /// The internal values worked out and used within one loop over the
    /// registers, written before they are read there: they need no slot in
    /// the state.
    fn loop_locals(&self, key: Key) -> Result<HashSet<ObjId>, String> {
        let d = self.d;
        let lay = &self.layouts[&key];
        let (order, second) = self.sequence(key)?;
        // The loop each node runs in, as Func::body splits them.
        let mut seg = 0usize;
        let mut rolled = HashSet::new();
        // For each object: the loops it is used in, and where in the order
        // it is first written and first read only.
        let mut used: HashMap<ObjId, (HashSet<usize>, usize, usize)> = HashMap::new();
        for (pos, &node) in order.iter().enumerate() {
            if pos == second {
                seg += 1;
            }
            let at = match node {
                Node::Loop(_) => {
                    seg += 1;
                    rolled.insert(seg);
                    let at = seg;
                    seg += 1;
                    at
                }
                Node::Call(_) | Node::Early(_) => {
                    seg += 1;
                    seg
                }
                _ => seg,
            };
            let (r, w) = self.access(key, node);
            let writes: HashSet<ObjId> = w.iter().map(|t| t.0).collect();
            for (o, written) in w.iter().map(|t| (t.0, true)).chain(r.iter().filter(|t| !writes.contains(&t.0)).map(|t| (t.0, false))) {
                let e = used.entry(o).or_insert_with(|| (HashSet::new(), usize::MAX, usize::MAX));
                e.0.insert(at);
                if written {
                    e.1 = e.1.min(pos);
                } else {
                    e.2 = e.2.min(pos);
                }
            }
        }
        Ok(used
            .into_iter()
            .filter(|(o, (segs, first_write, first_read))| {
                lay.statics.contains(o)
                    && d.object(*o).dir.is_none()
                    && !lay.elems.contains_key(o)
                    && segs.len() == 1
                    && !segs.iter().any(|s| rolled.contains(s))
                    && *first_write != usize::MAX
                    && first_write <= first_read
            })
            .map(|(o, _)| o)
            .collect())
    }

    /// The values written only by continuous assignments, all of them in
    /// one loop over the registers (see `Layout::fresh`).
    fn fresh_values(&self, key: Key) -> Result<HashSet<ObjId>, String> {
        let lay = &self.layouts[&key];
        let mut seg = 0usize;
        // For each value written: the loop its writers are in, if one, and
        // whether they are all continuous assignments.
        let mut writers: HashMap<ObjId, (Option<usize>, bool)> = HashMap::new();
        let (order, second) = self.sequence(key)?;
        for (pos, node) in order.into_iter().enumerate() {
            if pos == second {
                seg += 1;
            }
            match node {
                Node::Loop(_) => seg += 2,
                Node::Call(_) | Node::Early(_) => seg += 1,
                _ => {}
            }
            let (_, w) = self.access(key, node);
            for (o, _, _) in w {
                let e = writers.entry(o).or_insert((Some(seg), true));
                if e.0 != Some(seg) {
                    e.0 = None;
                }
                e.1 &= matches!(node, Node::Driver(_));
            }
        }
        Ok(writers
            .into_iter()
            .filter(|(o, (one, drivers))| one.is_some() && *drivers && lay.statics.contains(o) && !lay.elems.contains_key(o) && !lay.regs.contains(o))
            .map(|(o, _)| o)
            .collect())
    }

    /// The static values and the code of a scope and the generate blocks
    /// in it.
    fn collect(&mut self, scope: ScopeId, objs: &mut Vec<ObjId>, code: &mut Vec<Node>) -> Result<(), String> {
        let d = self.d;
        for item in &d.scope(scope).items {
            match *item {
                Item::Object(o) => {
                    let ob = d.object(o);
                    if matches!(ob.kind, ObjKind::Variable | ObjKind::Net(_)) && ob.lifetime == Lifetime::Static {
                        objs.push(o);
                    }
                }
                Item::Process(p) => {
                    let pr = &d.processes[p.0 as usize];
                    match pr.kind {
                        ProcKind::AlwaysComb => {
                            self.process_statics(pr.body, objs);
                            code.push(Node::Proc(p));
                        }
                        // Clocked processes, set apart by `layout`.
                        ProcKind::AlwaysFf | ProcKind::Always if self.clocked(p).is_some() => {
                            self.process_statics(pr.body, objs);
                            code.push(Node::Proc(p));
                        }
                        ProcKind::AlwaysFf => return Err("an always_ff that does not wait on edges is not supported yet".into()),
                        ProcKind::Initial if self.only_asserts(pr.body) => self.dropped.set(self.dropped.get() + 1),
                        k => return Err(format!("{k:?} processes are not supported yet")),
                    }
                }
                Item::Driver(dr) => {
                    let drv = &d.drivers[dr.0 as usize];
                    if !drv.delay.is_empty() || drv.strength.is_some() {
                        return Err("continuous assignments with delays or strengths are not supported yet".into());
                    }
                    code.push(Node::Driver(dr));
                }
                // An array's instances are one node, its first's.
                Item::Instance(i) if !self.arrays.values().any(|m| m[0] != i && m.contains(&i)) => code.push(Node::In(i)),
                Item::Instance(_) => {}
                Item::Cell(_) => return Err("gates and UDPs are not supported yet".into()),
                Item::Scope(s) => match d.scope(s).kind {
                    ScopeKind::Generate if self.is_generate_loop(s) => self.generate_loop(s, objs, code)?,
                    ScopeKind::Generate | ScopeKind::Block => self.collect(s, objs, code)?,
                    _ => {}
                },
                Item::Opaque(_) => self.dropped.set(self.dropped.get() + 1),
                Item::Typedef(..) | Item::Subroutine(_) | Item::Class(_) | Item::Import(..) => {}
            }
        }
        Ok(())
    }

    /// The objects that some select indexes by a value.
    fn indexed(&self, code: &[Node], ff: &[ProcId]) -> HashSet<ObjId> {
        let d = self.d;
        let mut exprs = Vec::new();
        for node in code {
            match *node {
                Node::Driver(dr) => exprs.extend([d.drivers[dr.0 as usize].lhs, d.drivers[dr.0 as usize].rhs]),
                Node::Loop(l) => {
                    for &dr in &self.loops[l].all {
                        exprs.extend([d.drivers[dr.0 as usize].lhs, d.drivers[dr.0 as usize].rhs]);
                    }
                }
                Node::Proc(p) => self.stmt_exprs(d.processes[p.0 as usize].body, &mut exprs),
                _ => {}
            }
        }
        for &p in ff {
            self.stmt_exprs(d.processes[p.0 as usize].body, &mut exprs);
        }
        let mut indexed = HashSet::new();
        while let Some(e) = exprs.pop() {
            let x = d.expr(e);
            if let ExprKind::Index = x.kind
                && let Some(&base) = x.args.first()
                && let ExprKind::Object(o) = &d.expr(base).kind
                && self.const_select(e).is_none()
            {
                indexed.insert(*o);
            }
            exprs.extend(x.args.iter().copied());
        }
        indexed
    }

    /// The arrays to keep padded (see [`Pad`]): internal ones of at least
    /// two elements wider than a piece and not a whole number of pieces,
    /// that some select indexes by a value.
    fn padded(&self, objs: &[ObjId], indexed: &HashSet<ObjId>, sh: Shape) -> HashMap<ObjId, Pad> {
        let d = self.d;
        let mut out = HashMap::new();
        for &o in objs {
            let ob = d.object(o);
            if ob.dir.is_some() || !indexed.contains(&o) || !(d.types.packed_range(ob.ty).is_some() || d.types.is_unpacked_array(ob.ty)) {
                continue;
            }
            let (w, ew) = (tw(&d.types, ob.ty), elem_bits(&d.types, ob.ty));
            let pb = sh.pb();
            if ew > pb && ew % pb != 0 && w % ew == 0 && w / ew >= 2 {
                out.insert(o, Pad { ew, n: w / ew, pb });
            }
        }
        out
    }

    /// The expressions a statement holds, at their tops.
    fn stmt_exprs(&self, s: StmtId, out: &mut Vec<ExprId>) {
        let d = self.d;
        match &d.stmt(s).kind {
            StmtKind::Block { stmts, .. } => stmts.iter().for_each(|&s| self.stmt_exprs(s, out)),
            StmtKind::Assign { lhs, rhs, .. } => out.extend([*lhs, *rhs]),
            StmtKind::Expr(e) | StmtKind::Return(Some(e)) => out.push(*e),
            StmtKind::If { cond, then, else_, .. } => {
                out.push(*cond);
                self.stmt_exprs(*then, out);
                if let Some(e) = else_ {
                    self.stmt_exprs(*e, out);
                }
            }
            StmtKind::Case { expr, items, default, .. } => {
                out.push(*expr);
                for (labels, body) in items {
                    out.extend(labels.iter().copied());
                    self.stmt_exprs(*body, out);
                }
                if let Some(s) = default {
                    self.stmt_exprs(*s, out);
                }
            }
            StmtKind::For { init, cond, step, body, .. } => {
                init.iter().chain(step).for_each(|&s| self.stmt_exprs(s, out));
                out.extend(cond.iter().copied());
                self.stmt_exprs(*body, out);
            }
            _ => {}
        }
    }

    /// The objects that every use selects one element of, with the same
    /// element width: these are kept an element to a slot. Ports are not,
    /// so that a parent and a child meet in one layout.
    fn element_only(&self, objs: &[ObjId], code: &[Node], sh: Shape) -> HashMap<ObjId, Elems> {
        let d = self.d;
        let mut uses: HashMap<ObjId, Option<u32>> = HashMap::new();
        for node in code {
            match *node {
                Node::Driver(dr) => {
                    let drv = &d.drivers[dr.0 as usize];
                    self.expr_uses(drv.lhs, &mut uses);
                    self.expr_uses(drv.rhs, &mut uses);
                }
                Node::Loop(l) => {
                    for &dr in &self.loops[l].all {
                        let drv = &d.drivers[dr.0 as usize];
                        self.expr_uses(drv.lhs, &mut uses);
                        self.expr_uses(drv.rhs, &mut uses);
                    }
                }
                Node::Proc(p) => self.stmt_uses(d.processes[p.0 as usize].body, &mut uses),
                Node::In(i) | Node::Call(i) | Node::Pre(i) | Node::Early(i) => {
                    for m in self.members(i) {
                        for c in &d.instance(m).connections {
                            if let Some(e) = c.expr {
                                self.expr_uses(e, &mut uses);
                            }
                        }
                    }
                }
            }
        }
        let mut out = HashMap::new();
        for &o in objs {
            let ob = d.object(o);
            let w = tw(&d.types, ob.ty);
            if let (None, Some(Some(ew))) = (ob.dir, uses.get(&o))
                && *ew <= sh.pb()
                && w % ew == 0
                && w / ew >= 2
                && (d.types.packed_range(ob.ty).is_some() || d.types.is_unpacked_array(ob.ty))
            {
                let lane = match Lane::of(*ew, sh, self.opts.bit_planes) {
                    Lane::B8 | Lane::B16 if !sh.scalar() => Lane::W32,
                    l => l,
                };
                out.insert(o, Elems { ew: *ew, n: w / ew, lane, stride: lane.piece_bytes(sh) });
            }
        }
        out
    }

    /// Notes each object an expression uses: `Some(width)` while every use
    /// selects one element of that width, `None` once one does not.
    fn expr_uses(&self, e: ExprId, uses: &mut HashMap<ObjId, Option<u32>>) {
        let d = self.d;
        let x = d.expr(e);
        let mut note = |o: ObjId, u: Option<u32>| {
            let slot = uses.entry(o).or_insert(u);
            if *slot != u {
                *slot = None;
            }
        };
        match &x.kind {
            ExprKind::Object(o) => note(*o, None),
            ExprKind::Index if matches!(d.expr(x.args[0]).kind, ExprKind::Object(_)) => {
                let ExprKind::Object(o) = d.expr(x.args[0]).kind else { unreachable!() };
                note(o, Some(tw(&d.types, x.ty)));
                self.expr_uses(x.args[1], uses);
            }
            _ => {
                for &a in &x.args {
                    self.expr_uses(a, uses);
                }
            }
        }
    }

    fn stmt_uses(&self, s: StmtId, uses: &mut HashMap<ObjId, Option<u32>>) {
        let d = self.d;
        let kind = &d.stmt(s).kind;
        if !matches!(kind, StmtKind::Null | StmtKind::Block { .. }) && self.only_asserts(s) {
            return;
        }
        match kind {
            StmtKind::Block { stmts, .. } => stmts.iter().for_each(|&s| self.stmt_uses(s, uses)),
            StmtKind::Assign { lhs, rhs, .. } => {
                self.expr_uses(*lhs, uses);
                self.expr_uses(*rhs, uses);
            }
            StmtKind::Expr(e) => self.expr_uses(*e, uses),
            StmtKind::If { cond, then, else_, .. } => {
                self.expr_uses(*cond, uses);
                self.stmt_uses(*then, uses);
                if let Some(e) = else_ {
                    self.stmt_uses(*e, uses);
                }
            }
            StmtKind::Case { expr, items, default, .. } => {
                self.expr_uses(*expr, uses);
                for (labels, body) in items {
                    labels.iter().for_each(|&l| self.expr_uses(l, uses));
                    self.stmt_uses(*body, uses);
                }
                if let Some(s) = default {
                    self.stmt_uses(*s, uses);
                }
            }
            StmtKind::For { init, cond, step, body, .. } => {
                init.iter().chain(step).for_each(|&s| self.stmt_uses(s, uses));
                if let Some(c) = cond {
                    self.expr_uses(*c, uses);
                }
                self.stmt_uses(*body, uses);
            }
            _ => {}
        }
    }

    /// The static variables declared in a process's blocks and loops.
    fn process_statics(&self, s: StmtId, objs: &mut Vec<ObjId>) {
        let d = self.d;
        let scope_objs = |sc: ScopeId, objs: &mut Vec<ObjId>| {
            for item in &d.scope(sc).items {
                if let Item::Object(o) = *item
                    && d.object(o).kind == ObjKind::Variable
                    && d.object(o).lifetime == Lifetime::Static
                    && !objs.contains(&o)
                {
                    objs.push(o);
                }
            }
        };
        match &d.stmt(s).kind {
            StmtKind::Block { scope, stmts, .. } => {
                if let Some(sc) = scope {
                    scope_objs(*sc, objs);
                }
                stmts.iter().for_each(|&s| self.process_statics(s, objs));
            }
            StmtKind::For { scope, body, .. } => {
                if let Some(sc) = scope {
                    scope_objs(*sc, objs);
                }
                self.process_statics(*body, objs);
            }
            StmtKind::If { then, else_, .. } => {
                self.process_statics(*then, objs);
                if let Some(e) = else_ {
                    self.process_statics(*e, objs);
                }
            }
            StmtKind::Case { items, default, .. } => {
                items.iter().for_each(|i| self.process_statics(i.1, objs));
                if let Some(s) = default {
                    self.process_statics(*s, objs);
                }
            }
            _ => {}
        }
    }

    /// Whether a generate scope holds a loop's iterations.
    fn is_generate_loop(&self, s: ScopeId) -> bool {
        let d = self.d;
        let mut kids = d.scope(s).items.iter().filter_map(|i| if let Item::Scope(k) = i { Some(*k) } else { None }).peekable();
        kids.peek().is_some() && kids.all(|k| d.scope(k).kind == ScopeKind::Generate && d.scope(k).index.is_some())
    }

    /// A generate loop's iterations: runs of them with the same code become
    /// loops; the rest are as written.
    fn generate_loop(&mut self, s: ScopeId, objs: &mut Vec<ObjId>, code: &mut Vec<Node>) -> Result<(), String> {
        let d = self.d;
        let iters: Vec<ScopeId> = d.scope(s).items.iter().filter_map(|i| if let Item::Scope(k) = i { Some(*k) } else { None }).collect();
        // Where every iteration has instances of the same specialisations,
        // in the same order, the instances at each place are an array.
        let insts: Vec<Vec<InstId>> = iters
            .iter()
            .map(|&i| {
                let mut v = Vec::new();
                self.iteration_instances(i, &mut v);
                v
            })
            .collect();
        if iters.len() >= 2
            && !insts[0].is_empty()
            && insts.iter().all(|v| v.len() == insts[0].len() && v.iter().zip(&insts[0]).all(|(a, b)| d.instance(*a).spec == d.instance(*b).spec))
        {
            for j in 0..insts[0].len() {
                self.arrays.insert(insts[0][j], insts.iter().map(|v| v[j]).collect());
            }
        }
        // Runs of iterations of the same shape.
        let shapes: Vec<Option<String>> = iters.iter().map(|&i| self.shape(i)).collect();
        let mut at = 0;
        while at < iters.len() {
            let mut end = at + 1;
            while end < iters.len() && shapes[at].is_some() && shapes[end] == shapes[at] {
                end += 1;
            }
            let big = || {
                let (mut drivers, mut params) = (Vec::new(), Vec::new());
                self.gather(iters[at], &mut drivers, &mut params);
                let body: usize = drivers.iter().map(|dr| self.expr_size(d.drivers[dr.0 as usize].lhs) + self.expr_size(d.drivers[dr.0 as usize].rhs)).sum();
                body * (end - at) > if body <= TINY_BODY { UNROLL_TINY_MAX } else { UNROLL_MAX }
            };
            let rolled = if end - at >= ROLL_MIN && big() { self.roll(&iters[at..end]) } else { None };
            match rolled {
                Some(r) => {
                    code.push(Node::Loop(self.loops.len()));
                    self.loops.push(r);
                }
                None => {
                    for &i in &iters[at..end] {
                        self.collect(i, objs, code)?;
                    }
                }
            }
            at = end;
        }
        Ok(())
    }

    /// The instances in an iteration of a generate loop, and in the generate
    /// blocks in it that are not loops, in order.
    fn iteration_instances(&self, s: ScopeId, out: &mut Vec<InstId>) {
        let d = self.d;
        for item in &d.scope(s).items {
            match *item {
                Item::Instance(i) => out.push(i),
                Item::Scope(k) if d.scope(k).kind == ScopeKind::Generate && !self.is_generate_loop(k) => self.iteration_instances(k, out),
                _ => {}
            }
        }
    }

    /// An iteration's code, as the source nodes it was made from, if it is
    /// only continuous assignments and parameters (in nested generate
    /// blocks that are not loops).
    fn shape(&self, s: ScopeId) -> Option<String> {
        let d = self.d;
        let mut out = String::new();
        for item in &d.scope(s).items {
            match *item {
                Item::Object(o) if matches!(d.object(o).kind, ObjKind::Param { .. }) => {
                    let _ = write!(out, "p{} ", d.name(d.object(o).name));
                }
                Item::Driver(dr) => {
                    let drv = &d.drivers[dr.0 as usize];
                    if !drv.delay.is_empty() || drv.strength.is_some() {
                        return None;
                    }
                    let _ = write!(out, "d{:?} ", drv.node);
                }
                Item::Scope(k) if d.scope(k).kind == ScopeKind::Generate && !self.is_generate_loop(k) => {
                    let _ = write!(out, "s{:?}({}) ", d.scope(k).node, self.shape(k)?);
                }
                Item::Typedef(..) | Item::Import(..) => {}
                _ => return None,
            }
        }
        Some(out)
    }

    /// An iteration's continuous assignments and parameters, in order.
    fn gather(&self, s: ScopeId, drivers: &mut Vec<DriverId>, params: &mut Vec<ObjId>) {
        let d = self.d;
        for item in &d.scope(s).items {
            match *item {
                Item::Object(o) => params.push(o),
                Item::Driver(dr) => drivers.push(dr),
                Item::Scope(k) => self.gather(k, drivers, params),
                _ => {}
            }
        }
    }

    /// The iterations as one loop, if every one's code is the first's with
    /// parameters that step evenly.
    fn roll(&self, iters: &[ScopeId]) -> Option<Rolled> {
        let d = self.d;
        let mut drivers = Vec::new();
        let mut params = Vec::new();
        for &i in iters {
            let (mut dv, mut pv) = (Vec::new(), Vec::new());
            self.gather(i, &mut dv, &mut pv);
            drivers.push(dv);
            params.push(pv);
        }
        let n = iters.len();
        let mut steps = Vec::new();
        for (k, &p) in params[0].iter().enumerate() {
            let val = |t: usize| d.object(params[t][k]).value.and_then(|c| d.konst(c).to_i64());
            let (a, b) = (val(0)?, val(1)?);
            if (0..n).any(|t| val(t) != Some(a + (b - a) * t as i64)) {
                return None;
            }
            if b != a {
                steps.push((p, a, b - a));
            }
        }
        for t in 1..n {
            let map: HashMap<ObjId, ObjId> = params[t].iter().copied().zip(params[0].iter().copied()).collect();
            for (&x, &y) in drivers[0].iter().zip(&drivers[t]) {
                let (x, y) = (&d.drivers[x.0 as usize], &d.drivers[y.0 as usize]);
                if !self.same_expr(x.lhs, y.lhs, &map) || !self.same_expr(x.rhs, y.rhs, &map) {
                    return None;
                }
            }
        }
        // An iteration must not read what another writes, so that running
        // them in order is running them at once.
        let mut writes: HashMap<ObjId, Vec<(u32, u32, usize)>> = HashMap::new();
        let mut reads = Vec::new();
        for (t, dv) in drivers.iter().enumerate() {
            let mut r = Vec::new();
            for &dr in dv {
                let drv = &d.drivers[dr.0 as usize];
                let mut w = Vec::new();
                self.reads(drv.rhs, &mut r);
                self.writes(drv.lhs, &mut w, &mut r);
                for (o, lo, hi) in w {
                    writes.entry(o).or_default().push((lo, hi, t));
                }
            }
            reads.push(r);
        }
        for (t, r) in reads.iter().enumerate() {
            for &(o, lo, hi) in r {
                if writes.get(&o).is_some_and(|v| v.iter().any(|&(wl, wh, u)| u != t && wl < hi && lo < wh)) {
                    return None;
                }
            }
        }
        // Element selects in range in every iteration.
        let mut in_range = HashSet::new();
        let mut ok: HashMap<ExprId, bool> = HashMap::new();
        for dv in &drivers {
            for (&x, &y) in drivers[0].iter().zip(dv) {
                let (x, y) = (&d.drivers[x.0 as usize], &d.drivers[y.0 as usize]);
                let mut pairs = Vec::new();
                self.pair_exprs(x.lhs, y.lhs, &mut pairs);
                self.pair_exprs(x.rhs, y.rhs, &mut pairs);
                for (t, e) in pairs {
                    if matches!(d.expr(t).kind, ExprKind::Index) {
                        let good = self.index_in_range(e);
                        *ok.entry(t).or_insert(true) &= good;
                    }
                }
            }
        }
        in_range.extend(ok.into_iter().filter(|e| e.1).map(|e| e.0));
        Some(Rolled { count: n as u32, body: drivers[0].clone(), steps, all: drivers.concat(), in_range })
    }

    /// A function's variables, its formals among them: those declared in
    /// its scope and the blocks within it, not its result.
    fn sub_locals(&self, sub: SubId) -> Vec<ObjId> {
        if let Some(v) = self.sub_locals.borrow().get(&sub) {
            return v.clone();
        }
        let d = self.d;
        let s = &d.subroutines[sub.0 as usize];
        let mut out: Vec<ObjId> = s.args.clone();
        let mut scopes = vec![s.scope];
        let mut stmts = s.body.clone();
        while let Some(st) = stmts.pop() {
            match &d.stmt(st).kind {
                StmtKind::Block { scope, stmts: b, .. } => {
                    scopes.extend(*scope);
                    stmts.extend(b);
                }
                StmtKind::For { scope, init, step, body, .. } => {
                    scopes.extend(*scope);
                    stmts.extend(init.iter().chain(step).chain([body]));
                }
                StmtKind::If { then, else_, .. } => stmts.extend(std::iter::once(*then).chain(*else_)),
                StmtKind::Case { items, default, .. } => stmts.extend(items.iter().map(|i| i.1).chain(*default)),
                _ => {}
            }
        }
        let result = d.scope(s.scope).names.get(&s.name).copied();
        for sc in scopes {
            for item in &d.scope(sc).items {
                if let Item::Object(o) = *item
                    && d.object(o).kind == ObjKind::Variable
                    && !out.contains(&o)
                    && result != Some(Binding::Object(o))
                {
                    out.push(o);
                }
            }
        }
        self.sub_locals.borrow_mut().insert(sub, out.clone());
        out
    }

    /// How many nodes an expression has.
    fn expr_size(&self, e: ExprId) -> usize {
        1 + self.d.expr(e).args.iter().map(|&a| self.expr_size(a)).sum::<usize>()
    }

    /// The pairs of corresponding nodes of two expressions of the same shape.
    fn pair_exprs(&self, a: ExprId, b: ExprId, out: &mut Vec<(ExprId, ExprId)>) {
        out.push((a, b));
        let (x, y) = (self.d.expr(a), self.d.expr(b));
        for (&p, &q) in x.args.iter().zip(&y.args) {
            self.pair_exprs(p, q, out);
        }
    }

    /// Whether an `Index`'s index is a constant in its base's range.
    fn index_in_range(&self, e: ExprId) -> bool {
        let d = self.d;
        let x = d.expr(e);
        let bt = d.expr(x.args[0]).ty;
        let Some(k) = self.fold(x.args[1]).and_then(|c| c.to_i64()) else { return false };
        let (l, r) = range_of(&d.types, bt);
        k >= l.min(r) && k <= l.max(r)
    }

    /// Whether two expressions are the same, with `map` taking the second's
    /// parameters to the first's.
    fn same_expr(&self, a: ExprId, b: ExprId, map: &HashMap<ObjId, ObjId>) -> bool {
        let d = self.d;
        let (x, y) = (d.expr(a), d.expr(b));
        if x.ty != y.ty || x.args.len() != y.args.len() {
            return false;
        }
        let same = match (&x.kind, &y.kind) {
            (ExprKind::Object(p), ExprKind::Object(q)) => p == q || map.get(q) == Some(p),
            (ExprKind::Const(p), ExprKind::Const(q)) => ops::const_eq(d.konst(*p), d.konst(*q)),
            (k, l) => k == l,
        };
        same && x.args.iter().zip(&y.args).all(|(&p, &q)| self.same_expr(p, q, map))
    }

    /// A process that runs on edges: what it waits on, and its body.
    fn clocked(&self, p: ProcId) -> Option<(&'d [crate::design::EventTerm], StmtId)> {
        let d = self.d;
        let pr = &d.processes[p.0 as usize];
        if !matches!(pr.kind, ProcKind::AlwaysFf | ProcKind::Always) {
            return None;
        }
        match &d.stmt(pr.body).kind {
            StmtKind::Timed(crate::design::Timing::Event(terms), body)
                if !terms.is_empty() && terms.iter().all(|t| matches!(t.edge, crate::design::Edge::Pos | crate::design::Edge::Neg)) =>
            {
                Some((terms.as_slice(), *body))
            }
            _ => None,
        }
    }

    /// What a statement assigns with nonblocking assignments.
    fn nba_writes(&self, s: StmtId, w: &mut Vec<(ObjId, u32, u32)>) {
        let d = self.d;
        match &d.stmt(s).kind {
            StmtKind::Assign { kind: AssignKind::NonBlocking, lhs, .. } => self.writes(*lhs, w, &mut Vec::new()),
            StmtKind::Block { stmts, .. } => stmts.iter().for_each(|&s| self.nba_writes(s, w)),
            StmtKind::If { then, else_, .. } => {
                self.nba_writes(*then, w);
                if let Some(e) = else_ {
                    self.nba_writes(*e, w);
                }
            }
            StmtKind::Case { items, default, .. } => {
                items.iter().for_each(|i| self.nba_writes(i.1, w));
                if let Some(e) = default {
                    self.nba_writes(*e, w);
                }
            }
            StmtKind::For { body, .. } => self.nba_writes(*body, w),
            _ => {}
        }
    }

    /// Whether a statement does nothing but check assertions, so leaving
    /// it out, conditions and all, changes no value. Its assertions are
    /// added to `asserts`.
    fn only_asserts(&self, s: StmtId) -> bool {
        let d = self.d;
        let all = |v: &mut dyn Iterator<Item = StmtId>| v.fold(true, |all, s| self.only_asserts(s) && all);
        match &d.stmt(s).kind {
            StmtKind::Null => true,
            StmtKind::Assert { .. } => {
                self.asserts.borrow_mut().insert(s);
                true
            }
            StmtKind::Block { join: None, stmts, .. } => all(&mut stmts.iter().copied()),
            StmtKind::If { then, else_, .. } => all(&mut std::iter::once(*then).chain(*else_)),
            StmtKind::Case { items, default, .. } => all(&mut items.iter().map(|i| i.1).chain(*default)),
            // The loop variable is the loop's own.
            StmtKind::For { scope: Some(_), body, .. } => self.only_asserts(*body),
            _ => false,
        }
    }

    /// The function for `key`, after its children's.
    fn function(&mut self, key: Key) -> Result<(), String> {
        if !self.done.insert(key) {
            return Ok(());
        }
        let mut kids: Vec<Key> = self.layouts[&key].kid_keys.values().copied().collect();
        kids.sort_unstable_by_key(|k| (k.0.0, k.1));
        kids.dedup();
        for k in kids {
            self.function(k)?;
        }
        let spec = key.0;
        let mut f = Func::new(self, key);
        let mut parts = String::new();
        match &self.layouts[&key].known {
            Some(k) => {
                let k = k.clone();
                f.run_loop_with(|f| f.known(&k))?;
            }
            None => match self.layouts[&key].part.clone() {
                None => {
                    let order = self.order(key)?;
                    f.body(&order)?;
                }
                // The two parts, and the whole as one then the other.
                Some((a, b)) => {
                    let name = fn_name(key);
                    let header = f.out.clone();
                    let params = header[header.find('(').unwrap() + 1..header.find(')').unwrap()].to_string();
                    for (part, nodes) in [("_a", &a), ("_b", &b)] {
                        let mut g = Func::new(self, key);
                        g.out = g.out.replacen(&format!("@{name}("), &format!("@{name}{part}("), 1);
                        g.body(nodes)?;
                        let text = g.finish();
                        let _ = writeln!(parts, "{text}");
                    }
                    let args = params.split(", ").collect::<Vec<_>>().join(", ");
                    let _ = writeln!(f.out, "  call void @{name}_a({args})\n  call void @{name}_b({args})");
                }
            },
        }
        let text = f.finish();
        let _ = writeln!(self.ir, "; {}, {} lanes\n{parts}{text}", self.d.name(self.d.spec(spec).name), key.1);
        if self.layouts[&key].clocked {
            for phase in [Phase::Ff, Phase::Commit] {
                let mut f = Func::new(self, key);
                let any = f.phase(phase)?;
                let text = if phase == Phase::Ff { f.finish_with(&any) } else { f.finish() };
                let _ = writeln!(self.ir, "{text}");
            }
            if self.top == Some(key) {
                let n = fn_name(key);
                let sh = self.layouts[&key].shape;
                let t = tys(sh.n);
                // The inputs compared with their copies, and copied: each a
                // loop over its registers, as they lie one after another.
                let mut cmp = String::new();
                let mut changed = "false".to_string();
                let mut from = "entry".to_string();
                for (k, &(at, lane, w, copy)) in self.watch.iter().enumerate() {
                    let ty = match (sh.scalar(), lane) {
                        (_, Lane::Bit) => "i16",
                        (true, Lane::B8) => "i8",
                        (true, Lane::B16) => "i16",
                        (true, Lane::W32) => "i32",
                        (_, Lane::W64) => "i64",
                        (false, Lane::B8) => t.b8,
                        (false, Lane::B16) => t.b16,
                        (false, Lane::W32) => t.w32,
                    };
                    let (unit, units) = (lane.reg_bytes(sh), lane.pieces(w) * sh.chunks as u64);
                    let zero = if ty.starts_with('<') { "zeroinitializer" } else { "0" };
                    let _ = write!(
                        cmp,
                        "  br label %W{k}\nW{k}:\n  %w{k} = phi i64 [ 0, %{from} ], [ %w{k}.next, %W{k} ]\n  \
                         %w{k}.in = phi {ty} [ {zero}, %{from} ], [ %w{k}.acc, %W{k} ]\n  %w{k}.a = add i64 %w{k}, {at}\n  \
                         %w{k}.p = getelementptr i8, ptr %s, i64 %w{k}.a\n  %w{k}.v = load {ty}, ptr %w{k}.p, align {unit}\n  \
                         %w{k}.b = add i64 %w{k}, {copy}\n  %w{k}.q = getelementptr i8, ptr %s, i64 %w{k}.b\n  \
                         %w{k}.u = load {ty}, ptr %w{k}.q, align {unit}\n  store {ty} %w{k}.v, ptr %w{k}.q, align {unit}\n  \
                         %w{k}.x = xor {ty} %w{k}.v, %w{k}.u\n  %w{k}.acc = or {ty} %w{k}.in, %w{k}.x\n  \
                         %w{k}.next = add i64 %w{k}, {unit}\n  %w{k}.end = icmp eq i64 %w{k}.next, {}\n  \
                         br i1 %w{k}.end, label %X{k}, label %W{k}\nX{k}:\n",
                        unit * units
                    );
                    // Whether any lane of it differs.
                    if ty.starts_with('<') {
                        let _ = write!(
                            cmp,
                            "  %w{k}.m = icmp ne {ty} %w{k}.acc, zeroinitializer\n  %w{k}.i = bitcast {} %w{k}.m to {}\n  \
                             %w{k}.ne = icmp ne {} %w{k}.i, 0\n",
                            t.m, t.bits, t.bits
                        );
                    } else {
                        let _ = writeln!(cmp, "  %w{k}.ne = icmp ne {ty} %w{k}.acc, 0");
                    }
                    let _ = writeln!(cmp, "  %w{k}.any = or i1 {changed}, %w{k}.ne");
                    changed = format!("%w{k}.any");
                    from = format!("X{k}");
                }
                let settled = self.settled;
                let settle = self.timed("step settle".into(), &format!("call void @{n}(ptr %s)"), "q0");
                let ff = self.timed("step ff".into(), &format!("%fired = call i1 @{n}_ff(ptr %s)"), "q1");
                let commit = self.timed("step commit".into(), &format!("call void @{n}_commit(ptr %s)"), "q2");
                let again = self.timed("step settle again".into(), &format!("call void @{n}(ptr %s)"), "q3");
                let _ = writeln!(
                    self.ir,
                    "; a step: settle, unless no input it reads has changed since it last did,\n\
                     ; sample the registers, and where an edge came, commit them and settle again\n\
                     define void @{n}_step(ptr %s) {{\nentry:\n{cmp}  %same = xor i1 {changed}, true\n  \
                     %settled.p = getelementptr i8, ptr %s, i64 {settled}\n  %settled = load i8, ptr %settled.p, align 1\n  \
                     %was = icmp ne i8 %settled, 0\n  %skip = and i1 %same, %was\n  br i1 %skip, label %sample, label %settle\n\
                     settle:\n{settle}  store i8 1, ptr %settled.p, align 1\n  br label %sample\n\
                     sample:\n{ff}  \
                     br i1 %fired, label %commit, label %done\ncommit:\n{commit}{again}  \
                     br label %done\ndone:\n  ret void\n}}\n"
                );
            }
        }
        Ok(())
    }

    /// The specialisation's code in an order where everything is worked
    /// out before it is read: a topological sort on the bits each node
    /// reads and writes.
    fn order(&self, key: Key) -> Result<Vec<Node>, String> {
        self.order_of(key).map_err(|stuck| {
            let nodes = &self.layouts[&key].code;
            let some: Vec<String> = stuck.iter().take(4).map(|&n| format!("{:?}", nodes[n])).collect();
            format!(
                "a combinational loop in {}: {} nodes wait on each other, such as {}",
                self.d.name(self.d.spec(key.0).name),
                stuck.len(),
                some.join(", ")
            )
        })
    }

    /// [`Self::order`], or the nodes left waiting on each other.
    fn order_of(&self, key: Key) -> Result<Vec<Node>, Vec<usize>> {
        let lay = &self.layouts[&key];
        let nodes = &lay.code;
        let mut reads = Vec::with_capacity(nodes.len());
        let mut writers: HashMap<ObjId, Vec<(u32, u32, usize)>> = HashMap::new();
        for (n, node) in nodes.iter().enumerate() {
            let (r, w) = self.access(key, *node);
            for (o, lo, hi) in w {
                writers.entry(o).or_default().push((lo, hi, n));
            }
            reads.push(r);
        }
        let mut succ: Vec<Vec<usize>> = vec![Vec::new(); nodes.len()];
        let mut preds = vec![0usize; nodes.len()];
        for (n, r) in reads.iter().enumerate() {
            let mut from = Vec::new();
            for &(o, lo, hi) in r {
                for &(wlo, whi, w) in writers.get(&o).map(|v| v.as_slice()).unwrap_or(&[]) {
                    if w != n && wlo < hi && lo < whi {
                        from.push(w);
                    }
                }
            }
            // A child's inputs before its call; a first call before the
            // inputs of the second.
            let at = |x: Node| nodes.iter().position(|&y| y == x).unwrap();
            match nodes[n] {
                Node::Call(i) => from.push(at(Node::In(i))),
                Node::Early(i) => from.push(at(Node::Pre(i))),
                Node::In(i) if lay.splits.contains_key(&i) => from.push(at(Node::Early(i))),
                _ => {}
            }
            from.sort_unstable();
            from.dedup();
            preds[n] = from.len();
            for w in from {
                succ[w].push(n);
            }
        }
        let mut ready: BinaryHeap<std::cmp::Reverse<usize>> =
            (0..nodes.len()).filter(|&n| preds[n] == 0).map(std::cmp::Reverse).collect();
        let mut order = Vec::with_capacity(nodes.len());
        while let Some(std::cmp::Reverse(n)) = ready.pop() {
            order.push(nodes[n]);
            for &s in &succ[n] {
                preds[s] -= 1;
                if preds[s] == 0 {
                    ready.push(std::cmp::Reverse(s));
                }
            }
        }
        if order.len() < nodes.len() {
            // Those left wait on a loop or are on one: the loops, with what
            // only waits on them taken off, a node no other left waits on
            // at a time.
            let mut left: Vec<bool> = (0..nodes.len()).map(|n| preds[n] > 0).collect();
            loop {
                let sinks: Vec<usize> = (0..nodes.len()).filter(|&n| left[n] && !succ[n].iter().any(|&s| left[s])).collect();
                if sinks.is_empty() {
                    break;
                }
                sinks.into_iter().for_each(|n| left[n] = false);
            }
            return Err((0..nodes.len()).filter(|&n| left[n]).collect());
        }
        Ok(order)
    }

    /// The bits a node reads and writes: (object, from, to) with `to`
    /// exclusive.
    fn access(&self, key: Key, node: Node) -> (Vec<Span>, Vec<Span>) {
        let d = self.d;
        let lay = &self.layouts[&key];
        let mut r = Vec::new();
        let mut w = Vec::new();
        let whole = |o: ObjId| (o, 0, tw(&d.types, d.object(o).ty).max(1));
        match node {
            Node::Proc(p) => {
                // The bits it reads and writes, as its statements select
                // them, so that processes that each work out some fields
                // of one struct are ordered by those fields.
                let pr = &d.processes[p.0 as usize];
                let (mut pr_r, mut pr_w) = (Vec::new(), Vec::new());
                self.stmt_access(pr.body, &mut pr_r, &mut pr_w, &mut Vec::new());
                r.extend(pr_r.into_iter().filter(|t| lay.statics.contains(&t.0)));
                w.extend(pr_w.into_iter().filter(|t| lay.statics.contains(&t.0)));
                let _ = whole;
            }
            Node::Driver(dr) => {
                let drv = &d.drivers[dr.0 as usize];
                self.reads(drv.rhs, &mut r);
                self.writes(drv.lhs, &mut w, &mut r);
            }
            Node::Loop(l) => {
                for &dr in &self.loops[l].all {
                    let drv = &d.drivers[dr.0 as usize];
                    self.reads(drv.rhs, &mut r);
                    self.writes(drv.lhs, &mut w, &mut r);
                }
            }
            Node::In(i) | Node::Pre(i) => {
                let pre = if let Node::Pre(_) = node { lay.splits.get(&i) } else { None };
                for (p, e) in self.member_connections(i) {
                    if p.dir == Dir::Input && pre.is_none_or(|s| p.object.is_some_and(|o| s.ins.contains(&o))) {
                        self.reads(e, &mut r);
                    }
                }
            }
            Node::Call(i) | Node::Early(i) => {
                let split = lay.splits.get(&i);
                for (p, e) in self.member_connections(i) {
                    let early = split.is_some_and(|s| p.object.is_some_and(|o| s.outs.contains(&o)));
                    if p.dir == Dir::Output && (split.is_none() || early == matches!(node, Node::Early(_))) {
                        self.writes(e, &mut w, &mut r);
                    }
                }
            }
        }
        (r, w)
    }

    /// `call`, a line of IR, timed into a counter for `what` where
    /// profiling; `t` names its temporaries.
    fn timed(&self, what: String, call: &str, t: &str) -> String {
        let Some(counter) = self.opts.profile else { return format!("  {call}\n") };
        let mut names = self.profile.borrow_mut();
        let k = names.len();
        names.push(what);
        format!(
            "  %{t}.t0 = call i64 @{counter}()\n  {call}\n  %{t}.t1 = call i64 @{counter}()\n  %{t}.dt = sub i64 %{t}.t1, %{t}.t0\n  \
             %{t}.at = getelementptr i8, ptr @sv_prof, i64 {}\n  %{t}.was = load i64, ptr %{t}.at, align 8\n  \
             %{t}.now = add i64 %{t}.was, %{t}.dt\n  store i64 %{t}.now, ptr %{t}.at, align 8\n  \
             %{t}.nat = getelementptr i8, ptr @sv_prof, i64 {}\n  %{t}.n = load i64, ptr %{t}.nat, align 8\n  \
             %{t}.n1 = add i64 %{t}.n, 1\n  store i64 %{t}.n1, ptr %{t}.nat, align 8\n",
            16 * k,
            16 * k + 8
        )
    }

    /// Whether a specialisation's settling code reads an input of its,
    /// itself or in a child, or only its clocked processes do (a clock).
    fn comb_reads(&self, key: Key, o: ObjId) -> bool {
        if let Some(&r) = self.comb_reads.borrow().get(&(key, o)) {
            return r;
        }
        let lay = &self.layouts[&key];
        let mut r = lay.known.is_some();
        for &node in &lay.code {
            match node {
                // An input the parent copies to a slot is read where it
                // settles, whatever the child does with it; one the child
                // reads where it is, as the child reads it.
                Node::In(i) | Node::Pre(i) => {
                    let child = lay.kid_keys[&i];
                    let args = lay.args.get(&i);
                    for (p, e) in self.member_connections(i) {
                        let mut rs = Vec::new();
                        self.reads(e, &mut rs);
                        let k = self.d.spec(child.0).ports.iter().position(|q| q.name == p.name);
                        let copied = !matches!(args.zip(k).map(|(a, k)| a[k]), Some(Arg::Alias(_)));
                        if p.dir == Dir::Input && rs.iter().any(|t| t.0 == o) && (copied || p.object.is_some_and(|q| self.comb_reads(child, q))) {
                            r = true;
                        }
                    }
                }
                Node::Call(_) | Node::Early(_) => {}
                _ => r |= self.access(key, node).0.iter().any(|t| t.0 == o),
            }
        }
        self.comb_reads.borrow_mut().insert((key, o), r);
        r
    }

    /// The pieces of a specialisation's combinational code (see [`Dep`]).
    fn deps(&self, key: Key) -> Vec<Dep> {
        let d = self.d;
        let lay = &self.layouts[&key];
        let mut out = Vec::new();
        for (n, &node) in lay.code.iter().enumerate() {
            match node {
                Node::Proc(p) => self.stmt_deps(d.processes[p.0 as usize].body, n, &mut Vec::new(), &mut out),
                // Each output from the inputs it depends on in the child.
                // (An array's outputs, each from any iteration's inputs.)
                Node::Call(i) => {
                    let pd = self.port_deps(lay.kid_keys[&i]);
                    let conns = self.member_connections(i);
                    for &(p, e) in &conns {
                        let Some(o) = p.object.filter(|_| p.dir == Dir::Output) else { continue };
                        let (mut w, mut r) = (Vec::new(), Vec::new());
                        self.writes(e, &mut w, &mut r);
                        for &(q, qe) in &conns {
                            if q.dir == Dir::Input && q.object.is_some_and(|qo| pd.get(&o).is_some_and(|s| s.contains(&qo))) {
                                self.reads(qe, &mut r);
                            }
                        }
                        out.push(Dep { node: n, w, r });
                    }
                }
                Node::In(_) | Node::Pre(_) | Node::Early(_) => {}
                Node::Driver(_) | Node::Loop(_) => {
                    let (r, w) = self.access(key, node);
                    out.push(Dep { node: n, w, r });
                }
            }
        }
        out
    }

    /// A statement's pieces: each assignment, from what it reads and the
    /// conditions it runs under (`ctl`; after a `break`, `continue` or
    /// `return`, the conditions it was under stay for the rest).
    fn stmt_deps(&self, s: StmtId, node: usize, ctl: &mut Vec<Span>, out: &mut Vec<Dep>) {
        let d = self.d;
        let piece = |lhs: Option<ExprId>, rhs: &[ExprId], ctl: &[Span], out: &mut Vec<Dep>| {
            let (mut w, mut r) = (Vec::new(), ctl.to_vec());
            for &e in rhs {
                self.reads(e, &mut r);
                self.call_access(e, &mut r, &mut w, &mut Vec::new());
            }
            if let Some(l) = lhs {
                self.writes(l, &mut w, &mut r);
            }
            if !w.is_empty() {
                out.push(Dep { node, w, r });
            }
        };
        match &d.stmt(s).kind {
            StmtKind::Block { stmts, .. } => stmts.iter().for_each(|&s| self.stmt_deps(s, node, ctl, out)),
            StmtKind::Assign { kind, lhs, rhs, .. } => {
                let mut rhs = vec![*rhs];
                if matches!(kind, AssignKind::Compound(_)) {
                    rhs.push(*lhs);
                }
                piece(Some(*lhs), &rhs, ctl, out);
            }
            StmtKind::Expr(x) => {
                let lhs = match d.expr(*x).kind {
                    ExprKind::Unary(UnOp::PreInc | UnOp::PostInc | UnOp::PreDec | UnOp::PostDec) => Some(d.expr(*x).args[0]),
                    _ => None,
                };
                piece(lhs, &[*x], ctl, out);
            }
            StmtKind::If { cond, then, else_, .. } => {
                let at = ctl.len();
                self.reads(*cond, ctl);
                self.stmt_deps(*then, node, ctl, out);
                if let Some(x) = else_ {
                    self.stmt_deps(*x, node, ctl, out);
                }
                if !stops(d, *then) && !else_.is_some_and(|x| stops(d, x)) {
                    ctl.truncate(at);
                }
            }
            StmtKind::Case { expr, items, default, .. } => {
                let at = ctl.len();
                self.reads(*expr, ctl);
                for (labels, _) in items {
                    labels.iter().for_each(|&l| self.reads(l, ctl));
                }
                for (_, body) in items {
                    self.stmt_deps(*body, node, ctl, out);
                }
                if let Some(x) = default {
                    self.stmt_deps(*x, node, ctl, out);
                }
                if !items.iter().any(|i| stops(d, i.1)) && !default.is_some_and(|x| stops(d, x)) {
                    ctl.truncate(at);
                }
            }
            StmtKind::For { init, cond, step, body, .. } => {
                init.iter().for_each(|&s| self.stmt_deps(s, node, ctl, out));
                let at = ctl.len();
                if let Some(c) = cond {
                    self.reads(*c, ctl);
                }
                // Twice, for what one iteration hands the next.
                for _ in 0..2 {
                    self.stmt_deps(*body, node, ctl, out);
                    step.iter().for_each(|&s| self.stmt_deps(s, node, ctl, out));
                }
                if !stops(d, *body) {
                    ctl.truncate(at);
                }
            }
            StmtKind::Return(Some(x)) => piece(None, &[*x], ctl, out),
            _ => {}
        }
    }

    /// For each output of a specialisation, the inputs its value depends
    /// on through its combinational code (none, for a register).
    fn port_deps(&self, key: Key) -> std::rc::Rc<HashMap<ObjId, HashSet<ObjId>>> {
        if let Some(r) = self.port_deps.borrow().get(&key) {
            return r.clone();
        }
        let d = self.d;
        let lay = &self.layouts[&key];
        let ports = &d.spec(key.0).ports;
        let ins: HashSet<ObjId> = ports.iter().filter(|p| p.dir == Dir::Input).filter_map(|p| p.object).collect();
        let mut out = HashMap::new();
        if lay.known.is_some() {
            for p in ports.iter().filter(|p| p.dir == Dir::Output) {
                out.insert(p.object.unwrap(), ins.clone());
            }
        } else {
            let deps = self.deps(key);
            let from = Self::sources(&deps, |o| ins.contains(&o).then_some(o));
            for p in ports.iter().filter(|p| p.dir == Dir::Output) {
                let Some(o) = p.object else { continue };
                let mut s = HashSet::new();
                for (k, dep) in deps.iter().enumerate() {
                    if dep.w.iter().any(|t| t.0 == o) {
                        s.extend(from[k].iter().copied());
                    }
                }
                out.insert(o, s);
            }
        }
        let r = std::rc::Rc::new(out);
        self.port_deps.borrow_mut().insert(key, r.clone());
        r
    }

    /// For each piece, the sources it depends on, through the pieces
    /// that write what it reads: `source` says which objects are sources,
    /// and names them.
    fn sources<T: Copy + Eq + std::hash::Hash>(deps: &[Dep], source: impl Fn(ObjId) -> Option<T>) -> Vec<HashSet<T>> {
        let mut writers: HashMap<ObjId, Vec<(u32, u32, usize)>> = HashMap::new();
        for (k, dep) in deps.iter().enumerate() {
            for &(o, lo, hi) in &dep.w {
                writers.entry(o).or_default().push((lo, hi, k));
            }
        }
        // Who each piece reads from.
        let mut from: Vec<HashSet<T>> = vec![HashSet::new(); deps.len()];
        let mut preds: Vec<Vec<usize>> = vec![Vec::new(); deps.len()];
        for (k, dep) in deps.iter().enumerate() {
            for &(o, lo, hi) in &dep.r {
                if let Some(t) = source(o) {
                    from[k].insert(t);
                }
                for &(wlo, whi, j) in writers.get(&o).map(|v| v.as_slice()).unwrap_or(&[]) {
                    if wlo < hi && lo < whi && j != k {
                        preds[k].push(j);
                    }
                }
            }
            preds[k].sort_unstable();
            preds[k].dedup();
        }
        loop {
            let mut changed = false;
            for k in 0..deps.len() {
                for j in preds[k].clone() {
                    if j != k && !from[j].is_subset(&from[k]) {
                        let add: Vec<T> = from[j].iter().copied().collect();
                        from[k].extend(add);
                        changed = true;
                    }
                }
            }
            if !changed {
                return from;
            }
        }
    }

    /// Calls children twice where the code loops only through them (see
    /// [`Split`]): for a child whose call is in a loop, the outputs that
    /// depend on no input the loop goes through come from a first call.
    fn split_calls(&mut self, key: Key) -> Result<(), String> {
        let d = self.d;
        let mut tried = HashSet::new();
        loop {
            let Err(stuck) = self.order_of(key) else { return Ok(()) };
            let lay = &self.layouts[&key];
            let cands: Vec<InstId> = stuck
                .iter()
                .filter_map(|&n| if let Node::Call(i) = lay.code[n] { Some(i) } else { None })
                .filter(|i| !lay.splits.contains_key(i) && !tried.contains(i))
                .collect();
            let Some(&i) = cands.first() else { return self.order(key).map(|_| ()) };
            tried.insert(i);
            // What the child's outputs reach in the parent, and so the
            // inputs that the loop goes through.
            let deps = self.deps(key);
            let call = lay.code.iter().position(|&x| x == Node::Call(i)).unwrap();
            let mut reached: Vec<Span> = deps.iter().filter(|x| x.node == call).flat_map(|x| x.w.iter().copied()).collect();
            let mut done = vec![false; deps.len()];
            loop {
                let mut more = false;
                for (k, dep) in deps.iter().enumerate() {
                    if !done[k] && dep.node != call && dep.r.iter().any(|&(o, lo, hi)| reached.iter().any(|&(ro, rlo, rhi)| ro == o && rlo < hi && lo < rhi)) {
                        done[k] = true;
                        more = true;
                        reached.extend(dep.w.iter().copied());
                    }
                }
                if !more {
                    break;
                }
            }
            let ck = self.layouts[&key].kid_keys[&i];
            let child = ck.0;
            let pd = self.port_deps(ck);
            let mut late = HashSet::new();
            let mut split = Split::default();
            for (p, e) in self.connections(i) {
                let Some(o) = p.object.filter(|_| p.dir == Dir::Input) else { continue };
                let mut r = Vec::new();
                self.reads(e, &mut r);
                if r.iter().any(|&(o, lo, hi)| reached.iter().any(|&(ro, rlo, rhi)| ro == o && rlo < hi && lo < rhi)) {
                    late.insert(o);
                }
            }
            for p in d.spec(child).ports.iter().filter(|p| p.dir == Dir::Output) {
                let Some(o) = p.object else { continue };
                let need = pd.get(&o).cloned().unwrap_or_default();
                if need.is_disjoint(&late) {
                    split.outs.insert(o);
                    split.ins.extend(need);
                }
            }
            if split.outs.is_empty() {
                continue;
            }
            let lay = self.layouts.get_mut(&key).unwrap();
            let at = lay.code.iter().position(|&x| x == Node::In(i)).unwrap();
            lay.code.splice(at..at, [Node::Pre(i), Node::Early(i)]);
            lay.splits.insert(i, split.clone());
            // A child with one instance gets functions for the two calls,
            // laid out again with them.
            if !self.parts.contains_key(&child) && d.instances.iter().filter(|x| x.spec == child).count() == 1 {
                self.parts.insert(child, split);
                self.layouts.remove(&ck);
                self.layout(ck)?;
            }
        }
    }

    /// The two parts of a child called twice (see `Layout::part`), if
    /// the first leaves out some of its code.
    fn part_of(&self, key: Key, split: &Split) -> Option<(Vec<Node>, Vec<Node>)> {
        let d = self.d;
        let lay = &self.layouts[&key];
        let order = self.order(key).ok()?;
        let acc: Vec<(Vec<Span>, Vec<Span>)> = order.iter().map(|&n| self.access(key, n)).collect();
        let overlaps = |a: &[Span], b: &[Span]| a.iter().any(|&(o, lo, hi)| b.iter().any(|&(p, plo, phi)| o == p && plo < hi && lo < phi));
        // The nodes a node waits on besides what it reads.
        let before = |n: Node| -> Vec<Node> {
            match n {
                Node::Call(i) => vec![Node::In(i)],
                Node::Early(i) => vec![Node::Pre(i)],
                Node::In(i) if lay.splits.contains_key(&i) => vec![Node::Early(i)],
                _ => Vec::new(),
            }
        };
        // What the first call's outputs need, from the last node back.
        let mut need: Vec<Span> = split.outs.iter().map(|&o| (o, 0, tw(&d.types, d.object(o).ty).max(1))).collect();
        let mut a = HashSet::new();
        let mut forced = HashSet::new();
        for (k, &n) in order.iter().enumerate().rev() {
            if forced.contains(&n) || overlaps(&acc[k].1, &need) {
                a.insert(n);
                need.extend(acc[k].0.iter().copied());
                forced.extend(before(n));
            }
        }
        // What reads an input the first call does not have.
        let late: Vec<Span> = d
            .spec(key.0)
            .ports
            .iter()
            .filter(|p| p.dir == Dir::Input)
            .filter_map(|p| p.object)
            .filter(|o| !split.ins.contains(o))
            .map(|o| (o, 0, tw(&d.types, d.object(o).ty).max(1)))
            .collect();
        let mut stale = late;
        let mut again = HashSet::new();
        for (k, &n) in order.iter().enumerate() {
            if overlaps(&acc[k].0, &stale) || before(n).iter().any(|b| again.contains(b)) {
                again.insert(n);
                stale.extend(acc[k].1.iter().copied());
            }
        }
        let first: Vec<Node> = order.iter().copied().filter(|n| a.contains(n)).collect();
        if first.len() == order.len() || first.is_empty() {
            return None;
        }
        let second: Vec<Node> =
            order.iter().copied().filter(|n| a.contains(n) && again.contains(n)).chain(order.iter().copied().filter(|n| !a.contains(n))).collect();
        Some((first, second))
    }

    /// The code in the order it runs, and where the second part of a child
    /// called twice starts, if it is.
    fn sequence(&self, key: Key) -> Result<(Vec<Node>, usize), String> {
        match &self.layouts[&key].part {
            Some((a, b)) => Ok((a.iter().chain(b).copied().collect(), a.len())),
            None => Ok((self.order(key)?, usize::MAX)),
        }
    }

    /// An instance node's ports and what each is connected to: every
    /// iteration's, for an array.
    fn member_connections(&self, i: InstId) -> Vec<(&'d crate::design::Port, ExprId)> {
        self.members(i).into_iter().flat_map(|m| self.connections(m)).collect()
    }

    /// A child's ports and what each is connected to.
    fn connections(&self, i: InstId) -> Vec<(&'d crate::design::Port, ExprId)> {
        let d = self.d;
        let inst = d.instance(i);
        d.spec(inst.spec)
            .ports
            .iter()
            .filter_map(|p| Some((p, inst.connections.iter().find(|c| c.port == p.name)?.expr?)))
            .collect()
    }

    /// What a statement reads and writes, with the functions it calls
    /// (each once: `seen`).
    fn stmt_access(&self, s: StmtId, r: &mut Vec<(ObjId, u32, u32)>, w: &mut Vec<(ObjId, u32, u32)>, seen: &mut Vec<SubId>) {
        let d = self.d;
        let e = |e: ExprId, r: &mut Vec<(ObjId, u32, u32)>, w: &mut Vec<(ObjId, u32, u32)>, seen: &mut Vec<SubId>| {
            self.reads(e, r);
            self.call_access(e, r, w, seen);
        };
        match &d.stmt(s).kind {
            StmtKind::Block { stmts, .. } => stmts.iter().for_each(|&s| self.stmt_access(s, r, w, seen)),
            StmtKind::Assign { kind, lhs, rhs, .. } => {
                e(*rhs, r, w, seen);
                if matches!(kind, AssignKind::Compound(_)) {
                    self.reads(*lhs, r);
                }
                self.writes(*lhs, w, r);
            }
            StmtKind::Expr(x) => {
                e(*x, r, w, seen);
                if let ExprKind::Unary(UnOp::PreInc | UnOp::PostInc | UnOp::PreDec | UnOp::PostDec) = d.expr(*x).kind {
                    self.writes(d.expr(*x).args[0], w, r);
                }
            }
            StmtKind::If { cond, then, else_, .. } => {
                e(*cond, r, w, seen);
                self.stmt_access(*then, r, w, seen);
                if let Some(x) = else_ {
                    self.stmt_access(*x, r, w, seen);
                }
            }
            StmtKind::Case { expr, items, default, .. } => {
                e(*expr, r, w, seen);
                for (labels, body) in items {
                    labels.iter().for_each(|&l| e(l, r, w, seen));
                    self.stmt_access(*body, r, w, seen);
                }
                if let Some(x) = default {
                    self.stmt_access(*x, r, w, seen);
                }
            }
            StmtKind::For { init, cond, step, body, .. } => {
                init.iter().chain(step).for_each(|&s| self.stmt_access(s, r, w, seen));
                if let Some(c) = cond {
                    e(*c, r, w, seen);
                }
                self.stmt_access(*body, r, w, seen);
            }
            StmtKind::Return(Some(x)) => e(*x, r, w, seen),
            _ => {}
        }
    }

    /// What the functions an expression calls read and write, and the
    /// targets of their output arguments.
    fn call_access(&self, e: ExprId, r: &mut Vec<(ObjId, u32, u32)>, w: &mut Vec<(ObjId, u32, u32)>, seen: &mut Vec<SubId>) {
        let d = self.d;
        let x = d.expr(e);
        if let ExprKind::Call(sub) = x.kind {
            let s = &d.subroutines[sub.0 as usize];
            for (i, &f) in s.args.iter().enumerate() {
                if d.object(f).dir == Some(Dir::Output)
                    && let Some(&a) = x.args.get(i)
                {
                    self.writes(a, w, r);
                }
            }
            if !seen.contains(&sub) {
                seen.push(sub);
                for &st in &s.body {
                    self.stmt_access(st, r, w, seen);
                }
            }
        }
        for &a in &x.args {
            self.call_access(a, r, w, seen);
        }
    }

    fn reads(&self, e: ExprId, out: &mut Vec<(ObjId, u32, u32)>) {
        let x = self.d.expr(e);
        match &x.kind {
            ExprKind::Object(o) => {
                if matches!(self.d.object(*o).kind, ObjKind::Variable | ObjKind::Net(_)) {
                    out.push((*o, 0, tw(&self.d.types, self.d.object(*o).ty).max(1)));
                }
            }
            ExprKind::Index | ExprKind::Part { .. } | ExprKind::Member(_) => {
                if let Some(r) = self.span(e) {
                    out.push(r);
                    for &a in &x.args[1..] {
                        self.reads(a, out);
                    }
                } else {
                    for &a in &x.args {
                        self.reads(a, out);
                    }
                }
            }
            _ => {
                for &a in &x.args {
                    self.reads(a, out);
                }
            }
        }
    }

    /// What an lvalue writes, and what its indices read.
    fn writes(&self, e: ExprId, w: &mut Vec<(ObjId, u32, u32)>, r: &mut Vec<(ObjId, u32, u32)>) {
        let x = self.d.expr(e);
        match &x.kind {
            ExprKind::Concat => {
                for &a in &x.args {
                    self.writes(a, w, r);
                }
            }
            _ => {
                if let Some(s) = self.span(e) {
                    w.push(s);
                }
                if matches!(x.kind, ExprKind::Index | ExprKind::Part { .. } | ExprKind::Member(_)) {
                    for &a in &x.args[1..] {
                        self.reads(a, r);
                    }
                }
            }
        }
    }

    /// The bits a select of an object covers, if it is rooted at an object:
    /// exact for constant selects, the whole object otherwise.
    fn span(&self, e: ExprId) -> Option<(ObjId, u32, u32)> {
        let d = self.d;
        let x = d.expr(e);
        match &x.kind {
            ExprKind::Object(o) => {
                matches!(d.object(*o).kind, ObjKind::Variable | ObjKind::Net(_)).then(|| (*o, 0, tw(&d.types, d.object(*o).ty).max(1)))
            }
            ExprKind::Index | ExprKind::Part { .. } | ExprKind::Member(_) => {
                let (o, lo, hi) = self.span(x.args[0])?;
                let whole = (o, 0, tw(&d.types, d.object(o).ty).max(1));
                match self.const_select(e) {
                    Some(Some((off, w))) if off >= 0 && (lo as i64 + off + w as i64) <= hi as i64 => {
                        let lo = lo + off as u32;
                        Some((o, lo, lo + w))
                    }
                    _ => Some(whole),
                }
            }
            _ => None,
        }
    }

    /// A select with constant indices: its bit offset in its base and its
    /// width (`None` inside if out of range); `None` if not constant.
    fn const_select(&self, e: ExprId) -> Option<Option<(i64, u32)>> {
        let d = self.d;
        let tys = &d.types;
        let x = d.expr(e);
        let bt = d.expr(x.args[0]).ty;
        match &x.kind {
            ExprKind::Index => {
                if !matches!(
                    tys.get(bt),
                    Type::Vector { .. } | Type::PackedArray { .. } | Type::Struct { packed: true, .. } | Type::Enum { .. } | Type::Unpacked { .. }
                ) {
                    return None;
                }
                let k = self.fold(x.args[1])?.to_i64()?;
                let ew = tw(tys, x.ty).max(1);
                Some(lsb_off(tys, bt, k, k).map(|off| (off * ew as i64, ew)))
            }
            ExprKind::Part { kind, width } => {
                let a = self.fold(x.args[1])?.to_i64()?;
                let (hi, lo) = match kind {
                    PartKind::Range => (a, self.fold(x.args[2])?.to_i64()?),
                    PartKind::Up => (a + *width as i64 - 1, a),
                    PartKind::Down => (a, a - *width as i64 + 1),
                };
                let ew = elem_bits(tys, bt).max(1);
                let start = sel_start(tys, bt, lo, hi);
                Some(Some((start * ew as i64, *width * ew)))
            }
            ExprKind::Member(i) => match tys.get(bt) {
                Type::Struct { packed: true, members, union, .. } => {
                    let w = tw(tys, members[*i as usize].ty);
                    let below: u32 = if *union { 0 } else { members[*i as usize + 1..].iter().map(|m| tw(tys, m.ty)).sum() };
                    Some(Some((below as i64, w)))
                }
                _ => None,
            },
            _ => None,
        }
    }

    /// The value of a constant expression: literals, parameters and what
    /// is built from them.
    fn fold(&self, e: ExprId) -> Option<Const> {
        // In a rolled loop, what is worked out from its stepping
        // parameters is not constant, and not remembered.
        if !self.loop_vars.borrow().is_empty() {
            return self.fold_new(e);
        }
        if let Some(c) = self.folds.borrow().get(&e) {
            return c.clone();
        }
        let c = self.fold_new(e);
        self.folds.borrow_mut().insert(e, c.clone());
        c
    }

    fn fold_new(&self, e: ExprId) -> Option<Const> {
        let d = self.d;
        let tys = &d.types;
        let x = d.expr(e);
        match &x.kind {
            ExprKind::Const(c) => Some(d.konst(*c).clone()),
            ExprKind::Object(o) => {
                let ob = d.object(*o);
                match ob.kind {
                    _ if self.loop_vars.borrow().contains(o) => None,
                    ObjKind::Param { .. } | ObjKind::EnumConst | ObjKind::Genvar => ob.value.map(|v| d.konst(v).clone()),
                    _ => None,
                }
            }
            ExprKind::Unary(op) => {
                if matches!(op, UnOp::PreInc | UnOp::PreDec | UnOp::PostInc | UnOp::PostDec) {
                    return None;
                }
                ops::unary(tys, *op, self.fold(x.args[0])?, x.ty)
            }
            // A constant side that decides `&&` or `||` decides it whatever
            // the other side is.
            ExprKind::Binary(op @ (BinOp::LogAnd | BinOp::LogOr)) => {
                let decides = |c: &Option<Const>| c.as_ref().and_then(ops::truth).filter(|&t| t == (*op == BinOp::LogOr));
                let (a, b) = (self.fold(x.args[0]), self.fold(x.args[1]));
                if let Some(t) = decides(&a).or(decides(&b)) {
                    return Some(Const::Bits(Bits::from_bool(t)));
                }
                let (a, b) = (ops::truth(&a?)?, ops::truth(&b?)?);
                Some(Const::Bits(Bits::from_bool(if *op == BinOp::LogAnd { a && b } else { a || b })))
            }
            ExprKind::Binary(op) => ops::binary(tys, *op, self.fold(x.args[0])?, self.fold(x.args[1])?, x.ty),
            ExprKind::Convert => Some(ops::convert(tys, &self.fold(x.args[0])?, x.ty)),
            ExprKind::SysCall(f) if x.args.len() == 1 => match d.name(*f) {
                // The ceiling of the log base 2, as an integer (20.8.1).
                "$clog2" => {
                    let v = ops::bits_of(&self.fold(x.args[0])?)?.to_u64()?;
                    let c = if v <= 1 { 0 } else { 64 - (v - 1).leading_zeros() as u64 };
                    Some(Const::Bits(Bits::from_u64(tw(tys, x.ty), tys.is_signed(x.ty), c)))
                }
                "$signed" | "$unsigned" => Some(ops::convert(tys, &self.fold(x.args[0])?, x.ty)),
                // The bounds of a packed type's first dimension (20.7), and
                // its width (20.6.2), from the argument's type.
                name @ ("$left" | "$right" | "$low" | "$high" | "$size" | "$bits") => {
                    let a = d.expr(x.args[0]);
                    let t = if let ExprKind::TypeRef(t) = a.kind { t } else { a.ty };
                    let (l, r) = range_of(tys, t);
                    let v = match name {
                        "$left" => l,
                        "$right" => r,
                        "$low" => l.min(r),
                        "$high" => l.max(r),
                        "$size" => (l - r).abs() + 1,
                        _ => tw(tys, t) as i64,
                    };
                    Some(Const::Bits(Bits::from_u64(tw(tys, x.ty), tys.is_signed(x.ty), v as u64)))
                }
                _ => None,
            },
            ExprKind::Cond => {
                if ops::truth(&self.fold(x.args[0])?)? {
                    self.fold(x.args[1])
                } else {
                    self.fold(x.args[2])
                }
            }
            ExprKind::Concat => {
                let parts: Option<Vec<Bits>> = x.args.iter().map(|&a| ops::bits_of(&self.fold(a)?)).collect();
                Some(Const::Bits(Bits::concat(&parts?)))
            }
            ExprKind::Member(i) => {
                let b = self.fold(x.args[0])?;
                let bt = d.expr(x.args[0]).ty;
                match (tys.get(bt), &b) {
                    (Type::Struct { packed: true, members, union, .. }, Const::Bits(bits)) => {
                        let w = tw(tys, members[*i as usize].ty);
                        let below: u32 = if *union { 0 } else { members[*i as usize + 1..].iter().map(|m| tw(tys, m.ty)).sum() };
                        Some(Const::Bits(bits.slice(below, w).with_signed(tys.is_signed(x.ty))))
                    }
                    (_, Const::Aggregate(v)) => v.get(*i as usize).cloned(),
                    _ => None,
                }
            }
            ExprKind::Index => {
                let b = self.fold(x.args[0])?;
                let i = self.fold(x.args[1])?;
                ops::index(tys, &b, d.expr(x.args[0]).ty, &i, x.ty)
            }
            _ => None,
        }
    }
}

/// One piece of a value in every lane (32 bits in vector code, 64 in
/// scalar code): a constant, or `%tN`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum P {
    K(u64),
    V(u32),
}

/// A mask, one bit per lane: a constant, or `%tN`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum M {
    K(bool),
    V(u32),
}

/// A `w`-bit value: its pieces, low first, with the bits above `w` zero.
#[derive(Clone, PartialEq, Debug)]
struct Val {
    w: u32,
    p: Vec<P>,
}

#[derive(Clone, Default)]
struct Env {
    vals: HashMap<ObjId, Val>,
    /// The static values written since the loop began.
    dirty: HashSet<ObjId>,
    /// Elements of objects kept an element to a slot, and those written.
    elems: HashMap<(ObjId, u32), Val>,
    edirty: HashSet<(ObjId, u32)>,
    /// The lanes that have left the innermost loop with `break`.
    brk: Option<M>,
    /// The lanes that have returned from the function being run.
    ret: Option<M>,
    /// The registers' next values from nonblocking assignments so far.
    nba: HashMap<ObjId, Val>,
    /// Values and elements loaded from the state and not since written:
    /// a load in one arm of a branch serves the rest of the loop, as the
    /// code has no branches, without a select of it where the arms meet.
    loaded: HashMap<ObjId, Val>,
    eloaded: HashMap<(ObjId, u32), Val>,
}

/// The IR types of a shape's values.
#[derive(Clone, Copy)]
struct Tys {
    /// A piece of a value in every lane.
    v: &'static str,
    /// A mask.
    m: &'static str,
    /// Lanes twice a piece's width, for products.
    wide: &'static str,
    /// Stored values: bytes, halves, and 32-bit pieces.
    b8: &'static str,
    b16: &'static str,
    w32: &'static str,
    /// An integer of a bit a lane.
    bits: &'static str,
}

fn tys(n: u32) -> Tys {
    match n {
        1 => Tys { v: "i64", m: "i1", wide: "i128", b8: "i8", b16: "i16", w32: "i32", bits: "i1" },
        4 => Tys { v: "<4 x i32>", m: "<4 x i1>", wide: "<4 x i64>", b8: "<4 x i8>", b16: "<4 x i16>", w32: "<4 x i32>", bits: "i4" },
        8 => Tys { v: "<8 x i32>", m: "<8 x i1>", wide: "<8 x i64>", b8: "<8 x i8>", b16: "<8 x i16>", w32: "<8 x i32>", bits: "i8" },
        16 => Tys { v: "<16 x i32>", m: "<16 x i1>", wide: "<16 x i64>", b8: "<16 x i8>", b16: "<16 x i16>", w32: "<16 x i32>", bits: "i16" },
        _ => unreachable!("{n} lanes a register"),
    }
}

/// One function being written.
struct Func<'l, 'd, 'a> {
    l: &'l Lowering<'d, 'a>,
    key: Key,
    /// Its shape, and what follows from it: the bits in a piece, and the
    /// types.
    sh: Shape,
    pb: u32,
    t: Tys,
    out: String,
    next: u32,
    env: Env,
    /// The label of the block being written.
    block: String,
    loops: u32,
    /// Rolled loops written so far.
    rolled: u32,
    /// In a rolled loop: static values are read from and written to the
    /// state as they are used, not kept for the end of the outer loop.
    mem: bool,
    /// The temporaries with the same value in every lane.
    uniform: HashSet<u32>,
    /// Temporaries that are 0 or 1 a lane, made from these masks.
    mask_of: HashMap<u32, M>,
    /// Masks that are a bit plane's 16 bits (`%tN`, an i16): logic on them
    /// is done on those.
    bits_of: HashMap<u32, u32>,
    /// In a rolled loop: the element selects known to be in range.
    in_range: HashSet<ExprId>,
    /// The functions being run, innermost last, with their results.
    calls: Vec<(SubId, Option<ObjId>)>,
    /// Bits of temporaries known to be zero in every lane.
    zeros: HashMap<u32, u64>,
    /// Temporaries that are 0 or 1 a lane, bit `j` of a piece.
    bit_of: HashMap<u32, (P, u32)>,
    /// Blocks written for arms run only where some lane needs them.
    guards: u32,
    /// Lowering an arm only to see its size: no arm within it is guarded.
    trial: bool,
    /// Running clocked processes: nonblocking assignments are allowed.
    ff: bool,
    /// The lanes running the code being written: those taking every arm
    /// it is in.
    path: M,
}

/// A piece as an operand: a constant, splat in vector code.
fn ps(p: P, scalar: bool) -> String {
    match p {
        P::K(k) if scalar => format!("{}", k as i64),
        P::K(k) => format!("splat (i32 {})", k as u32 as i32),
        P::V(n) => format!("%t{n}"),
    }
}

fn ms(m: M, scalar: bool) -> String {
    match m {
        M::K(b) if scalar => format!("{b}"),
        M::K(b) => format!("splat (i1 {b})"),
        M::V(n) => format!("%t{n}"),
    }
}

impl<'l, 'd, 'a> Func<'l, 'd, 'a> {
    fn new(l: &'l Lowering<'d, 'a>, key: Key) -> Self {
        let lay = &l.layouts[&key];
        let mut params = String::from("ptr %s");
        for k in 0..l.d.spec(key.0).ports.len() {
            if !lay.port_ix.is_empty() {
                let _ = write!(params, ", ptr %p{k}");
            }
        }
        let out = format!("define void @{}({params}) {{\nentry:\n", fn_name(key));
        let sh = lay.shape;
        Func {
            l,
            key,
            sh,
            pb: sh.pb(),
            t: tys(sh.n),
            out,
            next: 0,
            env: Env::default(),
            block: "entry".into(),
            loops: 0,
            rolled: 0,
            mem: false,
            uniform: HashSet::new(),
            mask_of: HashMap::new(),
            bits_of: HashMap::new(),
            in_range: HashSet::new(),
            calls: Vec::new(),
            ff: false,
            zeros: HashMap::new(),
            bit_of: HashMap::new(),
            guards: 0,
            trial: false,
            path: M::K(true),
        }
    }

    fn sc(&self) -> bool {
        self.sh.scalar()
    }

    fn ps(&self, p: P) -> String {
        ps(p, self.sc())
    }

    fn ms(&self, m: M) -> String {
        ms(m, self.sc())
    }

    /// Pieces in a `w`-bit value.
    fn np(&self, w: u32) -> usize {
        (w as usize).div_ceil(self.pb as usize).max(1)
    }

    fn ones_k(&self) -> u64 {
        ones_of(self.pb)
    }

    fn top_mask(&self, w: u32) -> u64 {
        top_mask(w, self.pb)
    }

    /// A constant piece: `x`'s low `pb` bits.
    fn k(&self, x: i64) -> P {
        P::K(x as u64 & self.ones_k())
    }

    fn zero(&self, w: u32) -> Val {
        Val { w, p: vec![P::K(0); self.np(w)] }
    }

    /// The value's low 64 bits, if it is a constant no wider than that.
    fn is_const(&self, v: &Val) -> Option<u64> {
        if v.p.len() as u32 * self.pb > 64 {
            return None;
        }
        let mut x = 0u64;
        for (k, p) in v.p.iter().enumerate() {
            match p {
                P::K(c) => x |= c << (self.pb * k as u32),
                P::V(_) => return None,
            }
        }
        Some(x)
    }

    /// A call of an intrinsic, declared once.
    fn intrinsic(&self, decl: String) {
        self.l.decls.borrow_mut().insert(decl);
    }

    fn finish(mut self) -> String {
        self.out.push_str("  ret void\n}\n");
        self.out
    }

    /// The function's end, returning `v`, an `i1`.
    fn finish_with(mut self, v: &str) -> String {
        let _ = write!(self.out, "  ret i1 {v}\n}}\n");
        self.out
    }

    fn d(&self) -> &'d Design<'a> {
        self.l.d
    }

    fn lay(&self) -> &'l Layout {
        &self.l.layouts[&self.key]
    }

    fn tmp(&mut self) -> u32 {
        self.next += 1;
        self.next
    }

    /// Splits the nodes into loops at the children's calls, and writes them.
    fn body(&mut self, order: &[Node]) -> Result<(), String> {
        let mut seg: Vec<Node> = Vec::new();
        for &n in order {
            if let Node::Loop(l) = n {
                self.run_loop(&seg)?;
                seg.clear();
                self.rolled_loop(l)?;
            } else if let Node::Call(i) | Node::Early(i) = n {
                self.run_loop(&seg)?;
                let child = self.lay().kid_keys[&i];
                let args = self.child_args(i);
                let part = match n {
                    _ if !self.lay().splits.contains_key(&i) || self.l.layouts[&child].part.is_none() => "",
                    Node::Early(_) => "_a",
                    _ => "_b",
                };
                let what = format!("{} {}{part}", self.d().name(self.d().spec(self.key.0).name), self.d().name(self.d().spec(child.0).name));
                let t = self.tmp();
                let text = self.l.timed(what, &format!("call void @{}{part}({args})", fn_name(child)), &format!("p{t}"));
                self.out.push_str(&text);
                seg = vec![n];
            } else {
                seg.push(n);
            }
        }
        self.run_loop(&seg)
    }

    /// The arguments of a call of child `i`'s functions: its state, and
    /// what it has for each port.
    fn child_args(&mut self, i: InstId) -> String {
        {
            {
                let off = self.lay().kids[&i];
                let t = self.tmp();
                let _ = writeln!(self.out, "  %t{t} = getelementptr i8, ptr %s, i64 {off}");
                let mut args = format!("ptr %t{t}");
                for a in self.lay().args[&i].clone() {
                    let p = match a {
                        Arg::Alias(o) => match self.slot(o) {
                            Some(Base::Port(k)) => format!("%p{k}"),
                            Some(Base::State(at)) => {
                                let t = self.tmp();
                                let _ = writeln!(self.out, "  %t{t} = getelementptr i8, ptr %s, i64 {at}");
                                format!("%t{t}")
                            }
                            None => unreachable!("an aliased value has a slot"),
                        },
                        Arg::Slot(at) => {
                            let t = self.tmp();
                            let _ = writeln!(self.out, "  %t{t} = getelementptr i8, ptr %s, i64 {at}");
                            format!("%t{t}")
                        }
                    };
                    let _ = write!(args, ", ptr {p}");
                }
                args
            }
        }
    }

    /// A loop over the group's registers running `nodes` in order.
    fn run_loop(&mut self, nodes: &[Node]) -> Result<(), String> {
        if nodes.is_empty() {
            return Ok(());
        }
        self.run_loop_with(|f| nodes.iter().try_for_each(|&n| f.node(n)))
    }

    /// A pass over the registers of lanes whose body `body` writes: a loop
    /// over them where there are more than one.
    fn run_loop_with(&mut self, body: impl FnOnce(&mut Self) -> Result<(), String>) -> Result<(), String> {
        let k = self.loops;
        self.loops += 1;
        let looped = self.looped();
        let mut prev = String::new();
        if looped {
            prev = std::mem::replace(&mut self.block, format!("L{k}"));
            let _ = writeln!(
                self.out,
                "  br label %L{k}\nL{k}:\n  %c{k} = phi i64 [ 0, %{prev} ], [ %c{k}.next, %L{k}.latch ]\n  %c{k}.off = shl i64 %c{k}, 6\n  \
                 %b{k} = getelementptr i8, ptr %s, i64 %c{k}.off\n  %c{k}.half = shl i64 %c{k}, 1\n  \
                 %h{k} = getelementptr i8, ptr %s, i64 %c{k}.half\n  %c{k}.b8 = shl i64 %c{k}, 4\n  \
                 %q{k} = getelementptr i8, ptr %s, i64 %c{k}.b8\n  %c{k}.b16 = shl i64 %c{k}, 5\n  \
                 %w{k} = getelementptr i8, ptr %s, i64 %c{k}.b16"
            );
        }
        self.env = Env::default();
        let path = std::mem::replace(&mut self.path, M::K(true));
        body(self)?;
        self.path = path;
        let mut dirty: Vec<ObjId> = self.env.dirty.iter().copied().collect();
        dirty.sort_unstable_by_key(|o| o.0);
        for o in dirty {
            let v = self.env.vals[&o].clone();
            let base = self.slot(o).expect("a written value with a slot");
            let lane = self.lay().lane(o);
            self.store(base, lane, &v);
        }
        self.flush_elems(None);
        if looped {
            let _ = writeln!(
                self.out,
                "  %c{k}.next = add i64 %c{k}, 1\n  %c{k}.end = icmp eq i64 %c{k}.next, {}\n  \
                 br i1 %c{k}.end, label %X{k}, label %L{k}\nX{k}:",
                self.sh.chunks
            );
            // The back edge leaves from the last block of the body.
            self.out = self.out.replace(&format!("%L{k}.latch"), &format!("%{}", self.block));
            self.block = format!("X{k}");
            let _ = prev;
        }
        Ok(())
    }

    /// Whether a pass over the registers is a loop: more than one.
    fn looped(&self) -> bool {
        self.sh.chunks > 1
    }

    /// Where this register's part of the state is, for values stored as
    /// `lane`.
    fn reg_base(&self, lane: Lane) -> String {
        if !self.looped() {
            return "%s".into();
        }
        let k = self.loops - 1;
        match lane {
            Lane::Bit => format!("%h{k}"),
            Lane::B8 => format!("%q{k}"),
            Lane::B16 => format!("%w{k}"),
            Lane::W32 | Lane::W64 => format!("%b{k}"),
        }
    }

    fn base(&self) -> String {
        self.reg_base(Lane::W32)
    }

    /// Where a value of this function is, if it has a place.
    fn slot(&self, o: ObjId) -> Option<Base> {
        let lay = self.lay();
        match lay.port_ix.get(&o) {
            Some(&k) => Some(Base::Port(k)),
            None => lay.objs.get(&o).map(|&at| Base::State(at)),
        }
    }

    /// For a value behind a pointer parameter, a pointer to this register's
    /// part of it, as `off` would be from the state; `None` otherwise.
    fn port_base(&mut self, base: Base, lane: Lane) -> Option<String> {
        let Base::Port(p) = base else { return None };
        if !self.looped() {
            return Some(format!("%p{p}"));
        }
        let k = self.loops - 1;
        let step = match lane {
            Lane::W32 | Lane::W64 => "off",
            Lane::Bit => "half",
            Lane::B8 => "b8",
            Lane::B16 => "b16",
        };
        let t = self.tmp();
        let _ = writeln!(self.out, "  %t{t} = getelementptr i8, ptr %p{p}, i64 %c{k}.{step}");
        Some(format!("%t{t}"))
    }

    fn load(&mut self, base: Base, lane: Lane, w: u32) -> Val {
        let stride = lane.piece_bytes(self.sh);
        self.load_with(base, lane, w, stride)
    }

    /// A value from the state, its pieces `stride` bytes apart.
    fn load_with(&mut self, base: Base, lane: Lane, w: u32, stride: u64) -> Val {
        let ptr = self.port_base(base, lane);
        let off = match base {
            Base::State(at) => at,
            Base::Port(_) => 0,
        };
        let b = ptr.unwrap_or_else(|| self.reg_base(lane));
        let (t, n, sc) = (self.t, self.sh.n as u64, self.sc());
        match lane {
            Lane::W32 | Lane::W64 => {
                let (ty, align) = match lane {
                    Lane::W64 => ("i64", 8),
                    _ => (t.w32, 4 * n),
                };
                let mut p = Vec::new();
                for i in 0..lane.pieces(w) {
                    let (a, v) = (self.tmp(), self.tmp());
                    let _ = writeln!(self.out, "  %t{a} = getelementptr i8, ptr {b}, i64 {}\n  %t{v} = load {ty}, ptr %t{a}, align {align}", off + i * stride);
                    if sc && lane == Lane::W32 {
                        let z = self.tmp();
                        let _ = writeln!(self.out, "  %t{z} = zext i32 %t{v} to i64");
                        self.zeros.insert(z, !0xffff_ffff);
                        p.push(P::V(z));
                    } else {
                        p.push(P::V(v));
                    }
                }
                Val { w, p }
            }
            Lane::Bit => {
                let (a, i, m) = (self.tmp(), self.tmp(), self.tmp());
                let _ = writeln!(
                    self.out,
                    "  %t{a} = getelementptr i8, ptr {b}, i64 {off}\n  %t{i} = load i16, ptr %t{a}, align 2\n  \
                     %t{m} = bitcast i16 %t{i} to {}",
                    t.m
                );
                self.bits_of.insert(m, i);
                self.bit(M::V(m), w)
            }
            Lane::B8 | Lane::B16 => {
                let (ty, align) = if lane == Lane::B8 { (t.b8, n) } else { (t.b16, 2 * n) };
                let (a, v, z) = (self.tmp(), self.tmp(), self.tmp());
                let _ = writeln!(
                    self.out,
                    "  %t{a} = getelementptr i8, ptr {b}, i64 {off}\n  %t{v} = load {ty}, ptr %t{a}, align {align}\n  \
                     %t{z} = zext {ty} %t{v} to {}",
                    t.v
                );
                self.zeros.insert(z, if lane == Lane::B8 { !0xff } else { !0xffff });
                Val { w, p: vec![P::V(z)] }
            }
        }
    }

    fn store(&mut self, base: Base, lane: Lane, v: &Val) {
        let stride = lane.piece_bytes(self.sh);
        self.store_with(base, lane, v, stride)
    }

    /// The bits of mask `m` as an integer, a bit a lane (16 lanes).
    fn mask_bits(&mut self, m: M) -> String {
        match m {
            M::K(b) => format!("{}", if b { -1 } else { 0 }),
            M::V(n) => match self.bits_of.get(&n) {
                Some(&i) => format!("%t{i}"),
                None => {
                    let t = self.tmp();
                    let _ = writeln!(self.out, "  %t{t} = bitcast {} %t{n} to {}", self.t.m, self.t.bits);
                    format!("%t{t}")
                }
            },
        }
    }

    /// Whether any lane of `m` is set, an `i1`.
    fn any(&mut self, m: M) -> String {
        match m {
            M::K(b) => format!("{b}"),
            M::V(n) if self.sc() => format!("%t{n}"),
            M::V(_) => {
                let bits = self.mask_bits(m);
                let t = self.tmp();
                let _ = writeln!(self.out, "  %t{t} = icmp ne {} {bits}, 0", self.t.bits);
                format!("%t{t}")
            }
        }
    }

    /// A value to the state, its pieces `stride` bytes apart.
    fn store_with(&mut self, base: Base, lane: Lane, v: &Val, stride: u64) {
        let ptr = self.port_base(base, lane);
        let off = match base {
            Base::State(at) => at,
            Base::Port(_) => 0,
        };
        let b = ptr.unwrap_or_else(|| self.reg_base(lane));
        let (t, n, sc) = (self.t, self.sh.n as u64, self.sc());
        match lane {
            Lane::W32 | Lane::W64 => {
                let (ty, align) = match lane {
                    Lane::W64 => ("i64", 8),
                    _ => (t.w32, 4 * n),
                };
                for (i, &p) in v.p.iter().enumerate() {
                    let x = if sc && lane == Lane::W32 {
                        match p {
                            P::K(k) => format!("{}", k as u32 as i32),
                            P::V(_) => {
                                let x = self.tmp();
                                let _ = writeln!(self.out, "  %t{x} = trunc i64 {} to i32", self.ps(p));
                                format!("%t{x}")
                            }
                        }
                    } else {
                        self.ps(p)
                    };
                    let a = self.tmp();
                    let _ = writeln!(self.out, "  %t{a} = getelementptr i8, ptr {b}, i64 {}\n  store {ty} {x}, ptr %t{a}, align {align}", off + i as u64 * stride);
                }
            }
            Lane::Bit => {
                let m = self.truth(v);
                let bits = self.mask_bits(m);
                let a = self.tmp();
                let _ = writeln!(self.out, "  %t{a} = getelementptr i8, ptr {b}, i64 {off}\n  store i16 {bits}, ptr %t{a}, align 2");
            }
            Lane::B8 | Lane::B16 => {
                let (ty, align) = if lane == Lane::B8 { (t.b8, n) } else { (t.b16, 2 * n) };
                let (a, x) = (self.tmp(), self.tmp());
                let _ = writeln!(
                    self.out,
                    "  %t{a} = getelementptr i8, ptr {b}, i64 {off}\n  %t{x} = trunc {} {} to {ty}\n  \
                     store {ty} %t{x}, ptr %t{a}, align {align}",
                    t.v,
                    self.ps(v.p[0])
                );
            }
        }
    }

    fn node(&mut self, n: Node) -> Result<(), String> {
        let d = self.d();
        match n {
            Node::Loop(_) => unreachable!("rolled loops are loops of their own"),
            Node::Driver(dr) => {
                let drv = &d.drivers[dr.0 as usize];
                if let Some((o, v)) = self.padded_copy(drv.lhs, drv.rhs) {
                    self.set(o, v);
                    return Ok(());
                }
                let v = self.expr(drv.rhs)?;
                self.assign(drv.lhs, v)
            }
            Node::Proc(p) => {
                let pr = &d.processes[p.0 as usize];
                self.stmt(pr.body)?;
                // Automatic variables end with the process.
                self.env.vals.retain(|o, _| d.object(*o).lifetime == Lifetime::Static);
                Ok(())
            }
            Node::In(i) | Node::Pre(i) => {
                let ck = self.lay().kid_keys[&i];
                let child = &self.l.layouts[&ck];
                let members = self.l.members(i);
                let args = self.lay().args[&i].clone();
                let pre = if let Node::Pre(_) = n { self.lay().splits.get(&i) } else { None };
                for (port, arg) in d.spec(ck.0).ports.iter().zip(args) {
                    // Values connected whole are where the child reads them;
                    // an unconnected input's slot stays zero.
                    let (Dir::Input, Arg::Slot(at)) = (port.dir, arg) else { continue };
                    if pre.is_some_and(|s| port.object.is_none_or(|o| !s.ins.contains(&o))) {
                        continue;
                    }
                    let o = port.object.ok_or("interface ports are not supported yet")?;
                    let w = tw(&d.types, d.object(o).ty);
                    let mut vals = Vec::with_capacity(members.len());
                    for &m in &members {
                        vals.push(match d.instance(m).connections.iter().find(|c| c.port == port.name).and_then(|c| c.expr) {
                            Some(e) => {
                                let v = self.expr(e)?;
                                Some(self.resize(&v, w, false))
                            }
                            None => None,
                        });
                    }
                    match &vals[..] {
                        [v] => {
                            if let Some(v) = v {
                                self.store(Base::State(at), child.lane(o), v);
                            }
                        }
                        _ => self.to_lanes(at, child.lane(o), w, child.shape, &vals),
                    }
                }
                Ok(())
            }
            Node::Call(i) | Node::Early(i) => {
                let ck = self.lay().kid_keys[&i];
                let child = &self.l.layouts[&ck];
                let members = self.l.members(i);
                let args = self.lay().args[&i].clone();
                let split = self.lay().splits.get(&i);
                for (port, arg) in d.spec(ck.0).ports.iter().zip(args) {
                    if let Some(s) = split
                        && port.object.is_some_and(|o| s.outs.contains(&o)) != matches!(n, Node::Early(_))
                    {
                        continue;
                    }
                    match (port.dir, arg) {
                        (Dir::Output, Arg::Slot(at)) => {
                            let o = port.object.ok_or("interface ports are not supported yet")?;
                            let ty = d.object(o).ty;
                            let cw = tw(&d.types, ty);
                            let mut regs = HashMap::new();
                            for (t, &m) in members.iter().enumerate() {
                                let Some(e) = d.instance(m).connections.iter().find(|c| c.port == port.name).and_then(|c| c.expr) else { continue };
                                let v = if members.len() == 1 {
                                    self.load(Base::State(at), child.lane(o), cw)
                                } else {
                                    self.from_lanes(at, child.lane(o), cw, child.shape, t, &mut regs)
                                };
                                let w = tw(&d.types, d.expr(e).ty);
                                let v = self.resize(&v, w, d.types.is_signed(ty));
                                self.assign(e, v)?;
                            }
                        }
                        (Dir::Input | Dir::Output, _) => {}
                        (dir, _) => return Err(format!("{dir:?} ports are not supported yet")),
                    }
                }
                Ok(())
            }
        }
    }

    /// 32-bit word `k` of lane `s` of `v`, an `i32` operand.
    fn word_of(&mut self, v: &Val, k: usize, s: u32) -> String {
        if self.sc() {
            let p = v.p.get(k / 2).copied().unwrap_or(P::K(0));
            let sh = 32 * (k % 2) as u32;
            if let P::K(x) = p {
                return format!("{}", (x >> sh) as u32 as i32);
            }
            let p = self.bin("lshr", p, P::K(sh as u64));
            let t = self.tmp();
            let _ = writeln!(self.out, "  %t{t} = trunc i64 {} to i32", self.ps(p));
            return format!("%t{t}");
        }
        match v.p.get(k).copied().unwrap_or(P::K(0)) {
            P::K(x) => format!("{}", x as u32 as i32),
            P::V(n) => {
                let t = self.tmp();
                let _ = writeln!(self.out, "  %t{t} = extractelement {} %t{n}, i64 {s}", self.t.v);
                format!("%t{t}")
            }
        }
    }

    /// An array of children's input, each iteration's value in `vals`, to
    /// the child's slot at `at` (stored as `lane`, for the child's shape
    /// `csh`): iteration `t`'s in the lanes as wide as this function's
    /// registers from `t` times that.
    fn to_lanes(&mut self, at: u64, lane: Lane, w: u32, csh: Shape, vals: &[Option<Val>]) {
        let sh = self.sh;
        if sh.n == 16 {
            // Registers of 16 both sides: this pass's register of an
            // iteration's lanes is one of the child's.
            let stride = lane.piece_bytes(csh);
            for (t, v) in vals.iter().enumerate() {
                if let Some(v) = v {
                    let blk = t as u64 * sh.chunks as u64 * lane.reg_bytes(csh);
                    self.store_with(Base::State(at + blk), lane, v, stride);
                }
            }
            return;
        }
        // Lane by lane: each of the child's registers made from the lanes it
        // holds, then stored as one.
        let (wp, nc, ct) = (sh.width() as usize, csh.n as usize, tys(csh.n));
        let lanes = vals.len() * wp;
        let words = if lane == Lane::W32 { pieces(w) } else { 1 };
        for k in 0..words {
            for r in 0..lanes.div_ceil(nc) {
                let mut acc = String::from("zeroinitializer");
                for j in 0..nc.min(lanes - r * nc) {
                    let l = r * nc + j;
                    let Some(v) = &vals[l / wp] else { continue };
                    let word = self.word_of(v, k, (l % wp) as u32);
                    let x = self.tmp();
                    let _ = writeln!(self.out, "  %t{x} = insertelement {} {acc}, i32 {word}, i64 {j}", ct.w32);
                    acc = format!("%t{x}");
                }
                let a = self.tmp();
                match lane {
                    Lane::W32 => {
                        let off = at + k as u64 * lane.piece_bytes(csh) + r as u64 * lane.reg_bytes(csh);
                        let _ = writeln!(self.out, "  %t{a} = getelementptr i8, ptr %s, i64 {off}\n  store {} {acc}, ptr %t{a}, align {}", ct.w32, 4 * nc);
                    }
                    Lane::B8 | Lane::B16 => {
                        let (ty, eb) = if lane == Lane::B8 { (ct.b8, 1) } else { (ct.b16, 2) };
                        let x = self.tmp();
                        let _ = writeln!(
                            self.out,
                            "  %t{a} = getelementptr i8, ptr %s, i64 {}\n  %t{x} = trunc {} {acc} to {ty}\n  store {ty} %t{x}, ptr %t{a}, align {}",
                            at + r as u64 * lane.reg_bytes(csh),
                            ct.w32,
                            eb * nc
                        );
                    }
                    Lane::Bit => {
                        let (m, b) = (self.tmp(), self.tmp());
                        let _ = writeln!(
                            self.out,
                            "  %t{a} = getelementptr i8, ptr %s, i64 {}\n  %t{m} = icmp ne {} {acc}, zeroinitializer\n  \
                             %t{b} = bitcast {} %t{m} to {}\n  store {} %t{b}, ptr %t{a}, align 2",
                            at + r as u64 * lane.reg_bytes(csh),
                            ct.w32,
                            ct.m,
                            ct.bits,
                            ct.bits
                        );
                    }
                    Lane::W64 => unreachable!("an array's child is not scalar"),
                }
            }
        }
    }

    /// Iteration `t`'s value of an array of children's output, from the
    /// child's slot at `at` (as [`Func::to_lanes`] lays it out); `regs`
    /// keeps the child's registers already loaded, by word and register.
    fn from_lanes(&mut self, at: u64, lane: Lane, w: u32, csh: Shape, t: usize, regs: &mut HashMap<(usize, usize), String>) -> Val {
        let sh = self.sh;
        if sh.n == 16 {
            let stride = lane.piece_bytes(csh);
            let blk = t as u64 * sh.chunks as u64 * lane.reg_bytes(csh);
            return self.load_with(Base::State(at + blk), lane, w, stride);
        }
        let (wp, nc, ct) = (sh.width() as usize, csh.n as usize, tys(csh.n));
        let words = if lane == Lane::W32 { pieces(w) } else { 1 };
        // Each word of each lane, an i32.
        let mut got: Vec<Vec<String>> = vec![Vec::new(); wp];
        for k in 0..words {
            for (s, got) in got.iter_mut().enumerate() {
                let l = t * wp + s;
                let (r, j) = (l / nc, l % nc);
                let reg = match regs.get(&(k, r)) {
                    Some(x) => x.clone(),
                    None => {
                        let a = self.tmp();
                        let x = match lane {
                            Lane::W32 => {
                                let (v, off) = (self.tmp(), at + k as u64 * lane.piece_bytes(csh) + r as u64 * lane.reg_bytes(csh));
                                let _ = writeln!(self.out, "  %t{a} = getelementptr i8, ptr %s, i64 {off}\n  %t{v} = load {}, ptr %t{a}, align {}", ct.w32, 4 * nc);
                                v
                            }
                            Lane::B8 | Lane::B16 => {
                                let (ty, eb) = if lane == Lane::B8 { (ct.b8, 1) } else { (ct.b16, 2) };
                                let (v, z) = (self.tmp(), self.tmp());
                                let _ = writeln!(
                                    self.out,
                                    "  %t{a} = getelementptr i8, ptr %s, i64 {}\n  %t{v} = load {ty}, ptr %t{a}, align {}\n  %t{z} = zext {ty} %t{v} to {}",
                                    at + r as u64 * lane.reg_bytes(csh),
                                    eb * nc,
                                    ct.w32
                                );
                                z
                            }
                            Lane::Bit => {
                                let (v, m, z) = (self.tmp(), self.tmp(), self.tmp());
                                let _ = writeln!(
                                    self.out,
                                    "  %t{a} = getelementptr i8, ptr %s, i64 {}\n  %t{v} = load {}, ptr %t{a}, align 2\n  \
                                     %t{m} = bitcast {} %t{v} to {}\n  %t{z} = zext {} %t{m} to {}",
                                    at + r as u64 * lane.reg_bytes(csh),
                                    ct.bits,
                                    ct.bits,
                                    ct.m,
                                    ct.m,
                                    ct.w32
                                );
                                z
                            }
                            Lane::W64 => unreachable!("an array's child is not scalar"),
                        };
                        let x = format!("%t{x}");
                        regs.insert((k, r), x.clone());
                        x
                    }
                };
                let e = self.tmp();
                let _ = writeln!(self.out, "  %t{e} = extractelement {} {reg}, i64 {j}", ct.w32);
                got.push(format!("%t{e}"));
            }
        }
        if self.sc() {
            let mut p = Vec::new();
            for q in 0..self.np(w) {
                let mut acc = P::K(0);
                for half in 0..2 {
                    let Some(word) = got[0].get(2 * q + half) else { break };
                    let z = self.tmp();
                    let _ = writeln!(self.out, "  %t{z} = zext i32 {word} to i64");
                    self.zeros.insert(z, !0xffff_ffff);
                    let x = self.bin("shl", P::V(z), P::K(32 * half as u64));
                    acc = self.bin("or", acc, x);
                }
                p.push(acc);
            }
            return Val { w, p };
        }
        let mut p = Vec::new();
        for k in 0..words {
            let mut acc = String::from("zeroinitializer");
            for (s, got) in got.iter().enumerate() {
                let x = self.tmp();
                let _ = writeln!(self.out, "  %t{x} = insertelement {} {acc}, i32 {}, i64 {s}", self.t.v, got[k]);
                acc = format!("%t{x}");
            }
            p.push(P::V(acc[2..].parse().unwrap()));
        }
        Val { w, p }
    }

    // ---- statements ----

    fn stmt(&mut self, s: StmtId) -> Result<(), String> {
        let d = self.d();
        if !matches!(d.stmt(s).kind, StmtKind::Null | StmtKind::Block { .. }) && self.l.only_asserts(s) {
            return Ok(());
        }
        match &d.stmt(s).kind {
            StmtKind::Null => Ok(()),
            StmtKind::Block { join: None, scope, stmts } => {
                if let Some(sc) = scope {
                    for item in &d.scope(*sc).items {
                        if let Item::Object(o) = *item {
                            let ob = d.object(o);
                            if ob.lifetime == Lifetime::Automatic && ob.kind == ObjKind::Variable {
                                let w = tw(&d.types, ob.ty);
                                let v = match ob.init {
                                    Some(e) => {
                                        let v = self.expr(e)?;
                                        self.resize(&v, w, false)
                                    }
                                    None => self.zero(w),
                                };
                                self.env.vals.insert(o, v);
                            }
                        }
                    }
                }
                for &s in stmts {
                    // After a `break` or `return` in some lanes, the rest
                    // of the block is for the others.
                    match self.stopped() {
                        Some(M::K(true)) => break,
                        Some(b @ M::V(_)) => {
                            let go = self.mnot(b);
                            self.branch(go, |f| f.stmt(s), |_| Ok(()))?;
                        }
                        _ => self.stmt(s)?,
                    }
                }
                Ok(())
            }
            StmtKind::Assign { kind: AssignKind::NonBlocking, lhs, rhs, timing: None } => {
                if !self.ff {
                    return Err("nonblocking assignments outside clocked processes are not supported yet".into());
                }
                if let Some((o, v)) = self.padded_copy(*lhs, *rhs) {
                    if !self.lay().regs.contains(&o) {
                        return Err(format!("{} is assigned nonblocking but is not a register here", d.name(d.object(o).name)));
                    }
                    if self.lay().in_place.contains(&o) {
                        self.set(o, v);
                    } else {
                        self.env.nba.insert(o, v);
                    }
                    return Ok(());
                }
                let v = self.expr(*rhs)?;
                self.assign_nb(*lhs, v)
            }
            StmtKind::Assign { kind: AssignKind::Blocking, lhs, rhs, timing: None } if self.padded_copy(*lhs, *rhs).is_some() => {
                let (o, v) = self.padded_copy(*lhs, *rhs).unwrap();
                self.set(o, v);
                Ok(())
            }
            StmtKind::Assign { kind, lhs, rhs, timing: None } => {
                let v = match kind {
                    AssignKind::Blocking => self.expr(*rhs)?,
                    AssignKind::Compound(op) => {
                        let ty = d.expr(*lhs).ty;
                        let a = self.expr(*lhs)?;
                        let b = self.expr(*rhs)?;
                        self.binary(*op, a, b, ty, ty)?
                    }
                    AssignKind::NonBlocking => return Err("nonblocking assignments are not supported yet".into()),
                };
                self.assign(*lhs, v)
            }
            StmtKind::Expr(e) => match d.expr(*e).kind {
                ExprKind::Call(sub) => self.call(sub, &d.expr(*e).args).map(|_| ()),
                ExprKind::Unary(op @ (UnOp::PreInc | UnOp::PostInc | UnOp::PreDec | UnOp::PostDec)) => {
                    let x = d.expr(*e).args[0];
                    let v = self.expr(x)?;
                    let one = self.konst(&Bits::from_u64(v.w, false, 1));
                    let v = if matches!(op, UnOp::PreInc | UnOp::PostInc) { self.add(&v, &one, false) } else { self.add(&v, &one, true) };
                    self.assign(x, v)
                }
                _ => Err("expression statements other than ++ and -- are not supported yet".into()),
            },
            StmtKind::If { cond, then, else_, .. } => {
                let (cond, then, else_) = (*cond, *then, *else_);
                let mask = move |f: &mut Self| -> Result<M, String> {
                    let c = f.expr(cond)?;
                    Ok(f.truth(&c))
                };
                self.cond_late(mask, then, else_)
            }
            StmtKind::Case { kind, expr, items, default, .. } => {
                let sel = self.expr(*expr)?;
                let st = d.expr(*expr).ty;
                self.case_chain(*kind, &sel, st, items, *default)
            }
            StmtKind::For { init, cond, step, body, .. } => {
                for &s in init {
                    self.stmt(s)?;
                }
                let n = self.trip_count(*cond, step)?;
                let outer = self.env.brk.replace(M::K(false));
                for _ in 0..n {
                    // The lanes still looping: those that have not broken
                    // out, where the condition holds.
                    let mut go = match self.stopped() {
                        Some(b) => self.mnot(b),
                        None => M::K(true),
                    };
                    if let Some(c) = cond {
                        let v = self.expr(*c)?;
                        let t = self.truth(&v);
                        go = self.mand(go, t);
                    }
                    if go == M::K(false) {
                        break;
                    }
                    let (body, step) = (*body, step.clone());
                    self.branch(
                        go,
                        |f| {
                            f.stmt(body)?;
                            // A lane that broke out does not step.
                            match f.stopped() {
                                Some(M::K(true)) => Ok(()),
                                Some(b @ M::V(_)) => {
                                    let on = f.mnot(b);
                                    f.branch(on, |g| step.iter().try_for_each(|&s| g.stmt(s)), |_| Ok(()))
                                }
                                _ => step.iter().try_for_each(|&s| f.stmt(s)),
                            }
                        },
                        |_| Ok(()),
                    )?;
                }
                self.env.brk = outer;
                Ok(())
            }
            StmtKind::Return(v) => {
                let Some(&(_, r)) = self.calls.last() else { return Err("return outside a function".into()) };
                if let (Some(e), Some(r)) = (v, r) {
                    let x = self.expr(*e)?;
                    let w = tw(&d.types, d.object(r).ty);
                    let x = self.resize(&x, w, d.types.is_signed(d.expr(*e).ty));
                    self.env.vals.insert(r, x);
                }
                self.env.ret = Some(M::K(true));
                Ok(())
            }
            StmtKind::Break => {
                if self.env.brk.is_none() {
                    return Err("break outside a loop".into());
                }
                self.env.brk = Some(M::K(true));
                Ok(())
            }
            StmtKind::Assert { .. } => {
                self.l.asserts.borrow_mut().insert(s);
                Ok(())
            }
            k => Err(format!("statement {} is not supported yet", stmt_name(k))),
        }
    }

    /// How many times a counted loop runs, from its condition and steps
    /// with the values it starts with, which must be constants.
    fn trip_count(&mut self, cond: Option<ExprId>, step: &[StmtId]) -> Result<u32, String> {
        self.trial(|f| {
            let mut n = 0u32;
            loop {
                let go = match cond {
                    Some(c) => {
                        let v = f.expr(c)?;
                        f.truth(&v)
                    }
                    None => M::K(true),
                };
                match go {
                    M::K(false) => return Ok(n),
                    M::K(true) if n < 65536 => {}
                    M::K(true) => return Err("a loop of more than 65536 iterations".into()),
                    M::V(_) => return Err("loops whose trip count is not a constant are not supported yet".into()),
                }
                step.iter().try_for_each(|&s| f.stmt(s))?;
                n += 1;
            }
        })
        .map(|(n, _)| n)
    }

    /// Runs `run` as a trial: nothing it writes or learns stays. Its
    /// result, and whether it changed no value (it only read, and loaded).
    fn trial<T>(&mut self, run: impl FnOnce(&mut Self) -> Result<T, String>) -> Result<(T, bool), String> {
        let saved = (self.env.clone(), self.out.len(), self.next, self.block.clone(), self.guards);
        let tables = (self.mask_of.clone(), self.bits_of.clone(), self.uniform.clone(), self.zeros.clone(), self.bit_of.clone());
        // (No arm within a trial is guarded: its blocks would outlive it.)
        let was = std::mem::replace(&mut self.trial, true);
        let r = run(self);
        self.trial = was;
        (self.block, self.guards) = (saved.3.clone(), saved.4);
        let (e, s) = (&self.env, &saved.0);
        let pure = e.vals == s.vals
            && e.elems == s.elems
            && e.nba == s.nba
            && e.dirty == s.dirty
            && e.edirty == s.edirty
            && e.brk == s.brk
            && e.ret == s.ret;
        self.env = saved.0;
        self.out.truncate(saved.1);
        self.next = saved.2;
        (self.mask_of, self.bits_of, self.uniform, self.zeros, self.bit_of) = tables;
        r.map(|x| (x, pure))
    }

    /// A case's items from the first, each item's match made where its
    /// arm meets the later ones'.
    fn case_chain(
        &mut self,
        kind: CaseKind,
        sel: &Val,
        st: TypeId,
        items: &[(Vec<ExprId>, StmtId)],
        default: Option<StmtId>,
    ) -> Result<(), String> {
        match items.split_first() {
            None => match default {
                Some(s) => self.stmt(s),
                None => Ok(()),
            },
            Some(((labels, body), rest)) => {
                let mask = |f: &mut Self| -> Result<M, String> {
                    let mut m = M::K(false);
                    for &l in labels {
                        let hit = f.case_match(kind, sel, st, l)?;
                        m = f.mor(m, hit);
                    }
                    Ok(m)
                };
                // A costly arm is guarded, which needs its match first.
                if self.worth_guarding(*body) {
                    let m = mask(self)?;
                    return match m {
                        M::V(_) if self.sc() => {
                            let mut arms: Vec<StmtId> = rest.iter().map(|i| i.1).chain(default).collect();
                            arms.push(*body);
                            self.diamond(m, &arms, |f| f.stmt(*body), Some(|f: &mut Self| f.case_chain(kind, sel, st, rest, default)))
                        }
                        M::V(_) => self.branch_guarded(m, *body, true, |f| f.case_chain(kind, sel, st, rest, default), None),
                        _ => self.branch(m, |f| f.stmt(*body), |f| f.case_chain(kind, sel, st, rest, default)),
                    };
                }
                self.branch_late(mask, |f| f.stmt(*body), |f| f.case_chain(kind, sel, st, rest, default))
            }
        }
    }

    /// An `if` on the condition `mask` makes: a costly arm guarded (its
    /// condition made first), else as [`Func::branch_late`].
    fn cond_late(&mut self, mask: impl Fn(&mut Self) -> Result<M, String>, then: StmtId, else_: Option<StmtId>) -> Result<(), String> {
        let costly = self.worth_guarding(then) || else_.is_some_and(|e| self.worth_guarding(e));
        if costly {
            let m = mask(self)?;
            return self.cond(m, then, else_);
        }
        self.branch_late(
            mask,
            |f| f.stmt(then),
            |f| match else_ {
                Some(e) => f.stmt(e),
                None => Ok(()),
            },
        )
    }

    /// An `if`: as [`Func::branch`], but an arm much bigger than the test of
    /// whether any lane takes it is run only where one does; in scalar
    /// code, both arms behind a branch.
    fn cond(&mut self, m: M, then: StmtId, else_: Option<StmtId>) -> Result<(), String> {
        let variable = matches!(m, M::V(_));
        let gt = variable && self.worth_guarding(then);
        let ge = else_.filter(|&e| variable && self.worth_guarding(e));
        if self.sc() && (gt || ge.is_some()) {
            let arms: Vec<StmtId> = std::iter::once(then).chain(else_).collect();
            return self.diamond(m, &arms, |f| f.stmt(then), else_.map(|e| move |f: &mut Self| f.stmt(e)));
        }
        self.branch_guarded(
            m,
            then,
            gt,
            |f| match else_ {
                Some(e) => f.stmt(e),
                None => Ok(()),
            },
            ge,
        )
    }

    /// Whether the arm `s` is worth a branch around it, taken only where
    /// some lane takes the arm (see [`GUARD_MIN`], and [`BRANCH_MIN`] for
    /// scalar code).
    fn worth_guarding(&mut self, s: StmtId) -> bool {
        let min = if self.sc() { self.l.opts.branch_min } else { self.l.opts.guard_min };
        if self.trial || self.mem || min == 0 {
            return false;
        }
        self.cost(|f| f.stmt(s)) >= min
    }

    /// How many instructions `f` writes, written and then taken back.
    fn cost(&mut self, f: impl FnOnce(&mut Self) -> Result<(), String>) -> usize {
        let saved = (self.env.clone(), self.out.len(), self.next, self.block.clone(), self.guards);
        let tables = (self.mask_of.clone(), self.bits_of.clone(), self.uniform.clone(), self.zeros.clone(), self.bit_of.clone());
        let was = std::mem::replace(&mut self.trial, true);
        let ok = f(self).is_ok();
        let n = self.out[saved.1..].lines().filter(|l| l.starts_with("  %") || l.starts_with("  store") || l.starts_with("  call")).count();
        self.trial = was;
        self.out.truncate(saved.1);
        (self.env, _, self.next, self.block, self.guards) = saved;
        (self.mask_of, self.bits_of, self.uniform, self.zeros, self.bit_of) = tables;
        // (Something it cannot lower fails again when lowered for real.)
        if ok { n } else { 0 }
    }

    /// [`Func::branch`] with `then` in a block of its own where `gt`, run
    /// only where some lane takes it, and the other arm likewise where it is
    /// `ge`, a statement. Each value either changes is joined after the block
    /// with a phi of its new value and the one it had.
    fn branch_guarded(
        &mut self,
        m: M,
        then: StmtId,
        gt: bool,
        else_: impl FnOnce(&mut Self) -> Result<(), String>,
        ge: Option<StmtId>,
    ) -> Result<(), String> {
        if !gt && ge.is_none() {
            return self.branch(m, |f| f.stmt(then), else_);
        }
        let before = self.env.clone();
        // The other arm first, as `b`, guarded on its own lanes if costly.
        let (path, nm) = (self.path, self.mnot(m));
        self.path = self.mand(path, nm);
        let ran = match ge {
            Some(e) => self.guard(nm, e, |f| f.stmt(e)),
            None => else_(self),
        };
        self.path = path;
        ran?;
        if !gt {
            let b = std::mem::replace(&mut self.env, before);
            (self.env.loaded, self.env.eloaded) = (b.loaded.clone(), b.eloaded.clone());
            return self.run_then_merge(m, then, b);
        }
        let mut b = self.env.clone();
        // In the block: `then` from the values before both arms (with what
        // has been loaded since), then the selects between it and `b`.
        self.guard(m, then, |f| {
            b.loaded = f.env.loaded.clone();
            b.eloaded = f.env.eloaded.clone();
            let mut start = before.clone();
            (start.loaded, start.eloaded) = (b.loaded.clone(), b.eloaded.clone());
            f.env = start;
            f.run_then_merge(m, then, b)
        })
    }

    /// `then` on the current values, then each value it or `b` changed
    /// selected by `m` (as the end of [`Func::branch`]).
    fn run_then_merge(&mut self, m: M, then: StmtId, mut b: Env) -> Result<(), String> {
        let before = self.env.clone();
        let path = self.path;
        self.path = self.mand(path, m);
        let ran = self.stmt(then);
        self.path = path;
        ran?;
        let mut a = std::mem::replace(&mut self.env, before);
        (self.env.loaded, self.env.eloaded) = (std::mem::take(&mut a.loaded), std::mem::take(&mut a.eloaded));
        for (o, v) in std::mem::take(&mut b.loaded) {
            self.env.loaded.entry(o).or_insert(v);
        }
        for (k, v) in std::mem::take(&mut b.eloaded) {
            self.env.eloaded.entry(k).or_insert(v);
        }
        self.merge(m, a, b);
        Ok(())
    }

    /// Runs `body` in a block of its own, entered only where some lane of
    /// `c` is set, on the current values; `s`, the statement it lowers, says
    /// what it may write, each loaded first so that the way around has it.
    fn guard(&mut self, c: M, s: StmtId, body: impl FnOnce(&mut Self) -> Result<(), String>) -> Result<(), String> {
        let M::V(n) = c else { unreachable!("a constant condition is not guarded") };
        let (mut r, mut w) = (Vec::new(), Vec::new());
        self.l.stmt_access(s, &mut r, &mut w, &mut Vec::new());
        self.preload(w);
        let any = self.any(M::V(n));
        let k = self.guards;
        self.guards += 1;
        let skip = self.env.clone();
        let from = std::mem::replace(&mut self.block, format!("G{k}"));
        let _ = writeln!(self.out, "  br i1 {any}, label %G{k}, label %J{k}\nG{k}:");
        body(self)?;
        let taken = std::mem::take(&mut self.env);
        let at = std::mem::replace(&mut self.block, format!("J{k}"));
        let _ = writeln!(self.out, "  br label %J{k}\nJ{k}:");
        self.env = self.join(skip, &from, taken, &at)?;
        Ok(())
    }

    /// Loads what an arm may write, each object of `w`, so that the way
    /// around it has their values.
    fn preload(&mut self, w: Vec<(ObjId, u32, u32)>) {
        let mut objs: Vec<ObjId> = w.into_iter().map(|t| t.0).collect();
        objs.sort_unstable_by_key(|o| o.0);
        objs.dedup();
        for o in objs {
            if self.lay().kept.contains(&o) {
                // Kept in the state, but a register's next value is not,
                // unless in place: its value from before, for the way around.
                if self.ff && self.lay().regs.contains(&o) && !self.lay().in_place.contains(&o) && !self.env.nba.contains_key(&o) {
                    let v = self.read(o);
                    self.env.loaded.insert(o, v);
                }
            } else if self.slot(o).is_some() || self.lay().elems.contains_key(&o) {
                self.read(o);
            }
        }
    }

    /// Scalar code's `if`: `then` where `m` holds and `else_` where it does
    /// not, each in a block of its own, and each value either changes
    /// joined after them with a phi. What the statements `arms` may write
    /// is loaded first, so that both ways have it.
    fn diamond(
        &mut self,
        m: M,
        arms: &[StmtId],
        then: impl FnOnce(&mut Self) -> Result<(), String>,
        else_: Option<impl FnOnce(&mut Self) -> Result<(), String>>,
    ) -> Result<(), String> {
        let M::V(n) = m else { unreachable!("a constant condition is not a branch") };
        let (mut r, mut w) = (Vec::new(), Vec::new());
        for &s in arms {
            self.l.stmt_access(s, &mut r, &mut w, &mut Vec::new());
        }
        self.preload(w);
        let before = self.env.clone();
        let k = self.guards;
        self.guards += 1;
        let other = if else_.is_some() { format!("E{k}") } else { format!("J{k}") };
        let from = std::mem::replace(&mut self.block, format!("T{k}"));
        let _ = writeln!(self.out, "  br i1 %t{n}, label %T{k}, label %{other}\nT{k}:");
        then(self)?;
        let a = std::mem::replace(&mut self.env, before.clone());
        let at_a = std::mem::replace(&mut self.block, other.clone());
        let _ = writeln!(self.out, "  br label %J{k}");
        let (b, at_b) = match else_ {
            Some(e) => {
                let _ = writeln!(self.out, "{other}:");
                e(self)?;
                let _ = writeln!(self.out, "  br label %J{k}");
                (std::mem::take(&mut self.env), std::mem::replace(&mut self.block, format!("J{k}")))
            }
            None => (before.clone(), from),
        };
        let _ = writeln!(self.out, "J{k}:");
        self.block = format!("J{k}");
        self.env = self.join2(&before, a, &at_a, b, &at_b)?;
        Ok(())
    }

    /// The values after two blocks, `a` from `at_a` and `b` from `at_b`,
    /// both from the values `before`: a phi of each either changed.
    fn join2(&mut self, before: &Env, a: Env, at_a: &str, b: Env, at_b: &str) -> Result<Env, String> {
        let d = self.d();
        let mut out = before.clone();
        out.dirty.extend(a.dirty.iter().chain(&b.dirty).copied());
        out.edirty.extend(a.edirty.iter().chain(&b.edirty).copied());
        let around = |f: &mut Self, o: ObjId| -> Result<Val, String> {
            match before.vals.get(&o).or(before.loaded.get(&o)) {
                Some(v) => Ok(v.clone()),
                None if f.slot(o).is_none() => Ok(f.zero(tw(&d.types, d.object(o).ty))),
                None => Err(format!("{} is not loaded before a branch", d.name(d.object(o).name))),
            }
        };
        let mut keys: Vec<ObjId> = a.vals.keys().chain(b.vals.keys()).copied().collect();
        keys.sort_unstable_by_key(|o| o.0);
        keys.dedup();
        for o in keys {
            let va = match a.vals.get(&o) {
                Some(v) => v.clone(),
                None => around(self, o)?,
            };
            let vb = match b.vals.get(&o) {
                Some(v) => v.clone(),
                None => around(self, o)?,
            };
            let v = self.phi(&va, at_a, &vb, at_b);
            out.vals.insert(o, v);
        }
        let mut keys: Vec<(ObjId, u32)> = a.elems.keys().chain(b.elems.keys()).copied().collect();
        keys.sort_unstable_by_key(|k| (k.0.0, k.1));
        keys.dedup();
        for k in keys {
            let was = before.elems.get(&k).or(before.eloaded.get(&k)).cloned();
            let va = a.elems.get(&k).cloned().or(was.clone()).ok_or("an element not loaded before a branch")?;
            let vb = b.elems.get(&k).cloned().or(was).ok_or("an element not loaded before a branch")?;
            let v = self.phi(&va, at_a, &vb, at_b);
            out.elems.insert(k, v);
        }
        let mut keys: Vec<ObjId> = a.nba.keys().chain(b.nba.keys()).copied().collect();
        keys.sort_unstable_by_key(|o| o.0);
        keys.dedup();
        for o in keys {
            let was = match before.nba.get(&o) {
                Some(v) => v.clone(),
                None => around(self, o)?,
            };
            let va = a.nba.get(&o).cloned().unwrap_or_else(|| was.clone());
            let vb = b.nba.get(&o).cloned().unwrap_or(was);
            let v = self.phi(&va, at_a, &vb, at_b);
            out.nba.insert(o, v);
        }
        let stop = |f: &mut Self, x: Option<M>, y: Option<M>| match (x, y) {
            (x, y) if x == y => x,
            (x, y) => Some(f.mphi(x.unwrap_or(M::K(false)), at_a, y.unwrap_or(M::K(false)), at_b)),
        };
        out.brk = stop(self, a.brk, b.brk);
        out.ret = stop(self, a.ret, b.ret);
        Ok(out)
    }

    /// The values after a guarded block: those it changed are phis of their
    /// values from it (`at`) and from around it (`from`).
    fn join(&mut self, skip: Env, from: &str, taken: Env, at: &str) -> Result<Env, String> {
        let d = self.d();
        let mut out = skip.clone();
        out.dirty.extend(taken.dirty.iter().copied());
        out.edirty.extend(taken.edirty.iter().copied());
        let around = |f: &mut Self, o: ObjId| -> Result<Val, String> {
            match skip.vals.get(&o).or(skip.loaded.get(&o)) {
                Some(v) => Ok(v.clone()),
                None if f.slot(o).is_none() => Ok(f.zero(f.width(o))),
                None => Err(format!("{} is not loaded before a guarded block", d.name(d.object(o).name))),
            }
        };
        let mut keys: Vec<ObjId> = taken.vals.keys().copied().collect();
        keys.sort_unstable_by_key(|o| o.0);
        for o in keys {
            let (vt, vs) = (taken.vals[&o].clone(), around(self, o)?);
            let v = self.phi(&vt, at, &vs, from);
            out.vals.insert(o, v);
        }
        let mut keys: Vec<(ObjId, u32)> = taken.elems.keys().copied().collect();
        keys.sort_unstable_by_key(|k| (k.0.0, k.1));
        for k in keys {
            let vs = skip.elems.get(&k).or(skip.eloaded.get(&k)).cloned().ok_or("an element not loaded before a guarded block")?;
            let v = self.phi(&taken.elems[&k].clone(), at, &vs, from);
            out.elems.insert(k, v);
        }
        let mut keys: Vec<ObjId> = taken.nba.keys().copied().collect();
        keys.sort_unstable_by_key(|o| o.0);
        for o in keys {
            let vs = match skip.nba.get(&o) {
                Some(v) => v.clone(),
                None => around(self, o)?,
            };
            let v = self.phi(&taken.nba[&o].clone(), at, &vs, from);
            out.nba.insert(o, v);
        }
        let stop = |f: &mut Self, t: Option<M>, s: Option<M>| match (t, s) {
            (t, s) if t == s => t,
            (t, s) => Some(f.mphi(t.unwrap_or(M::K(false)), at, s.unwrap_or(M::K(false)), from)),
        };
        out.brk = stop(self, taken.brk, skip.brk);
        out.ret = stop(self, taken.ret, skip.ret);
        Ok(out)
    }

    fn phi(&mut self, a: &Val, at: &str, b: &Val, from: &str) -> Val {
        let p = a
            .p
            .iter()
            .zip(&b.p)
            .map(|(&x, &y)| {
                if x == y {
                    return x;
                }
                let t = self.tmp();
                let _ = writeln!(self.out, "  %t{t} = phi {} [ {}, %{at} ], [ {}, %{from} ]", self.t.v, self.ps(x), self.ps(y));
                let z = self.kz(x) & self.kz(y);
                if z != 0 {
                    self.zeros.insert(t, z);
                }
                P::V(t)
            })
            .collect();
        Val { w: a.w, p }
    }

    fn mphi(&mut self, a: M, at: &str, b: M, from: &str) -> M {
        if a == b {
            return a;
        }
        let t = self.tmp();
        let _ = writeln!(self.out, "  %t{t} = phi {} [ {}, %{at} ], [ {}, %{from} ]", self.t.m, self.ms(a), self.ms(b));
        M::V(t)
    }

    /// Whether a case item matches, with `casez` and `casex` don't-cares in
    /// constant items (12.5.1).
    fn case_match(&mut self, kind: CaseKind, sel: &Val, st: TypeId, item: ExprId) -> Result<M, String> {
        let d = self.d();
        let w = sel.w;
        if let Some(Const::Bits(b)) = self.l.fold(item)
            && !b.is_known()
        {
            let b = b.resize(w, d.types.is_signed(d.expr(item).ty));
            let (mut care, mut val) = (Bits::zero(w, false), Bits::zero(w, false));
            for i in 0..w {
                let dc = match (kind, b.bit(i)) {
                    (_, Logic::Zero | Logic::One) => false,
                    (CaseKind::CaseZ, Logic::Z) | (CaseKind::CaseX, Logic::Z | Logic::X) => true,
                    _ => return Ok(M::K(false)),
                };
                if !dc {
                    care.set_bit(i, Logic::One);
                    val.set_bit(i, b.bit(i));
                }
            }
            let (care, val) = (self.konst(&care), self.konst(&val));
            let masked = self.bitwise("and", sel, &care);
            return Ok(self.eq(&masked, &val));
        }
        if kind == CaseKind::Inside {
            return self.inside_item(sel, st, item);
        }
        let v = self.expr(item)?;
        let v = self.resize(&v, w, d.types.is_signed(st));
        Ok(self.eq(sel, &v))
    }

    /// Whether `v` (of type `vt`) matches one item of `inside` (11.4.13):
    /// a range `[lo:hi]` holds it, or it equals the item, with the item's
    /// unknown bits matching anything.
    fn inside_item(&mut self, v: &Val, vt: TypeId, item: ExprId) -> Result<M, String> {
        let d = self.d();
        let tys = &d.types;
        let x = d.expr(item);
        if x.kind == ExprKind::Range {
            let (lo, hi) = (x.args[0], x.args[1]);
            let signed = tys.is_signed(vt) && tys.is_signed(d.expr(lo).ty) && tys.is_signed(d.expr(hi).ty);
            let (a, b) = (self.expr(lo)?, self.expr(hi)?);
            let n = v.w.max(a.w).max(b.w);
            let (v, a, b) = (self.resize(v, n, signed), self.resize(&a, n, signed), self.resize(&b, n, signed));
            let below = self.lt(&v, &a, signed);
            let above = self.lt(&b, &v, signed);
            let out = self.mor(below, above);
            return Ok(self.mnot(out));
        }
        let signed = tys.is_signed(vt) && tys.is_signed(x.ty);
        let n = v.w.max(tw(tys, x.ty));
        let v = self.resize(v, n, signed);
        if let Some(Const::Bits(b)) = self.l.fold(item)
            && !b.is_known()
        {
            let b = b.resize(n, signed);
            let (mut care, mut val) = (Bits::zero(n, false), Bits::zero(n, false));
            for i in 0..n {
                if matches!(b.bit(i), Logic::Zero | Logic::One) {
                    care.set_bit(i, Logic::One);
                    val.set_bit(i, b.bit(i));
                }
            }
            let (care, val) = (self.konst(&care), self.konst(&val));
            let masked = self.bitwise("and", &v, &care);
            return Ok(self.eq(&masked, &val));
        }
        let a = self.expr(item)?;
        let a = self.resize(&a, n, signed);
        Ok(self.eq(&v, &a))
    }

    /// Both arms, each on its own copy of the values, then a select of
    /// each value that differs. A constant condition runs one arm.
    ///
    /// The else arm is written first: in a chain of them (`if`/`else if`,
    /// a case's items) each arm's values then meet the rest's where they
    /// are made, rather than every arm's living to the chain's end.
    fn branch(
        &mut self,
        m: M,
        then: impl FnOnce(&mut Self) -> Result<(), String>,
        else_: impl FnOnce(&mut Self) -> Result<(), String>,
    ) -> Result<(), String> {
        match m {
            M::K(true) => then(self),
            M::K(false) => else_(self),
            M::V(_) => self.arms(Some(m), |_| Ok(m), then, else_),
        }
    }

    /// A branch on a condition made where the arms meet, after both, so
    /// it is not live across them: what `mask` makes when it is not a
    /// constant and changes no value (else it is made first, as usual).
    fn branch_late(
        &mut self,
        mask: impl Fn(&mut Self) -> Result<M, String>,
        then: impl FnOnce(&mut Self) -> Result<(), String>,
        else_: impl FnOnce(&mut Self) -> Result<(), String>,
    ) -> Result<(), String> {
        match self.trial(&mask)? {
            // (Not where values are kept in the state: an arm's stores to
            // them need its lanes while it is written.)
            (M::V(_), true) if self.lay().kept.is_empty() => self.arms(None, mask, then, else_),
            _ => {
                let m = mask(self)?;
                self.branch(m, then, else_)
            }
        }
    }

    /// Both arms of a branch, then its condition, `mask`, made from the
    /// values before them, and the selects.
    fn arms(
        &mut self,
        early: Option<M>,
        mask: impl FnOnce(&mut Self) -> Result<M, String>,
        then: impl FnOnce(&mut Self) -> Result<(), String>,
        else_: impl FnOnce(&mut Self) -> Result<(), String>,
    ) -> Result<(), String> {
        let before = self.env.clone();
        // The lanes running each arm, where the condition is known now
        // (stores to values kept in the state are made in those lanes).
        let path = self.path;
        if let Some(m) = early {
            let nm = self.mnot(m);
            self.path = self.mand(path, nm);
        }
        else_(self)?;
        let mut b = std::mem::replace(&mut self.env, before.clone());
        (self.env.loaded, self.env.eloaded) = (std::mem::take(&mut b.loaded), std::mem::take(&mut b.eloaded));
        if let Some(m) = early {
            self.path = self.mand(path, m);
        }
        then(self)?;
        self.path = path;
        let mut a = std::mem::replace(&mut self.env, before);
        (self.env.loaded, self.env.eloaded) = (std::mem::take(&mut a.loaded), std::mem::take(&mut a.eloaded));
        let m = mask(self)?;
        self.merge(m, a, b);
        Ok(())
    }

    /// The values after arms `a` (where `m`) and `b`, on the values before
    /// them: a select of each that differs.
    fn merge(&mut self, m: M, a: Env, b: Env) {
        let mut keys: Vec<ObjId> = a.vals.keys().chain(b.vals.keys()).copied().collect();
        keys.sort_unstable_by_key(|o| o.0);
        keys.dedup();
        for o in keys {
            let va = match a.vals.get(&o) {
                Some(v) => v.clone(),
                None => self.read(o),
            };
            let vb = match b.vals.get(&o) {
                Some(v) => v.clone(),
                None => self.read(o),
            };
            let v = if va == vb { va } else { self.select(m, &va, &vb) };
            self.env.vals.insert(o, v);
        }
        self.env.dirty.extend(a.dirty.into_iter().chain(b.dirty));
        let mut keys: Vec<(ObjId, u32)> = a.elems.keys().chain(b.elems.keys()).copied().collect();
        keys.sort_unstable_by_key(|k| (k.0.0, k.1));
        keys.dedup();
        for (o, i) in keys {
            let va = match a.elems.get(&(o, i)) {
                Some(v) => v.clone(),
                None => self.read_elem(o, i),
            };
            let vb = match b.elems.get(&(o, i)) {
                Some(v) => v.clone(),
                None => self.read_elem(o, i),
            };
            let v = if va == vb { va } else { self.select(m, &va, &vb) };
            self.env.elems.insert((o, i), v);
        }
        self.env.edirty.extend(a.edirty.into_iter().chain(b.edirty));
        let mut keys: Vec<ObjId> = a.nba.keys().chain(b.nba.keys()).copied().collect();
        keys.sort_unstable_by_key(|o| o.0);
        keys.dedup();
        for o in keys {
            let side = |f: &mut Self, x: &HashMap<ObjId, Val>| match x.get(&o).or(f.env.nba.get(&o)) {
                Some(v) => v.clone(),
                None => f.read(o),
            };
            let (va, vb) = (side(self, &a.nba), side(self, &b.nba));
            let v = if va == vb { va } else { self.select(m, &va, &vb) };
            self.env.nba.insert(o, v);
        }
        self.env.brk = self.merge_stop(m, a.brk, b.brk);
        self.env.ret = self.merge_stop(m, a.ret, b.ret);
    }

    /// A nonblocking assignment: the register's next value.
    fn assign_nb(&mut self, lhs: ExprId, v: Val) -> Result<(), String> {
        let d = self.d();
        let x = d.expr(lhs);
        match &x.kind {
            ExprKind::Concat => {
                let mut top: u32 = x.args.iter().map(|&a| tw(&d.types, d.expr(a).ty)).sum();
                for &a in &x.args {
                    let w = tw(&d.types, d.expr(a).ty);
                    top -= w;
                    let part = self.extract(&v, top, w);
                    self.assign_nb(a, part)?;
                }
                Ok(())
            }
            ExprKind::Object(_) | ExprKind::Index | ExprKind::Part { .. } | ExprKind::Member(_) => {
                let (o, off, w) = self.place(lhs)?;
                if !self.lay().regs.contains(&o) {
                    return Err(format!("{} is assigned nonblocking but is not a register here", d.name(d.object(o).name)));
                }
                if self.lay().in_place.contains(&o) {
                    let v = self.resize(&v, w, false);
                    return self.write_kept(o, off, w, &v);
                }
                let old = match self.env.nba.get(&o) {
                    Some(v) => v.clone(),
                    None => self.read(o),
                };
                let new = self.write_at(o, &old, off, w, &v)?;
                self.env.nba.insert(o, new);
                Ok(())
            }
            k => Err(format!("nonblocking assignments to {k:?} are not supported yet")),
        }
    }

    /// The function of a specialisation with registers for `phase`: its
    /// children's, then its own loop.
    /// Its `_ff` function returns whether an edge came in any lane, below
    /// it too (`i1`): without one, nothing changes, and the step need not
    /// commit or settle again.
    fn phase(&mut self, phase: Phase) -> Result<String, String> {
        let d = self.d();
        let ret = if phase == Phase::Ff { "define i1 @" } else { "define void @" };
        self.out = self.out.replacen(
            &format!("define void @{}(", fn_name(self.key)),
            &format!("{ret}{}{}(", fn_name(self.key), phase.suffix()),
            1,
        );
        let mut any = String::from("false");
        let fired = self.lay().fired;
        if phase == Phase::Ff && !self.lay().ff.is_empty() {
            let _ = writeln!(self.out, "  %fired = getelementptr i8, ptr %s, i64 {fired}\n  store i8 0, ptr %fired, align 1");
        }
        for n in self.lay().code.clone() {
            if let Node::Call(i) = n
                && self.l.layouts[&self.lay().kid_keys[&i]].clocked
            {
                let args = self.child_args(i);
                let ck = self.lay().kid_keys[&i];
                let child = format!("@{}{}", fn_name(ck), phase.suffix());
                let what = format!("{} {}{}", d.name(d.spec(self.key.0).name), d.name(d.spec(ck.0).name), phase.suffix());
                if phase == Phase::Ff {
                    let (t, o) = (self.tmp(), self.tmp());
                    let text = self.l.timed(what, &format!("%t{t} = call i1 {child}({args})"), &format!("p{t}"));
                    self.out.push_str(&text);
                    let _ = writeln!(self.out, "  %t{o} = or i1 {any}, %t{t}");
                    any = format!("%t{o}");
                } else {
                    let t = self.tmp();
                    let text = self.l.timed(what, &format!("call void {child}({args})"), &format!("p{t}"));
                    self.out.push_str(&text);
                }
            }
        }
        let lay = self.lay();
        if lay.regs.is_empty() && lay.ff.is_empty() {
            return Ok(any);
        }
        match phase {
            Phase::Ff => {
                self.run_loop_with(|f| f.ff_procs())?;
                let (l, z, o) = (self.tmp(), self.tmp(), self.tmp());
                let _ = writeln!(
                    self.out,
                    "  %t{l} = load i8, ptr %fired, align 1\n  %t{z} = icmp ne i8 %t{l}, 0\n  %t{o} = or i1 {any}, %t{z}"
                );
                Ok(format!("%t{o}"))
            }
            Phase::Commit if self.lay().shadow.is_empty() => Ok(any),
            Phase::Commit => self.run_loop_with(|f| {
                for o in f.lay().regs.clone() {
                    let w = tw(&d.types, d.object(o).ty);
                    let Some(&at) = f.lay().shadow.get(&o) else { continue };
                    let lane = f.lay().lane(o);
                    let v = f.load(Base::State(at), lane, w);
                    f.set(o, v);
                }
                Ok(())
            })
            .map(|_| any),
        }
    }

    /// Each clocked process where an edge it waits on came, on the
    /// settled values: the registers' next values, kept apart until
    /// every process has run.
    fn ff_procs(&mut self) -> Result<(), String> {
        let d = self.d();
        let planes = self.l.opts.bit_planes;
        let mut runs = Vec::new();
        let mut seen = HashMap::new();
        for p in self.lay().ff.clone() {
            let (terms, body) = self.l.clocked(p).unwrap();
            let mut fire = M::K(false);
            for t in terms {
                // An edge is a change of the expression's lowest bit
                // (9.4.2).
                let (now, before) = match seen.get(&t.expr) {
                    Some(&x) => x,
                    None => {
                        let v = self.expr(t.expr)?;
                        let now = self.extract(&v, 0, 1);
                        let now = self.truth(&now);
                        let at = self.lay().prev[&t.expr];
                        let b = self.load(Base::State(at), Lane::of(1, self.sh, planes), 1);
                        let before = self.truth(&b);
                        seen.insert(t.expr, (now, before));
                        (now, before)
                    }
                };
                let mut e = match t.edge {
                    crate::design::Edge::Pos => {
                        let nb = self.mnot(before);
                        self.mand(now, nb)
                    }
                    _ => {
                        let nn = self.mnot(now);
                        self.mand(nn, before)
                    }
                };
                if let Some(c) = t.iff {
                    let c = self.expr(c)?;
                    let c = self.truth(&c);
                    e = self.mand(e, c);
                }
                fire = self.mor(fire, e);
            }
            runs.push((fire, body));
        }
        let mut prevs: Vec<(ExprId, M)> = seen.into_iter().map(|(e, (now, _))| (e, now)).collect();
        prevs.sort_unstable_by_key(|p| p.0.0);
        for (e, now) in prevs {
            let at = self.lay().prev[&e];
            let v = self.bit(now, 1);
            self.store(Base::State(at), Lane::of(1, self.sh, planes), &v);
        }
        // Whether an edge came in any lane of this register, kept for the
        // function's result.
        let mut all = M::K(false);
        for r in &runs {
            all = self.mor(all, r.0);
        }
        if let M::V(_) = all {
            let nz = self.any(all);
            let (z, l, o) = (self.tmp(), self.tmp(), self.tmp());
            let _ = writeln!(
                self.out,
                "  %t{z} = zext i1 {nz} to i8\n  \
                 %t{l} = load i8, ptr %fired, align 1\n  %t{o} = or i8 %t{l}, %t{z}\n  store i8 %t{o}, ptr %fired, align 1"
            );
        } else if all == M::K(true) {
            let _ = writeln!(self.out, "  store i8 1, ptr %fired, align 1");
        }
        self.ff = true;
        for (fire, body) in runs {
            self.cond(fire, body, None)?;
            self.env.vals.retain(|o, _| d.object(*o).lifetime == Lifetime::Static);
        }
        self.ff = false;
        for o in self.lay().regs.clone() {
            let shadow = self.lay().shadow.get(&o).copied();
            let v = match (self.env.nba.get(&o), shadow) {
                (Some(v), _) => v.clone(),
                (None, Some(_)) => self.read(o),
                (None, None) => continue,
            };
            let lane = self.lay().lane(o);
            let at = match shadow {
                Some(at) => Base::State(at),
                None => self.slot(o).expect("a register has a slot"),
            };
            self.store(at, lane, &v);
        }
        Ok(())
    }

    /// The lanes stopped after a branch on `m`, from those stopped in each
    /// arm (none, where an arm says nothing).
    fn merge_stop(&mut self, m: M, a: Option<M>, b: Option<M>) -> Option<M> {
        match (a, b) {
            (Some(x), Some(y)) if x == y => Some(x),
            (x, y) if x.is_some() || y.is_some() => {
                let (x, y) = (x.unwrap_or(M::K(false)), y.unwrap_or(M::K(false)));
                let t = self.mand(m, x);
                let nm = self.mnot(m);
                let e = self.mand(nm, y);
                Some(self.mor(t, e))
            }
            _ => None,
        }
    }

    /// The lanes that have left the loop or returned from the function.
    fn stopped(&mut self) -> Option<M> {
        match (self.env.brk, self.env.ret) {
            (Some(a), Some(b)) => Some(self.mor(a, b)),
            (a, b) => a.or(b),
        }
    }

    /// A call of a function, written out where it is called: its formals
    /// and locals are values of this loop, and a `return` stops the lanes
    /// that take it.
    fn call(&mut self, sub: SubId, args: &[ExprId]) -> Result<Option<Val>, String> {
        let d = self.d();
        let s = &d.subroutines[sub.0 as usize];
        if s.kind != SubKind::Function {
            return Err(format!("task {} is not supported yet", d.name(s.name)));
        }
        if self.calls.iter().any(|c| c.0 == sub) || self.calls.len() > 64 {
            return Err(format!("{} calls itself", d.name(s.name)));
        }
        let mut ins = Vec::new();
        for (i, &f) in s.args.iter().enumerate() {
            let w = tw(&d.types, d.object(f).ty);
            let v = match (d.object(f).dir, args.get(i)) {
                (Some(Dir::Input) | None, Some(&a)) => {
                    let v = self.expr(a)?;
                    self.resize(&v, w, d.types.is_signed(d.expr(a).ty))
                }
                (Some(Dir::Output), _) | (_, None) => self.zero(w),
                (dir, _) => return Err(format!("{dir:?} arguments of {} are not supported yet", d.name(s.name))),
            };
            ins.push((f, v));
        }
        let r = match d.scope(s.scope).names.get(&s.name) {
            Some(Binding::Object(r)) => Some(*r),
            _ => None,
        };
        let locals = self.l.sub_locals(sub);
        let mut saved = Vec::new();
        for &o in locals.iter().chain(r.iter()) {
            saved.push((o, self.env.vals.remove(&o)));
        }
        for (f, v) in ins {
            self.env.vals.insert(f, v);
        }
        if let Some(r) = r {
            self.env.vals.insert(r, self.zero(tw(&d.types, d.object(r).ty)));
        }
        let outer = (self.env.brk.take(), self.env.ret.take());
        self.calls.push((sub, r));
        let mut result = Ok(());
        for &o in locals.iter() {
            if s.args.contains(&o) {
                continue;
            }
            let w = tw(&d.types, d.object(o).ty);
            let v = match d.object(o).init {
                Some(e) => self.expr(e).map(|v| self.resize(&v, w, false)),
                None => Ok(self.zero(w)),
            };
            match v {
                Ok(v) => {
                    self.env.vals.insert(o, v);
                }
                Err(e) => result = Err(e),
            }
        }
        for &st in &s.body {
            if result.is_err() {
                break;
            }
            result = match self.stopped() {
                Some(M::K(true)) => break,
                Some(m @ M::V(_)) => {
                    let go = self.mnot(m);
                    self.branch(go, |f| f.stmt(st), |_| Ok(()))
                }
                _ => self.stmt(st),
            };
        }
        self.calls.pop();
        (self.env.brk, self.env.ret) = outer;
        let value = r.map(|r| self.read(r));
        let mut outs = Vec::new();
        for (i, &f) in s.args.iter().enumerate() {
            if d.object(f).dir == Some(Dir::Output)
                && let Some(&a) = args.get(i)
            {
                outs.push((a, self.read(f)));
            }
        }
        for (o, v) in saved {
            match v {
                Some(v) => self.env.vals.insert(o, v),
                None => self.env.vals.remove(&o),
            };
        }
        result?;
        for (a, v) in outs {
            self.assign(a, v)?;
        }
        Ok(value)
    }

    /// The address of element `i` (a constant) of `o`, in this register's
    /// part of the state.
    fn elem_addr(&mut self, o: ObjId, i: u32) -> String {
        let e = self.lay().elems[&o];
        let base = self.reg_base(e.lane);
        let a = self.tmp();
        let _ = writeln!(self.out, "  %t{a} = getelementptr i8, ptr {base}, i64 {}", self.lay().objs[&o] + i as u64 * e.stride());
        format!("%t{a}")
    }

    /// Lane 0 of a piece, an `i64` operand (zero-extended from a lane).
    fn lane0(&mut self, p: P) -> String {
        match p {
            P::K(k) => format!("{k}"),
            P::V(n) if self.sc() => format!("%t{n}"),
            P::V(n) => {
                let (x, z) = (self.tmp(), self.tmp());
                let _ = writeln!(self.out, "  %t{x} = extractelement {} %t{n}, i64 0\n  %t{z} = zext i32 %t{x} to i64", self.t.v);
                format!("%t{z}")
            }
        }
    }

    /// The address of element `i`, the same in every lane (held at the
    /// last element).
    fn elem_addr_uniform(&mut self, o: ObjId, i: P, clamp: bool) -> String {
        if let P::K(k) = i {
            let n = self.lay().elems[&o].n;
            return self.elem_addr(o, k.min(n as u64 - 1) as u32);
        }
        let e = self.lay().elems[&o];
        let base = self.reg_base(e.lane);
        let off = self.lay().objs[&o];
        let ix = self.lane0(i);
        let t: Vec<u32> = (0..6).map(|_| self.tmp()).collect();
        // Held at the last element where the index may be out of range
        // (its value is then not used): a compare and select, which a
        // loop's invariants can be hoisted past, not a call.
        let ix = if clamp {
            let _ = writeln!(self.out, "  %t{} = icmp ugt i64 {ix}, {}\n  %t{} = select i1 %t{}, i64 {}, i64 {ix}", t[0], e.n - 1, t[1], t[0], e.n - 1);
            format!("%t{}", t[1])
        } else {
            ix
        };
        let _ = writeln!(
            self.out,
            "  %t{} = mul i64 {ix}, {}\n  %t{} = getelementptr i8, ptr {base}, i64 {off}\n  %t{} = getelementptr i8, ptr %t{}, i64 %t{}",
            t[2],
            e.stride(),
            t[3],
            t[4],
            t[3],
            t[2]
        );
        format!("%t{}", t[4])
    }

    /// The storage type of an element slot, and its alignment.
    fn slot_ty(&self, e: Elems) -> (&'static str, u64) {
        match e.lane {
            Lane::Bit => ("i16", 2),
            Lane::B8 => ("i8", 1),
            Lane::B16 => ("i16", 2),
            Lane::W32 if self.sc() => ("i32", 4),
            Lane::W32 => (self.t.w32, 4 * self.sh.n as u64),
            Lane::W64 => ("i64", 8),
        }
    }

    fn load_slot(&mut self, o: ObjId, addr: &str) -> Val {
        let e = self.lay().elems[&o];
        let (ty, align) = self.slot_ty(e);
        let t = self.tmp();
        let _ = writeln!(self.out, "  %t{t} = load {ty}, ptr {addr}, align {align}");
        if e.planes() {
            let m = self.tmp();
            let _ = writeln!(self.out, "  %t{m} = bitcast i16 %t{t} to {}", self.t.m);
            self.bits_of.insert(m, t);
            return self.bit(M::V(m), 1);
        }
        if ty != self.t.v {
            let z = self.tmp();
            let _ = writeln!(self.out, "  %t{z} = zext {ty} %t{t} to {}", self.t.v);
            self.zeros.insert(z, !self.top_mask(e.ew));
            return Val { w: e.ew, p: vec![P::V(z)] };
        }
        Val { w: e.ew, p: vec![P::V(t)] }
    }

    fn store_slot(&mut self, o: ObjId, addr: &str, v: &Val) {
        let e = self.lay().elems[&o];
        let (ty, align) = self.slot_ty(e);
        if e.planes() {
            let m = self.truth(v);
            let bits = self.mask_bits(m);
            let _ = writeln!(self.out, "  store i16 {bits}, ptr {addr}, align 2");
        } else {
            let v = self.resize(v, e.ew, false);
            let x = if ty != self.t.v {
                let x = self.tmp();
                let _ = writeln!(self.out, "  %t{x} = trunc {} {} to {ty}", self.t.v, self.ps(v.p[0]));
                format!("%t{x}")
            } else {
                self.ps(v.p[0])
            };
            let _ = writeln!(self.out, "  store {ty} {x}, ptr {addr}, align {align}");
        }
    }

    /// Element `i` (a constant) of an object kept an element to a slot.
    fn read_elem(&mut self, o: ObjId, i: u32) -> Val {
        if let Some(v) = self.env.elems.get(&(o, i)).or(self.env.eloaded.get(&(o, i))) {
            return v.clone();
        }
        let a = self.elem_addr(o, i);
        let v = self.load_slot(o, &a);
        self.env.eloaded.insert((o, i), v.clone());
        v
    }

    fn write_elem(&mut self, o: ObjId, i: u32, v: Val) {
        let ew = self.lay().elems[&o].ew;
        let v = self.resize(&v, ew, false);
        if self.mem {
            let a = self.elem_addr(o, i);
            self.store_slot(o, &a, &v);
        } else {
            self.env.edirty.insert((o, i));
        }
        self.env.elems.insert((o, i), v);
    }

    /// Stores the elements written since the loop began: of `o`, or of all.
    fn flush_elems(&mut self, o: Option<ObjId>) {
        let mut dirty: Vec<(ObjId, u32)> = self.env.edirty.iter().copied().filter(|k| o.is_none_or(|o| k.0 == o)).collect();
        dirty.sort_unstable_by_key(|k| (k.0.0, k.1));
        for k in dirty {
            self.env.edirty.remove(&k);
            let v = self.env.elems[&k].clone();
            let a = self.elem_addr(k.0, k.1);
            self.store_slot(k.0, &a, &v);
        }
    }

    /// Which element of an object kept an element to a slot `e` (an
    /// `Index` of it) selects: a constant, or one a lane, with whether it
    /// is in range.
    fn elem_index(&mut self, e: ExprId) -> Result<(P, M), String> {
        let d = self.d();
        let x = d.expr(e);
        let bt = d.expr(x.args[0]).ty;
        let (l, r) = range_of(&d.types, bt);
        let idx = match self.l.fold(x.args[1]) {
            Some(c) => self.konst_of(&c, tw(&d.types, d.expr(x.args[1]).ty))?,
            None => self.expr(x.args[1])?,
        };
        let signed = d.types.is_signed(d.expr(x.args[1]).ty);
        let k = idx.p[0];
        if self.in_range.contains(&e) {
            let i = if l >= r { self.bin("sub", k, self.k(r)) } else { self.bin("sub", self.k(r), k) };
            return Ok((i, M::K(true)));
        }
        let mut ok = M::K(true);
        for &hp in &idx.p[1..] {
            let z = self.icmp("eq", hp, P::K(0));
            ok = self.mand(ok, z);
        }
        let (min, max) = (l.min(r), l.max(r));
        let (ge, le) = if signed { ("sge", "sle") } else { ("uge", "ule") };
        let a = self.icmp(ge, k, self.k(min));
        let b = self.icmp(le, k, self.k(max));
        ok = self.mand(ok, a);
        ok = self.mand(ok, b);
        let i = if l >= r { self.bin("sub", k, self.k(r)) } else { self.bin("sub", self.k(r), k) };
        Ok((i, ok))
    }

    /// `o[...]` read, for an object kept an element to a slot.
    fn elem_read(&mut self, o: ObjId, e: ExprId) -> Result<Val, String> {
        let el = self.lay().elems[&o];
        let (i, ok) = self.elem_index(e)?;
        let v = match i {
            P::K(k) if k < el.n as u64 => self.read_elem(o, k as u32),
            P::K(_) => self.zero(el.ew),
            _ if self.is_uniform(i) => {
                let a = self.elem_addr_uniform(o, i, ok != M::K(true));
                self.load_slot(o, &a)
            }
            // A different element a lane: each, chosen by a select.
            _ => {
                let mut acc = self.zero(el.ew);
                for k in 0..el.n {
                    let hit = self.icmp("eq", i, P::K(k as u64));
                    let v = self.read_elem(o, k);
                    acc = self.select(hit, &v, &acc);
                }
                acc
            }
        };
        if ok == M::K(true) {
            return Ok(v);
        }
        let z = self.zero(el.ew);
        Ok(self.select(ok, &v, &z))
    }

    /// `o[...] = v`, for an object kept an element to a slot.
    fn elem_write(&mut self, o: ObjId, e: ExprId, v: Val) -> Result<(), String> {
        let el = self.lay().elems[&o];
        let (i, ok) = self.elem_index(e)?;
        match i {
            P::K(k) if k < el.n as u64 => {
                let k = k as u32;
                let v = match ok {
                    M::K(true) => v,
                    _ => {
                        let old = self.read_elem(o, k);
                        self.select(ok, &v, &old)
                    }
                };
                self.write_elem(o, k, v);
            }
            P::K(_) => {}
            _ if self.is_uniform(i) && self.mem => {
                // Any element may change: none is remembered.
                self.flush_elems(Some(o));
                self.env.elems.retain(|k, _| k.0 != o);
                self.env.eloaded.retain(|k, _| k.0 != o);
                let a = self.elem_addr_uniform(o, i, ok != M::K(true));
                let v = match ok {
                    M::K(true) => v,
                    _ => {
                        let old = self.load_slot(o, &a);
                        self.select(ok, &v, &old)
                    }
                };
                self.store_slot(o, &a, &v);
            }
            _ => {
                for k in 0..el.n {
                    let hit = self.icmp("eq", i, P::K(k as u64));
                    let hit = self.mand(hit, ok);
                    let old = self.read_elem(o, k);
                    let new = self.select(hit, &v, &old);
                    self.write_elem(o, k, new);
                }
            }
        }
        Ok(())
    }

    /// The object kept an element to a slot that an `Index` selects from
    /// directly, if its element is what the `Index` gives.
    fn elem_target(&self, e: ExprId) -> Option<ObjId> {
        let d = self.d();
        let x = d.expr(e);
        if !matches!(x.kind, ExprKind::Index) {
            return None;
        }
        let ExprKind::Object(o) = d.expr(x.args[0]).kind else { return None };
        let el = self.lay().elems.get(&o)?;
        (tw(&d.types, x.ty) == el.ew).then_some(o)
    }

    /// An object's value: from this loop's values, from the state, or zero
    /// for an automatic variable not yet set.
    fn read(&mut self, o: ObjId) -> Val {
        if let Some(&e) = self.lay().elems.get(&o) {
            // The whole value, from its elements.
            let mut acc = self.zero(e.n * e.ew);
            for i in 0..e.n {
                let v = self.read_elem(o, i);
                acc = self.insert(&acc, i * e.ew, e.ew, &v);
            }
            return acc;
        }
        if self.lay().kept.contains(&o) {
            let w = self.width(o);
            let p = (0..self.np(w)).map(|j| self.load_piece(o, j)).collect();
            return Val { w, p };
        }
        if let Some(v) = self.env.vals.get(&o).or(self.env.loaded.get(&o)) {
            return v.clone();
        }
        let w = self.width(o);
        match self.slot(o) {
            Some(base) => {
                let lane = self.lay().lane(o);
                let v = self.load(base, lane, w);
                self.env.loaded.insert(o, v.clone());
                v
            }
            None => {
                let v = self.zero(w);
                self.env.vals.insert(o, v.clone());
                v
            }
        }
    }

    /// An object's width as it is kept: padded, for a [`Pad`].
    fn width(&self, o: ObjId) -> u32 {
        match self.lay().pads.get(&o) {
            Some(p) => p.width(),
            None => tw(&self.d().types, self.d().object(o).ty),
        }
    }

    /// A value of a padded array's type, padded.
    fn pad(&mut self, p: Pad, v: &Val) -> Val {
        let mut acc = self.zero(p.width());
        for k in 0..p.n {
            let x = self.extract(v, k * p.ew, p.ew);
            acc = self.insert(&acc, k * p.stride(), p.ew, &x);
        }
        acc
    }

    fn unpad(&mut self, p: Pad, v: &Val) -> Val {
        let mut acc = self.zero(p.n * p.ew);
        for k in 0..p.n {
            let x = self.extract(v, k * p.stride(), p.ew);
            acc = self.insert(&acc, k * p.ew, p.ew, &x);
        }
        acc
    }

    /// Where a select of a padded array is in it as kept, if within one
    /// element.
    fn padded_off(&self, p: Pad, off: Off, w: u32) -> Option<Off> {
        match off {
            Off::K(k) if k % p.ew + w <= p.ew => Some(Off::K(k / p.ew * p.stride() + k % p.ew)),
            Off::Dec { i, n, stride, add, ok } if stride == p.ew && add + w <= p.ew => Some(Off::Dec { i, n, stride: p.stride(), add, ok }),
            _ => None,
        }
    }

    /// A whole padded array copied from another of the same shape: its
    /// value as kept.
    fn padded_copy(&mut self, lhs: ExprId, rhs: ExprId) -> Option<(ObjId, Val)> {
        let d = self.d();
        let ExprKind::Object(a) = d.expr(lhs).kind else { return None };
        let mut x = d.expr(rhs);
        while matches!(x.kind, ExprKind::Convert) && tw(&d.types, x.ty) == tw(&d.types, d.expr(x.args[0]).ty) {
            x = d.expr(x.args[0]);
        }
        let ExprKind::Object(b) = x.kind else { return None };
        let pa = *self.lay().pads.get(&a)?;
        (self.lay().pads.get(&b) == Some(&pa)).then(|| (a, self.read(b)))
    }

    /// Piece `j` of a value kept in the state, from there.
    fn load_piece(&mut self, o: ObjId, j: usize) -> P {
        let lane = self.piece_lane();
        let at = self.lay().objs[&o] + j as u64 * lane.piece_bytes(self.sh);
        let (a, v) = (self.tmp(), self.tmp());
        let base = self.base();
        let _ = writeln!(self.out, "  %t{a} = getelementptr i8, ptr {base}, i64 {at}\n  %t{v} = load {}, ptr %t{a}, align {}", self.t.v, lane.reg_bytes(self.sh));
        P::V(v)
    }

    /// Stores `x` as piece `j` of a padded array in the lanes of `m` that
    /// run this code; `old` is what is there.
    fn store_piece_over(&mut self, o: ObjId, j: usize, x: P, m: M, old: Option<P>) {
        let m = self.mand(m, self.path);
        if m == M::K(false) || old == Some(x) {
            return;
        }
        let x = match (m, old) {
            (M::K(true), _) => x,
            (m, Some(old)) => self.sel(m, x, old),
            (m, None) => {
                let old = self.load_piece(o, j);
                self.sel(m, x, old)
            }
        };
        let lane = self.piece_lane();
        let at = self.lay().objs[&o] + j as u64 * lane.piece_bytes(self.sh);
        let a = self.tmp();
        let base = self.base();
        let _ = writeln!(self.out, "  %t{a} = getelementptr i8, ptr {base}, i64 {at}\n  store {} {}, ptr %t{a}, align {}", self.t.v, self.ps(x), lane.reg_bytes(self.sh));
    }

    fn store_piece(&mut self, o: ObjId, j: usize, x: P, m: M) {
        self.store_piece_over(o, j, x, m, None)
    }

    /// Where a select of a value kept in the state is there (see
    /// [`Self::padded_off`]), if it is a constant or one of few elements.
    fn kept_off(&self, o: ObjId, off: Off, w: u32) -> Option<Off> {
        match (self.lay().pads.get(&o), off) {
            (Some(&p), off) => self.padded_off(p, off, w),
            (None, off @ (Off::K(_) | Off::Dec { .. })) => Some(off),
            _ => None,
        }
    }

    /// A value as its type has it, from how it is kept.
    fn logical(&mut self, o: ObjId, v: &Val) -> Val {
        match self.lay().pads.get(&o).copied() {
            Some(p) => self.unpad(p, v),
            None => v.clone(),
        }
    }

    /// The `w` bits at `off` in `o`.
    fn read_at(&mut self, o: ObjId, off: Off, w: u32) -> Val {
        if !self.lay().kept.contains(&o) {
            let v = self.read(o);
            return self.extract_off(&v, off, w);
        }
        let pb = self.pb;
        let n = self.width(o).div_ceil(pb);
        match self.kept_off(o, off, w) {
            // The pieces the bits are in, of each element they may be in.
            Some(Off::K(k)) => {
                let (q, r) = ((k / pb) as usize, k % pb);
                let span = (r + w).div_ceil(pb) as usize;
                let v = Val { w: pb * span as u32, p: (0..span).map(|j| self.load_piece(o, q + j)).collect() };
                self.extract(&v, r, w)
            }
            Some(Off::Dec { i, n: count, stride, add, ok }) if stride % pb == 0 => {
                let (q, r) = ((add / pb) as usize, add % pb);
                let span = (r + w).div_ceil(pb) as usize;
                let mut acc = vec![P::K(0); span];
                for k in 0..count {
                    let at = (k * stride / pb) as usize + q;
                    if at + span > n as usize {
                        break;
                    }
                    let hit = self.icmp("eq", i, P::K(k as u64));
                    if hit == M::K(false) {
                        continue;
                    }
                    for (j, a) in acc.iter_mut().enumerate() {
                        let x = self.load_piece(o, at + j);
                        *a = self.sel(hit, x, *a);
                    }
                }
                let acc = Val { w: pb * span as u32, p: acc };
                let x = self.extract(&acc, r, w);
                let z = self.zero(w);
                self.select(ok, &x, &z)
            }
            _ => {
                let v = self.read(o);
                let v = self.logical(o, &v);
                self.extract_off(&v, off, w)
            }
        }
    }

    /// Sets the `w` bits at `off` in a value kept in the state to `x`.
    fn write_kept(&mut self, o: ObjId, off: Off, w: u32, x: &Val) -> Result<(), String> {
        let pb = self.pb;
        let n = self.width(o).div_ceil(pb) as usize;
        // Each element's pieces that the bits are in, where `hit`.
        let put = |f: &mut Self, at: usize, r: u32, hit: M| {
            let span = (r + w).div_ceil(pb) as usize;
            let old: Vec<P> = (0..span).map(|j| f.load_piece(o, at + j)).collect();
            let v = f.insert(&Val { w: pb * span as u32, p: old.clone() }, r, w, x);
            for j in 0..span {
                f.store_piece_over(o, at + j, v.p[j], hit, Some(old[j]));
            }
        };
        match self.kept_off(o, off, w) {
            Some(Off::K(k)) => put(self, (k / pb) as usize, k % pb, M::K(true)),
            Some(Off::Dec { i, n: count, stride, add, ok }) if stride % pb == 0 => {
                for k in 0..count {
                    let at = ((k * stride + add) / pb) as usize;
                    if at + (add % pb + w).div_ceil(pb) as usize > n {
                        break;
                    }
                    let hit = self.icmp("eq", i, P::K(k as u64));
                    let hit = self.mand(hit, ok);
                    if hit != M::K(false) {
                        put(self, at, add % pb, hit);
                    }
                }
            }
            _ => {
                let old = self.read(o);
                let u = self.logical(o, &old);
                let u = self.insert_off(&u, off, w, x)?;
                let new = match self.lay().pads.get(&o).copied() {
                    Some(p) => self.pad(p, &u),
                    None => u,
                };
                for j in 0..new.p.len() {
                    self.store_piece_over(o, j, new.p[j], M::K(true), Some(old.p[j]));
                }
            }
        }
        Ok(())
    }

    /// `old`, the value of `o` as kept, with the `w` bits at `off` set to `v`.
    fn write_at(&mut self, o: ObjId, old: &Val, off: Off, w: u32, v: &Val) -> Result<Val, String> {
        match self.lay().pads.get(&o).copied() {
            Some(p) => match self.padded_off(p, off, w) {
                Some(off) => self.insert_off(old, off, w, v),
                None => {
                    let u = self.unpad(p, old);
                    let u = self.insert_off(&u, off, w, v)?;
                    Ok(self.pad(p, &u))
                }
            },
            None => self.insert_off(old, off, w, v),
        }
    }

    fn set(&mut self, o: ObjId, v: Val) {
        if self.lay().kept.contains(&o) {
            for (j, &x) in v.p.iter().enumerate() {
                self.store_piece(o, j, x, M::K(true));
            }
            return;
        }
        if let Some(&e) = self.lay().elems.get(&o) {
            for i in 0..e.n {
                let x = self.extract(&v, i * e.ew, e.ew);
                self.write_elem(o, i, x);
            }
            return;
        }
        if let Some(base) = self.slot(o) {
            if self.mem {
                let lane = self.lay().lane(o);
                self.store(base, lane, &v);
            } else {
                self.env.dirty.insert(o);
            }
        }
        self.env.vals.insert(o, v);
    }

    fn assign(&mut self, lhs: ExprId, v: Val) -> Result<(), String> {
        let d = self.d();
        let x = d.expr(lhs);
        match &x.kind {
            ExprKind::Object(o) => {
                let w = tw(&d.types, d.object(*o).ty);
                let v = self.resize(&v, w, false);
                let v = match self.lay().pads.get(o).copied() {
                    Some(p) => self.pad(p, &v),
                    None => v,
                };
                self.set(*o, v);
                Ok(())
            }
            ExprKind::Concat => {
                let mut top: u32 = x.args.iter().map(|&a| tw(&d.types, d.expr(a).ty)).sum();
                for &a in &x.args {
                    let w = tw(&d.types, d.expr(a).ty);
                    top -= w;
                    let part = self.extract(&v, top, w);
                    self.assign(a, part)?;
                }
                Ok(())
            }
            ExprKind::Index | ExprKind::Part { .. } | ExprKind::Member(_) => {
                if let Some(o) = self.elem_target(lhs) {
                    return self.elem_write(o, lhs, v);
                }
                let (o, off, w) = self.place(lhs)?;
                if self.lay().kept.contains(&o) {
                    let v = self.resize(&v, w, false);
                    return self.write_kept(o, off, w, &v);
                }
                if self.mem
                    && !self.lay().elems.contains_key(&o)
                    && !self.lay().kept.contains(&o)
                    && self.lay().lane(o) == self.piece_lane()
                    && w <= self.pb
                    && self.lay().objs.contains_key(&o)
                    && let Some((p, ok)) = self.uniform_off(off)
                {
                    self.mem_insert(o, p, ok, w, &v);
                    return Ok(());
                }
                let old = if self.lay().fresh.contains(&o) && !self.env.vals.contains_key(&o) {
                    self.zero(self.width(o))
                } else {
                    self.read(o)
                };
                let new = self.write_at(o, &old, off, w, &v)?;
                self.set(o, new);
                Ok(())
            }
            k => Err(format!("assignments to {k:?} are not supported yet")),
        }
    }

    /// Where a select on the left of an assignment is: its object, its bit
    /// offset there, its width.
    fn place(&mut self, e: ExprId) -> Result<(ObjId, Off, u32), String> {
        let d = self.d();
        let x = d.expr(e);
        if let ExprKind::Object(o) = x.kind {
            return Ok((o, Off::K(0), tw(&d.types, x.ty)));
        }
        let (o, base, _) = self.place(x.args[0])?;
        let (off, w) = self.offset(e)?;
        let off = match (base, off) {
            (Off::K(a), Off::K(b)) => Off::K(a + b),
            (Off::Neg(_), _) | (_, Off::Neg(_)) => return Err("a part select partly out of range as a target".into()),
            (Off::K(k), Off::Dec { i, n, stride, add, ok }) | (Off::Dec { i, n, stride, add, ok }, Off::K(k)) => {
                Off::Dec { i, n, stride, add: add + k, ok }
            }
            (a, b) => {
                let (pa, ma) = self.off_var(a);
                let (pb, mb) = self.off_var(b);
                let p = self.bin("add", pa, pb);
                let m = self.mand(ma, mb);
                Off::V(p, m)
            }
        };
        Ok((o, off, w))
    }

    /// A select's bit offset as a value a lane, and whether it is in range.
    fn off_var(&mut self, off: Off) -> (P, M) {
        match off {
            Off::K(k) => (P::K(k as u64), M::K(true)),
            Off::Neg(_) => (P::K(0), M::K(false)),
            Off::V(p, m) => (p, m),
            Off::Dec { i, stride, add, ok, .. } => {
                let p = if stride.is_power_of_two() {
                    self.bin("shl", i, P::K(stride.trailing_zeros() as u64))
                } else {
                    self.bin("mul", i, P::K(stride as u64))
                };
                (self.bin("add", p, P::K(add as u64)), ok)
            }
        }
    }

    /// Whether choosing among `n` elements costs less than shifting a
    /// value of `len` pieces, for `w` bits.
    fn decodes(&self, len: usize, n: u32, w: u32) -> bool {
        n as usize * (3 * self.np(w) + 2) < 3 * len * len
    }

    /// The `w` bits of `v` at `off`.
    fn extract_off(&mut self, v: &Val, off: Off, w: u32) -> Val {
        match off {
            Off::K(k) => self.extract(v, k, w),
            Off::Neg(k) => {
                let v = self.resize(v, v.w + k, false);
                let v = self.shl_const(&v, k as u64);
                self.extract(&v, 0, w)
            }
            // Elements at whole pieces: the pieces chosen, then one shift.
            Off::Dec { i, n, stride, add, ok } if stride % self.pb == 0 && self.decodes(v.p.len(), n, w) => {
                let pb = self.pb;
                let (q, r) = ((add / pb) as usize, add % pb);
                let span = (r + w).div_ceil(pb) as usize;
                let mut acc = vec![P::K(0); span];
                for k in 0..n {
                    let at = (k * stride / pb) as usize + q;
                    if at >= v.p.len() {
                        break;
                    }
                    let hit = self.icmp("eq", i, P::K(k as u64));
                    for (j, a) in acc.iter_mut().enumerate() {
                        let x = v.p.get(at + j).copied().unwrap_or(P::K(0));
                        *a = self.sel(hit, x, *a);
                    }
                }
                let acc = Val { w: pb * span as u32, p: acc };
                let x = self.extract(&acc, r, w);
                let z = self.zero(w);
                self.select(ok, &x, &z)
            }
            Off::Dec { i, n, stride, add, ok } if self.decodes(v.p.len(), n, w) => {
                let mut acc = self.zero(w);
                for k in 0..n {
                    if k * stride + add >= v.w {
                        break;
                    }
                    let hit = self.icmp("eq", i, P::K(k as u64));
                    let x = self.extract(v, k * stride + add, w);
                    acc = self.select(hit, &x, &acc);
                }
                let z = self.zero(w);
                self.select(ok, &acc, &z)
            }
            _ => {
                let (p, ok) = self.off_var(off);
                let v = self.lshr_var(v, p, None);
                let v = self.extract(&v, 0, w);
                let z = self.zero(w);
                self.select(ok, &v, &z)
            }
        }
    }

    /// `v` with the `w` bits at `off` replaced by `x`.
    fn insert_off(&mut self, v: &Val, off: Off, w: u32, x: &Val) -> Result<Val, String> {
        Ok(match off {
            Off::K(k) => self.insert(v, k, w, x),
            Off::Neg(_) => return Err("a part select partly out of range as a target".into()),
            // Elements at whole pieces: `x` shifted into place once, then
            // each element's pieces chosen.
            Off::Dec { i, n, stride, add, ok } if stride % self.pb == 0 && self.decodes(v.p.len(), n, w) => {
                let pb = self.pb;
                let (q, r) = ((add / pb) as usize, add % pb);
                let span = (r + w).div_ceil(pb) as usize;
                let zero = self.zero(pb * span as u32);
                let xs = self.insert(&zero, r, w, x);
                let ones = self.ones(w);
                let m = self.insert(&zero, r, w, &ones);
                let mut acc = v.clone();
                for k in 0..n {
                    let at = (k * stride / pb) as usize + q;
                    if at >= v.p.len() {
                        break;
                    }
                    let hit = self.icmp("eq", i, P::K(k as u64));
                    let hit = self.mand(hit, ok);
                    for j in 0..span.min(v.p.len() - at) {
                        let old = acc.p[at + j];
                        let new = match m.p[j] {
                            P::K(all) if all == self.ones_k() => xs.p[j],
                            mj => {
                                let keep = self.bin("xor", mj, P::K(self.ones_k()));
                                let kept = self.bin("and", old, keep);
                                self.bin("or", kept, xs.p[j])
                            }
                        };
                        acc.p[at + j] = self.sel(hit, new, old);
                    }
                }
                self.norm(acc)
            }
            Off::Dec { i, n, stride, add, ok } if self.decodes(v.p.len(), n, w) => {
                let mut acc = v.clone();
                for k in 0..n {
                    if k * stride + add >= v.w {
                        break;
                    }
                    let hit = self.icmp("eq", i, P::K(k as u64));
                    let hit = self.mand(hit, ok);
                    let new = self.insert(&acc, k * stride + add, w, x);
                    acc = self.select(hit, &new, &acc);
                }
                acc
            }
            _ => {
                let (p, ok) = self.off_var(off);
                self.insert_var(v, p, ok, w, x)
            }
        })
    }

    /// The object, offset and width of a select of selects of an object,
    /// if one is variable: read as one select, not each in turn.
    fn read_place(&mut self, e: ExprId) -> Result<Option<(ObjId, Off, u32)>, String> {
        let d = self.d();
        let mut at = e;
        let mut var = false;
        loop {
            let x = d.expr(at);
            match x.kind {
                ExprKind::Index | ExprKind::Part { .. } | ExprKind::Member(_) => {
                    if !matches!(x.kind, ExprKind::Member(_)) && self.l.const_select(at).is_none() {
                        var = true;
                    }
                    at = x.args[0];
                }
                ExprKind::Object(o)
                    if (var || self.lay().kept.contains(&o)) && matches!(d.object(o).kind, ObjKind::Variable | ObjKind::Net(_)) =>
                {
                    break;
                }
                _ => return Ok(None),
            }
        }
        match self.place(e) {
            Ok(p) => Ok(Some(p)),
            Err(_) => Ok(None),
        }
    }

    /// The bit offset of a select in its base, and its width.
    fn offset(&mut self, e: ExprId) -> Result<(Off, u32), String> {
        let d = self.d();
        let tys = &d.types;
        let x = d.expr(e);
        if let Some(c) = self.l.const_select(e) {
            return Ok(match c {
                Some((off, w)) => (Off::K(off as u32).with_sign(off), w),
                None => (Off::V(P::K(0), M::K(false)), tw(tys, x.ty)),
            });
        }
        let bt = d.expr(x.args[0]).ty;
        let (l, r) = range_of(tys, bt);
        let idx = self.expr(x.args[1])?;
        let signed = tys.is_signed(d.expr(x.args[1]).ty);
        let (lo_index, ew, w) = match &x.kind {
            ExprKind::Index => {
                let ew = tw(tys, x.ty).max(1);
                (idx.clone(), ew, ew)
            }
            ExprKind::Part { kind, width } => {
                let ew = elem_bits(tys, bt).max(1);
                let lo = match kind {
                    PartKind::Up => idx.clone(),
                    PartKind::Down => {
                        let k = self.konst(&Bits::from_u64(idx.w, false, *width as u64 - 1));
                        self.add(&idx, &k, true)
                    }
                    PartKind::Range => return Err("a part select with variable bounds".into()),
                };
                (lo, ew, *width * ew)
            }
            _ => return Err(format!("a select {:?} with variable indices", x.kind)),
        };
        // Elements from the least significant end: k - r if descending,
        // r - (k + n - 1) if ascending; in range if min <= k <= max - n + 1.
        let n = (w / ew) as i64;
        let k = lo_index.p[0];
        let mut ok = M::K(true);
        for &hp in &lo_index.p[1..] {
            let z = self.icmp("eq", hp, P::K(0));
            ok = self.mand(ok, z);
        }
        let (min, max) = (l.min(r), l.max(r));
        let (lt, le) = if signed { ("sge", "sle") } else { ("uge", "ule") };
        let a = self.icmp(lt, k, self.k(min));
        // A part select that runs off the top reads and writes the bits
        // still in range.
        let top = if matches!(x.kind, ExprKind::Part { .. }) && l >= r { max } else { max - n + 1 };
        let b = self.icmp(le, k, self.k(top));
        ok = self.mand(ok, a);
        ok = self.mand(ok, b);
        let elems = if l >= r { self.bin("sub", k, self.k(r)) } else { self.bin("sub", self.k(r - n + 1), k) };
        if matches!(x.kind, ExprKind::Index) {
            let n = (max - min + 1) as u32;
            // (A constant here, an unrolled loop's variable, is one element.)
            if let (P::K(k), M::K(true)) = (elems, ok)
                && k < n as u64
            {
                return Ok((Off::K(k as u32 * ew), w));
            }
            return Ok((Off::Dec { i: elems, n, stride: ew, add: 0, ok }, w));
        }
        let bits = if ew.is_power_of_two() { self.bin("shl", elems, P::K(ew.trailing_zeros() as u64)) } else { self.bin("mul", elems, P::K(ew as u64)) };
        Ok((Off::V(bits, ok), w))
    }

    /// A run of iterations as a loop around a loop over the group's
    /// registers: what depends only on the iteration (the stepping
    /// parameters, the elements' addresses and range checks) is worked out
    /// once an iteration, not once a register.
    fn rolled_loop(&mut self, l: usize) -> Result<(), String> {
        let d = self.d();
        let lp = &self.l.loops[l];
        let k = self.rolled;
        self.rolled += 1;
        let prev = std::mem::replace(&mut self.block, format!("R{k}"));
        let _ = writeln!(self.out, "  br label %R{k}\nR{k}:\n  %r{k} = phi i32 [ 0, %{prev} ], [ %r{k}.next, %R{k}.latch ]");
        let mut steps = Vec::new();
        for &(o, first, step) in &lp.steps {
            let (a, b, c, t) = (self.tmp(), self.tmp(), self.tmp(), self.tmp());
            let _ = writeln!(self.out, "  %t{a} = mul i32 %r{k}, {}\n  %t{b} = add i32 %t{a}, {}", step as i32, first as i32);
            if self.sc() {
                let _ = writeln!(self.out, "  %t{t} = sext i32 %t{b} to i64");
            } else {
                let v = self.t.v;
                let _ = writeln!(
                    self.out,
                    "  %t{c} = insertelement {v} poison, i32 %t{b}, i64 0\n  %t{t} = shufflevector {v} %t{c}, {v} poison, {v} zeroinitializer"
                );
            }
            self.uniform.insert(t);
            let w = tw(&d.types, d.object(o).ty);
            let mut p = vec![P::K(0); self.np(w)];
            p[0] = P::V(t);
            if self.np(w) > 1 {
                let f = self.bin("ashr", P::V(t), P::K(self.pb as u64 - 1));
                for q in &mut p[1..] {
                    *q = f;
                }
            }
            let v = self.norm(Val { w, p });
            steps.push((o, v));
        }
        self.mem = true;
        self.in_range = lp.in_range.clone();
        *self.l.loop_vars.borrow_mut() = lp.steps.iter().map(|s| s.0).collect();
        let result = self.run_loop_with(|f| {
            f.env.vals.extend(steps);
            lp.body.iter().try_for_each(|&dr| {
                let drv = &d.drivers[dr.0 as usize];
                let v = f.expr(drv.rhs)?;
                f.assign(drv.lhs, v)
            })
        });
        self.l.loop_vars.borrow_mut().clear();
        self.mem = false;
        self.in_range.clear();
        result?;
        let _ = writeln!(
            self.out,
            "  %r{k}.next = add i32 %r{k}, 1\n  %r{k}.end = icmp eq i32 %r{k}.next, {}\n  \
             br i1 %r{k}.end, label %Y{k}, label %R{k}\nY{k}:",
            lp.count
        );
        self.out = self.out.replace(&format!("%R{k}.latch"), &format!("%{}", self.block));
        self.block = format!("Y{k}");
        Ok(())
    }

    /// The static object an expression names, in a rolled loop, where
    /// selects of it go to the state.
    fn in_memory(&self, e: ExprId) -> Option<ObjId> {
        match self.d().expr(e).kind {
            ExprKind::Object(o)
                if self.mem
                    && self.lay().objs.contains_key(&o)
                    && !self.lay().elems.contains_key(&o)
                    && !self.lay().kept.contains(&o)
                    && self.lay().lane(o) == self.piece_lane() =>
            {
                Some(o)
            }
            _ => None,
        }
    }

    /// How values in whole pieces are stored.
    fn piece_lane(&self) -> Lane {
        if self.sc() { Lane::W64 } else { Lane::W32 }
    }

    /// A select's offset if it is the same in every lane.
    fn uniform_off(&mut self, off: Off) -> Option<(P, M)> {
        match off {
            Off::K(k) => Some((P::K(k as u64), M::K(true))),
            Off::V(p, ok) if self.is_uniform(p) => Some((p, ok)),
            Off::Dec { i, .. } if self.is_uniform(i) => Some(self.off_var(off)),
            _ => None,
        }
    }

    fn is_uniform(&self, p: P) -> bool {
        match p {
            P::K(_) => true,
            P::V(n) => self.uniform.contains(&n),
        }
    }

    /// The addresses of the two pieces holding bits `off..off + pb` of
    /// `o` (the second held at the last piece), and the shift within the
    /// first.
    fn piece_addrs(&mut self, o: ObjId, off: P) -> (String, String, P) {
        let d = self.d();
        let base = self.base();
        let start = self.lay().objs[&o];
        let lane = self.piece_lane();
        let stride = lane.piece_bytes(self.sh);
        let pb = self.pb as u64;
        let last = lane.pieces(tw(&d.types, d.object(o).ty)) - 1;
        let r = self.bin("and", off, P::K(pb - 1));
        if let P::K(k) = off {
            let q = (k / pb).min(last);
            let q1 = (q + 1).min(last);
            let (a, b) = (self.tmp(), self.tmp());
            let _ = writeln!(
                self.out,
                "  %t{a} = getelementptr i8, ptr {base}, i64 {}\n  %t{b} = getelementptr i8, ptr {base}, i64 {}",
                start + q * stride,
                start + q1 * stride
            );
            return (format!("%t{a}"), format!("%t{b}"), r);
        }
        let x = self.lane0(off);
        let n: Vec<u32> = (0..10).map(|_| self.tmp()).collect();
        let _ = writeln!(
            self.out,
            "  %t{0} = lshr i64 {x}, {lg}\n  %t{1} = icmp ugt i64 %t{0}, {last}\n  %t{2} = select i1 %t{1}, i64 {last}, i64 %t{0}\n  \
             %t{3} = add i64 %t{2}, 1\n  %t{4} = icmp ugt i64 %t{3}, {last}\n  %t{5} = select i1 %t{4}, i64 {last}, i64 %t{3}\n  \
             %t{6} = mul i64 %t{2}, {stride}\n  %t{7} = mul i64 %t{5}, {stride}\n  \
             %t{8} = getelementptr i8, ptr {base}, i64 {start}",
            n[0],
            n[1],
            n[2],
            n[3],
            n[4],
            n[5],
            n[6],
            n[7],
            n[8],
            lg = self.pb.trailing_zeros()
        );
        let (a, b) = (self.tmp(), self.tmp());
        let _ = writeln!(
            self.out,
            "  %t{a} = getelementptr i8, ptr %t{}, i64 %t{}\n  %t{b} = getelementptr i8, ptr %t{}, i64 %t{}",
            n[8], n[6], n[8], n[7]
        );
        (format!("%t{a}"), format!("%t{b}"), r)
    }

    fn load_at(&mut self, addr: &str) -> P {
        let t = self.tmp();
        let ty = if self.sc() { "i64" } else { self.t.v };
        let _ = writeln!(self.out, "  %t{t} = load {ty}, ptr {addr}, align {}", self.piece_lane().reg_bytes(self.sh));
        P::V(t)
    }

    /// Bits `off..off + w` (`w` at most a piece) of `o`, read from the
    /// state.
    fn mem_extract(&mut self, o: ObjId, off: P, ok: M, w: u32) -> Val {
        let pb = self.pb as u64;
        let (a0, a1, r) = self.piece_addrs(o, off);
        let (v0, v1) = (self.load_at(&a0), self.load_at(&a1));
        let lo = self.bin("lshr", v0, r);
        let ri = self.bin("sub", P::K(pb), r);
        let ri = self.bin("and", ri, P::K(pb - 1));
        let rz = self.icmp("eq", r, P::K(0));
        let hi = self.bin("shl", v1, ri);
        let hi = self.sel(rz, P::K(0), hi);
        let v = self.bin("or", lo, hi);
        let v = self.norm(Val { w, p: vec![v] });
        let z = self.zero(w);
        self.select(ok, &v, &z)
    }

    /// Writes `x` to bits `off..off + w` (`w` at most a piece) of `o` in
    /// the state.
    fn mem_insert(&mut self, o: ObjId, off: P, ok: M, w: u32, x: &Val) {
        let pb = self.pb as u64;
        let (a0, a1, r) = self.piece_addrs(o, off);
        let (v0, v1) = (self.load_at(&a0), self.load_at(&a1));
        let x = self.resize(x, w, false).p[0];
        let m = P::K(self.top_mask(w));
        let ri = self.bin("sub", P::K(pb), r);
        let ri = self.bin("and", ri, P::K(pb - 1));
        let rz = self.icmp("eq", r, P::K(0));
        let m0 = self.bin("shl", m, r);
        let x0 = self.bin("shl", x, r);
        let m1 = self.bin("lshr", m, ri);
        let m1 = self.sel(rz, P::K(0), m1);
        let x1 = self.bin("lshr", x, ri);
        let x1 = self.sel(rz, P::K(0), x1);
        let all = self.ones_k();
        let put = |f: &mut Self, v: P, m: P, x: P| {
            let keep = f.bin("xor", m, P::K(all));
            let kept = f.bin("and", v, keep);
            let n = f.bin("or", kept, x);
            f.sel(ok, n, v)
        };
        let n1 = put(self, v1, m1, x1);
        let n0 = put(self, v0, m0, x0);
        // The second piece first: where both are the last piece, the
        // first's write is the one that stays.
        let (ty, align) = (if self.sc() { "i64" } else { self.t.v }, self.piece_lane().reg_bytes(self.sh));
        for (a, n) in [(a1, n1), (a0, n0)] {
            let _ = writeln!(self.out, "  store {ty} {}, ptr {a}, align {align}", self.ps(n));
        }
        self.env.vals.remove(&o);
        self.env.loaded.remove(&o);
    }

    /// The function of a specialisation proved to count zeros.
    fn known(&mut self, k: &Known) -> Result<(), String> {
        let x = self.read(k.input);
        let n = x.p.len();
        let pb = self.pb;
        let z0 = self.zero(k.width);
        let zero = self.eq(&x, &z0);
        // A constant of the piece's width made of a repeated byte.
        let bytes = |b: u8| u64::from_ne_bytes([b; 8]) & ones_of(pb);
        let count = if k.count == Count::Ones {
            // Each piece's ones, in two-bit, then four-bit fields, then
            // bytes, which a multiply adds into the top byte.
            let mut c = P::K(0);
            for &p in &x.p {
                let a = self.bin("lshr", p, P::K(1));
                let a = self.bin("and", a, P::K(bytes(0x55)));
                let p = self.bin("sub", p, a);
                let a = self.bin("and", p, P::K(bytes(0x33)));
                let b = self.bin("lshr", p, P::K(2));
                let b = self.bin("and", b, P::K(bytes(0x33)));
                let p = self.bin("add", a, b);
                let a = self.bin("lshr", p, P::K(4));
                let p = self.bin("add", p, a);
                let p = self.bin("and", p, P::K(bytes(0x0F)));
                let p = self.bin("mul", p, P::K(bytes(0x01)));
                let p = self.bin("lshr", p, P::K(pb as u64 - 8));
                c = self.bin("add", c, p);
            }
            c
        } else if k.count == Count::Leading {
            // From the top piece down; the top piece's bits moved up to
            // the top of its lane.
            let top_bits = k.width - pb * (n as u32 - 1);
            let top = self.bin("shl", x.p[n - 1], P::K((pb - top_bits) as u64));
            let mut c = self.ctlz(top);
            if n == 2 {
                let lo = self.ctlz(x.p[0]);
                let lo = self.bin("add", lo, P::K(top_bits as u64));
                let z = self.icmp("eq", x.p[1], P::K(0));
                c = self.sel(z, lo, c);
            }
            c
        } else {
            // pb - ctlz(~p & (p - 1)), the trailing zeros of a piece.
            let all = self.ones_k();
            let tz = |f: &mut Self, p: P| {
                let a = f.bin("xor", p, P::K(all));
                let b = f.bin("sub", p, P::K(1));
                let c = f.bin("and", a, b);
                let l = f.ctlz(c);
                f.bin("sub", P::K(pb as u64), l)
            };
            let mut c = tz(self, x.p[0]);
            if n == 2 {
                let hi = tz(self, x.p[1]);
                let hi = self.bin("add", hi, P::K(pb as u64));
                let z = self.icmp("eq", x.p[0], P::K(0));
                c = self.sel(z, hi, c);
            }
            c
        };
        let d = self.d();
        for &(o, out) in &k.outs {
            let w = tw(&d.types, d.object(o).ty);
            let v = match out {
                Out::Count { at_zero } => {
                    let c = if k.count == Count::Ones && at_zero == 0 { count } else { self.sel(zero, self.k(at_zero as i64), count) };
                    let v = Val { w: pb, p: vec![c] };
                    self.resize(&v, w, false)
                }
                Out::Zero { inverted } => {
                    let m = if inverted { self.mnot(zero) } else { zero };
                    self.bit(m, w)
                }
                Out::Const(c) => self.konst(&Bits::from_u64(w, false, c)),
            };
            self.set(o, v);
        }
        Ok(())
    }

    fn ctlz(&mut self, p: P) -> P {
        if let P::K(k) = p {
            return P::K((k.leading_zeros() - (64 - self.pb)) as u64);
        }
        let v = self.t.v;
        let name = if self.sc() { "llvm.ctlz.i64".to_string() } else { format!("llvm.ctlz.v{}i32", self.sh.n) };
        self.intrinsic(format!("declare {v} @{name}({v}, i1)"));
        let t = self.tmp();
        let _ = writeln!(self.out, "  %t{t} = call {v} @{name}({v} {}, i1 false)", self.ps(p));
        P::V(t)
    }

    // ---- expressions ----

    fn expr(&mut self, e: ExprId) -> Result<Val, String> {
        let d = self.d();
        let tys = &d.types;
        let x = d.expr(e);
        if !matches!(x.kind, ExprKind::Object(_) | ExprKind::Const(_))
            && let Some(c) = self.l.fold(e)
        {
            return self.konst_of(&c, tw(tys, x.ty));
        }
        let w = tw(tys, x.ty);
        match &x.kind {
            ExprKind::Const(c) => self.konst_of(d.konst(*c), w),
            ExprKind::Object(o) => {
                let ob = d.object(*o);
                match ob.kind {
                    ObjKind::Variable | ObjKind::Net(_) => {
                        if ob.lifetime == Lifetime::Static && !self.lay().statics.contains(o) && !self.env.vals.contains_key(o) {
                            return Err(format!("{} is not in this module", d.name(ob.name)));
                        }
                        let v = self.read(*o);
                        Ok(match self.lay().pads.get(o).copied() {
                            Some(p) => self.unpad(p, &v),
                            None => v,
                        })
                    }
                    _ => {
                        if let Some(v) = self.env.vals.get(o) {
                            let v = v.clone();
                            return Ok(self.resize(&v, w, true));
                        }
                        let c = self.l.fold(e).ok_or_else(|| format!("{} has no value", d.name(ob.name)))?;
                        self.konst_of(&c, w)
                    }
                }
            }
            ExprKind::Unary(op) => {
                let a = self.expr(x.args[0])?;
                self.unary(*op, a, x.ty)
            }
            ExprKind::Binary(op) => {
                let a = self.expr(x.args[0])?;
                let b = self.expr(x.args[1])?;
                self.binary(*op, a, b, x.ty, d.expr(x.args[0]).ty)
            }
            ExprKind::Cond => {
                let c = self.expr(x.args[0])?;
                match self.truth(&c) {
                    M::K(true) => self.expr(x.args[1]),
                    M::K(false) => self.expr(x.args[2]),
                    m => {
                        let a = self.expr(x.args[1])?;
                        let b = self.expr(x.args[2])?;
                        let (a, b) = (self.resize(&a, w, false), self.resize(&b, w, false));
                        Ok(self.select(m, &a, &b))
                    }
                }
            }
            ExprKind::Concat => {
                let mut acc = self.zero(w);
                let mut top = w;
                // A replication of zero is no part of the value (11.4.12.1).
                for &a in x.args.iter().filter(|&&a| d.expr(a).kind != ExprKind::Replicate(0)) {
                    let v = self.expr(a)?;
                    top -= v.w;
                    acc = self.insert(&acc, top, v.w, &v);
                }
                Ok(acc)
            }
            ExprKind::Replicate(n) => {
                let v = self.expr(x.args[0])?;
                // One bit: all ones or none, the bit's negation; a few
                // that fit a piece, a multiply.
                if v.w == 1 && w > 1 {
                    let fill = match v.p[0] {
                        P::V(t) if self.mask_of.contains_key(&t) => {
                            let m = self.mask_of[&t];
                            self.sel(m, P::K(!0), P::K(0))
                        }
                        p => self.bin("sub", P::K(0), p),
                    };
                    return Ok(self.norm(Val { w, p: vec![fill; self.np(w)] }));
                }
                if v.w > 0 && w <= self.pb && *n > 1 {
                    let k: u64 = (0..*n).fold(0u64, |k, i| k | 1 << (i * v.w));
                    let p = self.bin("mul", v.p[0], P::K(k));
                    return Ok(self.norm(Val { w, p: vec![p] }));
                }
                let mut acc = self.zero(w);
                for k in 0..*n {
                    acc = self.insert(&acc, k * v.w, v.w, &v);
                }
                Ok(acc)
            }
            ExprKind::Index | ExprKind::Part { .. } | ExprKind::Member(_) => {
                if let Some(o) = self.elem_target(e) {
                    return self.elem_read(o, e);
                }
                if let Some(o) = self.in_memory(x.args[0]) {
                    let (off, w2) = self.offset(e)?;
                    if let Some((p, ok)) = self.uniform_off(off)
                        && w2 <= self.pb
                    {
                        return Ok(self.mem_extract(o, p, ok, w2));
                    }
                }
                if let Some((o, off, w)) = self.read_place(e)? {
                    return Ok(self.read_at(o, off, w));
                }
                let base = self.expr(x.args[0])?;
                let (off, w) = self.offset(e)?;
                Ok(self.extract_off(&base, off, w))
            }
            ExprKind::SysCall(f) => match d.name(*f) {
                // The same bits, read as signed or unsigned (11.7).
                "$signed" | "$unsigned" if x.args.len() == 1 => {
                    let v = self.expr(x.args[0])?;
                    Ok(self.resize(&v, w, tys.is_signed(d.expr(x.args[0]).ty)))
                }
                name => Err(format!("{name} is not supported yet")),
            },
            // A packed target's pattern is its elements concatenated, the
            // first the most significant (10.9).
            ExprKind::Pattern if tw(tys, x.ty) > 0 && !x.args.is_empty() => {
                let widths: Vec<u32> = match tys.get(x.ty) {
                    Type::Struct { packed: true, union: false, members, .. } => members.iter().map(|m| tw(tys, m.ty)).collect(),
                    Type::Unpacked { elem, .. } => vec![tw(tys, *elem); x.args.len()],
                    Type::Struct { .. } => return Err("an assignment pattern for a packed union is not supported yet".into()),
                    _ => vec![w / x.args.len() as u32; x.args.len()],
                };
                if widths.len() != x.args.len() || widths.iter().sum::<u32>() != w {
                    return Err("an assignment pattern that does not fill its type".into());
                }
                let mut acc = self.zero(w);
                let mut top = w;
                for (&a, &ew) in x.args.iter().zip(&widths) {
                    let v = self.expr(a)?;
                    let v = self.resize(&v, ew, tys.is_signed(d.expr(a).ty));
                    top -= ew;
                    acc = self.insert(&acc, top, ew, &v);
                }
                Ok(acc)
            }
            ExprKind::Call(sub) => {
                let v = self.call(*sub, &x.args)?.ok_or("a call of a void function as a value")?;
                Ok(self.resize(&v, w, tys.is_signed(x.ty)))
            }
            ExprKind::Inside => {
                let v = self.expr(x.args[0])?;
                let vt = d.expr(x.args[0]).ty;
                let mut m = M::K(false);
                for &r in &x.args[1..] {
                    let hit = self.inside_item(&v, vt, r)?;
                    m = self.mor(m, hit);
                }
                Ok(self.bit(m, w))
            }
            ExprKind::Convert => {
                let st = d.expr(x.args[0]).ty;
                if !tys.is_integral(st) || !tys.is_integral(x.ty) {
                    return Err(format!("conversions from {} to {} are not supported yet", tys.describe(st, &d.syms), tys.describe(x.ty, &d.syms)));
                }
                let v = self.expr(x.args[0])?;
                Ok(self.resize(&v, w, tys.is_signed(st)))
            }
            k => Err(format!("{} is not supported yet", expr_name(k))),
        }
    }

    fn konst_of(&mut self, c: &Const, w: u32) -> Result<Val, String> {
        let b = const_bits(c, w).ok_or("a constant that is not a packed value")?;
        let b = b.known_or_zero();
        Ok(self.resize(&self.konst(&b), w, b.signed()))
    }

    fn konst(&self, b: &Bits) -> Val {
        let w = b.width().max(1);
        let pb = self.pb;
        let p = (0..self.np(w) as u32)
            .map(|k| {
                let lo = pb * k;
                if lo >= b.width() { P::K(0) } else { P::K(b.slice(lo, pb.min(b.width() - lo)).to_u64().unwrap_or(0)) }
            })
            .collect();
        Val { w, p }
    }

    fn unary(&mut self, op: UnOp, a: Val, ty: TypeId) -> Result<Val, String> {
        let w = tw(&self.d().types, ty);
        Ok(match op {
            UnOp::Plus => self.resize(&a, w, false),
            UnOp::Neg => {
                let a = self.resize(&a, w, false);
                self.add(&self.zero(w), &a, true)
            }
            UnOp::Not => {
                let a = self.resize(&a, w, false);
                self.not(&a)
            }
            UnOp::LogNot => {
                let m = self.truth(&a);
                let m = self.mnot(m);
                self.bit(m, w)
            }
            UnOp::RedOr | UnOp::RedNor => {
                let mut m = self.truth(&a);
                if op == UnOp::RedNor {
                    m = self.mnot(m);
                }
                self.bit(m, w)
            }
            UnOp::RedAnd | UnOp::RedNand => {
                let ones = self.ones(a.w);
                let mut m = self.eq(&a, &ones);
                if op == UnOp::RedNand {
                    m = self.mnot(m);
                }
                self.bit(m, w)
            }
            UnOp::RedXor | UnOp::RedXnor => {
                let mut x = a.p[0];
                for &p in &a.p[1..] {
                    x = self.bin("xor", x, p);
                }
                let pb = self.pb as u64;
                for s in [32, 16, 8, 4, 2, 1].into_iter().filter(|&s| s < pb) {
                    let y = self.bin("lshr", x, P::K(s));
                    x = self.bin("xor", x, y);
                }
                let mut x = self.bin("and", x, P::K(1));
                if op == UnOp::RedXnor {
                    x = self.bin("xor", x, P::K(1));
                }
                let v = Val { w: 1, p: vec![x] };
                self.resize(&v, w, false)
            }
            _ => return Err(format!("{op:?} in an expression is not supported yet")),
        })
    }

    /// `op` on `a` and `b`; `ty` is the result's type, `at` the operands'.
    fn binary(&mut self, op: BinOp, a: Val, b: Val, ty: TypeId, at: TypeId) -> Result<Val, String> {
        let tys = &self.d().types;
        let w = tw(tys, ty);
        let signed = tys.is_signed(at);
        Ok(match op {
            BinOp::And | BinOp::Or | BinOp::Xor | BinOp::Xnor => {
                let (a, b) = (self.resize(&a, w, false), self.resize(&b, w, false));
                let v = match op {
                    BinOp::And => self.bitwise("and", &a, &b),
                    BinOp::Or => self.bitwise("or", &a, &b),
                    _ => self.bitwise("xor", &a, &b),
                };
                if op == BinOp::Xnor { self.not(&v) } else { v }
            }
            BinOp::Add | BinOp::Sub => {
                let (a, b) = (self.resize(&a, w, false), self.resize(&b, w, false));
                self.add(&a, &b, op == BinOp::Sub)
            }
            BinOp::Mul if w <= self.pb => {
                let (a, b) = (self.resize(&a, w, false), self.resize(&b, w, false));
                let p = self.bin("mul", a.p[0], b.p[0]);
                self.norm(Val { w, p: vec![p] })
            }
            BinOp::Mul => {
                let (a, b) = (self.resize(&a, w, false), self.resize(&b, w, false));
                self.mul_wide(&a, &b)
            }
            BinOp::Shl | BinOp::AShl | BinOp::Shr | BinOp::AShr => {
                let a = self.resize(&a, w, false);
                let fill = (op == BinOp::AShr && tys.is_signed(ty)).then(|| self.sign_fill(&a));
                match self.is_const(&b) {
                    Some(n) => {
                        if matches!(op, BinOp::Shl | BinOp::AShl) {
                            self.shl_const(&a, n)
                        } else {
                            self.lshr_const(&a, n, fill)
                        }
                    }
                    _ => {
                        let mut ok = M::K(true);
                        for &hp in &b.p[1..] {
                            let z = self.icmp("eq", hp, P::K(0));
                            ok = self.mand(ok, z);
                        }
                        let v = if matches!(op, BinOp::Shl | BinOp::AShl) { self.shl_var(&a, b.p[0]) } else { self.lshr_var(&a, b.p[0], fill) };
                        let big = match fill {
                            Some(f) => Val { w, p: vec![f; a.p.len()] },
                            None => self.zero(w),
                        };
                        let big = self.norm(big);
                        self.select(ok, &v, &big)
                    }
                }
            }
            BinOp::Eq | BinOp::Ne | BinOp::CaseEq | BinOp::CaseNe => {
                let n = a.w.max(b.w);
                let (a, b) = (self.resize(&a, n, signed), self.resize(&b, n, signed));
                let mut m = self.eq(&a, &b);
                if matches!(op, BinOp::Ne | BinOp::CaseNe) {
                    m = self.mnot(m);
                }
                self.bit(m, w)
            }
            BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
                let n = a.w.max(b.w);
                let (a, b) = (self.resize(&a, n, signed), self.resize(&b, n, signed));
                let m = match op {
                    BinOp::Lt => self.lt(&a, &b, signed),
                    BinOp::Gt => self.lt(&b, &a, signed),
                    BinOp::Le => {
                        let m = self.lt(&b, &a, signed);
                        self.mnot(m)
                    }
                    _ => {
                        let m = self.lt(&a, &b, signed);
                        self.mnot(m)
                    }
                };
                self.bit(m, w)
            }
            BinOp::LogAnd | BinOp::LogOr | BinOp::Implies | BinOp::Equiv => {
                let (x, y) = (self.truth(&a), self.truth(&b));
                let m = match op {
                    BinOp::LogAnd => self.mand(x, y),
                    BinOp::LogOr => self.mor(x, y),
                    BinOp::Implies => {
                        let nx = self.mnot(x);
                        self.mor(nx, y)
                    }
                    _ => {
                        let ne = self.mxor(x, y);
                        self.mnot(ne)
                    }
                };
                self.bit(m, w)
            }
            _ => return Err(format!("{op:?} on values that are not constant is not supported yet")),
        })
    }

    // ---- values ----

    /// `a * b` in `a`'s width, wider than a piece: each pair of pieces
    /// multiplied in lanes twice as wide, the products' halves added by
    /// column in those, and the carries passed up.
    fn mul_wide(&mut self, a: &Val, b: &Val) -> Val {
        let n = a.p.len();
        let (v, wt, sc, pb) = (self.t.v, self.t.wide, self.sc(), self.pb);
        let wide = |f: &mut Self, p: P| -> String {
            match p {
                P::K(k) if sc => format!("{k}"),
                P::K(k) => format!("splat (i64 {k})"),
                P::V(t) => {
                    let x = f.tmp();
                    let _ = writeln!(f.out, "  %t{x} = zext {v} %t{t} to {wt}");
                    format!("%t{x}")
                }
            }
        };
        let a64: Vec<Option<String>> = a.p.iter().map(|&p| (p != P::K(0)).then(|| wide(self, p))).collect();
        let b64: Vec<Option<String>> = b.p.iter().map(|&p| (p != P::K(0)).then(|| wide(self, p))).collect();
        let half = if sc { "64".to_string() } else { "splat (i64 32)".to_string() };
        // The columns' sums: the low halves of the products of pieces i and
        // j in column i + j, their high halves in the next.
        let mut cols: Vec<Vec<String>> = vec![Vec::new(); n];
        for i in 0..n {
            for j in 0..n - i {
                let (Some(x), Some(y)) = (&a64[i], &b64[j]) else { continue };
                let p = self.tmp();
                let _ = writeln!(self.out, "  %t{p} = mul {wt} {x}, {y}");
                let (t, lo, hi) = (self.tmp(), self.tmp(), self.tmp());
                let _ = writeln!(self.out, "  %t{t} = trunc {wt} %t{p} to {v}\n  %t{lo} = zext {v} %t{t} to {wt}\n  %t{hi} = lshr {wt} %t{p}, {half}");
                cols[i + j].push(format!("%t{lo}"));
                if i + j + 1 < n {
                    cols[i + j + 1].push(format!("%t{hi}"));
                }
            }
        }
        let _ = pb;
        let mut carry: Option<String> = None;
        let mut out = Vec::with_capacity(n);
        for col in cols.iter_mut().take(n) {
            col.extend(carry.take());
            let mut sum = col.first().cloned();
            for t in col.iter().skip(1) {
                let s = self.tmp();
                let _ = writeln!(self.out, "  %t{s} = add {wt} {}, {t}", sum.unwrap());
                sum = Some(format!("%t{s}"));
            }
            match sum {
                None => out.push(P::K(0)),
                Some(s) => {
                    let (lo, c) = (self.tmp(), self.tmp());
                    let _ = writeln!(self.out, "  %t{lo} = trunc {wt} {s} to {v}\n  %t{c} = lshr {wt} {s}, {half}");
                    out.push(P::V(lo));
                    carry = Some(format!("%t{c}"));
                }
            }
        }
        self.norm(Val { w: a.w, p: out })
    }

    /// A mask as a `w`-bit 0 or 1.
    fn bit(&mut self, m: M, w: u32) -> Val {
        let p0 = match m {
            M::K(b) => P::K(b as u64),
            M::V(n) => {
                let t = self.tmp();
                let _ = writeln!(self.out, "  %t{t} = zext {} %t{n} to {}", self.t.m, self.t.v);
                self.mask_of.insert(t, m);
                self.zeros.insert(t, !1);
                if self.uniform.contains(&n) {
                    self.uniform.insert(t);
                }
                P::V(t)
            }
        };
        let mut p = vec![P::K(0); self.np(w)];
        p[0] = p0;
        Val { w, p }
    }

    fn ones(&self, w: u32) -> Val {
        let mut p = vec![P::K(self.ones_k()); self.np(w)];
        *p.last_mut().unwrap() = P::K(self.top_mask(w));
        Val { w, p }
    }

    fn norm(&mut self, mut v: Val) -> Val {
        let t = self.top_mask(v.w);
        if t != self.ones_k() {
            let last = *v.p.last().unwrap();
            *v.p.last_mut().unwrap() = self.bin("and", last, P::K(t));
        }
        v
    }

    fn not(&mut self, a: &Val) -> Val {
        if a.w == 1
            && let P::V(t) = a.p[0]
            && let Some(&m) = self.mask_of.get(&t)
        {
            let n = self.mnot(m);
            return self.bit(n, 1);
        }
        let all = self.ones_k();
        let p = a.p.iter().map(|&x| self.bin("xor", x, P::K(all))).collect();
        self.norm(Val { w: a.w, p })
    }

    fn bitwise(&mut self, op: &str, a: &Val, b: &Val) -> Val {
        let p = a.p.iter().zip(&b.p).map(|(&x, &y)| self.bin(op, x, y)).collect();
        Val { w: a.w, p }
    }

    /// `a + b`, or `a - b` as `a + ~b + 1`, in `a`'s width.
    fn add(&mut self, a: &Val, b: &Val, sub: bool) -> Val {
        let mut carry = P::K(sub as u64);
        let all = self.ones_k();
        let mut p = Vec::with_capacity(a.p.len());
        for (&x, &y) in a.p.iter().zip(&b.p) {
            let y = if sub { self.bin("xor", y, P::K(all)) } else { y };
            let s1 = self.bin("add", x, y);
            let s = self.bin("add", s1, carry);
            if p.len() + 1 < a.p.len() {
                let c1 = self.icmp("ult", s1, x);
                let c2 = self.icmp("ult", s, s1);
                let c = self.mor(c1, c2);
                carry = self.bit(c, self.pb).p[0];
            }
            p.push(s);
        }
        self.norm(Val { w: a.w, p })
    }

    fn eq(&mut self, a: &Val, b: &Val) -> M {
        let mut m = M::K(true);
        for (&x, &y) in a.p.iter().zip(&b.p) {
            let e = self.icmp("eq", x, y);
            m = self.mand(m, e);
        }
        m
    }

    fn lt(&mut self, a: &Val, b: &Val, signed: bool) -> M {
        let n = a.p.len();
        let pb = self.pb;
        let mut m = M::K(false);
        for k in 0..n {
            let (x, y, pred) = if k == n - 1 && signed {
                let sh = P::K((pb - (a.w - pb * (n as u32 - 1))) as u64);
                let x = self.bin("shl", a.p[k], sh);
                let x = self.bin("ashr", x, sh);
                let y = self.bin("shl", b.p[k], sh);
                let y = self.bin("ashr", y, sh);
                (x, y, "slt")
            } else {
                (a.p[k], b.p[k], "ult")
            };
            let l = self.icmp(pred, x, y);
            let e = self.icmp("eq", x, y);
            let below = self.mand(e, m);
            m = self.mor(l, below);
        }
        m
    }

    fn truth(&mut self, a: &Val) -> M {
        let mut x = a.p[0];
        for &p in &a.p[1..] {
            x = self.bin("or", x, p);
        }
        if let P::V(t) = x
            && let Some(&m) = self.mask_of.get(&t)
        {
            return m;
        }
        self.icmp("ne", x, P::K(0))
    }

    /// All ones where the sign bit is set.
    fn sign_fill(&mut self, a: &Val) -> P {
        let top = *a.p.last().unwrap();
        let pb = self.pb;
        let sh = pb - (a.w - pb * (a.p.len() as u32 - 1));
        let x = self.bin("shl", top, P::K(sh as u64));
        self.bin("ashr", x, P::K(pb as u64 - 1))
    }

    /// `v` cut or extended to `w` bits.
    fn resize(&mut self, v: &Val, w: u32, signed: bool) -> Val {
        if w == v.w {
            return v.clone();
        }
        let n = self.np(w);
        if w < v.w {
            let p = v.p[..n].to_vec();
            return self.norm(Val { w, p });
        }
        let mut p = v.p.clone();
        if signed {
            let fill = self.sign_fill(v);
            let above = self.bin("and", fill, P::K(!self.top_mask(v.w) & self.ones_k()));
            let last = p.len() - 1;
            p[last] = self.bin("or", p[last], above);
            p.resize(n, fill);
        } else {
            p.resize(n, P::K(0));
        }
        self.norm(Val { w, p })
    }

    /// Bits `lo..lo + w` of `v`.
    fn extract(&mut self, v: &Val, lo: u32, w: u32) -> Val {
        let pb = self.pb;
        // One bit: a test of it, which is a mask (vptestmd).
        if w == 1 && lo < v.w && matches!(v.p[(lo / pb) as usize], P::V(_)) {
            let piece = v.p[(lo / pb) as usize];
            let x = self.bin("and", piece, P::K(1 << (lo % pb)));
            let m = self.icmp("ne", x, P::K(0));
            let b = self.bit(m, 1);
            if let (P::V(t), M::V(_)) = (b.p[0], m) {
                self.bit_of.insert(t, (piece, lo % pb));
            }
            return b;
        }
        // Only the pieces the bits are in.
        let first = (lo / pb) as usize;
        if first > 0 && first < v.p.len() {
            let end = (((lo + w).min(v.w) as usize).div_ceil(pb as usize) + 1).min(v.p.len());
            let part = Val { w: v.w.min(pb * end as u32) - pb * first as u32, p: v.p[first..end].to_vec() };
            return self.extract(&part, lo % pb, w);
        }
        let s = self.lshr_const(v, lo as u64, None);
        self.resize(&s, w, false)
    }

    /// `v` with bits `lo..lo + w` replaced by `x`.
    fn insert(&mut self, v: &Val, lo: u32, w: u32, x: &Val) -> Val {
        if lo >= v.w {
            return v.clone();
        }
        let pb = self.pb;
        // A bit of another value moved here: shifted into place and kept,
        // not tested and extended.
        if w == 1
            && let P::V(t) = x.p[0]
            && let Some(&(src, j)) = self.bit_of.get(&t)
        {
            let (q, b) = ((lo / pb) as usize, lo % pb);
            let moved = if b >= j { self.bin("shl", src, P::K((b - j) as u64)) } else { self.bin("lshr", src, P::K((j - b) as u64)) };
            let moved = self.bin("and", moved, P::K(1 << b));
            let kept = self.bin("and", v.p[q], P::K(!(1u64 << b) & self.ones_k()));
            let mut p = v.p.clone();
            p[q] = self.bin("or", kept, moved);
            return Val { w: v.w, p };
        }
        let x = self.resize(x, w, false);
        let x = self.resize(&x, v.w, false);
        let x = self.shl_const(&x, lo as u64);
        let m = self.ones(w);
        let m = self.resize(&m, v.w, false);
        let m = self.shl_const(&m, lo as u64);
        let keep = self.not(&m);
        let kept = self.bitwise("and", v, &keep);
        self.bitwise("or", &kept, &x)
    }

    fn insert_var(&mut self, v: &Val, off: P, ok: M, w: u32, x: &Val) -> Val {
        let x = self.resize(x, w, false);
        let x = self.resize(&x, v.w, false);
        let x = self.shl_var(&x, off);
        let m = self.ones(w);
        let m = self.resize(&m, v.w, false);
        let m = self.shl_var(&m, off);
        let keep = self.not(&m);
        let kept = self.bitwise("and", v, &keep);
        let new = self.bitwise("or", &kept, &x);
        self.select(ok, &new, v)
    }

    fn shl_const(&mut self, v: &Val, n: u64) -> Val {
        if n >= v.w as u64 {
            return self.zero(v.w);
        }
        let pb = self.pb as u64;
        let (q, r) = ((n / pb) as usize, n % pb);
        let get = |k: isize| if k < 0 { P::K(0) } else { v.p[k as usize] };
        let mut p = Vec::with_capacity(v.p.len());
        for k in 0..v.p.len() as isize {
            let hi = self.bin("shl", get(k - q as isize), P::K(r));
            let lo = if r == 0 { P::K(0) } else { self.bin("lshr", get(k - q as isize - 1), P::K(pb - r)) };
            p.push(self.bin("or", hi, lo));
        }
        self.norm(Val { w: v.w, p })
    }

    /// `v >> n`, shifting in `fill` (zeros if `None`).
    fn lshr_const(&mut self, v: &Val, n: u64, fill: Option<P>) -> Val {
        let f = fill.unwrap_or(P::K(0));
        let len = v.p.len();
        let mut src = v.p.clone();
        if fill.is_some() {
            let above = self.bin("and", f, P::K(!self.top_mask(v.w) & self.ones_k()));
            src[len - 1] = self.bin("or", src[len - 1], above);
        }
        if n >= v.w as u64 {
            return self.norm(Val { w: v.w, p: vec![f; len] });
        }
        let pb = self.pb as u64;
        let (q, r) = ((n / pb) as usize, n % pb);
        let get = |k: usize| if k < len { src[k] } else { f };
        let mut p = Vec::with_capacity(len);
        for k in 0..len {
            let lo = self.bin("lshr", get(k + q), P::K(r));
            let hi = if r == 0 { P::K(0) } else { self.bin("shl", get(k + q + 1), P::K(pb - r)) };
            p.push(self.bin("or", lo, hi));
        }
        self.norm(Val { w: v.w, p })
    }

    /// `v << s` for a shift in each lane; zero if `s >= v.w`.
    fn shl_var(&mut self, v: &Val, s: P) -> Val {
        if let P::K(n) = s {
            return self.shl_const(v, n);
        }
        let len = v.p.len();
        let (q, r, ri, rz) = self.split_shift(s);
        let mut p = Vec::with_capacity(len);
        for k in 0..len {
            let mut acc = P::K(0);
            for t in 0..=k {
                let hi = self.bin("shl", v.p[k - t], r);
                let lo = if k > t {
                    let x = self.bin("lshr", v.p[k - t - 1], ri);
                    self.sel(rz, P::K(0), x)
                } else {
                    P::K(0)
                };
                let term = self.bin("or", hi, lo);
                let hit = self.icmp("eq", q, P::K(t as u64));
                acc = self.sel(hit, term, acc);
            }
            p.push(acc);
        }
        let big = self.icmp("uge", s, P::K(v.w as u64));
        let v2 = self.norm(Val { w: v.w, p });
        let z = self.zero(v.w);
        self.select(big, &z, &v2)
    }

    /// `v >> s` for a shift in each lane, shifting in `fill`.
    fn lshr_var(&mut self, v: &Val, s: P, fill: Option<P>) -> Val {
        if let P::K(n) = s {
            return self.lshr_const(v, n, fill);
        }
        let f = fill.unwrap_or(P::K(0));
        let len = v.p.len();
        let mut src = v.p.clone();
        if fill.is_some() {
            let above = self.bin("and", f, P::K(!self.top_mask(v.w) & self.ones_k()));
            src[len - 1] = self.bin("or", src[len - 1], above);
        }
        let get = |k: usize| if k < len { src[k] } else { f };
        let (q, r, ri, rz) = self.split_shift(s);
        let mut p = Vec::with_capacity(len);
        for k in 0..len {
            // Whole pieces past the top bring in only the fill.
            let mut acc = f;
            for t in 0..len - k {
                let lo = self.bin("lshr", get(k + t), r);
                let x = self.bin("shl", get(k + t + 1), ri);
                let hi = self.sel(rz, P::K(0), x);
                let term = self.bin("or", lo, hi);
                let hit = self.icmp("eq", q, P::K(t as u64));
                acc = self.sel(hit, term, acc);
            }
            p.push(acc);
        }
        let big = self.icmp("uge", s, P::K(v.w as u64));
        let v2 = self.norm(Val { w: v.w, p });
        let b = self.norm(Val { w: v.w, p: vec![f; len] });
        self.select(big, &b, &v2)
    }

    /// A shift's whole pieces, the bits within a piece, `pb - bits` (mod
    /// pb) and whether the bits are zero, so no shift is by a piece or more.
    fn split_shift(&mut self, s: P) -> (P, P, P, M) {
        let pb = self.pb as u64;
        let q = self.bin("lshr", s, P::K(pb.trailing_zeros() as u64));
        let r = self.bin("and", s, P::K(pb - 1));
        let ri = self.bin("sub", P::K(pb), r);
        let ri = self.bin("and", ri, P::K(pb - 1));
        let rz = self.icmp("eq", r, P::K(0));
        (q, r, ri, rz)
    }

    fn select(&mut self, m: M, a: &Val, b: &Val) -> Val {
        let p = a.p.iter().zip(&b.p).map(|(&x, &y)| self.sel(m, x, y)).collect();
        Val { w: a.w, p }
    }

    // ---- instructions, folding constants ----

    /// The bits of `p` known to be zero in every lane (those above a piece
    /// among them).
    fn kz(&self, p: P) -> u64 {
        let above = !self.ones_k();
        above
            | match p {
                P::K(c) => !c,
                P::V(t) => self.zeros.get(&t).copied().unwrap_or(0),
            }
    }

    /// `x` as a signed number of the piece's width.
    fn signed(&self, x: u64) -> i64 {
        let s = 64 - self.pb;
        ((x << s) as i64) >> s
    }

    fn bin(&mut self, op: &str, a: P, b: P) -> P {
        let (all, pb) = (self.ones_k(), self.pb as u64);
        // A mask that clears only bits known to be zero changes nothing;
        // one that keeps only those, everything.
        let (za, zb) = (self.kz(a), self.kz(b));
        match op {
            "and" if za | zb == !0 => return P::K(0),
            "and" if matches!(b, P::K(m) if !za & !m & all == 0) => return a,
            "and" if matches!(a, P::K(m) if !zb & !m & all == 0) => return b,
            "or" | "xor" if za == !0 => return b,
            "or" | "xor" if zb == !0 => return a,
            _ => {}
        }
        // Logic on values made from masks is logic on the masks.
        if let ("and" | "or" | "xor", P::V(x), P::V(y)) = (op, a, b)
            && let (Some(&mx), Some(&my)) = (self.mask_of.get(&x), self.mask_of.get(&y))
        {
            let m = self.mlogic(op, mx, my);
            return self.bit(m, 1).p[0];
        }
        if let (P::K(x), P::K(y)) = (a, b) {
            return P::K(
                all & match op {
                    "and" => x & y,
                    "or" => x | y,
                    "xor" => x ^ y,
                    "add" => x.wrapping_add(y),
                    "sub" => x.wrapping_sub(y),
                    "mul" => x.wrapping_mul(y),
                    "shl" if y < pb => x << y,
                    "lshr" if y < pb => x >> y,
                    "shl" | "lshr" => 0,
                    "ashr" => (self.signed(x) >> y.min(pb - 1)) as u64,
                    _ => unreachable!("{op}"),
                },
            );
        }
        match (op, a, b) {
            ("and", P::K(0), _) | ("and", _, P::K(0)) => return P::K(0),
            ("and", P::K(m), x) | ("and", x, P::K(m)) if m == all => return x,
            ("or" | "xor" | "add", P::K(0), x) | ("or" | "xor" | "add" | "sub" | "shl" | "lshr" | "ashr", x, P::K(0)) => {
                return x;
            }
            ("or", P::K(m), _) | ("or", _, P::K(m)) if m == all => return P::K(all),
            ("shl" | "lshr", P::K(0), _) => return P::K(0),
            ("shl" | "lshr", _, P::K(n)) if n >= pb => return P::K(0),
            ("and" | "or", x, y) if x == y => return x,
            ("xor", x, y) if x == y => return P::K(0),
            _ => {}
        }
        let t = self.tmp();
        let _ = writeln!(self.out, "  %t{t} = {op} {} {}, {}", self.t.v, self.ps(a), self.ps(b));
        if self.is_uniform(a) && self.is_uniform(b) {
            self.uniform.insert(t);
        }
        // What is known of its bits.
        let lead = |z: u64| (!z & all).leading_zeros() - (64 - pb as u32);
        let z = match (op, b) {
            ("and", _) => za | zb,
            ("or" | "xor", _) => za & zb,
            ("shl", P::K(s)) => (za << s) | ((1u64 << s) - 1),
            ("lshr", P::K(s)) => (za >> s) | !(all >> s),
            // A sum has at most one bit more than its wider operand.
            ("add", _) => match lead(za).min(lead(zb)).saturating_sub(1) {
                0 => 0,
                l => !(all >> l),
            },
            _ => 0,
        } & all;
        if z != 0 {
            self.zeros.insert(t, z);
        }
        P::V(t)
    }

    fn icmp(&mut self, pred: &str, a: P, b: P) -> M {
        if let (P::V(x), P::K(k @ (0 | 1)), "eq" | "ne") = (a, b, pred)
            && let Some(&m) = self.mask_of.get(&x)
        {
            return if (k == 1) == (pred == "eq") { m } else { self.mnot(m) };
        }
        if let (P::K(x), P::K(y)) = (a, b) {
            let (sx, sy) = (self.signed(x), self.signed(y));
            return M::K(match pred {
                "eq" => x == y,
                "ne" => x != y,
                "ult" => x < y,
                "ule" => x <= y,
                "ugt" => x > y,
                "uge" => x >= y,
                "slt" => sx < sy,
                "sle" => sx <= sy,
                "sgt" => sx > sy,
                "sge" => sx >= sy,
                _ => unreachable!("{pred}"),
            });
        }
        match (pred, b) {
            ("uge", P::K(0)) => return M::K(true),
            ("ult", P::K(0)) => return M::K(false),
            _ => {}
        }
        if a == b {
            return M::K(matches!(pred, "eq" | "ule" | "uge" | "sle" | "sge"));
        }
        let t = self.tmp();
        let _ = writeln!(self.out, "  %t{t} = icmp {pred} {} {}, {}", self.t.v, self.ps(a), self.ps(b));
        if self.is_uniform(a) && self.is_uniform(b) {
            self.uniform.insert(t);
        }
        M::V(t)
    }

    fn sel(&mut self, m: M, a: P, b: P) -> P {
        match m {
            M::K(true) => a,
            M::K(false) => b,
            M::V(_) if a == b => a,
            M::V(n) => {
                let t = self.tmp();
                let _ = writeln!(self.out, "  %t{t} = select {} %t{n}, {v} {}, {v} {}", self.t.m, self.ps(a), self.ps(b), v = self.t.v);
                if self.uniform.contains(&n) && self.is_uniform(a) && self.is_uniform(b) {
                    self.uniform.insert(t);
                }
                let z = self.kz(a) & self.kz(b) & self.ones_k();
                if z != 0 {
                    self.zeros.insert(t, z);
                }
                P::V(t)
            }
        }
    }

    fn mlogic(&mut self, op: &str, a: M, b: M) -> M {
        if let (M::K(x), M::K(y)) = (a, b) {
            return M::K(match op {
                "and" => x & y,
                "or" => x | y,
                _ => x ^ y,
            });
        }
        match (op, a, b) {
            ("and", M::K(false), _) | ("and", _, M::K(false)) => return M::K(false),
            ("and", M::K(true), x) | ("and", x, M::K(true)) => return x,
            ("or", M::K(true), _) | ("or", _, M::K(true)) => return M::K(true),
            ("or" | "xor", M::K(false), x) | ("or" | "xor", x, M::K(false)) => return x,
            _ => {}
        }
        // Masks from bit planes: the logic on their 16 bits, a scalar
        // instruction, and a mask of that only for what needs one.
        let bits = |f: &Self, m: M| match m {
            M::K(b) => Some(format!("{}", if b { -1 } else { 0 })),
            M::V(n) => f.bits_of.get(&n).map(|i| format!("%t{i}")),
        };
        if let (Some(x), Some(y)) = (bits(self, a), bits(self, b)) {
            let (i, t) = (self.tmp(), self.tmp());
            let _ = writeln!(self.out, "  %t{i} = {op} i16 {x}, {y}\n  %t{t} = bitcast i16 %t{i} to {}", self.t.m);
            self.bits_of.insert(t, i);
            return M::V(t);
        }
        let t = self.tmp();
        let _ = writeln!(self.out, "  %t{t} = {op} {} {}, {}", self.t.m, self.ms(a), self.ms(b));
        M::V(t)
    }

    fn mand(&mut self, a: M, b: M) -> M {
        self.mlogic("and", a, b)
    }

    fn mor(&mut self, a: M, b: M) -> M {
        self.mlogic("or", a, b)
    }

    fn mxor(&mut self, a: M, b: M) -> M {
        self.mlogic("xor", a, b)
    }

    fn mnot(&mut self, a: M) -> M {
        self.mlogic("xor", a, M::K(true))
    }
}

/// A select's bit offset: a constant, a constant below zero (part selects
/// that start out of range), or one per lane with whether it is in range.
#[derive(Clone, Copy)]
enum Off {
    K(u32),
    Neg(u32),
    V(P, M),
    /// Element `i` (from the least significant) of `n`, each `stride`
    /// bits, then `add` bits up; in range where `ok`. A select of one of
    /// few wide elements is then one select an element of the bits it
    /// takes, not a shift of the whole value.
    Dec { i: P, n: u32, stride: u32, add: u32, ok: M },
}

impl Off {
    fn with_sign(self, off: i64) -> Off {
        if off < 0 { Off::Neg((-off) as u32) } else { self }
    }
}

/// Whether a statement holds a `break`, `continue` or `return`.
fn stops(d: &Design, s: StmtId) -> bool {
    match &d.stmt(s).kind {
        StmtKind::Break | StmtKind::Continue | StmtKind::Return(_) => true,
        StmtKind::Block { stmts, .. } => stmts.iter().any(|&s| stops(d, s)),
        StmtKind::If { then, else_, .. } => stops(d, *then) || else_.is_some_and(|x| stops(d, x)),
        StmtKind::Case { items, default, .. } => items.iter().any(|i| stops(d, i.1)) || default.is_some_and(|x| stops(d, x)),
        StmtKind::For { body, .. } => stops(d, *body),
        _ => false,
    }
}

fn stmt_name(k: &StmtKind) -> &'static str {
    match k {
        StmtKind::Block { .. } => "fork",
        StmtKind::Assign { .. } => "assignment with timing",
        StmtKind::Forever(_) => "forever",
        StmtKind::Repeat(..) => "repeat",
        StmtKind::While(..) => "while",
        StmtKind::DoWhile(..) => "do-while",
        StmtKind::Foreach { .. } => "foreach",
        StmtKind::Timed(..) => "timing control",
        StmtKind::Wait(..) => "wait",
        StmtKind::Return(_) => "return",
        StmtKind::Break => "break",
        StmtKind::Continue => "continue",
        _ => "this statement",
    }
}

fn expr_name(k: &ExprKind) -> &'static str {
    match k {
        ExprKind::Hier { .. } => "a hierarchical reference",
        ExprKind::Call(_) => "a function call",
        ExprKind::SysCall(_) => "a system function call",
        ExprKind::Method(_) => "a method call",
        ExprKind::Streaming { .. } => "a streaming concatenation",
        ExprKind::Pattern => "an assignment pattern",
        ExprKind::Inside => "inside",
        _ => "this expression",
    }
}

// ---- Proving what a specialisation computes ----

/// A value as one diagram a bit, low bit first.
type Bits2 = Vec<Bdd>;

/// Why a symbolic evaluation stopped: something it does not model, or
/// diagrams too big. Either way the specialisation keeps its own logic.
struct Unproved;

impl From<TooBig> for Unproved {
    fn from(_: TooBig) -> Unproved {
        Unproved
    }
}

type Sr<T> = Result<T, Unproved>;

/// Evaluates a specialisation's code on diagrams: what each value is, as a
/// function of the input's bits.
struct Sym<'s, 'd, 'a> {
    l: &'s Lowering<'d, 'a>,
    m: Manager,
    env: FxMap<ObjId, Bits2>,
}

impl<'d, 'a> Lowering<'d, 'a> {
    /// What a combinational specialisation with one input of up to 64 bits
    /// computes, if every output is proved, for every input, to be a count
    /// of its trailing (or leading) zeros away from zero, whether it is
    /// zero, or a constant.
    fn recognise(&self, key: Key) -> Option<Known> {
        let d = self.d;
        let spec = key.0;
        let lay = &self.layouts[&key];
        if lay.code.is_empty() || lay.clocked {
            return None;
        }
        let ports = &d.spec(spec).ports;
        let mut input = None;
        let mut outs = Vec::new();
        for p in ports {
            let o = p.object?;
            match p.dir {
                Dir::Input if input.is_none() => input = Some(o),
                Dir::Output => outs.push(o),
                _ => return None,
            }
        }
        let input = input?;
        let w = tw(&d.types, d.object(input).ty);
        if !(2..=64).contains(&w) || outs.is_empty() {
            return None;
        }
        let order = self.order(key).ok()?;
        self.sym_orders.borrow_mut().insert(spec, order.clone());
        let mut s = Sym { l: self, m: Manager::new(1 << 20), env: FxMap::default() };
        let x: Bits2 = (0..w).map(|i| s.m.var(i)).collect::<Result<_, _>>().ok()?;
        s.env.insert(input, x.clone());
        s.run(&order).ok()?;
        // The counts, where the input is not zero, in enough bits.
        let nb = 32 - w.leading_zeros();
        let mut zero = TRUE;
        for &b in &x {
            let nb = s.m.not(b).ok()?;
            zero = s.m.and(zero, nb).ok()?;
        }
        let nonzero = s.m.not(zero).ok()?;
        let mut trailing = s.konst(w as u64, nb);
        for i in (0..w).rev() {
            let k = s.konst(i as u64, nb);
            trailing = s.ite_vec(x[i as usize], &k, &trailing).ok()?;
        }
        let mut leading = s.konst(w as u64, nb);
        for i in 0..w {
            let k = s.konst((w - 1 - i) as u64, nb);
            leading = s.ite_vec(x[i as usize], &k, &leading).ok()?;
        }
        // The ones, added a bit at a time.
        let mut ones = s.konst(0, nb);
        for &b in &x {
            let mut one = vec![FALSE; nb as usize];
            one[0] = b;
            ones = s.add(&ones, &one, FALSE).ok()?;
        }
        let mut dir = None;
        let mut known = Vec::new();
        for o in outs {
            let ow = tw(&d.types, d.object(o).ty) as usize;
            let bits = s.env.get(&o).cloned().unwrap_or_else(|| vec![FALSE; ow]);
            let out = if bits.iter().all(|&b| b <= TRUE) {
                Out::Const(bits.iter().enumerate().fold(0, |v, (i, &b)| v | ((b == TRUE) as u64) << i))
            } else if ow == 1 && bits[0] == zero {
                Out::Zero { inverted: false }
            } else if ow == 1 && bits[0] == nonzero {
                Out::Zero { inverted: true }
            } else {
                let matches = |s: &mut Sym, c: &Bits2| -> Result<bool, TooBig> {
                    for k in 0..ow.max(c.len()) {
                        let a = bits.get(k).copied().unwrap_or(FALSE);
                        let b = c.get(k).copied().unwrap_or(FALSE);
                        if s.m.and(a, nonzero)? != s.m.and(b, nonzero)? {
                            return Ok(false);
                        }
                    }
                    Ok(true)
                };
                let lead = if matches(&mut s, &trailing).ok()? {
                    Count::Trailing
                } else if matches(&mut s, &leading).ok()? {
                    Count::Leading
                } else if matches(&mut s, &ones).ok()? {
                    Count::Ones
                } else {
                    return None;
                };
                if dir.is_some_and(|d| d != lead) {
                    return None;
                }
                dir = Some(lead);
                let at_zero = bits.iter().enumerate().fold(0, |v, (i, &b)| v | (s.m.at_zero(b) as u64) << i);
                Out::Count { at_zero }
            };
            known.push((o, out));
        }
        Some(Known { input, width: w, count: dir?, outs: known })
    }
}

impl<'s, 'd, 'a> Sym<'s, 'd, 'a> {
    fn d(&self) -> &'d Design<'a> {
        self.l.d
    }

    fn konst(&self, v: u64, w: u32) -> Bits2 {
        (0..w).map(|i| if i < 64 && v >> i & 1 == 1 { TRUE } else { FALSE }).collect()
    }

    fn const_of(&self, c: &Const, w: usize) -> Sr<Bits2> {
        let b = ops::bits_of(c).ok_or(Unproved)?.known_or_zero();
        let sign = b.signed() && b.width() > 0 && b.bit(b.width() - 1) == Logic::One;
        Ok((0..w as u32)
            .map(|i| {
                let one = if i < b.width() { b.bit(i) == Logic::One } else { sign };
                if one { TRUE } else { FALSE }
            })
            .collect())
    }

    fn resize(&self, v: &Bits2, w: usize, signed: bool) -> Bits2 {
        let fill = if signed { *v.last().unwrap_or(&FALSE) } else { FALSE };
        (0..w).map(|i| v.get(i).copied().unwrap_or(fill)).collect()
    }

    fn ite_vec(&mut self, c: Bdd, a: &Bits2, b: &Bits2) -> Result<Bits2, TooBig> {
        a.iter().zip(b).map(|(&x, &y)| self.m.ite(c, x, y)).collect()
    }

    fn any(&mut self, v: &Bits2) -> Sr<Bdd> {
        let mut r = FALSE;
        for &b in v {
            r = self.m.or(r, b)?;
        }
        Ok(r)
    }

    /// A value all of whose bits are constant, as an integer.
    fn as_const(v: &Bits2) -> Option<i64> {
        if v.len() > 63 || v.iter().any(|&b| b > TRUE) {
            return None;
        }
        Some(v.iter().enumerate().fold(0, |a, (i, &b)| a | ((b == TRUE) as i64) << i))
    }

    /// A specialisation's code, in order.
    fn run(&mut self, order: &[Node]) -> Sr<()> {
        let d = self.d();
        for &node in order {
            match node {
                Node::Driver(dr) => self.driver(dr)?,
                Node::Loop(l) => self.l.loops[l].all.iter().try_for_each(|&dr| self.driver(dr))?,
                Node::Proc(p) => self.stmt(d.processes[p.0 as usize].body)?,
                Node::In(_) => {}
                Node::Call(i) => self.l.members(i).into_iter().try_for_each(|m| self.call(m))?,
                Node::Pre(_) | Node::Early(_) => return Err(Unproved),
            }
        }
        Ok(())
    }

    /// A child instance: its inputs, its code on values of its own, then
    /// its outputs.
    fn call(&mut self, i: InstId) -> Sr<()> {
        let d = self.d();
        let inst = d.instance(i);
        let order = match self.l.sym_orders.borrow().get(&inst.spec) {
            Some(o) => o.clone(),
            None => {
                let key = self.l.layouts.keys().find(|k| k.0 == inst.spec).copied().ok_or(Unproved)?;
                self.l.order(key).map_err(|_| Unproved)?
            }
        };
        let mut env = FxMap::default();
        let conn = |p: &crate::design::Port| inst.connections.iter().find(|c| c.port == p.name).and_then(|c| c.expr);
        for p in &d.spec(inst.spec).ports {
            let o = p.object.ok_or(Unproved)?;
            if p.dir == Dir::Input {
                let w = tw(&d.types, d.object(o).ty) as usize;
                let v = match conn(p) {
                    Some(e) => self.expr(e)?,
                    None => vec![FALSE; w],
                };
                env.insert(o, self.resize(&v, w, false));
            }
        }
        let outer = std::mem::replace(&mut self.env, env);
        let ran = self.run(&order);
        let inner = std::mem::replace(&mut self.env, outer);
        ran?;
        for p in &d.spec(inst.spec).ports {
            match p.dir {
                Dir::Input => {}
                Dir::Output => {
                    let o = p.object.ok_or(Unproved)?;
                    let Some(e) = conn(p) else { continue };
                    let ty = d.object(o).ty;
                    let v = inner.get(&o).cloned().unwrap_or_else(|| vec![FALSE; tw(&d.types, ty) as usize]);
                    let v = self.resize(&v, tw(&d.types, d.expr(e).ty) as usize, d.types.is_signed(ty));
                    self.assign(e, v)?;
                }
                _ => return Err(Unproved),
            }
        }
        Ok(())
    }

    fn driver(&mut self, dr: DriverId) -> Sr<()> {
        let drv = &self.d().drivers[dr.0 as usize];
        let v = self.expr(drv.rhs)?;
        self.assign(drv.lhs, v)
    }

    fn eq(&mut self, a: &Bits2, b: &Bits2) -> Sr<Bdd> {
        let mut r = TRUE;
        for (&x, &y) in a.iter().zip(b) {
            let d = self.m.xor(x, y)?;
            let e = self.m.not(d)?;
            r = self.m.and(r, e)?;
        }
        Ok(r)
    }

    /// `a < b`, unsigned.
    fn lt(&mut self, a: &Bits2, b: &Bits2) -> Sr<Bdd> {
        let mut r = FALSE;
        for (&x, &y) in a.iter().zip(b) {
            // From the low bit up: this bit decides unless equal.
            let nx = self.m.not(x)?;
            let less = self.m.and(nx, y)?;
            let d = self.m.xor(x, y)?;
            let same = self.m.not(d)?;
            let keep = self.m.and(same, r)?;
            r = self.m.or(less, keep)?;
        }
        Ok(r)
    }

    fn add(&mut self, a: &Bits2, b: &Bits2, mut carry: Bdd) -> Sr<Bits2> {
        let mut out = Vec::with_capacity(a.len());
        for (&x, &y) in a.iter().zip(b) {
            let xy = self.m.xor(x, y)?;
            out.push(self.m.xor(xy, carry)?);
            let both = self.m.and(x, y)?;
            let prop = self.m.and(xy, carry)?;
            carry = self.m.or(both, prop)?;
        }
        Ok(out)
    }

    /// Where a select is in its base: its bit offset (possibly below zero)
    /// and width, from constant indices.
    fn select_at(&mut self, e: ExprId) -> Sr<(i64, usize)> {
        if let Some(Some((off, w))) = self.l.const_select(e) {
            return Ok((off, w as usize));
        }
        let d = self.d();
        let tys = &d.types;
        let x = d.expr(e);
        let bt = d.expr(x.args[0]).ty;
        let idx = self.expr(x.args[1])?;
        let k = Self::as_const(&idx).ok_or(Unproved)?;
        let k = if tys.is_signed(d.expr(x.args[1]).ty) && idx.len() < 64 && k >> (idx.len() - 1) & 1 == 1 {
            k - (1 << idx.len())
        } else {
            k
        };
        match &x.kind {
            ExprKind::Index => {
                let ew = tw(tys, x.ty).max(1) as i64;
                // Out of range: past the top, so that it reads zeros.
                let off = lsb_off(tys, bt, k, k).unwrap_or(tw(tys, bt) as i64);
                Ok((off * ew, ew as usize))
            }
            ExprKind::Part { kind: kind @ (PartKind::Up | PartKind::Down), width } => {
                let ew = elem_bits(tys, bt).max(1) as i64;
                let (hi, lo) = if *kind == PartKind::Up { (k + *width as i64 - 1, k) } else { (k, k - *width as i64 + 1) };
                let start = sel_start(tys, bt, lo, hi);
                Ok((start * ew, (*width as i64 * ew) as usize))
            }
            _ => Err(Unproved),
        }
    }

    fn expr(&mut self, e: ExprId) -> Sr<Bits2> {
        let d = self.d();
        let tys = &d.types;
        let x = d.expr(e);
        let w = tw(tys, x.ty) as usize;
        if !matches!(x.kind, ExprKind::Object(_)) && let Some(c) = self.l.fold(e) {
            return self.const_of(&c, w);
        }
        let arg = |s: &mut Self, i: usize| -> Sr<Bits2> {
            let v = s.expr(x.args[i])?;
            let at = d.expr(x.args[i]).ty;
            Ok(s.resize(&v, w, tys.is_signed(at)))
        };
        Ok(match &x.kind {
            ExprKind::Const(c) => self.const_of(d.konst(*c), w)?,
            ExprKind::Object(o) => match d.object(*o).kind {
                ObjKind::Variable | ObjKind::Net(_) => {
                    let ow = tw(tys, d.object(*o).ty) as usize;
                    let v = self.env.get(o).cloned().unwrap_or_else(|| vec![FALSE; ow]);
                    self.resize(&v, w, false)
                }
                _ => self.const_of(&self.l.fold(e).ok_or(Unproved)?, w)?,
            },
            ExprKind::Unary(UnOp::Not) => {
                let a = arg(self, 0)?;
                a.iter().map(|&b| self.m.not(b)).collect::<Result<_, _>>()?
            }
            ExprKind::Unary(op @ (UnOp::LogNot | UnOp::RedOr | UnOp::RedNor | UnOp::RedAnd | UnOp::RedNand | UnOp::RedXor | UnOp::RedXnor)) => {
                let a = self.expr(x.args[0])?;
                let mut r = match op {
                    UnOp::RedAnd | UnOp::RedNand => {
                        let mut r = TRUE;
                        for &b in &a {
                            r = self.m.and(r, b)?;
                        }
                        r
                    }
                    UnOp::RedXor | UnOp::RedXnor => {
                        let mut r = FALSE;
                        for &b in &a {
                            r = self.m.xor(r, b)?;
                        }
                        r
                    }
                    _ => self.any(&a)?,
                };
                if matches!(op, UnOp::LogNot | UnOp::RedNor | UnOp::RedNand | UnOp::RedXnor) {
                    r = self.m.not(r)?;
                }
                self.resize(&vec![r], w, false)
            }
            ExprKind::Binary(op @ (BinOp::And | BinOp::Or | BinOp::Xor | BinOp::Xnor)) => {
                let (a, b) = (arg(self, 0)?, arg(self, 1)?);
                let mut v = Vec::with_capacity(w);
                for (&p, &q) in a.iter().zip(&b) {
                    v.push(match op {
                        BinOp::And => self.m.and(p, q)?,
                        BinOp::Or => self.m.or(p, q)?,
                        BinOp::Xor => self.m.xor(p, q)?,
                        _ => {
                            let t = self.m.xor(p, q)?;
                            self.m.not(t)?
                        }
                    });
                }
                v
            }
            ExprKind::Binary(op @ (BinOp::Add | BinOp::Sub)) => {
                let (a, b) = (arg(self, 0)?, arg(self, 1)?);
                if *op == BinOp::Add {
                    self.add(&a, &b, FALSE)?
                } else {
                    let nb = b.iter().map(|&q| self.m.not(q)).collect::<Result<Vec<_>, _>>()?;
                    self.add(&a, &nb, TRUE)?
                }
            }
            ExprKind::Binary(op) if matches!(op, BinOp::Eq | BinOp::Ne | BinOp::CaseEq | BinOp::CaseNe | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge) => {
                let at = d.expr(x.args[0]).ty;
                let signed = tys.is_signed(at);
                let n = tw(tys, at).max(tw(tys, d.expr(x.args[1]).ty)) as usize;
                let a = self.expr(x.args[0])?;
                let b = self.expr(x.args[1])?;
                let (mut a, mut b) = (self.resize(&a, n, signed), self.resize(&b, n, signed));
                if signed {
                    // Signed order is unsigned order with the sign bits
                    // flipped.
                    a[n - 1] = self.m.not(a[n - 1])?;
                    b[n - 1] = self.m.not(b[n - 1])?;
                }
                let r = match op {
                    BinOp::Eq | BinOp::CaseEq => self.eq(&a, &b)?,
                    BinOp::Ne | BinOp::CaseNe => {
                        let e = self.eq(&a, &b)?;
                        self.m.not(e)?
                    }
                    BinOp::Lt => self.lt(&a, &b)?,
                    BinOp::Gt => self.lt(&b, &a)?,
                    BinOp::Le => {
                        let g = self.lt(&b, &a)?;
                        self.m.not(g)?
                    }
                    _ => {
                        let l = self.lt(&a, &b)?;
                        self.m.not(l)?
                    }
                };
                self.resize(&vec![r], w, false)
            }
            ExprKind::Binary(op @ (BinOp::LogAnd | BinOp::LogOr)) => {
                let a = self.expr(x.args[0])?;
                let b = self.expr(x.args[1])?;
                let (p, q) = (self.any(&a)?, self.any(&b)?);
                let r = if *op == BinOp::LogAnd { self.m.and(p, q)? } else { self.m.or(p, q)? };
                self.resize(&vec![r], w, false)
            }
            ExprKind::Binary(op @ (BinOp::Shl | BinOp::AShl | BinOp::Shr | BinOp::AShr)) => {
                let a = arg(self, 0)?;
                let n = self.expr(x.args[1])?;
                let n = Self::as_const(&n).ok_or(Unproved)? as usize;
                let fill = if *op == BinOp::AShr && tys.is_signed(x.ty) { *a.last().unwrap_or(&FALSE) } else { FALSE };
                (0..w)
                    .map(|i| match op {
                        BinOp::Shl | BinOp::AShl => if i >= n { a[i - n] } else { FALSE },
                        _ => a.get(i + n).copied().unwrap_or(fill),
                    })
                    .collect()
            }
            ExprKind::Cond => {
                let c = self.expr(x.args[0])?;
                let c = self.any(&c)?;
                let (a, b) = (arg(self, 1)?, arg(self, 2)?);
                self.ite_vec(c, &a, &b)?
            }
            ExprKind::Concat => {
                let mut v = Vec::with_capacity(w);
                for &a in x.args.iter().rev() {
                    v.extend(self.expr(a)?);
                }
                self.resize(&v, w, false)
            }
            ExprKind::Replicate(n) => {
                let a = self.expr(x.args[0])?;
                let v: Bits2 = (0..*n).flat_map(|_| a.iter().copied()).collect();
                self.resize(&v, w, false)
            }
            ExprKind::Index | ExprKind::Part { .. } | ExprKind::Member(_) => {
                let base = self.expr(x.args[0])?;
                let (off, sw) = self.select_at(e)?;
                let v: Bits2 = (0..sw as i64).map(|i| if off + i >= 0 { base.get((off + i) as usize).copied().unwrap_or(FALSE) } else { FALSE }).collect();
                self.resize(&v, w, false)
            }
            ExprKind::Convert => {
                let st = d.expr(x.args[0]).ty;
                if !tys.is_integral(st) || !tys.is_integral(x.ty) {
                    return Err(Unproved);
                }
                let a = self.expr(x.args[0])?;
                self.resize(&a, w, tys.is_signed(st))
            }
            _ => return Err(Unproved),
        })
    }

    fn assign(&mut self, lhs: ExprId, v: Bits2) -> Sr<()> {
        let d = self.d();
        let x = d.expr(lhs);
        match &x.kind {
            ExprKind::Concat => {
                let mut at = 0;
                for &a in x.args.iter().rev() {
                    let w = tw(&d.types, d.expr(a).ty) as usize;
                    let part: Bits2 = (0..w).map(|i| v.get(at + i).copied().unwrap_or(FALSE)).collect();
                    self.assign(a, part)?;
                    at += w;
                }
                Ok(())
            }
            _ => {
                let (o, off, w) = self.place(lhs)?;
                let ow = tw(&d.types, d.object(o).ty) as usize;
                let mut cur = self.env.get(&o).cloned().unwrap_or_else(|| vec![FALSE; ow]);
                for i in 0..w {
                    let at = off + i as i64;
                    if at >= 0 && (at as usize) < ow {
                        cur[at as usize] = v.get(i).copied().unwrap_or(FALSE);
                    }
                }
                self.env.insert(o, cur);
                Ok(())
            }
        }
    }

    /// The object a target selects from, the bit offset, the width.
    fn place(&mut self, e: ExprId) -> Sr<(ObjId, i64, usize)> {
        let d = self.d();
        let x = d.expr(e);
        match &x.kind {
            ExprKind::Object(o) if matches!(d.object(*o).kind, ObjKind::Variable | ObjKind::Net(_)) => {
                Ok((*o, 0, tw(&d.types, x.ty) as usize))
            }
            ExprKind::Index | ExprKind::Part { .. } | ExprKind::Member(_) => {
                let (o, base, _) = self.place(x.args[0])?;
                let (off, w) = self.select_at(e)?;
                Ok((o, base + off, w))
            }
            _ => Err(Unproved),
        }
    }

    fn stmt(&mut self, s: StmtId) -> Sr<()> {
        let d = self.d();
        let kind = &d.stmt(s).kind;
        if !matches!(kind, StmtKind::Null | StmtKind::Block { .. }) && self.l.only_asserts(s) {
            return Ok(());
        }
        match kind {
            StmtKind::Null => Ok(()),
            StmtKind::Block { join: None, scope, stmts } => {
                if let Some(sc) = scope {
                    for item in &d.scope(*sc).items {
                        if let Item::Object(o) = *item
                            && d.object(o).lifetime == Lifetime::Automatic
                        {
                            let w = tw(&d.types, d.object(o).ty) as usize;
                            let v = match d.object(o).init {
                                Some(e) => self.expr(e)?,
                                None => vec![FALSE; w],
                            };
                            self.env.insert(o, self.resize(&v, w, false));
                        }
                    }
                }
                stmts.iter().try_for_each(|&s| self.stmt(s))
            }
            StmtKind::Assign { kind: AssignKind::Blocking, lhs, rhs, timing: None } => {
                let v = self.expr(*rhs)?;
                self.assign(*lhs, v)
            }
            StmtKind::Expr(e) => match d.expr(*e).kind {
                ExprKind::Unary(op @ (UnOp::PreInc | UnOp::PostInc | UnOp::PreDec | UnOp::PostDec)) => {
                    let t = d.expr(*e).args[0];
                    let v = self.expr(t)?;
                    let one = self.konst(1, v.len() as u32);
                    let r = if matches!(op, UnOp::PreInc | UnOp::PostInc) {
                        self.add(&v, &one, FALSE)?
                    } else {
                        let n = one.iter().map(|&q| self.m.not(q)).collect::<Result<Vec<_>, _>>()?;
                        self.add(&v, &n, TRUE)?
                    };
                    self.assign(t, r)
                }
                _ => Err(Unproved),
            },
            StmtKind::If { cond, then, else_, .. } => {
                let c = self.expr(*cond)?;
                let c = self.any(&c)?;
                let (then, else_) = (*then, *else_);
                self.branch(c, |s| s.stmt(then), |s| else_.map_or(Ok(()), |e| s.stmt(e)))
            }
            StmtKind::For { init, cond, step, body, .. } => {
                for &s in init {
                    self.stmt(s)?;
                }
                for _ in 0..65536 {
                    if let Some(c) = cond {
                        let v = self.expr(*c)?;
                        match self.any(&v)? {
                            TRUE => {}
                            FALSE => return Ok(()),
                            _ => return Err(Unproved),
                        }
                    }
                    self.stmt(*body)?;
                    step.iter().try_for_each(|&s| self.stmt(s))?;
                }
                Err(Unproved)
            }
            _ => Err(Unproved),
        }
    }

    /// Both arms on copies of the values, merged bit by bit on `c`.
    fn branch(&mut self, c: Bdd, then: impl FnOnce(&mut Self) -> Sr<()>, else_: impl FnOnce(&mut Self) -> Sr<()>) -> Sr<()> {
        match c {
            TRUE => return then(self),
            FALSE => return else_(self),
            _ => {}
        }
        let before = self.env.clone();
        then(self)?;
        let a = std::mem::replace(&mut self.env, before);
        else_(self)?;
        let b = std::mem::take(&mut self.env);
        let mut keys: Vec<ObjId> = a.keys().chain(b.keys()).copied().collect();
        keys.sort_unstable_by_key(|o| o.0);
        keys.dedup();
        for o in keys {
            let ow = tw(&self.d().types, self.d().object(o).ty) as usize;
            let va = a.get(&o).cloned().unwrap_or_else(|| vec![FALSE; ow]);
            let vb = b.get(&o).cloned().unwrap_or_else(|| vec![FALSE; ow]);
            let v = self.ite_vec(c, &va, &vb)?;
            self.env.insert(o, v);
        }
        Ok(())
    }
}
