//! DPI-C: SystemVerilog calling C, and C calling SystemVerilog (clause 35,
//! Annex H).
//!
//! **Libraries.** The shared libraries named by `-sv_lib` and the like are
//! opened when the run starts. An import's C name (its `c_identifier`, or
//! its SystemVerilog name) is looked up in them in order, then in
//! everything the process has loaded, so that `sin` or `malloc` from the C
//! library need no library at all.
//!
//! **Imports.** Arguments go as Annex H says: small inputs by value, the
//! rest by reference to their C form (`marshal`), open arrays by a handle
//! that the `svdpi.h` functions read (`OpenArray`). The call itself is
//! made by `abi`, since the signature is known only at run time. Outputs
//! are read back from the C form and assigned to the actual arguments
//! when the call returns (35.6.2).
//!
//! **Exports.** C code calls an exported function by its C name, so the
//! name has to be a symbol before the libraries that call it are loaded.
//! `stub` writes a small C file with a function of each exported name,
//! which hands its arguments to `dispatch` here, builds it with the C
//! compiler, and loads it. Which instance's subroutine runs depends on the
//! calling chain's scope (35.5.3): the import's own, or what `svSetScope`
//! set.
//!
//! **Time.** An exported task may wait, and C code cannot be suspended in
//! the middle, so a `context` import task in a design that exports tasks
//! runs on a worker thread of its own. While it runs the simulation waits
//! for it; when it calls an export, the worker waits while the simulation
//! runs the export, as part of the calling process, for as long as the
//! export takes. Only one of the two runs at a time, so the C code may use
//! the `svdpi.h` and VPI functions as on the simulation's thread. Other
//! imports run on the simulation's thread.

mod abi;
pub mod api;
mod stub;

use super::*;
use crate::ast::{Kind, NodeId};
use crate::design::ops;
use crate::hash::FxMap;
use std::cell::Cell;
use std::ffi::{CStr, CString, c_char, c_void};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, Sender, channel};

/// What the simulation knows of the design's DPI declarations, and the
/// state of the run's C side.
pub(crate) struct Dpi {
    imports: FxMap<SubId, Import>,
    /// The exported names, each with the subroutines exported under it and
    /// the scopes they are exported from; the stub of `exports[k]` calls
    /// `dispatch(k, ...)`.
    exports: Vec<Export>,
    /// Whether some export is a task, so that imported tasks may wait.
    export_tasks: bool,
    libs: Vec<*mut c_void>,
    /// Boxed, as C code holds their addresses.
    #[allow(clippy::vec_box)]
    scopes: Vec<Box<api::ScopeRec>>,
    scope_index: FxMap<(u32, ScopeId), usize>,
    /// Scopes by hierarchical name, made when first asked for.
    names: Option<FxMap<String, (u32, ScopeId)>>,
    /// Strings handed to C as outputs of exports, which stay valid a
    /// while (H.8.10).
    strings: VecDeque<CString>,
    idle: Vec<Rc<Worker>>,
    /// Imported tasks running on workers, by process, innermost last.
    jobs: FxMap<u32, Vec<Job>>,
}

struct Import {
    c_name: CString,
    context: bool,
    /// Per argument: an open unpacked or packed dimension (35.5.6.1).
    open: Vec<bool>,
    open_packed: Vec<bool>,
    /// The C function, once looked up.
    func: Option<usize>,
}

struct Export {
    c_name: String,
    subs: Vec<(SubId, ScopeId)>,
    task: bool,
}

/// The calling chain's state, on the thread running the chain's C code.
#[derive(Clone, Copy, Default)]
pub(crate) struct Chain {
    /// The current scope, an `api::ScopeRec`.
    pub(crate) scope: usize,
    /// The chain was disabled while it waited for an export (35.9).
    pub(crate) disabled: bool,
}

thread_local! {
    /// The simulation whose C code this thread is running.
    static SIM: Cell<*mut c_void> = const { Cell::new(std::ptr::null_mut()) };
    pub(crate) static CHAIN: Cell<Chain> = const { Cell::new(Chain { scope: 0, disabled: false }) };
    /// On a worker: its channels to the simulation.
    static LINK: std::cell::RefCell<Option<Link>> = const { std::cell::RefCell::new(None) };
}

/// The simulation of this thread's C code, if there is one.
pub(crate) fn current_sim<'x>() -> Option<&'x mut Sim<'static, 'static, 'static>> {
    let p = SIM.with(|s| s.get());
    // SAFETY: `SIM` is set while the simulation waits for this thread's C
    // code, and cleared before the simulation can go away.
    if p.is_null() { None } else { Some(unsafe { &mut *(p as *mut Sim<'static, 'static, 'static>) }) }
}

// ---- workers ----

/// From the simulation to a worker.
enum ToWorker {
    Run { func: usize, args: Vec<abi::Arg>, chain: Chain, sim: usize },
    /// An export finished: whether the chain was disabled meanwhile.
    Reply { disabled: bool },
}

/// From a worker to the simulation.
enum FromWorker {
    Done,
    Export { idx: usize, args: usize, ret: usize, scope: usize },
}

struct Worker {
    tx: Sender<ToWorker>,
    rx: Receiver<FromWorker>,
}

struct Link {
    tx: Sender<FromWorker>,
    rx: Receiver<ToWorker>,
}

fn spawn_worker() -> Result<Worker, String> {
    let (tx, wrx) = channel();
    let (wtx, rx) = channel();
    std::thread::Builder::new()
        .name("epitome-dpi".into())
        .stack_size(64 << 20)
        .spawn(move || {
            LINK.with(|l| *l.borrow_mut() = Some(Link { tx: wtx, rx: wrx }));
            loop {
                let msg = LINK.with(|l| l.borrow().as_ref().unwrap().rx.recv());
                let Ok(ToWorker::Run { func, args, chain, sim }) = msg else {
                    if msg.is_err() {
                        break;
                    }
                    continue;
                };
                set_thread_sim(sim as *mut c_void);
                CHAIN.with(|c| c.set(chain));
                // SAFETY: the function and its arguments were checked
                // against the import's declaration.
                unsafe { abi::call(func as *const c_void, &args) };
                set_thread_sim(std::ptr::null_mut());
                if LINK.with(|l| l.borrow().as_ref().unwrap().tx.send(FromWorker::Done)).is_err() {
                    break;
                }
            }
        })
        .map_err(|e| format!("cannot start a DPI thread: {e}"))?;
    Ok(Worker { tx, rx })
}

fn set_thread_sim(p: *mut c_void) {
    SIM.with(|s| s.set(p));
    vpi::set_thread_current(p);
}

/// An imported task running on a worker.
struct Job {
    worker: Rc<Worker>,
    cx: Cx,
    sub: SubId,
    actuals: Vec<ExprId>,
    /// Where the calling process goes on.
    resume: u32,
    marshal: Marshal,
    /// The exported task running for it now.
    export: Option<ExportCall>,
}

struct ExportCall {
    /// The depth of the export's frame in the process.
    depth: usize,
    sub: SubId,
    args: usize,
    /// Where the task's result goes: 1 if it was disabled (35.9).
    ret: usize,
}

/// Where a call's C arguments live while it runs.
#[derive(Default)]
struct Marshal {
    args: Vec<abi::Arg>,
    slots: Vec<Slot>,
    strings: Vec<CString>,
    /// Boxed, as the C code holds their addresses.
    #[allow(clippy::vec_box)]
    opens: Vec<Box<api::OpenArray>>,
}

enum Slot {
    /// Passed by value.
    Value,
    /// Passed by reference to this buffer.
    Buf(Vec<u64>),
    /// Passed as an open array handle: `opens[k]`.
    Open(usize),
}

// ---- C forms of values ----

/// How a type is laid out on the C side (H.7).
#[derive(Clone, Debug)]
enum CKind {
    /// `char`, `short`, `int`, `long long` and their unsigned forms.
    Int(u8, bool),
    /// A `bit` or `logic` scalar: `svBit`, `svLogic`.
    Scalar,
    /// A packed type: `svBitVecVal` or `svLogicVecVal` words.
    Vec(u32, bool, bool),
    Real,
    ShortReal,
    Str,
    Chandle,
    Struct(Vec<TypeId>),
    /// A fixed unpacked array, `[l:r]`.
    Array(TypeId, i64, i64),
    /// A dynamic array or queue, which only an open array formal takes.
    Dynamic,
    Other,
}

fn c_kind(tys: &Types, ty: TypeId) -> CKind {
    use crate::design::types::{Atom, RealKind};
    match tys.get(ty) {
        Type::Vector { atom, dims, signed } if dims.is_empty() => match atom {
            Atom::Byte => CKind::Int(1, *signed),
            Atom::ShortInt => CKind::Int(2, *signed),
            Atom::Int => CKind::Int(4, *signed),
            Atom::LongInt => CKind::Int(8, *signed),
            Atom::Bit | Atom::Logic | Atom::Reg => CKind::Scalar,
            Atom::Integer | Atom::Time => CKind::Vec(atom.width(), true, *signed),
        },
        Type::Vector { .. } | Type::PackedArray { .. } | Type::Struct { packed: true, .. } => {
            CKind::Vec(tys.width(ty), tys.is_four_state(ty), tys.is_signed(ty))
        }
        Type::Enum { base, .. } => c_kind(tys, *base),
        Type::Real(RealKind::ShortReal) => CKind::ShortReal,
        Type::Real(_) => CKind::Real,
        Type::String => CKind::Str,
        Type::Chandle => CKind::Chandle,
        Type::Struct { packed: false, union: false, members, .. } => CKind::Struct(members.iter().map(|m| m.ty).collect()),
        Type::Unpacked { elem, dim: UDim::Range(l, r) } => CKind::Array(*elem, *l, *r),
        Type::Unpacked { dim: UDim::Dynamic | UDim::Queue(_), .. } => CKind::Dynamic,
        _ => CKind::Other,
    }
}

fn words(width: u32) -> usize {
    (width.max(1) as usize).div_ceil(32)
}

/// The size and alignment of a type's C form.
fn c_size(tys: &Types, ty: TypeId) -> (usize, usize) {
    match c_kind(tys, ty) {
        CKind::Int(n, _) => (n as usize, n as usize),
        CKind::Scalar => (1, 1),
        CKind::Vec(w, four, _) => (words(w) * if four { 8 } else { 4 }, 4),
        CKind::Real | CKind::Str | CKind::Chandle | CKind::Dynamic | CKind::Other => (8, 8),
        CKind::ShortReal => (4, 4),
        CKind::Struct(members) => {
            let (mut size, mut align) = (0usize, 1usize);
            for m in members {
                let (s, a) = c_size(tys, m);
                size = size.next_multiple_of(a) + s;
                align = align.max(a);
            }
            (size.next_multiple_of(align).max(1), align)
        }
        CKind::Array(elem, l, r) => {
            let (s, a) = c_size(tys, elem);
            (s * ((l - r).unsigned_abs() as usize + 1), a)
        }
    }
}

/// The low 64 bits of an integral value.
fn low64(v: &Const) -> u64 {
    match v {
        Const::Bits(b) => {
            let b = b.known_or_zero();
            b.chunk32(0).0 as u64 | (b.chunk32(1).0 as u64) << 32
        }
        Const::Real(r) => *r as i64 as u64,
        _ => 0,
    }
}

fn read_u(p: &[u8], n: usize) -> u64 {
    let mut b = [0u8; 8];
    b[..n].copy_from_slice(&p[..n]);
    u64::from_le_bytes(b)
}

fn logic_code(l: Logic) -> u8 {
    match l {
        Logic::Zero => 0,
        Logic::One => 1,
        Logic::Z => 2,
        Logic::X => 3,
    }
}

fn code_logic(c: u8) -> Logic {
    match c & 3 {
        0 => Logic::Zero,
        1 => Logic::One,
        2 => Logic::Z,
        _ => Logic::X,
    }
}

fn bits_of(tys: &Types, v: &Const, ty: TypeId) -> Bits {
    match ops::convert(tys, v, ty) {
        Const::Bits(b) => b,
        _ => Bits::zero(tys.width(ty).max(1), false),
    }
}

/// The C string for a value, kept alive in `strings`.
fn c_string(v: &Const, strings: &mut Vec<CString>) -> u64 {
    let bytes: Vec<u8> = match v {
        Const::Str(s) => s.iter().copied().filter(|&c| c != 0).collect(),
        Const::Bits(b) => b.to_str_bytes(),
        _ => vec![],
    };
    let s = CString::new(bytes).unwrap_or_default();
    let p = s.as_ptr() as u64;
    strings.push(s);
    p
}

fn from_c_string(p: u64) -> Const {
    if p == 0 {
        return Const::Str(vec![]);
    }
    // SAFETY: C code gives a string as a null-terminated one (H.8.10).
    Const::Str(unsafe { CStr::from_ptr(p as *const c_char) }.to_bytes().to_vec())
}

/// Writes `v`, of type `ty`, in its C form at the start of `out`.
fn put(tys: &Types, ty: TypeId, v: &Const, out: &mut [u8], strings: &mut Vec<CString>) {
    match c_kind(tys, ty) {
        CKind::Int(n, _) => out[..n as usize].copy_from_slice(&low64(v).to_le_bytes()[..n as usize]),
        CKind::Scalar => out[0] = logic_code(bits_of(tys, v, ty).bit(0)),
        CKind::Vec(w, four, _) => {
            let b = match v {
                // An open packed formal keeps the actual's width.
                Const::Bits(b) if b.width() == w => b.clone(),
                _ => bits_of(tys, v, ty),
            };
            for k in 0..words(w) {
                let (a, u) = b.chunk32(k as u32);
                if four {
                    out[k * 8..k * 8 + 4].copy_from_slice(&a.to_le_bytes());
                    out[k * 8 + 4..k * 8 + 8].copy_from_slice(&u.to_le_bytes());
                } else {
                    out[k * 4..k * 4 + 4].copy_from_slice(&(a & !u).to_le_bytes());
                }
            }
        }
        CKind::Real => {
            let r = match v {
                Const::Real(r) => *r,
                Const::Bits(b) => b.to_f64(),
                _ => 0.0,
            };
            out[..8].copy_from_slice(&r.to_le_bytes());
        }
        CKind::ShortReal => {
            let r = match v {
                Const::Real(r) => *r,
                Const::Bits(b) => b.to_f64(),
                _ => 0.0,
            };
            out[..4].copy_from_slice(&(r as f32).to_le_bytes());
        }
        CKind::Str => out[..8].copy_from_slice(&c_string(v, strings).to_le_bytes()),
        CKind::Chandle => {
            let p = match v {
                Const::Bits(_) => low64(v),
                _ => 0,
            };
            out[..8].copy_from_slice(&p.to_le_bytes());
        }
        CKind::Struct(members) => {
            let items: &[Const] = match v {
                Const::Aggregate(items) => items,
                _ => &[],
            };
            let mut at = 0usize;
            for (k, m) in members.iter().enumerate() {
                let (s, a) = c_size(tys, *m);
                at = at.next_multiple_of(a);
                let item = items.get(k).cloned().unwrap_or_else(|| ops::default_value(tys, *m));
                put(tys, *m, &item, &mut out[at..at + s], strings);
                at += s;
            }
        }
        CKind::Array(elem, l, r) => {
            let items: &[Const] = match v {
                Const::Aggregate(items) => items,
                _ => &[],
            };
            let (s, _) = c_size(tys, elem);
            let n = (l - r).unsigned_abs() as usize + 1;
            for c in 0..n {
                // The lowest index comes first in C (H.7.3); the value
                // keeps the left one first.
                let k = if l <= r { c } else { n - 1 - c };
                let item = items.get(k).cloned().unwrap_or_else(|| ops::default_value(tys, elem));
                put(tys, elem, &item, &mut out[c * s..(c + 1) * s], strings);
            }
        }
        CKind::Dynamic | CKind::Other => {}
    }
}

/// Reads a value of type `ty` from its C form at the start of `p`.
fn get(tys: &Types, ty: TypeId, p: &[u8]) -> Const {
    match c_kind(tys, ty) {
        CKind::Int(n, signed) => {
            let v = read_u(p, n as usize);
            Const::Bits(Bits::from_u64(n as u32 * 8, signed, v))
        }
        CKind::Scalar => ops::convert(tys, &Const::Bits(Bits::of_logic(code_logic(p[0]))), ty),
        CKind::Vec(w, four, signed) => {
            let chunks = (0..words(w)).map(|k| {
                if four {
                    (read_u(&p[k * 8..], 4) as u32, read_u(&p[k * 8 + 4..], 4) as u32)
                } else {
                    (read_u(&p[k * 4..], 4) as u32, 0)
                }
            });
            Const::Bits(Bits::from_chunks32(w, signed, chunks))
        }
        CKind::Real => Const::Real(f64::from_le_bytes(p[..8].try_into().unwrap())),
        CKind::ShortReal => Const::Real(f32::from_le_bytes(p[..4].try_into().unwrap()) as f64),
        CKind::Str => from_c_string(read_u(p, 8)),
        CKind::Chandle => match read_u(p, 8) {
            0 => Const::Null,
            a => Const::Bits(Bits::from_u64(64, false, a)),
        },
        CKind::Struct(members) => {
            let mut at = 0usize;
            let mut items = Vec::new();
            for m in members {
                let (s, a) = c_size(tys, m);
                at = at.next_multiple_of(a);
                items.push(get(tys, m, &p[at..at + s]));
                at += s;
            }
            Const::Aggregate(items)
        }
        CKind::Array(elem, l, r) => {
            let (s, _) = c_size(tys, elem);
            let n = (l - r).unsigned_abs() as usize + 1;
            let mut items: Vec<Const> = (0..n).map(|c| get(tys, elem, &p[c * s..(c + 1) * s])).collect();
            if l > r {
                items.reverse();
            }
            Const::Aggregate(items)
        }
        CKind::Dynamic | CKind::Other => ops::default_value(tys, ty),
    }
}

/// Whether an input of this kind goes by value (H.8.7).
fn by_value(k: &CKind) -> bool {
    matches!(k, CKind::Int(..) | CKind::Scalar | CKind::Real | CKind::ShortReal | CKind::Str | CKind::Chandle)
}

fn small_arg(tys: &Types, ty: TypeId, v: &Const, strings: &mut Vec<CString>) -> abi::Arg {
    match c_kind(tys, ty) {
        CKind::Int(n, signed) => {
            let x = low64(v);
            let bits = n as u32 * 8;
            let x = if bits < 64 && signed { ((x << (64 - bits)) as i64 >> (64 - bits)) as u64 } else if bits < 64 { x & ((1u64 << bits) - 1) } else { x };
            abi::Arg::Int(x, n)
        }
        CKind::Scalar => abi::Arg::Int(logic_code(bits_of(tys, v, ty).bit(0)) as u64, 1),
        CKind::Real => abi::Arg::F64(match v {
            Const::Real(r) => *r,
            Const::Bits(b) => b.to_f64(),
            _ => 0.0,
        }),
        CKind::ShortReal => abi::Arg::F32(match v {
            Const::Real(r) => *r as f32,
            Const::Bits(b) => b.to_f64() as f32,
            _ => 0.0,
        }),
        CKind::Str => abi::Arg::Int(c_string(v, strings), 8),
        CKind::Chandle => abi::Arg::Int(if matches!(v, Const::Bits(_)) { low64(v) } else { 0 }, 8),
        _ => abi::Arg::Int(0, 8),
    }
}

/// A function's result, from the registers.
fn result(tys: &Types, ty: TypeId, r: abi::Ret) -> Const {
    if ty == Types::VOID {
        return Const::Null;
    }
    match c_kind(tys, ty) {
        CKind::Int(n, signed) => {
            let bits = n as u32 * 8;
            let x = if bits < 64 { r.int & ((1u64 << bits) - 1) } else { r.int };
            ops::convert(tys, &Const::Bits(Bits::from_u64(bits, signed, x)), ty)
        }
        CKind::Scalar => ops::convert(tys, &Const::Bits(Bits::of_logic(code_logic(r.int as u8))), ty),
        CKind::Real => Const::Real(r.f64()),
        CKind::ShortReal => Const::Real(r.f32() as f64),
        CKind::Str => from_c_string(r.int),
        CKind::Chandle => {
            if r.int == 0 {
                Const::Null
            } else {
                Const::Bits(Bits::from_u64(64, false, r.int))
            }
        }
        _ => ops::default_value(tys, ty),
    }
}

fn bytes(b: &[u64]) -> &[u8] {
    // SAFETY: plain words seen as bytes.
    unsafe { std::slice::from_raw_parts(b.as_ptr() as *const u8, b.len() * 8) }
}

fn bytes_mut(b: &mut [u64]) -> &mut [u8] {
    // SAFETY: plain words seen as bytes.
    unsafe { std::slice::from_raw_parts_mut(b.as_mut_ptr() as *mut u8, b.len() * 8) }
}

fn buffer(size: usize) -> Vec<u64> {
    vec![0u64; size.div_ceil(8).max(1)]
}

// ---- dynamic loading ----

#[cfg_attr(target_os = "linux", link(name = "dl"))]
unsafe extern "C" {
    fn dlopen(path: *const c_char, mode: i32) -> *mut c_void;
    fn dlsym(handle: *mut c_void, name: *const c_char) -> *mut c_void;
    fn dlerror() -> *const c_char;
}

#[cfg(target_os = "macos")]
const RTLD_GLOBAL: i32 = 0x8;
#[cfg(not(target_os = "macos"))]
const RTLD_GLOBAL: i32 = 0x100;
const RTLD_LAZY: i32 = 0x1;
const RTLD_NOW: i32 = 0x2;
#[cfg(target_os = "macos")]
const RTLD_DEFAULT: *mut c_void = -2isize as *mut c_void;
#[cfg(not(target_os = "macos"))]
const RTLD_DEFAULT: *mut c_void = std::ptr::null_mut();

fn open_lib(path: &Path, mode: i32) -> Result<*mut c_void, String> {
    let c = CString::new(path.display().to_string()).map_err(|e| e.to_string())?;
    // SAFETY: loading a library the user named runs its initialisers.
    let h = unsafe { dlopen(c.as_ptr(), mode) };
    if h.is_null() {
        let e = unsafe { dlerror() };
        let msg = if e.is_null() { String::new() } else { unsafe { CStr::from_ptr(e) }.to_string_lossy().into_owned() };
        return Err(format!("cannot load {}: {msg}", path.display()));
    }
    Ok(h)
}

/// Sets an exported task's `int` result.
fn set_task_result(ret: usize, v: i32) {
    if ret != 0 {
        // SAFETY: the stub's result variable, an `int` for a task.
        unsafe { *(ret as *mut i32) = v };
    }
}

/// The C function that the export stubs call.
unsafe extern "C" fn dispatch(idx: i32, args: *mut *mut c_void, ret: *mut c_void) {
    let scope = CHAIN.with(|c| c.get().scope);
    let on_worker = LINK.with(|l| l.borrow().is_some());
    if on_worker {
        let sent = LINK.with(|l| {
            l.borrow().as_ref().unwrap().tx.send(FromWorker::Export { idx: idx as usize, args: args as usize, ret: ret as usize, scope })
        });
        let reply = if sent.is_ok() { LINK.with(|l| l.borrow().as_ref().unwrap().rx.recv()) } else { Err(std::sync::mpsc::RecvError) };
        let disabled = match reply {
            Ok(ToWorker::Reply { disabled }) => disabled,
            _ => {
                // The simulation has gone: unwind the C code.
                set_thread_sim(std::ptr::null_mut());
                true
            }
        };
        CHAIN.with(|c| c.set(Chain { scope, disabled: c.get().disabled || disabled }));
        return;
    }
    if let Some(s) = current_sim() {
        s.dpi_export_here(idx as usize, args as usize, ret as usize, scope);
    }
    CHAIN.with(|c| c.set(Chain { scope, ..c.get() }));
}

impl Dpi {
    fn empty() -> Dpi {
        Dpi {
            imports: FxMap::default(),
            exports: vec![],
            export_tasks: false,
            libs: vec![],
            scopes: vec![],
            scope_index: FxMap::default(),
            names: None,
            strings: VecDeque::new(),
            idle: vec![],
            jobs: FxMap::default(),
        }
    }

    /// The design's DPI declarations; `None` without any.
    pub(super) fn for_design(d: &Design, ast: &Ast) -> Option<Box<Dpi>> {
        let mut dpi = Dpi::empty();
        // An import's prototype is the child of its declaration.
        let mut decl_of: FxMap<NodeId, NodeId> = FxMap::default();
        for (i, n) in ast.nodes.iter().enumerate() {
            if n.kind == Kind::DpiImport {
                let id = NodeId(i as u32);
                if let Some(p) = ast.child(id, Kind::Prototype) {
                    decl_of.insert(p, id);
                }
            }
        }
        for (i, s) in d.subroutines.iter().enumerate() {
            let Some(&decl) = decl_of.get(&s.node) else { continue };
            if !s.qualifiers.iter().any(|&q| d.name(q) == "import") {
                continue;
            }
            let c_name = match ast.child(decl, Kind::Name) {
                Some(n) => ast.token(n).ident_name().to_string(),
                None => d.name(s.name).to_string(),
            };
            let (open, open_packed) = open_formals(d, ast, s);
            dpi.imports.insert(
                SubId(i as u32),
                Import {
                    c_name: CString::new(c_name).unwrap_or_default(),
                    context: s.qualifiers.iter().any(|&q| d.name(q) == "context"),
                    open,
                    open_packed,
                    func: None,
                },
            );
        }
        // Exports are kept as syntax in the scope they are declared in.
        for (k, sc) in d.scopes.iter().enumerate() {
            for item in &sc.items {
                let Item::Opaque(n) = *item else { continue };
                if ast.kind(n) != Kind::DpiExport {
                    continue;
                }
                let names: Vec<NodeId> = ast.children(n).iter().copied().filter(|&c| ast.kind(c) == Kind::Name).collect();
                let Some(&last) = names.last() else { continue };
                let sv_name = ast.token(last).ident_name();
                let c_name = ast.token(names[0]).ident_name().to_string();
                let Some(sub) = sc.names.iter().find_map(|(s, b)| match b {
                    Binding::Subroutine(x) if d.name(*s) == sv_name => Some(*x),
                    _ => None,
                }) else {
                    continue;
                };
                let task = d.sub(sub).kind == SubKind::Task;
                dpi.export_tasks |= task;
                let scope = ScopeId(k as u32);
                match dpi.exports.iter_mut().find(|e| e.c_name == c_name) {
                    Some(e) => e.subs.push((sub, scope)),
                    None => dpi.exports.push(Export { c_name, subs: vec![(sub, scope)], task }),
                }
            }
        }
        if dpi.imports.is_empty() && dpi.exports.is_empty() {
            return None;
        }
        Some(Box::new(dpi))
    }
}

/// Which formals of an import have an open unpacked dimension, and which
/// an open packed one (`bit [] x`), which the declaration's syntax shows.
fn open_formals(d: &Design, ast: &Ast, s: &Subroutine) -> (Vec<bool>, Vec<bool>) {
    let n = s.args.len();
    let open: Vec<bool> = s.args.iter().map(|&f| matches!(d.types.get(d.object(f).ty), Type::Unpacked { dim: UDim::Dynamic | UDim::Queue(_), .. })).collect();
    let mut packed = Vec::new();
    if let Some(ports) = ast.child(s.node, Kind::TfPorts) {
        let mut cur = false;
        for &port in ast.children(ports) {
            if ast.kind(port) != Kind::TfPort {
                continue;
            }
            for &k in ast.children(port) {
                match ast.kind(k) {
                    Kind::Declarator => packed.push(cur),
                    Kind::Keyword => {}
                    _ => cur = ast.children(k).iter().any(|&c| ast.kind(c) == Kind::UnsizedDim),
                }
            }
        }
    }
    if packed.len() != n {
        packed = vec![false; n];
    }
    let open = open.iter().zip(&packed).map(|(a, b)| *a || *b).collect();
    (open, packed)
}

/// An expression without the conversions the elaborator wraps around an
/// actual argument, which an open formal does not make.
fn peel(d: &Design, mut x: ExprId) -> ExprId {
    while let ExprKind::Convert = d.expr(x).kind {
        x = d.expr(x).args[0];
    }
    x
}

impl Sim<'_, '_, '_> {
    fn dpi(&mut self) -> &mut Dpi {
        self.dpi.as_mut().unwrap()
    }

    /// Whether formal `i` of `sub` is an open array, whose actual is passed
    /// as it is.
    pub(super) fn dpi_open_formal(&self, sub: SubId, i: usize) -> bool {
        self.dpi.as_ref().and_then(|d| d.imports.get(&sub)).is_some_and(|imp| imp.open.get(i).copied().unwrap_or(false))
    }

    pub(super) fn dpi_actual(&self, sub: SubId, i: usize, a: ExprId) -> ExprId {
        if self.dpi_open_formal(sub, i) { peel(self.d, a) } else { a }
    }

    pub(super) fn is_dpi_import(&self, sub: SubId) -> bool {
        self.dpi.as_ref().is_some_and(|d| d.imports.contains_key(&sub))
    }

    /// Loads the libraries, with the stubs for the design's exports first.
    pub(super) fn dpi_start(&mut self, libs: &[PathBuf]) -> Result<(), String> {
        if libs.is_empty() {
            return Ok(());
        }
        if self.dpi.is_none() {
            self.dpi = Some(Box::new(Dpi::empty()));
        }
        if !self.dpi().exports.is_empty() {
            let d = self.d;
            let decls: Vec<stub::Decl> = self
                .dpi
                .as_ref()
                .unwrap()
                .exports
                .iter()
                .map(|e| {
                    let sub = d.sub(e.subs[0].0);
                    let tys = &d.types;
                    let params = sub
                        .args
                        .iter()
                        .map(|&f| {
                            let o = d.object(f);
                            let k = c_kind(tys, o.ty);
                            if o.dir.unwrap_or(Dir::Input) == Dir::Input && by_value(&k) { (stub::c_type(&k), false) } else { ("void *", true) }
                        })
                        .collect();
                    let ret = if e.task {
                        "int"
                    } else if sub.ret == Types::VOID {
                        "void"
                    } else {
                        stub::c_type(&c_kind(tys, sub.ret))
                    };
                    stub::Decl { name: e.c_name.clone(), params, ret }
                })
                .collect();
            // Without them the libraries still load, unless they call an
            // export.
            match stub::build(&decls) {
                Ok(path) => {
                    let h = open_lib(&path, RTLD_NOW | RTLD_GLOBAL)?;
                    let slot = unsafe { dlsym(h, c"epitome_dpi_dispatch".as_ptr()) };
                    if slot.is_null() {
                        return Err(format!("{}: the export stubs have no dispatcher", path.display()));
                    }
                    // SAFETY: the stub library's dispatcher variable.
                    unsafe { *(slot as *mut usize) = dispatch as *const () as usize };
                }
                Err(e) => eprintln!("epitome-sv: warning: C code cannot call the design's DPI exports: {e}"),
            }
        }
        for lib in libs {
            let h = open_lib(lib, RTLD_LAZY | RTLD_GLOBAL)?;
            self.dpi().libs.push(h);
        }
        Ok(())
    }

    /// The C function of an import, looked up the first time.
    fn dpi_func(&mut self, sub: SubId) -> Option<usize> {
        let imp = self.dpi().imports.get(&sub)?;
        if let Some(f) = imp.func {
            return Some(f);
        }
        let name = imp.c_name.clone();
        let mut f = std::ptr::null_mut();
        for &h in &self.dpi().libs {
            f = unsafe { dlsym(h, name.as_ptr()) };
            if !f.is_null() {
                break;
            }
        }
        if f.is_null() {
            f = unsafe { dlsym(RTLD_DEFAULT, name.as_ptr()) };
        }
        if f.is_null() {
            let n = self.d.name(self.d.sub(sub).name).to_string();
            self.dpi_fatal(format!("DPI import `{n}`: no C function `{}` in the loaded libraries (35.4)", name.to_string_lossy()));
            return None;
        }
        self.dpi().imports.get_mut(&sub).unwrap().func = Some(f as usize);
        Some(f as usize)
    }

    fn dpi_fatal(&mut self, msg: String) {
        let _ = writeln!(self.out, "Error: {msg}");
        eprintln!("epitome-sv: error: {msg}");
        self.outcome.errors += 1;
        self.outcome.failed = true;
        self.finishing = true;
    }

    // ---- scopes ----

    /// The record standing for an instantiated scope, an `svScope`.
    pub(crate) fn dpi_scope(&mut self, inst: u32, sc: ScopeId) -> usize {
        if let Some(&k) = self.dpi().scope_index.get(&(inst, sc)) {
            return &*self.dpi().scopes[k] as *const api::ScopeRec as usize;
        }
        let name = self.dpi_scope_name(inst, sc);
        let rec = Box::new(api::ScopeRec::new(inst, sc, name));
        let p = &*rec as *const api::ScopeRec as usize;
        let dpi = self.dpi();
        dpi.scope_index.insert((inst, sc), dpi.scopes.len());
        dpi.scopes.push(rec);
        p
    }

    fn dpi_scope_name(&self, inst: u32, sc: ScopeId) -> String {
        let s = self.d.scope(sc);
        match s.kind {
            ScopeKind::Package => s.name.map(|n| self.d.name(n).to_string()).unwrap_or_default(),
            ScopeKind::Unit => "$unit".into(),
            _ if inst == u32::MAX => "$unit".into(),
            _ => self.scope_full(inst, sc),
        }
    }

    /// The instantiated scope a subroutine is declared in: the scope around
    /// it, in the instance it runs in, or in no instance for a package or
    /// the compilation unit.
    fn decl_scope(&self, inst: u32, sub: SubId) -> (u32, ScopeId) {
        let s = self.d.sub(sub);
        let decl = self.d.scope(s.scope).parent.unwrap_or(s.scope);
        let mut cur = Some(decl);
        while let Some(c) = cur {
            match self.d.scope(c).kind {
                ScopeKind::Package | ScopeKind::Unit => return (u32::MAX, decl),
                ScopeKind::Spec => break,
                _ => cur = self.d.scope(c).parent,
            }
        }
        (self.def_inst(inst, s.scope), decl)
    }

    /// The scope with a hierarchical name, for `svGetScopeFromName`.
    pub(crate) fn dpi_scope_named(&mut self, name: &str) -> usize {
        if self.dpi().names.is_none() {
            // The named generate blocks of each specialisation.
            let mut gens: FxMap<ScopeId, Vec<ScopeId>> = FxMap::default();
            for (k, s) in self.d.scopes.iter().enumerate() {
                if s.kind == ScopeKind::Generate && s.name.is_some()
                    && let Some(top) = self.spec_scope_of(ScopeId(k as u32))
                {
                    gens.entry(top).or_default().push(ScopeId(k as u32));
                }
            }
            let mut names = FxMap::default();
            for inst in 0..self.insts.len() as u32 {
                let spec = self.insts[inst as usize].spec;
                let top = self.d.spec(spec).scope;
                names.insert(self.inst_path(inst), (inst, top));
                for &g in gens.get(&top).map(Vec::as_slice).unwrap_or(&[]) {
                    names.entry(self.scope_full(inst, g)).or_insert((inst, g));
                }
            }
            for &p in &self.d.packages {
                if let Some(n) = self.d.scope(p).name {
                    names.entry(self.d.name(n).to_string()).or_insert((u32::MAX, p));
                }
            }
            self.dpi().names = Some(names);
        }
        let name = name.strip_prefix("$root.").unwrap_or(name);
        match self.dpi().names.as_ref().unwrap().get(name).copied() {
            Some((inst, sc)) => self.dpi_scope(inst, sc),
            None => 0,
        }
    }

    /// The specialisation's scope a scope is inside.
    fn spec_scope_of(&self, mut sc: ScopeId) -> Option<ScopeId> {
        loop {
            if self.d.scope(sc).kind == ScopeKind::Spec {
                return Some(sc);
            }
            sc = self.d.scope(sc).parent?;
        }
    }

    // ---- calling imports ----

    /// Puts the arguments of a call of `sub` in their C forms.
    fn dpi_marshal(&mut self, cx: &Cx, sub: SubId, locals: &Locals, actuals: &[ExprId]) -> Marshal {
        let tys = self.types();
        let formals = self.d.sub(sub).args.clone();
        let (open, open_packed) = {
            let imp = &self.dpi().imports[&sub];
            (imp.open.clone(), imp.open_packed.clone())
        };
        let mut m = Marshal::default();
        for (i, &f) in formals.iter().enumerate() {
            let o = self.d.object(f);
            let (dir, ty) = (o.dir.unwrap_or(Dir::Input), o.ty);
            let v = locals.borrow().iter().rev().find(|(k, _)| *k == f).map(|(_, v)| v.clone()).unwrap_or_else(|| ops::default_value(tys, ty));
            if open[i] {
                let a = actuals.get(i).map(|&a| peel(self.d, a));
                let (aty, shape) = match a {
                    Some(a) if dir == Dir::Output => (self.d.expr(a).ty, self.eval(cx, a)),
                    Some(a) => (self.d.expr(a).ty, v.clone()),
                    None => (ty, v.clone()),
                };
                let oa = api::OpenArray::new(tys, ty, aty, &shape, open_packed[i], dir != Dir::Output, &mut m.strings);
                let b = Box::new(oa);
                m.args.push(abi::Arg::Int(&*b as *const api::OpenArray as u64, 8));
                m.slots.push(Slot::Open(m.opens.len()));
                m.opens.push(b);
            } else if dir == Dir::Input && by_value(&c_kind(tys, ty)) {
                m.args.push(small_arg(tys, ty, &v, &mut m.strings));
                m.slots.push(Slot::Value);
            } else {
                let mut buf = buffer(c_size(tys, ty).0);
                if dir != Dir::Output {
                    put(tys, ty, &v, bytes_mut(&mut buf), &mut m.strings);
                }
                m.args.push(abi::Arg::Int(buf.as_ptr() as u64, 8));
                m.slots.push(Slot::Buf(buf));
            }
        }
        m
    }

    /// Assigns the outputs of a finished call to its actual arguments.
    fn dpi_copy_out(&mut self, cx: &Cx, sub: SubId, m: &Marshal, actuals: &[ExprId]) {
        let tys = self.types();
        let formals = self.d.sub(sub).args.clone();
        for (i, &f) in formals.iter().enumerate() {
            let o = self.d.object(f);
            if !matches!(o.dir, Some(Dir::Output | Dir::Inout | Dir::Ref)) {
                continue;
            }
            let Some(&a) = actuals.get(i) else { continue };
            let v = match &m.slots[i] {
                Slot::Value => continue,
                Slot::Buf(b) => get(tys, o.ty, bytes(b)),
                Slot::Open(k) => m.opens[*k].read(tys),
            };
            let a = self.dpi_actual(sub, i, a);
            let t = self.target(cx, a);
            self.write_cx(cx, &t, v, false);
        }
    }

    /// The chain an import starts: its declaration's scope.
    fn dpi_chain(&mut self, cx: &Cx, sub: SubId) -> Chain {
        let (inst, sc) = self.decl_scope(cx.inst, sub);
        Chain { scope: self.dpi_scope(inst, sc), disabled: false }
    }

    /// Calls C code on this thread.
    fn dpi_invoke(&mut self, func: usize, args: &[abi::Arg], chain: Chain) -> abi::Ret {
        let me = self as *mut Self as *mut c_void;
        let saved_sim = SIM.with(|s| s.replace(me));
        let saved_vpi = vpi::swap_thread_current(me);
        let saved = CHAIN.with(|c| c.replace(chain));
        // SAFETY: the arguments are of the kinds the import declares.
        let r = unsafe { abi::call(func as *const c_void, args) };
        CHAIN.with(|c| c.set(saved));
        vpi::swap_thread_current(saved_vpi);
        SIM.with(|s| s.set(saved_sim));
        r
    }

    /// A call of an imported function, or of an imported task that cannot
    /// wait.
    pub(super) fn dpi_call(&mut self, cx: &Cx, sub: SubId, locals: &Locals, actuals: &[ExprId]) -> Const {
        let ret_ty = self.d.sub(sub).ret;
        if !abi::SUPPORTED {
            self.dpi_fatal("DPI calls are not supported on this platform".into());
            return ops::default_value(self.types(), ret_ty);
        }
        let Some(func) = self.dpi_func(sub) else { return ops::default_value(self.types(), ret_ty) };
        let m = self.dpi_marshal(cx, sub, locals, actuals);
        let chain = self.dpi_chain(cx, sub);
        let r = self.dpi_invoke(func, &m.args, chain);
        self.dpi_copy_out(cx, sub, &m, actuals);
        if self.d.sub(sub).kind == SubKind::Task { Const::Null } else { result(self.types(), ret_ty, r) }
    }

    /// A call of an imported task: on a worker if it may call an exported
    /// task, which may wait.
    pub(super) fn dpi_call_task(&mut self, p: u32, cx: &Cx, sub: SubId, locals: &Locals, actuals: &[ExprId]) -> Flow {
        let worker = self.dpi().export_tasks && self.dpi().imports[&sub].context;
        if !worker || !abi::SUPPORTED {
            self.dpi_call(cx, sub, locals, actuals);
            return Flow::Next;
        }
        let Some(func) = self.dpi_func(sub) else { return Flow::Next };
        let w = match self.dpi().idle.pop() {
            Some(w) => w,
            None => match spawn_worker() {
                Ok(w) => Rc::new(w),
                Err(e) => {
                    self.dpi_fatal(e);
                    return Flow::Next;
                }
            },
        };
        let marshal = self.dpi_marshal(cx, sub, locals, actuals);
        let chain = self.dpi_chain(cx, sub);
        let sim = self as *mut Self as usize;
        let _ = w.tx.send(ToWorker::Run { func, args: marshal.args.clone(), chain, sim });
        let resume = self.procs[p as usize].pc;
        let job = Job { worker: w, cx: cx.clone(), sub, actuals: actuals.to_vec(), resume, marshal, export: None };
        self.dpi().jobs.entry(p).or_default().push(job);
        self.dpi_wait(p)
    }

    /// Waits for the innermost worker of process `p` to finish or to call
    /// an export.
    fn dpi_wait(&mut self, p: u32) -> Flow {
        loop {
            let w = self.dpi().jobs[&p].last().unwrap().worker.clone();
            match w.rx.recv() {
                Ok(FromWorker::Done) => {
                    let job = self.dpi().jobs.get_mut(&p).unwrap().pop().unwrap();
                    self.dpi().idle.push(job.worker.clone());
                    self.dpi_copy_out(&job.cx, job.sub, &job.marshal, &job.actuals);
                    self.procs[p as usize].pc = job.resume;
                    return Flow::Next;
                }
                Ok(FromWorker::Export { idx, args, ret, scope }) => {
                    let Some((sub, inst)) = self.dpi_export_target(idx, scope) else {
                        let _ = w.tx.send(ToWorker::Reply { disabled: false });
                        continue;
                    };
                    if self.d.sub(sub).kind == SubKind::Function {
                        self.dpi_run_export_function(sub, inst, args, ret);
                        let _ = w.tx.send(ToWorker::Reply { disabled: false });
                        continue;
                    }
                    // The export runs as part of the process, for as long
                    // as it takes.
                    let Some(&entry) = self.sub_entry.get(&sub) else {
                        let _ = w.tx.send(ToWorker::Reply { disabled: false });
                        continue;
                    };
                    let locals = self.dpi_export_locals(sub, args);
                    let frame = Frame { cx: Cx { inst, locals, this: None }, ret: None, sub: Some(sub), counters: HashMap::new(), iters: HashMap::new(), ret_value: None };
                    self.procs[p as usize].frames.push(frame);
                    let depth = self.procs[p as usize].frames.len();
                    self.procs[p as usize].pc = entry;
                    self.dpi().jobs.get_mut(&p).unwrap().last_mut().unwrap().export = Some(ExportCall { depth, sub, args, ret });
                    return Flow::Next;
                }
                Err(_) => {
                    self.dpi().jobs.get_mut(&p).unwrap().pop();
                    self.dpi_fatal("a DPI worker thread ended".into());
                    return Flow::Done;
                }
            }
        }
    }

    /// When a process returns from an exported task that a worker's C code
    /// called: hands the outputs back, and waits for the C code again.
    pub(super) fn dpi_task_return(&mut self, p: u32) -> Option<Flow> {
        let depth = self.procs[p as usize].frames.len();
        let dpi = self.dpi.as_mut()?;
        let job = dpi.jobs.get_mut(&p)?.last_mut()?;
        if job.export.as_ref().is_none_or(|e| e.depth != depth) {
            return None;
        }
        let call = job.export.take().unwrap();
        let w = job.worker.clone();
        let f = self.procs[p as usize].frames.pop().unwrap();
        self.dpi_export_outputs(call.sub, call.args, &f.cx.locals);
        set_task_result(call.ret, 0);
        let _ = w.tx.send(ToWorker::Reply { disabled: false });
        Some(self.dpi_wait(p))
    }

    /// A process is killed: its C code is told so, and is waited for
    /// (35.9).
    pub(super) fn dpi_killed(&mut self, p: u32) {
        let Some(dpi) = self.dpi.as_mut() else { return };
        let Some(jobs) = dpi.jobs.remove(&p) else { return };
        for job in jobs.into_iter().rev() {
            let w = job.worker.clone();
            if let Some(e) = &job.export {
                set_task_result(e.ret, 1);
                let _ = w.tx.send(ToWorker::Reply { disabled: true });
            }
            loop {
                match w.rx.recv() {
                    Ok(FromWorker::Done) => {
                        self.dpi().idle.push(w);
                        break;
                    }
                    // Calling an export once disabled is an error (35.9 d).
                    Ok(FromWorker::Export { idx, ret, .. }) => {
                        if self.dpi().exports.get(idx).is_some_and(|e| e.task) {
                            set_task_result(ret, 1);
                        }
                        let _ = w.tx.send(ToWorker::Reply { disabled: true });
                    }
                    Err(_) => break,
                }
            }
        }
    }

    // ---- exports ----

    /// The subroutine and instance an export call reaches from `scope`.
    fn dpi_export_target(&mut self, idx: usize, scope: usize) -> Option<(SubId, u32)> {
        let e = self.dpi().exports.get(idx)?;
        let name = e.c_name.clone();
        let subs = e.subs.clone();
        let (inst, sc) = if scope == 0 {
            (u32::MAX, self.d.unit)
        } else {
            // SAFETY: scopes are records this simulation made.
            let r = unsafe { &*(scope as *const api::ScopeRec) };
            (r.inst, r.scope)
        };
        // Exported from the scope itself or a scope around it in the same
        // instance; or from a package or the compilation unit.
        let mut cur = Some(sc);
        while let Some(c) = cur {
            if let Some(&(sub, _)) = subs.iter().find(|(_, s)| *s == c) {
                let i = if inst == u32::MAX { 0 } else { self.def_inst(inst, self.d.sub(sub).scope) };
                return Some((sub, i));
            }
            if self.d.scope(c).kind == ScopeKind::Spec {
                break;
            }
            cur = self.d.scope(c).parent;
        }
        for &(sub, s) in &subs {
            if matches!(self.d.scope(s).kind, ScopeKind::Package | ScopeKind::Unit) {
                return Some((sub, if inst == u32::MAX { 0 } else { inst }));
            }
        }
        let where_ = if scope == 0 { "no scope (svSetScope was not called)".to_string() } else { format!("scope `{}`", self.dpi_scope_name(inst, sc)) };
        self.dpi_fatal(format!("DPI export `{name}` is not exported from {where_} (35.5.3)"));
        None
    }

    /// An export called by C code running on this thread.
    fn dpi_export_here(&mut self, idx: usize, args: usize, ret: usize, scope: usize) {
        let Some((sub, inst)) = self.dpi_export_target(idx, scope) else { return };
        if self.d.sub(sub).kind == SubKind::Task {
            let n = self.d.name(self.d.sub(sub).name).to_string();
            self.dpi_fatal(format!(
                "exported task `{n}` called from C code that cannot wait: only a `context` imported task may call an exported task (35.8)"
            ));
            set_task_result(ret, 0);
            return;
        }
        self.dpi_run_export_function(sub, inst, args, ret);
    }

    /// The locals of an export called from C, from its C arguments.
    fn dpi_export_locals(&mut self, sub: SubId, args: usize) -> Locals {
        let tys = self.types();
        let locals: Locals = Rc::default();
        let args = args as *const *const u8;
        for (i, &f) in self.d.sub(sub).args.iter().enumerate() {
            let o = self.d.object(f);
            let ty = o.ty;
            let v = if o.dir == Some(Dir::Output) || args.is_null() {
                ops::default_value(tys, ty)
            } else {
                // SAFETY: each argument points at its C value (`stub`).
                let p = unsafe { *args.add(i) };
                if p.is_null() {
                    ops::default_value(tys, ty)
                } else {
                    let (size, _) = c_size(tys, ty);
                    get(tys, ty, unsafe { std::slice::from_raw_parts(p, size) })
                }
            };
            locals.borrow_mut().push((f, ops::convert(tys, &v, ty)));
        }
        locals
    }

    /// Writes an export's outputs back to its C arguments.
    fn dpi_export_outputs(&mut self, sub: SubId, args: usize, locals: &Locals) {
        let tys = self.types();
        let args = args as *const *mut u8;
        if args.is_null() {
            return;
        }
        let mut strings = Vec::new();
        for (i, &f) in self.d.sub(sub).args.iter().enumerate() {
            let o = self.d.object(f);
            if !matches!(o.dir, Some(Dir::Output | Dir::Inout | Dir::Ref)) {
                continue;
            }
            let v = locals.borrow().iter().rev().find(|(k, _)| *k == f).map(|(_, v)| v.clone()).unwrap_or(Const::Null);
            let p = unsafe { *args.add(i) };
            if p.is_null() {
                continue;
            }
            let (size, _) = c_size(tys, o.ty);
            // SAFETY: the caller allocated the argument (H.8.5).
            put(tys, o.ty, &v, unsafe { std::slice::from_raw_parts_mut(p, size) }, &mut strings);
        }
        self.dpi_keep_strings(strings);
    }

    fn dpi_keep_strings(&mut self, strings: Vec<CString>) {
        let dpi = self.dpi();
        dpi.strings.extend(strings);
        while dpi.strings.len() > 256 {
            dpi.strings.pop_front();
        }
    }

    fn dpi_run_export_function(&mut self, sub: SubId, inst: u32, args: usize, ret: usize) {
        let locals = self.dpi_export_locals(sub, args);
        let v = self.invoke_function(inst, sub, &locals, None);
        self.dpi_export_outputs(sub, args, &locals);
        let ret_ty = self.d.sub(sub).ret;
        if ret != 0 && ret_ty != Types::VOID {
            let tys = self.types();
            let (size, _) = c_size(tys, ret_ty);
            let mut strings = Vec::new();
            // SAFETY: the stub's result variable, of the result's C type.
            put(tys, ret_ty, &v, unsafe { std::slice::from_raw_parts_mut(ret as *mut u8, size) }, &mut strings);
            self.dpi_keep_strings(strings);
        }
    }

    // ---- time ----

    /// The time unit and precision of a scope's design element, or the
    /// simulation's precision for none (H.13).
    pub(crate) fn dpi_time_scale(&self, scope: usize) -> (i8, i8) {
        if scope == 0 {
            return (self.precision, self.precision);
        }
        // SAFETY: scopes are records this simulation made.
        let r = unsafe { &*(scope as *const api::ScopeRec) };
        if r.inst == u32::MAX || r.inst as usize >= self.insts.len() {
            return self.d.unit_time;
        }
        let sp = self.d.spec(self.insts[r.inst as usize].spec);
        (sp.time_unit, sp.time_precision)
    }

    pub(crate) fn dpi_now(&self) -> (u64, i8) {
        (self.time, self.precision)
    }
}
