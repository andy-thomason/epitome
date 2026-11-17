//! VPI: the C interface of clauses 36–38, served to applications (such as
//! cocotb) loaded into a simulation.
//!
//! An application is a shared library whose `vlog_startup_routines` array
//! registers callbacks. From them it walks the design, reads and writes
//! values, and waits for times, for changes and for the regions of a time
//! slot (4.4.3). The C functions themselves are made by `export_vpi!` in
//! the executable that loads the library, so that the library finds them
//! there; each works on the simulation running on its thread.
//!
//! Handles are numbers, not pointers: an index into a table and a
//! generation, so that a freed handle is caught rather than followed.
//!
//! How the scheduler serves the application:
//!
//! - value-change callbacks run after the process (or update) that made
//!   the change, with the value it made;
//! - `cbReadWriteSynch` callbacks run once the active, inactive and NBA
//!   regions are empty, and the regions run again after them;
//! - `cbReadOnlySynch` callbacks run after that, before Postponed;
//! - `cbNextSimTime` callbacks run when time moves on, before anything
//!   else at the new time, and `cbAfterDelay` ones at their time, before
//!   the processes it wakes;
//! - `vpiNoDelay` writes happen at once and wake what watches them;
//!   `vpiInertialDelay` and the transport delays are nonblocking updates
//!   at their time; `vpiForceFlag` and `vpiReleaseFlag` act as `force`
//!   and `release` (10.6).

#![allow(non_upper_case_globals)]

use super::*;
use crate::hash::FxMap;
use std::cell::Cell;
use std::ffi::{CStr, CString, c_char, c_void};
use std::path::PathBuf;
use std::ptr::null_mut;

/// The constants of the standard's `vpi_user.h` and `sv_vpi_user.h`
/// listings that this implementation uses, under their names there.
pub mod consts {
    // Object types.
    pub const vpiConstant: i32 = 7;
    pub const vpiIntegerVar: i32 = 25;
    pub const vpiIterator: i32 = 27;
    pub const vpiMemory: i32 = 29;
    pub const vpiMemoryWord: i32 = 30;
    pub const vpiModule: i32 = 32;
    pub const vpiNamedEvent: i32 = 34;
    pub const vpiNet: i32 = 36;
    pub const vpiNetBit: i32 = 37;
    pub const vpiParameter: i32 = 41;
    pub const vpiPort: i32 = 44;
    pub const vpiRealVar: i32 = 47;
    pub const vpiReg: i32 = 48;
    pub const vpiRegBit: i32 = 49;
    pub const vpiSysFuncCall: i32 = 56;
    pub const vpiSysTaskCall: i32 = 57;
    pub const vpiTimeVar: i32 = 63;
    pub const vpiUserSystf: i32 = 67;
    pub const vpiCallback: i32 = 107;
    pub const vpiNetArray: i32 = 114;
    pub const vpiRange: i32 = 115;
    pub const vpiRegArray: i32 = 116;
    pub const vpiGenScopeArray: i32 = 133;
    pub const vpiGenScope: i32 = 134;
    pub const vpiLongIntVar: i32 = 610;
    pub const vpiShortIntVar: i32 = 611;
    pub const vpiIntVar: i32 = 612;
    pub const vpiShortRealVar: i32 = 613;
    pub const vpiByteVar: i32 = 614;
    pub const vpiStringVar: i32 = 616;
    pub const vpiEnumVar: i32 = 617;
    pub const vpiStructVar: i32 = 618;
    pub const vpiBitVar: i32 = 620;
    pub const vpiPackedArrayVar: i32 = 623;
    pub const vpiInstance: i32 = 745;

    // Relations.
    pub const vpiIndex: i32 = 78;
    pub const vpiLeftRange: i32 = 79;
    pub const vpiParent: i32 = 81;
    pub const vpiRightRange: i32 = 83;
    pub const vpiScope: i32 = 84;
    pub const vpiSysTfCall: i32 = 85;
    pub const vpiArgument: i32 = 89;
    pub const vpiBit: i32 = 90;
    pub const vpiInternalScope: i32 = 92;
    pub const vpiVariables: i32 = 100;

    // Properties.
    pub const vpiUndefined: i32 = -1;
    pub const vpiType: i32 = 1;
    pub const vpiName: i32 = 2;
    pub const vpiFullName: i32 = 3;
    pub const vpiSize: i32 = 4;
    pub const vpiFile: i32 = 5;
    pub const vpiLineNo: i32 = 6;
    pub const vpiTopModule: i32 = 7;
    pub const vpiDefName: i32 = 9;
    pub const vpiTimeUnit: i32 = 11;
    pub const vpiTimePrecision: i32 = 12;
    pub const vpiScalar: i32 = 17;
    pub const vpiVector: i32 = 18;
    pub const vpiDirection: i32 = 20;
    pub const vpiInput: i32 = 1;
    pub const vpiOutput: i32 = 2;
    pub const vpiInout: i32 = 3;
    pub const vpiNoDirection: i32 = 5;
    pub const vpiNetType: i32 = 22;
    pub const vpiWire: i32 = 1;
    pub const vpiArray: i32 = 28;
    pub const vpiConstType: i32 = 40;
    pub const vpiDecConst: i32 = 1;
    pub const vpiRealConst: i32 = 2;
    pub const vpiBinaryConst: i32 = 3;
    pub const vpiStringConst: i32 = 6;
    pub const vpiIntConst: i32 = 7;
    pub const vpiSigned: i32 = 65;
    pub const vpiLocalParam: i32 = 70;

    // Value formats.
    pub const vpiBinStrVal: i32 = 1;
    pub const vpiOctStrVal: i32 = 2;
    pub const vpiDecStrVal: i32 = 3;
    pub const vpiHexStrVal: i32 = 4;
    pub const vpiScalarVal: i32 = 5;
    pub const vpiIntVal: i32 = 6;
    pub const vpiRealVal: i32 = 7;
    pub const vpiStringVal: i32 = 8;
    pub const vpiVectorVal: i32 = 9;
    pub const vpiStrengthVal: i32 = 10;
    pub const vpiTimeVal: i32 = 11;
    pub const vpiObjTypeVal: i32 = 12;
    pub const vpiSuppressVal: i32 = 13;

    // Scalar values.
    pub const vpi0: i32 = 0;
    pub const vpi1: i32 = 1;
    pub const vpiZ: i32 = 2;
    pub const vpiX: i32 = 3;
    pub const vpiH: i32 = 4;
    pub const vpiL: i32 = 5;
    pub const vpiDontCare: i32 = 6;

    // `vpi_put_value` flags.
    pub const vpiNoDelay: i32 = 1;
    pub const vpiInertialDelay: i32 = 2;
    pub const vpiTransportDelay: i32 = 3;
    pub const vpiPureTransportDelay: i32 = 4;
    pub const vpiForceFlag: i32 = 5;
    pub const vpiReleaseFlag: i32 = 6;
    pub const vpiCancelEvent: i32 = 7;
    pub const vpiReturnEvent: i32 = 0x1000;

    // Time types.
    pub const vpiScaledRealTime: i32 = 1;
    pub const vpiSimTime: i32 = 2;
    pub const vpiSuppressTime: i32 = 3;

    // Callback reasons.
    pub const cbValueChange: i32 = 1;
    pub const cbAtStartOfSimTime: i32 = 5;
    pub const cbReadWriteSynch: i32 = 6;
    pub const cbReadOnlySynch: i32 = 7;
    pub const cbNextSimTime: i32 = 8;
    pub const cbAfterDelay: i32 = 9;
    pub const cbEndOfCompile: i32 = 10;
    pub const cbStartOfSimulation: i32 = 11;
    pub const cbEndOfSimulation: i32 = 12;
    pub const cbAtEndOfSimTime: i32 = 31;

    // `vpi_control` operations.
    pub const vpiStop: i32 = 66;
    pub const vpiFinish: i32 = 67;

    // System task and function kinds.
    pub const vpiSysTask: i32 = 1;
    pub const vpiSysFunc: i32 = 2;

    // Error states and levels.
    pub const vpiRun: i32 = 3;
    pub const vpiError: i32 = 3;
}

use consts::*;

/// `vpiHandle`.
pub type Handle = *mut c_void;

/// `s_vpi_time`.
#[repr(C)]
pub struct VpiTime {
    pub type_: i32,
    pub high: u32,
    pub low: u32,
    pub real: f64,
}

/// `s_vpi_vecval`: a 32-bit chunk; `aval`/`bval` bits are 0/0 for 0, 1/0
/// for 1, 0/1 for z and 1/1 for x.
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct VecVal {
    pub aval: i32,
    pub bval: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union ValueUnion {
    pub str_: *mut c_char,
    pub scalar: i32,
    pub integer: i32,
    pub real: f64,
    pub time: *mut VpiTime,
    pub vector: *mut VecVal,
    pub misc: *mut c_char,
}

/// `s_vpi_value`.
#[repr(C)]
pub struct VpiValue {
    pub format: i32,
    pub value: ValueUnion,
}

pub type CbRoutine = unsafe extern "C" fn(*mut CbData) -> i32;
pub type TfRoutine = unsafe extern "C" fn(*mut c_char) -> i32;

/// `s_cb_data`.
#[repr(C)]
pub struct CbData {
    pub reason: i32,
    pub cb_rtn: Option<CbRoutine>,
    pub obj: Handle,
    pub time: *mut VpiTime,
    pub value: *mut VpiValue,
    pub index: i32,
    pub user_data: *mut c_char,
}

/// `s_vpi_systf_data`.
#[repr(C)]
pub struct SystfData {
    pub type_: i32,
    pub sysfunctype: i32,
    pub tfname: *const c_char,
    pub calltf: Option<TfRoutine>,
    pub compiletf: Option<TfRoutine>,
    pub sizetf: Option<TfRoutine>,
    pub user_data: *mut c_char,
}

/// `s_vpi_vlog_info`.
#[repr(C)]
pub struct VlogInfo {
    pub argc: i32,
    pub argv: *mut *mut c_char,
    pub product: *mut c_char,
    pub version: *mut c_char,
}

/// `s_vpi_error_info`.
#[repr(C)]
pub struct ErrorInfo {
    pub state: i32,
    pub level: i32,
    pub message: *mut c_char,
    pub product: *mut c_char,
    pub code: *mut c_char,
    pub file: *mut c_char,
    pub line: i32,
}

/// What to load into a simulation.
#[derive(Default)]
pub struct Setup {
    /// The libraries, in the order their startup routines run.
    pub libs: Vec<PathBuf>,
    /// The command line, as `vpi_get_vlog_info` reports it.
    pub argv: Vec<String>,
}

/// Simulates `d` with the VPI applications in `setup` loaded.
pub fn simulate(d: &Design, ast: &Ast, out: &mut dyn Write, opts: &Options, setup: &Setup) -> Result<Outcome, String> {
    let mut s = Sim::new(d, ast, out, opts);
    s.vpi = Some(Box::new(Vpi::new(&setup.argv)));
    let me = &mut s as *mut Sim as *mut c_void;
    let saved = CURRENT.with(|c| c.replace(me));
    let r = s.vpi_start(&setup.libs);
    if r.is_ok() {
        s.run();
        s.vpi_end();
    }
    CURRENT.with(|c| c.set(saved));
    r.map(|()| s.outcome)
}

thread_local! {
    /// The simulation the C functions work on.
    static CURRENT: Cell<*mut c_void> = const { Cell::new(null_mut()) };
}

/// The state of the interface, kept by the simulation.
pub(crate) struct Vpi {
    handles: Vec<(u32, Option<HObj>)>,
    free: Vec<u32>,
    cbs: Vec<(u32, Option<Cb>)>,
    free_cbs: Vec<u32>,
    /// Callbacks waiting for the end of the active regions, for the end of
    /// the time slot, for time to move on, and for the start and end of
    /// the run.
    rw: Vec<(u32, u32)>,
    ro: Vec<(u32, u32)>,
    next: Vec<(u32, u32)>,
    start: Vec<(u32, u32)>,
    end: Vec<(u32, u32)>,
    /// Value-change callbacks by the slot they watch, and the slots that
    /// changed since they were last looked at.
    by_slot: FxMap<usize, Vec<(u32, u32)>>,
    pub(crate) dirty: Vec<usize>,
    systfs: Vec<Systf>,
    /// The system task or function calls running now, innermost last.
    calls: Vec<Call>,
    error: Option<CString>,
    /// Recent strings handed out, which stay valid a while.
    strings: VecDeque<CString>,
    vectors: VecDeque<Vec<VecVal>>,
    times: VecDeque<Box<VpiTime>>,
    argv: Vec<CString>,
    argv_ptrs: Vec<*mut c_char>,
    product: CString,
    version: CString,
    libs: Vec<*mut c_void>,
}

struct Systf {
    name: String,
    data: SystfData,
    /// What `data.tfname` points into.
    _name_c: CString,
}

struct Call {
    tf: usize,
    cx: Cx,
    args: Vec<ExprId>,
    ret: Option<Const>,
}

struct Cb {
    reason: i32,
    rtn: CbRoutine,
    obj: Handle,
    user_data: *mut c_char,
    index: i32,
    time_type: i32,
    format: i32,
    /// For a value change: what is watched, and its value when last seen.
    watch: Option<(Sig, Const)>,
}

/// An object a handle stands for, with its names.
#[derive(Clone)]
struct HObj {
    obj: Obj,
    name: String,
    full: String,
}

#[derive(Clone)]
enum Obj {
    /// A module (or interface, or program) instance: the simulation's
    /// instance.
    Module(u32),
    /// A generate block of an instance.
    Gen(u32, ScopeId),
    /// The blocks of a generate loop.
    GenArray(u32, ScopeId),
    Sig(Sig),
    Param(u32, ObjId),
    /// A constant: a range bound or an index.
    Const(Const),
    /// A dimension's range, `[l:r]`.
    Range(i64, i64),
    Iter(Vec<HObj>, usize),
    /// A callback, by its place in the table and generation.
    Callback(u32, u32),
    /// A registered system task or function.
    Systf,
    /// A call of one being run, by its depth, and an argument of it.
    Call(usize),
    Arg(usize, usize),
}

/// A variable or net, or an element or bits of one.
#[derive(Clone)]
struct Sig {
    inst: u32,
    obj: ObjId,
    slot: usize,
    path: Vec<Sel>,
    shape: Shape,
    /// Its type, when it has one of its own (the whole object or an
    /// element of an unpacked array), for conversions.
    ty: TypeId,
    /// The index that selected it from its parent.
    index: Option<i64>,
    /// How many selects made it: 0 for the object itself.
    depth: u32,
}

#[derive(Clone, Debug)]
enum Shape {
    /// An unpacked array with a fixed range.
    Array { elem: TypeId, range: (i64, i64) },
    /// An integral value: its packed dimensions, outermost first (none for
    /// one bit).
    Packed { dims: Vec<(i64, i64)>, signed: bool, four: bool },
    Real,
    Str,
    Event,
    Other,
}

impl Shape {
    fn width(&self) -> u32 {
        match self {
            Shape::Packed { dims, .. } => dims.iter().map(|&(l, r)| (l - r).unsigned_abs() as u32 + 1).product(),
            _ => 0,
        }
    }
}

impl Vpi {
    fn new(argv: &[String]) -> Vpi {
        let argv: Vec<CString> = argv.iter().map(|a| CString::new(a.as_str()).unwrap_or_default()).collect();
        let mut argv_ptrs: Vec<*mut c_char> = argv.iter().map(|a| a.as_ptr() as *mut c_char).collect();
        argv_ptrs.push(null_mut());
        Vpi {
            handles: Vec::new(),
            free: Vec::new(),
            cbs: Vec::new(),
            free_cbs: Vec::new(),
            rw: Vec::new(),
            ro: Vec::new(),
            next: Vec::new(),
            start: Vec::new(),
            end: Vec::new(),
            by_slot: FxMap::default(),
            dirty: Vec::new(),
            systfs: Vec::new(),
            calls: Vec::new(),
            error: None,
            strings: VecDeque::new(),
            vectors: VecDeque::new(),
            times: VecDeque::new(),
            argv,
            argv_ptrs,
            product: CString::new("epitome-sv").unwrap(),
            version: CString::new(env!("CARGO_PKG_VERSION")).unwrap(),
            libs: Vec::new(),
        }
    }

    fn handle(&mut self, o: HObj) -> Handle {
        let i = match self.free.pop() {
            Some(i) => {
                self.handles[i as usize].1 = Some(o);
                i
            }
            None => {
                self.handles.push((0, Some(o)));
                self.handles.len() as u32 - 1
            }
        };
        let generation = self.handles[i as usize].0;
        (((generation as u64) << 32) | (i as u64 + 1)) as usize as Handle
    }

    fn index(&self, h: Handle) -> Option<usize> {
        let v = h as usize as u64;
        let i = (v & 0xffff_ffff).checked_sub(1)? as usize;
        let generation = (v >> 32) as u32;
        let (g, o) = self.handles.get(i)?;
        (*g == generation && o.is_some()).then_some(i)
    }

    fn get(&self, h: Handle) -> Option<&HObj> {
        self.index(h).and_then(|i| self.handles[i].1.as_ref())
    }

    fn release(&mut self, h: Handle) -> bool {
        let Some(i) = self.index(h) else { return false };
        self.handles[i].1 = None;
        self.handles[i].0 = self.handles[i].0.wrapping_add(1);
        self.free.push(i as u32);
        true
    }

    fn add_cb(&mut self, cb: Cb) -> (u32, u32) {
        let i = match self.free_cbs.pop() {
            Some(i) => {
                self.cbs[i as usize].1 = Some(cb);
                i
            }
            None => {
                self.cbs.push((0, Some(cb)));
                self.cbs.len() as u32 - 1
            }
        };
        (i, self.cbs[i as usize].0)
    }

    fn cb(&self, id: (u32, u32)) -> Option<&Cb> {
        let (g, c) = self.cbs.get(id.0 as usize)?;
        if *g == id.1 { c.as_ref() } else { None }
    }

    fn drop_cb(&mut self, id: (u32, u32)) -> Option<Cb> {
        let e = self.cbs.get_mut(id.0 as usize)?;
        if e.0 != id.1 || e.1.is_none() {
            return None;
        }
        e.0 = e.0.wrapping_add(1);
        let cb = e.1.take();
        self.free_cbs.push(id.0);
        cb
    }

    /// A string that stays valid for the next few calls.
    fn string(&mut self, s: &str) -> *mut c_char {
        let c = CString::new(s.replace('\0', "")).unwrap_or_default();
        let p = c.as_ptr() as *mut c_char;
        self.strings.push_back(c);
        if self.strings.len() > 32 {
            self.strings.pop_front();
        }
        p
    }

    fn time_buf(&mut self, t: VpiTime) -> *mut VpiTime {
        let mut b = Box::new(t);
        let p = &mut *b as *mut VpiTime;
        self.times.push_back(b);
        if self.times.len() > 32 {
            self.times.pop_front();
        }
        p
    }
}

/// Whether to log every call to standard error (`EPITOME_VPI_TRACE` set).
fn tracing() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("EPITOME_VPI_TRACE").is_some_and(|v| !v.is_empty()))
}

/// Logs a call and what it returned, when tracing.
fn traced<R: std::fmt::Debug>(call: impl FnOnce() -> String, r: R) -> R {
    if tracing() {
        eprintln!("vpi: {} -> {r:?}", call());
    }
    r
}

/// Runs `f` on the simulation of this thread, if there is one, after
/// forgetting the last call's error.
fn with<R>(default: R, f: impl FnOnce(&mut Sim<'static, 'static, 'static>) -> R) -> R {
    with_error(default, |s| {
        s.vpi_mut().error = None;
        f(s)
    })
}

fn with_error<R>(default: R, f: impl FnOnce(&mut Sim<'static, 'static, 'static>) -> R) -> R {
    let p = CURRENT.with(|c| c.get());
    if p.is_null() {
        return default;
    }
    // SAFETY: `CURRENT` is set, for the length of the run, to the
    // simulation on this thread, and the simulation does not touch itself
    // while the application's code runs.
    let s = unsafe { &mut *(p as *mut Sim<'static, 'static, 'static>) };
    if s.vpi.is_none() {
        return default;
    }
    f(s)
}

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

impl Sim<'_, '_, '_> {
    // ---- starting and ending ----

    /// Loads the libraries, runs their startup routines, and then the
    /// start-of-simulation callbacks.
    fn vpi_start(&mut self, libs: &[PathBuf]) -> Result<(), String> {
        for lib in libs {
            let path = CString::new(lib.display().to_string()).map_err(|e| e.to_string())?;
            // SAFETY: loading a library the user named runs its code.
            let h = unsafe { dlopen(path.as_ptr(), RTLD_LAZY | RTLD_GLOBAL) };
            if h.is_null() {
                let e = unsafe { dlerror() };
                let msg = if e.is_null() { String::new() } else { unsafe { CStr::from_ptr(e) }.to_string_lossy().into_owned() };
                return Err(format!("cannot load {}: {msg}", lib.display()));
            }
            self.vpi.as_mut().unwrap().libs.push(h);
            let sym = unsafe { dlsym(h, c"vlog_startup_routines".as_ptr()) };
            if sym.is_null() {
                return Err(format!("{} has no vlog_startup_routines", lib.display()));
            }
            // A null-terminated array of routines (36.9.1).
            let mut at = sym as *const Option<unsafe extern "C" fn()>;
            while let Some(f) = unsafe { *at } {
                self.set_current();
                unsafe { f() };
                at = unsafe { at.add(1) };
            }
        }
        let ids = std::mem::take(&mut self.vpi.as_mut().unwrap().start);
        for id in ids {
            self.vpi_fire(id, None);
        }
        Ok(())
    }

    fn vpi_end(&mut self) {
        let ids = std::mem::take(&mut self.vpi.as_mut().unwrap().end);
        for id in ids {
            self.vpi_fire(id, None);
        }
    }

    fn set_current(&mut self) {
        let me = self as *mut Sim as *mut c_void;
        CURRENT.with(|c| c.set(me));
    }

    // ---- the scheduler's side ----

    /// Value-change callbacks for what changed since they last ran.
    pub(super) fn vpi_deliver(&mut self) {
        loop {
            let Some(v) = self.vpi.as_mut() else { return };
            if v.dirty.is_empty() {
                return;
            }
            let mut slots = std::mem::take(&mut v.dirty);
            slots.sort_unstable();
            slots.dedup();
            let mut fire = Vec::new();
            for s in slots {
                let ids = self.vpi.as_ref().unwrap().by_slot.get(&s).cloned().unwrap_or_default();
                for id in ids {
                    let Some(sig) = self.vpi.as_ref().unwrap().cb(id).and_then(|c| c.watch.as_ref()).map(|w| w.0.clone()) else {
                        continue;
                    };
                    let now = self.sig_read(&sig);
                    let v = self.vpi.as_mut().unwrap();
                    let Some(Cb { watch: Some((_, old)), .. }) = v.cbs[id.0 as usize].1.as_mut() else { continue };
                    if !same(old, &now) {
                        *old = now.clone();
                        fire.push((id, now));
                    }
                }
            }
            for (id, val) in fire {
                self.vpi_fire(id, Some(val));
                if self.finishing {
                    return;
                }
            }
        }
    }

    /// The read-write callbacks, each followed by the active regions
    /// again, until none is left; then the read-only ones.
    pub(super) fn vpi_sync(&mut self) {
        loop {
            let ids = std::mem::take(&mut self.vpi.as_mut().unwrap().rw);
            if ids.is_empty() {
                break;
            }
            for id in ids {
                self.vpi_fire(id, None);
            }
            self.slot();
            if self.finishing {
                return;
            }
        }
        loop {
            let ids = std::mem::take(&mut self.vpi.as_mut().unwrap().ro);
            if ids.is_empty() {
                break;
            }
            for id in ids {
                self.vpi_fire(id, None);
            }
        }
    }

    /// Time moves on to `self.time`.
    pub(super) fn vpi_next_time(&mut self) {
        let ids = std::mem::take(&mut self.vpi.as_mut().unwrap().next);
        for id in ids {
            self.vpi_fire(id, None);
        }
    }

    /// A timed entry: an `cbAfterDelay` or `cbAtStartOfSimTime` callback
    /// to run now, or a synchronisation callback to queue for this slot.
    pub(super) fn vpi_timed(&mut self, id: (u32, u32)) {
        let v = self.vpi.as_mut().unwrap();
        match v.cb(id).map(|c| c.reason) {
            Some(cbReadWriteSynch) => v.rw.push(id),
            Some(cbReadOnlySynch | cbAtEndOfSimTime) => v.ro.push(id),
            Some(_) => self.vpi_fire(id, None),
            None => {}
        }
    }

    /// Runs a callback. Callbacks other than value changes run once.
    fn vpi_fire(&mut self, id: (u32, u32), value: Option<Const>) {
        let v = self.vpi.as_mut().unwrap();
        let Some(cb) = v.cb(id) else { return };
        let (reason, rtn, obj, user_data, index, time_type, format) =
            (cb.reason, cb.rtn, cb.obj, cb.user_data, cb.index, cb.time_type, cb.format);
        if reason != cbValueChange {
            v.drop_cb(id);
        }
        let unit = self.handle_unit(obj);
        let mut time = self.time_struct(if time_type == vpiScaledRealTime { vpiScaledRealTime } else { vpiSimTime }, unit);
        let mut val = VpiValue { format: vpiSuppressVal, value: ValueUnion { integer: 0 } };
        if let Some(c) = value
            && format != vpiSuppressVal
        {
            val.format = format;
            let (sh, ty) = match self.vpi.as_ref().unwrap().cb(id).and_then(|c| c.watch.as_ref()) {
                Some((s, _)) => (s.shape.clone(), s.ty),
                None => (Shape::Other, Types::UNKNOWN),
            };
            self.format_value(&c, &sh, ty, &mut val);
        }
        let mut data = CbData { reason, cb_rtn: Some(rtn), obj, time: &mut time, value: &mut val, index, user_data };
        self.set_current();
        // SAFETY: the routine and its data are the application's own.
        unsafe { rtn(&mut data) };
    }

    /// Notes a change of slot `s` for the value-change callbacks.
    pub(super) fn vpi_changed(&mut self, s: usize) {
        if let Some(v) = self.vpi.as_mut()
            && v.by_slot.contains_key(&s)
        {
            v.dirty.push(s);
        }
    }

    /// A system task or function the design calls that an application
    /// registered: runs its `calltf`.
    pub(super) fn vpi_systf(&mut self, cx: &Cx, name: &str, args: &[ExprId]) -> Option<Const> {
        let v = self.vpi.as_mut()?;
        let tf = v.systfs.iter().position(|t| t.name == name)?;
        let calltf = v.systfs[tf].data.calltf;
        let user_data = v.systfs[tf].data.user_data;
        v.calls.push(Call { tf, cx: cx.clone(), args: args.to_vec(), ret: None });
        if let Some(f) = calltf {
            self.set_current();
            unsafe { f(user_data) };
        }
        let call = self.vpi.as_mut().unwrap().calls.pop().unwrap();
        Some(call.ret.unwrap_or(Const::Bits(Bits::zero(32, false))))
    }

    // ---- the design as objects ----

    fn vpi_mut(&mut self) -> &mut Vpi {
        self.vpi.as_mut().unwrap()
    }

    fn new_handle(&mut self, o: HObj) -> Handle {
        self.vpi_mut().handle(o)
    }

    fn obj_of(&self, h: Handle) -> Option<HObj> {
        self.vpi.as_ref().unwrap().get(h).cloned()
    }

    fn error(&mut self, msg: impl Into<String>) {
        let v = self.vpi_mut();
        v.error = Some(CString::new(msg.into()).unwrap_or_default());
    }

    fn module_obj(&self, inst: u32) -> HObj {
        let i = self.d.instance(self.insts[inst as usize].elab);
        let mut name = self.d.name(i.name).to_string();
        if let Some(ix) = i.index {
            name += &format!("[{ix}]");
        }
        HObj { obj: Obj::Module(inst), name, full: self.inst_path(inst) }
    }

    fn tops(&self) -> Vec<HObj> {
        (0..self.insts.len() as u32).filter(|&i| self.insts[i as usize].parent.is_none()).map(|i| self.module_obj(i)).collect()
    }

    /// The scope of a module or generate block.
    fn scope_of(&self, o: &Obj) -> Option<(u32, ScopeId)> {
        match *o {
            Obj::Module(i) => Some((i, self.d.spec(self.insts[i as usize].spec).scope)),
            Obj::Gen(i, s) => Some((i, s)),
            _ => None,
        }
    }

    /// Whether a generate scope holds the blocks of a loop.
    fn is_loop(&self, sc: ScopeId) -> bool {
        let s = self.d.scope(sc);
        s.index.is_none()
            && !s.items.is_empty()
            && s.items.iter().all(|i| matches!(i, Item::Scope(b) if self.d.scope(*b).index.is_some()))
    }

    /// Everything directly inside a module or generate block: instances,
    /// generate blocks (a loop's blocks one by one, and the loop as an
    /// array), and objects.
    fn children(&self, parent: &HObj) -> Vec<HObj> {
        let mut out = Vec::new();
        match &parent.obj {
            Obj::GenArray(inst, sc) => {
                for item in &self.d.scope(*sc).items {
                    if let Item::Scope(b) = item {
                        out.push(self.gen_obj(*inst, *b, &parent.full));
                    }
                }
                return out;
            }
            Obj::Sig(s) => {
                if let Some((lo, hi)) = self.index_range(s) {
                    let (l, r) = (lo, hi);
                    let step = if l <= r { 1 } else { -1 };
                    let mut i = l;
                    loop {
                        if let Some(o) = self.select(parent, i) {
                            out.push(o);
                        }
                        if i == r {
                            break;
                        }
                        i += step;
                    }
                }
                return out;
            }
            Obj::Call(c) => {
                let n = self.vpi.as_ref().unwrap().calls.get(*c).map_or(0, |c| c.args.len());
                for a in 0..n {
                    out.push(HObj { obj: Obj::Arg(*c, a), name: String::new(), full: String::new() });
                }
                return out;
            }
            _ => {}
        }
        let Some((inst, sc)) = self.scope_of(&parent.obj) else { return out };
        self.scope_children(inst, sc, &parent.full, &mut out);
        out
    }

    fn scope_children(&self, inst: u32, sc: ScopeId, full: &str, out: &mut Vec<HObj>) {
        for item in &self.d.scope(sc).items {
            match *item {
                Item::Instance(c) => {
                    if let Some(&k) = self.kids.get(&(inst, c)) {
                        out.push(self.module_obj(k));
                    }
                }
                Item::Scope(s) => {
                    let scope = self.d.scope(s);
                    if scope.kind != ScopeKind::Generate {
                        continue;
                    }
                    match scope.name {
                        None => self.scope_children(inst, s, full, out),
                        Some(n) => {
                            if self.is_loop(s) {
                                out.push(HObj { obj: Obj::GenArray(inst, s), name: self.d.name(n).into(), full: format!("{full}.{}", self.d.name(n)) });
                                for b in &scope.items {
                                    if let Item::Scope(b) = b {
                                        out.push(self.gen_obj(inst, *b, full));
                                    }
                                }
                            } else {
                                out.push(self.gen_obj(inst, s, full));
                            }
                        }
                    }
                }
                Item::Object(o) => {
                    if let Some(h) = self.object_obj(inst, o, full) {
                        out.push(h);
                    }
                }
                _ => {}
            }
        }
    }

    fn gen_obj(&self, inst: u32, b: ScopeId, parent_full: &str) -> HObj {
        let scope = self.d.scope(b);
        let mut name = scope.name.map_or_else(String::new, |n| self.d.name(n).to_string());
        if let Some(ix) = scope.index {
            name += &format!("[{ix}]");
        }
        HObj { obj: Obj::Gen(inst, b), full: format!("{parent_full}.{name}"), name }
    }

    fn object_obj(&self, inst: u32, o: ObjId, parent_full: &str) -> Option<HObj> {
        let ob = self.d.object(o);
        let name = self.d.name(ob.name).to_string();
        let full = format!("{parent_full}.{name}");
        let obj = match ob.kind {
            ObjKind::Param { .. } => Obj::Param(inst, o),
            ObjKind::Variable | ObjKind::Net(_) => {
                if !matches!(self.homes[o.0 as usize], Home::Frame(..) | Home::Global(_)) {
                    return None;
                }
                let slot = self.slot_of_obj(inst, o)?;
                let shape = self.shape(ob.ty);
                if matches!(shape, Shape::Other) {
                    return None;
                }
                Obj::Sig(Sig { inst, obj: o, slot, path: vec![], shape, ty: ob.ty, index: None, depth: 0 })
            }
            _ => return None,
        };
        Some(HObj { obj, name, full })
    }

    fn shape(&self, ty: TypeId) -> Shape {
        let t = self.types();
        match t.get(ty) {
            Type::Unpacked { elem, dim: UDim::Range(l, r) } => Shape::Array { elem: *elem, range: (*l, *r) },
            Type::Real(_) => Shape::Real,
            Type::String => Shape::Str,
            Type::Event => Shape::Event,
            _ if t.is_integral(ty) => Shape::Packed { dims: self.packed_dims(ty), signed: t.is_signed(ty), four: t.is_four_state(ty) },
            _ => Shape::Other,
        }
    }

    /// An integral type's packed dimensions, outermost first.
    fn packed_dims(&self, ty: TypeId) -> Vec<(i64, i64)> {
        let t = self.types();
        match t.get(ty) {
            Type::Vector { atom, dims, .. } => {
                if dims.is_empty() && !atom.is_scalar() {
                    vec![(atom.width() as i64 - 1, 0)]
                } else {
                    dims.clone()
                }
            }
            Type::PackedArray { elem, dim, .. } => {
                let mut v = vec![*dim];
                v.extend(self.packed_dims(*elem));
                v
            }
            Type::Enum { base, .. } => self.packed_dims(*base),
            _ => {
                let w = t.width(ty);
                if w > 1 { vec![(w as i64 - 1, 0)] } else { vec![] }
            }
        }
    }

    /// The range an index of a signal takes.
    fn index_range(&self, s: &Sig) -> Option<(i64, i64)> {
        match &s.shape {
            Shape::Array { range, .. } => Some(*range),
            Shape::Packed { dims, .. } => dims.first().copied(),
            _ => None,
        }
    }

    /// The ranges of a signal's dimensions, unpacked then packed, outermost
    /// first.
    fn ranges(&self, s: &Sig) -> Vec<HObj> {
        let mut out = Vec::new();
        let mut shape = s.shape.clone();
        while let Shape::Array { elem, range } = shape {
            out.push(range);
            shape = self.shape(elem);
        }
        if let Shape::Packed { dims, .. } = shape {
            out.extend(dims);
        }
        out.into_iter().map(|(l, r)| HObj { obj: Obj::Range(l, r), name: String::new(), full: String::new() }).collect()
    }

    /// Element or bits `i` of a signal, or block `i` of a generate loop.
    fn select(&self, parent: &HObj, i: i64) -> Option<HObj> {
        let name = format!("{}[{i}]", parent.name);
        let full = format!("{}[{i}]", parent.full);
        match &parent.obj {
            Obj::GenArray(inst, sc) => {
                for item in &self.d.scope(*sc).items {
                    if let Item::Scope(b) = item
                        && self.d.scope(*b).index == Some(i)
                    {
                        return Some(HObj { obj: Obj::Gen(*inst, *b), name, full });
                    }
                }
                None
            }
            Obj::Sig(s) => {
                let mut e = s.clone();
                e.index = Some(i);
                e.depth += 1;
                match &s.shape {
                    Shape::Array { elem, range: (l, r) } => {
                        let off = if l <= r { i - l } else { l - i };
                        if off < 0 || off > (l - r).abs() {
                            return None;
                        }
                        e.path.push(Sel::Elem(off as usize));
                        e.shape = self.shape(*elem);
                        e.ty = *elem;
                    }
                    Shape::Packed { dims, signed, four } => {
                        let &(l, r) = dims.first()?;
                        let off = if l >= r { i - r } else { r - i };
                        if off < 0 || off > (l - r).abs() {
                            return None;
                        }
                        let rest = dims[1..].to_vec();
                        let w = Shape::Packed { dims: rest.clone(), signed: false, four: *four }.width();
                        let lo = off as u32 * w;
                        match e.path.last_mut() {
                            Some(Sel::Bits(base, bw)) => {
                                *base += lo;
                                *bw = w;
                            }
                            _ => e.path.push(Sel::Bits(lo, w)),
                        }
                        let _ = signed;
                        e.shape = Shape::Packed { dims: rest, signed: false, four: *four };
                        e.ty = Types::UNKNOWN;
                    }
                    _ => return None,
                }
                Some(HObj { obj: Obj::Sig(e), name, full })
            }
            _ => None,
        }
    }

    fn vpi_type(&self, o: &HObj) -> i32 {
        match &o.obj {
            Obj::Module(_) => vpiModule,
            Obj::Gen(..) => vpiGenScope,
            Obj::GenArray(..) => vpiGenScopeArray,
            Obj::Param(..) => vpiParameter,
            Obj::Const(_) => vpiConstant,
            Obj::Range(..) => vpiRange,
            Obj::Iter(..) => vpiIterator,
            Obj::Callback(..) => vpiCallback,
            Obj::Systf => vpiUserSystf,
            Obj::Call(c) => {
                let v = self.vpi.as_ref().unwrap();
                let tf = v.calls.get(*c).map_or(0, |c| c.tf);
                if v.systfs.get(tf).is_some_and(|t| t.data.type_ == vpiSysFunc) { vpiSysFuncCall } else { vpiSysTaskCall }
            }
            Obj::Arg(..) => vpiConstant,
            Obj::Sig(s) => self.sig_type(s),
        }
    }

    fn sig_type(&self, s: &Sig) -> i32 {
        let net = matches!(self.d.object(s.obj).kind, ObjKind::Net(_));
        match &s.shape {
            Shape::Array { .. } => {
                if net {
                    vpiNetArray
                } else {
                    vpiRegArray
                }
            }
            Shape::Packed { dims, .. } if s.depth > 0 && s.ty == Types::UNKNOWN => {
                // Bits of a vector, or a packed element of one.
                match (net, dims.is_empty()) {
                    (true, true) => vpiNetBit,
                    (false, true) => vpiRegBit,
                    (true, false) => vpiNet,
                    (false, false) => vpiReg,
                }
            }
            _ if net => vpiNet,
            Shape::Real => vpiRealVar,
            Shape::Str => vpiStringVar,
            Shape::Event => vpiNamedEvent,
            Shape::Packed { .. } => {
                use crate::design::types::Atom;
                match self.types().get(s.ty) {
                    Type::Vector { atom: Atom::Bit, .. } => vpiBitVar,
                    Type::Vector { atom: Atom::Byte, .. } => vpiByteVar,
                    Type::Vector { atom: Atom::ShortInt, .. } => vpiShortIntVar,
                    Type::Vector { atom: Atom::Int, .. } => vpiIntVar,
                    Type::Vector { atom: Atom::LongInt, .. } => vpiLongIntVar,
                    Type::Vector { atom: Atom::Integer, .. } => vpiIntegerVar,
                    _ if !self.types().is_four_state(s.ty) => vpiBitVar,
                    _ => vpiReg,
                }
            }
            Shape::Other => 0,
        }
    }

    /// Whether `o` belongs in an iteration of type `t`.
    fn of_type(&self, o: &HObj, t: i32) -> bool {
        let ty = self.vpi_type(o);
        match t {
            vpiInternalScope => matches!(o.obj, Obj::Gen(..) | Obj::Module(_)),
            vpiInstance => matches!(o.obj, Obj::Module(_)),
            vpiVariables => matches!(o.obj, Obj::Sig(_)) && !matches!(ty, vpiNet | vpiNetArray),
            vpiMemory => ty == vpiRegArray,
            _ => ty == t,
        }
    }

    // ---- values ----

    fn sig_read(&self, s: &Sig) -> Const {
        let mut v = self.read_slot(s.slot);
        for sel in &s.path {
            v = match (sel, v) {
                (Sel::Elem(i), Const::Aggregate(items)) => items.get(*i).cloned().unwrap_or(Const::Null),
                (Sel::Bits(lo, w), Const::Bits(b)) => Const::Bits(b.slice(*lo, *w)),
                _ => Const::Null,
            };
        }
        v
    }

    fn obj_value(&mut self, o: &HObj) -> Option<Const> {
        match &o.obj {
            Obj::Sig(s) => Some(self.sig_read(s)),
            Obj::Param(_, p) => self.d.object(*p).value.map(|c| self.d.konst(c).clone()),
            Obj::Const(c) => Some(c.clone()),
            Obj::Arg(c, a) => {
                let (cx, x) = {
                    let call = self.vpi.as_ref().unwrap().calls.get(*c)?;
                    (call.cx.clone(), *call.args.get(*a)?)
                };
                Some(self.eval(&cx, x))
            }
            _ => None,
        }
    }

    /// Fills `v` with `c` in the format it asks for.
    fn format_value(&mut self, c: &Const, shape: &Shape, ty: TypeId, v: &mut VpiValue) {
        let bits = || -> Bits {
            match c {
                Const::Bits(b) => b.clone(),
                Const::Real(r) => Bits::from_f64(64, true, *r),
                Const::Str(s) => Bits::from_str_bytes(s),
                Const::Aggregate(_) => crate::design::ops::pack(c),
                _ => Bits::x(shape.width().max(1)),
            }
        };
        let _ = ty;
        if v.format == vpiObjTypeVal {
            v.format = match (c, shape) {
                (Const::Real(_), _) => vpiRealVal,
                (Const::Str(_), _) => vpiStringVal,
                (Const::Bits(b), _) if b.width() == 1 => vpiScalarVal,
                (Const::Bits(b), _) if b.width() <= 32 && b.is_known() => vpiIntVal,
                _ => vpiVectorVal,
            };
        }
        match v.format {
            vpiBinStrVal => {
                let s = bits().to_radix(1).to_lowercase();
                v.value.str_ = self.vpi_mut().string(&s);
            }
            vpiOctStrVal => {
                let s = bits().to_radix(3).to_lowercase();
                v.value.str_ = self.vpi_mut().string(&s);
            }
            vpiHexStrVal => {
                let s = bits().to_radix(4).to_lowercase();
                v.value.str_ = self.vpi_mut().string(&s);
            }
            vpiDecStrVal => {
                let s = match c {
                    Const::Real(r) => format!("{}", r.round() as i64),
                    _ => bits().to_decimal(),
                };
                v.value.str_ = self.vpi_mut().string(&s);
            }
            vpiScalarVal => {
                v.value.scalar = match bits().bit(0) {
                    Logic::Zero => vpi0,
                    Logic::One => vpi1,
                    Logic::Z => vpiZ,
                    Logic::X => vpiX,
                };
            }
            vpiIntVal => {
                v.value.integer = match c {
                    Const::Real(r) => r.round() as i32,
                    _ => {
                        let b = bits().known_or_zero();
                        let w = b.width().max(1);
                        b.resize(64.max(w), b.signed()).slice(0, 32).to_u64().unwrap_or(0) as u32 as i32
                    }
                };
            }
            vpiRealVal => {
                v.value.real = match c {
                    Const::Real(r) => *r,
                    _ => bits().known_or_zero().to_f64(),
                };
            }
            vpiStringVal => {
                let bytes = match c {
                    Const::Str(s) => s.clone(),
                    _ => bits().known_or_zero().to_str_bytes(),
                };
                let s: Vec<u8> = bytes.into_iter().filter(|&b| b != 0).collect();
                v.value.str_ = self.vpi_mut().string(&String::from_utf8_lossy(&s));
            }
            vpiVectorVal => {
                let b = bits();
                let n = b.width().max(1).div_ceil(32);
                let mut words = vec![VecVal::default(); n as usize];
                for i in 0..b.width() {
                    let (a, bv) = match b.bit(i) {
                        Logic::Zero => (0, 0),
                        Logic::One => (1, 0),
                        Logic::Z => (0, 1),
                        Logic::X => (1, 1),
                    };
                    let w = &mut words[(i / 32) as usize];
                    w.aval |= a << (i % 32);
                    w.bval |= bv << (i % 32);
                }
                let vpi = self.vpi_mut();
                vpi.vectors.push_back(words);
                if vpi.vectors.len() > 32 {
                    vpi.vectors.pop_front();
                }
                v.value.vector = vpi.vectors.back_mut().unwrap().as_mut_ptr();
            }
            vpiTimeVal => {
                let t = bits().known_or_zero().resize(64, false).to_u64().unwrap_or(0);
                let p = self.vpi_mut().time_buf(VpiTime { type_: vpiSimTime, high: (t >> 32) as u32, low: t as u32, real: 0.0 });
                v.value.time = p;
            }
            vpiSuppressVal => {}
            f => self.error(format!("vpi_get_value: format {f} is not supported")),
        }
    }

    /// A value given by the application, as a value of `shape`.
    fn parse_value(&mut self, v: &VpiValue, shape: &Shape) -> Option<Const> {
        let (width, signed, four) = match shape {
            Shape::Packed { signed, four, .. } => (shape.width(), *signed, *four),
            Shape::Real | Shape::Str => (64, true, false),
            _ => (32, false, true),
        };
        let cstr = |p: *mut c_char| -> String {
            if p.is_null() { String::new() } else { unsafe { CStr::from_ptr(p) }.to_string_lossy().into_owned() }
        };
        let fit = |b: Bits| b.resize(width.max(1), b.signed()).with_signed(signed);
        let b = unsafe {
            match v.format {
                vpiBinStrVal | vpiOctStrVal | vpiHexStrVal | vpiDecStrVal => {
                    let s = cstr(v.value.str_);
                    let base = match v.format {
                        vpiBinStrVal => 2,
                        vpiOctStrVal => 8,
                        vpiHexStrVal => 16,
                        _ => 10,
                    };
                    let neg = base == 10 && s.starts_with('-');
                    let digits = s.trim_start_matches('-');
                    let b = crate::design::value::parse_digits(digits, base, None, false).ok()?;
                    let b = if neg { b.resize(b.width() + 1, false).neg() } else { b };
                    fit(b.resize(b.width().max(width), neg))
                }
                vpiScalarVal => {
                    let l = match v.value.scalar {
                        vpi0 | vpiL => Logic::Zero,
                        vpi1 | vpiH => Logic::One,
                        vpiZ => Logic::Z,
                        _ => Logic::X,
                    };
                    fit(Bits::of_logic(l))
                }
                vpiIntVal => fit(Bits::from_i64(64, true, v.value.integer as i64)),
                vpiRealVal => {
                    if matches!(shape, Shape::Real) {
                        return Some(Const::Real(v.value.real));
                    }
                    fit(Bits::from_f64(64, true, v.value.real))
                }
                vpiStringVal => {
                    let s = cstr(v.value.str_);
                    if matches!(shape, Shape::Str) {
                        return Some(Const::Str(s.into_bytes()));
                    }
                    fit(Bits::from_str_bytes(s.as_bytes()).resize(width.max(1), false))
                }
                vpiVectorVal => {
                    if v.value.vector.is_null() {
                        return None;
                    }
                    let mut b = Bits::zero(width.max(1), signed);
                    for i in 0..width {
                        let w = *v.value.vector.add((i / 32) as usize);
                        let (a, bv) = ((w.aval >> (i % 32)) & 1, (w.bval >> (i % 32)) & 1);
                        b.set_bit(
                            i,
                            match (a, bv) {
                                (0, 0) => Logic::Zero,
                                (1, 0) => Logic::One,
                                (0, _) => Logic::Z,
                                _ => Logic::X,
                            },
                        );
                    }
                    b
                }
                vpiTimeVal => {
                    if v.value.time.is_null() {
                        return None;
                    }
                    let t = &*v.value.time;
                    fit(Bits::from_u64(64, false, (t.high as u64) << 32 | t.low as u64))
                }
                f => {
                    self.error(format!("vpi_put_value: format {f} is not supported"));
                    return None;
                }
            }
        };
        Some(match shape {
            Shape::Real => Const::Real(b.known_or_zero().to_f64()),
            Shape::Str => Const::Str(b.to_str_bytes()),
            _ => Const::Bits(if four { b } else { b.known_or_zero() }),
        })
    }

    fn sig_target(&self, s: &Sig) -> Target {
        Target { place: Place::Slot(s.slot), path: s.path.clone(), ty: if s.path.is_empty() { s.ty } else { Types::UNKNOWN } }
    }

    /// A write from outside, at once.
    fn sig_write(&mut self, s: &Sig, v: Const) {
        let t = self.sig_target(s);
        let cx = Cx { inst: u32::MAX, locals: Rc::default(), this: None };
        self.write_cx(&cx, &t, v, false);
    }

    fn sig_force(&mut self, s: &Sig, v: Const) {
        let slot = self.find(s.slot);
        let bits = match s.path.as_slice() {
            [Sel::Bits(lo, w)] => Some((*lo, *w)),
            [] => None,
            _ => {
                self.error("vpi_put_value: only a whole object or its bits can be forced");
                return;
            }
        };
        let old = self.pcont.entry(slot).or_default().force.take();
        if let Some((p, _)) = old
            && p != VPI_FORCE
        {
            self.end_proc(p);
        }
        self.pcont.entry(slot).or_default().force = Some((VPI_FORCE, bits));
        let saved = self.privileged.replace(VPI_FORCE);
        self.sig_write(s, v);
        self.privileged = saved;
    }

    fn sig_release(&mut self, s: &Sig) {
        let slot = self.find(s.slot);
        let Some(pc) = self.pcont.get_mut(&slot) else { return };
        let old = pc.force.take();
        if let Some((p, _)) = old
            && p != VPI_FORCE
        {
            self.end_proc(p);
        }
        let is_net = matches!(self.d.object(s.obj).kind, ObjKind::Net(_));
        // The drivers run now, so that the value the release hands back is
        // theirs.
        for q in self.released(slot, is_net) {
            self.step(q);
        }
        if self.pcont.get(&slot).is_some_and(|pc| pc.force.is_none() && pc.assign.is_none()) {
            self.pcont.remove(&slot);
        }
    }

    /// The time unit of the scope a handle is in, as a power of ten of a
    /// second.
    fn handle_unit(&self, h: Handle) -> i8 {
        let inst = match self.vpi.as_ref().and_then(|v| v.get(h)).map(|o| &o.obj) {
            Some(Obj::Module(i) | Obj::Gen(i, _) | Obj::Param(i, _)) => *i,
            Some(Obj::Sig(s)) => s.inst,
            _ => u32::MAX,
        };
        self.unit_of(inst)
    }

    fn time_struct(&self, ty: i32, unit: i8) -> VpiTime {
        let t = self.time;
        let real = t as f64 / 10f64.powi((unit - self.precision) as i32);
        VpiTime { type_: ty, high: (t >> 32) as u32, low: t as u32, real }
    }

    /// A delay given by the application, in ticks of the global precision.
    fn delay_ticks(&self, t: *const VpiTime, unit: i8) -> u64 {
        if t.is_null() {
            return 0;
        }
        let t = unsafe { &*t };
        match t.type_ {
            vpiScaledRealTime => (t.real * 10f64.powi((unit - self.precision) as i32)).round().max(0.0) as u64,
            vpiSimTime => (t.high as u64) << 32 | t.low as u64,
            _ => 0,
        }
    }

    // ---- the C functions' bodies ----

    fn api_register_cb(&mut self, data: &CbData) -> Handle {
        let Some(rtn) = data.cb_rtn else {
            self.error("vpi_register_cb: no routine");
            return null_mut();
        };
        let time_type = if data.time.is_null() { vpiSuppressTime } else { unsafe { (*data.time).type_ } };
        let format = if data.value.is_null() { vpiSuppressVal } else { unsafe { (*data.value).format } };
        let mut cb = Cb { reason: data.reason, rtn, obj: data.obj, user_data: data.user_data, index: data.index, time_type, format, watch: None };
        let unit = self.handle_unit(data.obj);
        match data.reason {
            cbValueChange => {
                let Some(HObj { obj: Obj::Sig(s), .. }) = self.obj_of(data.obj) else {
                    self.error("vpi_register_cb: cbValueChange needs a net or variable");
                    return null_mut();
                };
                let now = self.sig_read(&s);
                let slot = self.find(s.slot);
                cb.watch = Some((s, now));
                let id = self.vpi_mut().add_cb(cb);
                self.vpi_mut().by_slot.entry(slot).or_default().push(id);
                return self.cb_handle(id);
            }
            cbAfterDelay | cbAtStartOfSimTime => {
                let d = self.delay_ticks(data.time, unit);
                let at = if data.reason == cbAfterDelay { self.time + d } else { d.max(self.time) };
                let id = self.vpi_mut().add_cb(cb);
                self.wheel.entry(at).or_default().push(Timed::Vpi(id));
                return self.cb_handle(id);
            }
            cbReadWriteSynch | cbReadOnlySynch | cbAtEndOfSimTime => {
                let d = self.delay_ticks(data.time, unit);
                let reason = data.reason;
                let id = self.vpi_mut().add_cb(cb);
                if d > 0 {
                    self.wheel.entry(self.time + d).or_default().push(Timed::Vpi(id));
                } else if reason == cbReadWriteSynch {
                    self.vpi_mut().rw.push(id);
                } else {
                    self.vpi_mut().ro.push(id);
                }
                return self.cb_handle(id);
            }
            cbNextSimTime | cbStartOfSimulation | cbEndOfSimulation | cbEndOfCompile => {
                let reason = data.reason;
                let id = self.vpi_mut().add_cb(cb);
                let v = self.vpi_mut();
                match reason {
                    cbNextSimTime => v.next.push(id),
                    cbEndOfSimulation => v.end.push(id),
                    _ => v.start.push(id),
                }
                return self.cb_handle(id);
            }
            r => self.error(format!("vpi_register_cb: reason {r} is not supported")),
        }
        null_mut()
    }

    fn cb_handle(&mut self, id: (u32, u32)) -> Handle {
        self.new_handle(HObj { obj: Obj::Callback(id.0, id.1), name: String::new(), full: String::new() })
    }

    fn api_remove_cb(&mut self, h: Handle) -> i32 {
        let Some(HObj { obj: Obj::Callback(i, g), .. }) = self.obj_of(h) else {
            self.error("vpi_remove_cb: not a callback handle");
            return 0;
        };
        let v = self.vpi_mut();
        if let Some(cb) = v.drop_cb((i, g))
            && let Some((s, _)) = cb.watch
        {
            let slot = self.find(s.slot);
            let v = self.vpi_mut();
            if let Some(list) = v.by_slot.get_mut(&slot) {
                list.retain(|&id| id != (i, g));
                if list.is_empty() {
                    v.by_slot.remove(&slot);
                }
            }
        }
        // Lists of other kinds skip a callback that is gone.
        self.vpi_mut().release(h);
        1
    }

    fn api_handle(&mut self, ty: i32, h: Handle) -> Handle {
        if h.is_null() {
            if ty == vpiSysTfCall {
                let v = self.vpi_mut();
                if let Some(c) = v.calls.len().checked_sub(1) {
                    let name = v.systfs[v.calls[c].tf].name.clone();
                    return self.new_handle(HObj { obj: Obj::Call(c), name: name.clone(), full: name });
                }
            }
            return null_mut();
        }
        let Some(o) = self.obj_of(h) else {
            self.error("vpi_handle: bad handle");
            return null_mut();
        };
        let found = match (ty, &o.obj) {
            (vpiLeftRange | vpiRightRange, Obj::Range(l, r)) => {
                let v = if ty == vpiLeftRange { *l } else { *r };
                Some(HObj { obj: Obj::Const(Const::Bits(Bits::from_i64(32, true, v))), name: String::new(), full: String::new() })
            }
            (vpiLeftRange | vpiRightRange, Obj::Sig(s)) => self.index_range(s).map(|(l, r)| {
                let v = if ty == vpiLeftRange { l } else { r };
                HObj { obj: Obj::Const(Const::Bits(Bits::from_i64(32, true, v))), name: String::new(), full: String::new() }
            }),
            (vpiIndex, Obj::Sig(Sig { index: Some(i), .. })) => {
                Some(HObj { obj: Obj::Const(Const::Bits(Bits::from_i64(32, true, *i))), name: String::new(), full: String::new() })
            }
            (vpiParent, Obj::Sig(s)) if s.depth > 0 => self.parent_sig(&o, s),
            (vpiModule | vpiScope | vpiParent, _) => self.scope_above(&o, ty == vpiModule),
            _ => None,
        };
        match found {
            Some(f) => self.new_handle(f),
            None => null_mut(),
        }
    }

    /// The signal a select was made from.
    fn parent_sig(&self, o: &HObj, s: &Sig) -> Option<HObj> {
        let mut cur = self.object_obj(s.inst, s.obj, &self.scope_full(s.inst, self.d.object(s.obj).scope))?;
        // Walk down again by the indices on the name, all but the last.
        let suffix = &o.full[cur.full.len()..];
        let idx: Vec<i64> = suffix.split('[').skip(1).filter_map(|p| p.trim_end_matches(']').parse().ok()).collect();
        for &i in &idx[..idx.len().saturating_sub(1)] {
            cur = self.select(&cur, i)?;
        }
        Some(cur)
    }

    /// The full name of the scope `sc` of instance `inst`.
    fn scope_full(&self, inst: u32, sc: ScopeId) -> String {
        let spec_scope = self.d.spec(self.insts[inst as usize].spec).scope;
        let mut parts = Vec::new();
        let mut cur = Some(sc);
        let mut skip_container = false;
        while let Some(c) = cur {
            if c == spec_scope {
                break;
            }
            let s = self.d.scope(c);
            if s.kind == ScopeKind::Generate
                && let Some(n) = s.name
            {
                if skip_container && s.index.is_none() {
                    skip_container = false;
                } else {
                    let mut name = self.d.name(n).to_string();
                    if let Some(ix) = s.index {
                        name += &format!("[{ix}]");
                        skip_container = true;
                    }
                    parts.push(name);
                }
            }
            cur = s.parent;
        }
        let mut full = self.inst_path(inst);
        for p in parts.iter().rev() {
            full += ".";
            full += p;
        }
        full
    }

    /// The module (or, unless `module`, the nearest scope) above `o`.
    fn scope_above(&self, o: &HObj, module: bool) -> Option<HObj> {
        let (inst, sc) = match &o.obj {
            Obj::Sig(s) => (s.inst, self.d.object(s.obj).scope),
            Obj::Param(i, p) => (*i, self.d.object(*p).scope),
            Obj::Gen(i, s) | Obj::GenArray(i, s) => {
                let parent = self.d.scope(*s).parent?;
                (*i, parent)
            }
            Obj::Module(i) => {
                let p = self.insts[*i as usize].parent?;
                return Some(self.module_obj(p));
            }
            _ => return None,
        };
        let spec_scope = self.d.spec(self.insts[inst as usize].spec).scope;
        let mut cur = Some(sc);
        while let Some(c) = cur {
            if c == spec_scope {
                return Some(self.module_obj(inst));
            }
            let s = self.d.scope(c);
            if !module && s.kind == ScopeKind::Generate && s.name.is_some() && !self.is_loop(c) {
                let full = self.scope_full(inst, c);
                let name = full.rsplit('.').next().unwrap_or("").to_string();
                return Some(HObj { obj: Obj::Gen(inst, c), name, full });
            }
            cur = s.parent;
        }
        Some(self.module_obj(inst))
    }

    fn api_handle_by_name(&mut self, name: &str, scope: Handle) -> Handle {
        let mut parts = split_name(name);
        let mut cur = if scope.is_null() {
            if parts.first().map(String::as_str) == Some("$root") {
                parts.remove(0);
            }
            if parts.is_empty() {
                return null_mut();
            }
            let first = parts.remove(0);
            match self.tops().into_iter().find(|t| t.name == first) {
                Some(t) => t,
                None => return null_mut(),
            }
        } else {
            match self.obj_of(scope) {
                Some(o) => o,
                None => {
                    self.error("vpi_handle_by_name: bad scope handle");
                    return null_mut();
                }
            }
        };
        for p in parts {
            match self.child_named(&cur, &p) {
                Some(c) => cur = c,
                None => return null_mut(),
            }
        }
        self.new_handle(cur)
    }

    fn child_named(&self, parent: &HObj, name: &str) -> Option<HObj> {
        let kids = self.children(parent);
        if let Some(k) = kids.iter().find(|k| k.name == name) {
            return Some(k.clone());
        }
        // `a[1][2]`: the object, then its selects.
        let open = name.find('[')?;
        let mut cur = kids.into_iter().find(|k| k.name == name[..open])?;
        for part in name[open..].split('[').skip(1) {
            let i: i64 = part.strip_suffix(']')?.trim().parse().ok()?;
            cur = self.select(&cur, i)?;
        }
        Some(cur)
    }

    fn api_handle_by_index(&mut self, h: Handle, i: i32) -> Handle {
        let Some(o) = self.obj_of(h) else {
            self.error("vpi_handle_by_index: bad handle");
            return null_mut();
        };
        match self.select(&o, i as i64) {
            Some(s) => self.new_handle(s),
            None => null_mut(),
        }
    }

    fn api_iterate(&mut self, ty: i32, h: Handle) -> Handle {
        let items: Vec<HObj> = if h.is_null() {
            match ty {
                vpiModule | vpiInstance | vpiInternalScope => self.tops(),
                _ => vec![],
            }
        } else {
            let Some(o) = self.obj_of(h) else {
                self.error("vpi_iterate: bad handle");
                return null_mut();
            };
            match &o.obj {
                Obj::Sig(s) if ty == vpiRange => self.ranges(s),
                // Of arrays and vectors: their elements or bits.
                Obj::Sig(_) if matches!(ty, vpiRange | vpiParameter | vpiModule | vpiInternalScope | vpiPort | vpiArgument) => vec![],
                Obj::Sig(_) | Obj::GenArray(..) => self.children(&o),
                Obj::Call(_) if ty == vpiArgument => self.children(&o),
                _ => self.children(&o).into_iter().filter(|k| self.of_type(k, ty)).collect(),
            }
        };
        if items.is_empty() {
            return null_mut();
        }
        self.new_handle(HObj { obj: Obj::Iter(items, 0), name: String::new(), full: String::new() })
    }

    fn api_scan(&mut self, h: Handle) -> Handle {
        let v = self.vpi_mut();
        let next = match v.index(h).and_then(|i| v.handles[i].1.as_mut()) {
            Some(HObj { obj: Obj::Iter(items, at), .. }) => {
                let o = items.get(*at).cloned();
                *at += 1;
                o
            }
            _ => {
                self.error("vpi_scan: not an iterator");
                return null_mut();
            }
        };
        match next {
            Some(o) => self.new_handle(o),
            None => {
                // An iterator is freed when it runs out (37.2.2).
                self.vpi_mut().release(h);
                null_mut()
            }
        }
    }

    fn api_get(&mut self, prop: i32, h: Handle) -> i64 {
        if h.is_null() {
            return match prop {
                vpiTimePrecision => self.precision as i64,
                vpiTimeUnit => self.d.unit_time.0 as i64,
                _ => vpiUndefined as i64,
            };
        }
        let Some(o) = self.obj_of(h) else {
            self.error("vpi_get: bad handle");
            return vpiUndefined as i64;
        };
        let t = self.vpi_type(&o);
        let packed = |s: &Sig| matches!(s.shape, Shape::Packed { .. });
        let r: i64 = match (prop, &o.obj) {
            (vpiType, _) => t as i64,
            (vpiSize, Obj::Sig(s)) => match &s.shape {
                Shape::Array { range: (l, r), .. } => (l - r).abs() + 1,
                Shape::Packed { .. } => s.shape.width() as i64,
                Shape::Str => match self.sig_read(s) {
                    Const::Str(b) => b.len() as i64,
                    _ => 0,
                },
                _ => 1,
            },
            (vpiSize, Obj::Param(..) | Obj::Const(_) | Obj::Arg(..)) => match self.obj_value(&o) {
                Some(Const::Bits(b)) => b.width() as i64,
                Some(Const::Str(s)) => s.len() as i64 * 8,
                Some(_) => 1,
                None => vpiUndefined as i64,
            },
            (vpiSize, Obj::GenArray(_, sc)) => self.d.scope(*sc).items.len() as i64,
            (vpiSize, Obj::Range(l, r)) => (l - r).abs() + 1,
            (vpiSize, Obj::Iter(items, _)) => items.len() as i64,
            (vpiVector, Obj::Sig(s)) => (packed(s) && (s.shape.width() > 1 || !matches!(t, vpiReg | vpiNet | vpiRegBit | vpiNetBit | vpiBitVar))) as i64,
            (vpiScalar, Obj::Sig(s)) => (packed(s) && s.shape.width() == 1 && matches!(t, vpiReg | vpiNet | vpiRegBit | vpiNetBit | vpiBitVar)) as i64,
            (vpiSigned, Obj::Sig(s)) => matches!(s.shape, Shape::Packed { signed: true, .. }) as i64,
            (vpiSigned, Obj::Param(..) | Obj::Const(_)) => matches!(self.obj_value(&o), Some(Const::Bits(b)) if b.signed()) as i64,
            (vpiArray, Obj::Sig(s)) => matches!(s.shape, Shape::Array { .. }) as i64,
            (vpiDirection, Obj::Sig(s)) if s.depth == 0 => (match self.d.object(s.obj).dir {
                Some(Dir::Input) => vpiInput,
                Some(Dir::Output) => vpiOutput,
                Some(Dir::Inout | Dir::Ref | Dir::ConstRef) => vpiInout,
                None => vpiNoDirection,
            }) as i64,
            (vpiNetType, Obj::Sig(s)) if matches!(self.d.object(s.obj).kind, ObjKind::Net(_)) => vpiWire as i64,
            (vpiTopModule, Obj::Module(i)) => self.insts[*i as usize].parent.is_none() as i64,
            (vpiLocalParam, Obj::Param(_, p)) => matches!(self.d.object(*p).kind, ObjKind::Param { local: true }) as i64,
            (vpiConstType, Obj::Param(..) | Obj::Const(_) | Obj::Arg(..)) => (match self.obj_value(&o) {
                Some(Const::Real(_)) => vpiRealConst,
                Some(Const::Str(_)) => vpiStringConst,
                Some(Const::Bits(b)) if b.is_known() && b.width() <= 32 => vpiIntConst,
                Some(Const::Bits(b)) if b.is_known() => vpiDecConst,
                _ => vpiBinaryConst,
            }) as i64,
            (vpiTimeUnit, _) => self.handle_unit(h) as i64,
            (vpiTimePrecision, Obj::Module(i)) => self.d.spec(self.insts[*i as usize].spec).time_precision as i64,
            (vpiTimePrecision, _) => self.precision as i64,
            _ => vpiUndefined as i64,
        };
        r
    }

    fn api_get_str(&mut self, prop: i32, h: Handle) -> *mut c_char {
        let Some(o) = self.obj_of(h) else {
            if !h.is_null() {
                self.error("vpi_get_str: bad handle");
            }
            return null_mut();
        };
        let s = match (prop, &o.obj) {
            (vpiName, Obj::Iter(..) | Obj::Callback(..) | Obj::Const(_) | Obj::Range(..)) => return null_mut(),
            (vpiName, _) => o.name.clone(),
            (vpiFullName, Obj::Iter(..) | Obj::Callback(..) | Obj::Const(_) | Obj::Range(..) | Obj::Call(_) | Obj::Arg(..)) => return null_mut(),
            (vpiFullName, _) => o.full.clone(),
            (vpiDefName, Obj::Module(i)) => self.d.name(self.d.spec(self.insts[*i as usize].spec).name).to_string(),
            (vpiType, _) => type_name(self.vpi_type(&o)).to_string(),
            _ => return null_mut(),
        };
        self.vpi_mut().string(&s)
    }

    fn api_get_value(&mut self, h: Handle, v: &mut VpiValue) {
        let Some(o) = self.obj_of(h) else {
            self.error("vpi_get_value: bad handle");
            return;
        };
        let Some(c) = self.obj_value(&o) else {
            self.error("vpi_get_value: the object has no value");
            return;
        };
        let (shape, ty) = match &o.obj {
            Obj::Sig(s) => (s.shape.clone(), s.ty),
            _ => match &c {
                Const::Bits(b) => (Shape::Packed { dims: if b.width() > 1 { vec![(b.width() as i64 - 1, 0)] } else { vec![] }, signed: b.signed(), four: true }, Types::UNKNOWN),
                Const::Real(_) => (Shape::Real, Types::REAL),
                Const::Str(_) => (Shape::Str, Types::STRING),
                _ => (Shape::Other, Types::UNKNOWN),
            },
        };
        self.format_value(&c, &shape, ty, v);
    }

    fn api_put_value(&mut self, h: Handle, v: Option<&mut VpiValue>, time: *const VpiTime, flags: i32) -> Handle {
        let Some(o) = self.obj_of(h) else {
            self.error("vpi_put_value: bad handle");
            return null_mut();
        };
        let mode = flags & 0xfff;
        if let Obj::Call(c) = o.obj {
            let Some(v) = v else { return null_mut() };
            let val = self.parse_value(v, &Shape::Packed { dims: vec![(31, 0)], signed: true, four: true });
            if let Some(call) = self.vpi_mut().calls.get_mut(c) {
                call.ret = val;
            }
            return null_mut();
        }
        let Obj::Sig(s) = &o.obj else {
            self.error("vpi_put_value: the object cannot be written");
            return null_mut();
        };
        if mode == vpiReleaseFlag {
            self.sig_release(s);
            if let Some(v) = v {
                let now = self.sig_read(s);
                self.format_value(&now, &s.shape.clone(), s.ty, v);
            }
            return null_mut();
        }
        let Some(v) = v else {
            self.error("vpi_put_value: no value");
            return null_mut();
        };
        let Some(val) = self.parse_value(v, &s.shape) else { return null_mut() };
        match mode {
            vpiNoDelay => self.sig_write(s, val),
            vpiForceFlag => self.sig_force(s, val),
            vpiInertialDelay | vpiTransportDelay | vpiPureTransportDelay => {
                let unit = self.handle_unit(h);
                let d = self.delay_ticks(time, unit);
                let t = self.sig_target(s);
                let cx = Cx { inst: u32::MAX, locals: Rc::default(), this: None };
                if d == 0 {
                    self.nba.push((t, val, cx));
                } else {
                    self.after(d, Timed::Nba(t, val, cx));
                }
            }
            vpiCancelEvent => {}
            m => self.error(format!("vpi_put_value: delay mode {m} is not supported")),
        }
        null_mut()
    }

    fn api_get_time(&mut self, h: Handle, t: &mut VpiTime) {
        let unit = if h.is_null() { self.d.unit_time.0 } else { self.handle_unit(h) };
        let ty = if t.type_ == vpiScaledRealTime { vpiScaledRealTime } else { vpiSimTime };
        *t = self.time_struct(ty, unit);
    }

    fn api_control(&mut self, op: i32) -> i32 {
        match op {
            vpiFinish | vpiStop => {
                self.finishing = true;
                self.outcome.finished = true;
                1
            }
            _ => 0,
        }
    }

    fn api_register_systf(&mut self, d: &SystfData) -> Handle {
        if d.tfname.is_null() {
            self.error("vpi_register_systf: no name");
            return null_mut();
        }
        let name = unsafe { CStr::from_ptr(d.tfname) }.to_string_lossy().into_owned();
        let name_c = CString::new(name.clone()).unwrap_or_default();
        let data = SystfData {
            type_: d.type_,
            sysfunctype: d.sysfunctype,
            tfname: name_c.as_ptr(),
            calltf: d.calltf,
            compiletf: d.compiletf,
            sizetf: d.sizetf,
            user_data: d.user_data,
        };
        let v = self.vpi_mut();
        v.systfs.push(Systf { name: name.clone(), data, _name_c: name_c });
        self.new_handle(HObj { obj: Obj::Systf, name: name.clone(), full: name })
    }

    fn api_free(&mut self, h: Handle) -> i32 {
        self.vpi_mut().release(h) as i32
    }

    fn api_chk_error(&mut self, info: *mut ErrorInfo) -> i32 {
        let v = self.vpi_mut();
        let Some(msg) = v.error.as_ref() else { return 0 };
        if !info.is_null() {
            let i = unsafe { &mut *info };
            i.state = vpiRun;
            i.level = vpiError;
            i.message = msg.as_ptr() as *mut c_char;
            i.product = v.product.as_ptr() as *mut c_char;
            i.code = c"".as_ptr() as *mut c_char;
            i.file = null_mut();
            i.line = 0;
        }
        vpiError
    }

    fn api_vlog_info(&mut self, info: &mut VlogInfo) -> i32 {
        let v = self.vpi_mut();
        info.argc = v.argv.len() as i32;
        info.argv = v.argv_ptrs.as_mut_ptr();
        info.product = v.product.as_ptr() as *mut c_char;
        info.version = v.version.as_ptr() as *mut c_char;
        1
    }

    fn api_compare(&mut self, a: Handle, b: Handle) -> i32 {
        let (Some(x), Some(y)) = (self.obj_of(a), self.obj_of(b)) else { return 0 };
        let same = match (&x.obj, &y.obj) {
            (Obj::Sig(p), Obj::Sig(q)) => p.slot == q.slot && format!("{:?}", p.path) == format!("{:?}", q.path),
            (Obj::Module(p), Obj::Module(q)) => p == q,
            (Obj::Gen(i, s), Obj::Gen(j, t)) | (Obj::GenArray(i, s), Obj::GenArray(j, t)) => i == j && s == t,
            (Obj::Param(i, p), Obj::Param(j, q)) => i == j && p == q,
            (Obj::Callback(i, g), Obj::Callback(j, k)) => i == j && g == k,
            _ => false,
        };
        same as i32
    }
}

/// The force of an application, in `Pcont::force`, in place of a process.
pub(super) const VPI_FORCE: u32 = u32::MAX - 1;

/// `a.b[3].c` as `a`, `b[3]`, `c`; an escaped name runs to white space.
fn split_name(name: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut depth = 0;
    let mut escaped = false;
    for c in name.chars() {
        if escaped {
            if c.is_whitespace() {
                escaped = false;
            } else {
                cur.push(c);
            }
            continue;
        }
        match c {
            '\\' if cur.is_empty() => escaped = true,
            '[' => {
                depth += 1;
                cur.push(c);
            }
            ']' => {
                depth -= 1;
                cur.push(c);
            }
            '.' if depth == 0 => out.push(std::mem::take(&mut cur)),
            _ => cur.push(c),
        }
    }
    out.push(cur);
    out
}

fn type_name(t: i32) -> &'static str {
    match t {
        vpiConstant => "vpiConstant",
        vpiIntegerVar => "vpiIntegerVar",
        vpiIterator => "vpiIterator",
        vpiMemory => "vpiMemory",
        vpiMemoryWord => "vpiMemoryWord",
        vpiModule => "vpiModule",
        vpiNamedEvent => "vpiNamedEvent",
        vpiNet => "vpiNet",
        vpiNetBit => "vpiNetBit",
        vpiParameter => "vpiParameter",
        vpiPort => "vpiPort",
        vpiRealVar => "vpiRealVar",
        vpiReg => "vpiReg",
        vpiRegBit => "vpiRegBit",
        vpiSysFuncCall => "vpiSysFuncCall",
        vpiSysTaskCall => "vpiSysTaskCall",
        vpiTimeVar => "vpiTimeVar",
        vpiUserSystf => "vpiUserSystf",
        vpiCallback => "vpiCallback",
        vpiNetArray => "vpiNetArray",
        vpiRange => "vpiRange",
        vpiRegArray => "vpiRegArray",
        vpiGenScopeArray => "vpiGenScopeArray",
        vpiGenScope => "vpiGenScope",
        vpiLongIntVar => "vpiLongIntVar",
        vpiShortIntVar => "vpiShortIntVar",
        vpiIntVar => "vpiIntVar",
        vpiShortRealVar => "vpiShortRealVar",
        vpiByteVar => "vpiByteVar",
        vpiStringVar => "vpiStringVar",
        vpiEnumVar => "vpiEnumVar",
        vpiStructVar => "vpiStructVar",
        vpiBitVar => "vpiBitVar",
        vpiPackedArrayVar => "vpiPackedArrayVar",
        _ => "vpiUndefined",
    }
}

/// The C functions, which `export_vpi!` exports under their names.
///
/// # Safety
///
/// Each is called by C code with the arguments the standard gives it:
/// pointers either null or to valid structures.
#[allow(clippy::missing_safety_doc)]
pub mod c {
    use super::*;

    pub unsafe extern "C" fn vpi_register_cb(data: *mut CbData) -> Handle {
        if data.is_null() {
            return null_mut();
        }
        traced(|| format!("vpi_register_cb({data:?})"), with(null_mut(), |s| s.api_register_cb(unsafe { &*data })))
    }

    pub unsafe extern "C" fn vpi_remove_cb(h: Handle) -> i32 {
        traced(|| format!("vpi_remove_cb({h:?})"), with(0, |s| s.api_remove_cb(h)))
    }

    pub unsafe extern "C" fn vpi_register_systf(data: *mut SystfData) -> Handle {
        if data.is_null() {
            return null_mut();
        }
        traced(|| format!("vpi_register_systf({data:?})"), with(null_mut(), |s| s.api_register_systf(unsafe { &*data })))
    }

    pub unsafe extern "C" fn vpi_handle(ty: i32, h: Handle) -> Handle {
        traced(|| format!("vpi_handle({ty:?}, {h:?})"), with(null_mut(), |s| s.api_handle(ty, h)))
    }

    pub unsafe extern "C" fn vpi_handle_by_name(name: *mut c_char, scope: Handle) -> Handle {
        if name.is_null() {
            return null_mut();
        }
        let name = unsafe { CStr::from_ptr(name) }.to_string_lossy().into_owned();
        traced(|| format!("vpi_handle_by_name({name:?}, {scope:?})"), with(null_mut(), |s| s.api_handle_by_name(&name, scope)))
    }

    pub unsafe extern "C" fn vpi_handle_by_index(h: Handle, i: i32) -> Handle {
        traced(|| format!("vpi_handle_by_index({h:?}, {i:?})"), with(null_mut(), |s| s.api_handle_by_index(h, i)))
    }

    pub unsafe extern "C" fn vpi_iterate(ty: i32, h: Handle) -> Handle {
        traced(|| format!("vpi_iterate({ty:?}, {h:?})"), with(null_mut(), |s| s.api_iterate(ty, h)))
    }

    pub unsafe extern "C" fn vpi_scan(h: Handle) -> Handle {
        traced(|| format!("vpi_scan({h:?})"), with(null_mut(), |s| s.api_scan(h)))
    }

    pub unsafe extern "C" fn vpi_get(prop: i32, h: Handle) -> i32 {
        traced(|| format!("vpi_get({prop:?}, {h:?})"), with(vpiUndefined, |s| s.api_get(prop, h) as i32))
    }

    pub unsafe extern "C" fn vpi_get64(prop: i32, h: Handle) -> i64 {
        traced(|| format!("vpi_get64({prop:?}, {h:?})"), with(vpiUndefined as i64, |s| s.api_get(prop, h)))
    }

    pub unsafe extern "C" fn vpi_get_str(prop: i32, h: Handle) -> *mut c_char {
        let p = with(null_mut(), |s| s.api_get_str(prop, h));
        if tracing() {
            let text = if p.is_null() { None } else { Some(unsafe { CStr::from_ptr(p) }.to_string_lossy()) };
            eprintln!("vpi: vpi_get_str({prop}, {h:?}) -> {text:?}");
        }
        p
    }

    pub unsafe extern "C" fn vpi_get_value(h: Handle, v: *mut VpiValue) {
        if v.is_null() {
            return;
        }
        with((), |s| s.api_get_value(h, unsafe { &mut *v }));
        if tracing() {
            eprintln!("vpi: vpi_get_value({h:?}, {v:?}) -> format {}", unsafe { (*v).format });
        }
    }

    pub unsafe extern "C" fn vpi_put_value(h: Handle, v: *mut VpiValue, t: *mut VpiTime, flags: i32) -> Handle {
        traced(|| format!("vpi_put_value({h:?}, {v:?}, {t:?}, {flags:?})"), with(null_mut(), |s| s.api_put_value(h, unsafe { v.as_mut() }, t, flags)))
    }

    pub unsafe extern "C" fn vpi_get_time(h: Handle, t: *mut VpiTime) {
        if t.is_null() {
            return;
        }
        with((), |s| s.api_get_time(h, unsafe { &mut *t }));
        if tracing() {
            let t = unsafe { &*t };
            eprintln!("vpi: vpi_get_time({h:?}) -> {}", (t.high as u64) << 32 | t.low as u64);
        }
    }

    /// Variadic in C; the arguments after the operation are not read.
    pub unsafe extern "C" fn vpi_control(op: i32) -> i32 {
        traced(|| format!("vpi_control({op:?})"), with(0, |s| s.api_control(op)))
    }

    pub unsafe extern "C" fn vpi_free_object(h: Handle) -> i32 {
        traced(|| format!("vpi_free_object({h:?})"), with(0, |s| s.api_free(h)))
    }

    pub unsafe extern "C" fn vpi_release_handle(h: Handle) -> i32 {
        traced(|| format!("vpi_release_handle({h:?})"), with(0, |s| s.api_free(h)))
    }

    pub unsafe extern "C" fn vpi_chk_error(info: *mut ErrorInfo) -> i32 {
        traced(|| format!("vpi_chk_error({info:?})"), with_error(0, |s| s.api_chk_error(info)))
    }

    pub unsafe extern "C" fn vpi_get_vlog_info(info: *mut VlogInfo) -> i32 {
        if info.is_null() {
            return 0;
        }
        traced(|| format!("vpi_get_vlog_info({info:?})"), with(0, |s| s.api_vlog_info(unsafe { &mut *info })))
    }

    pub unsafe extern "C" fn vpi_compare_objects(a: Handle, b: Handle) -> i32 {
        traced(|| format!("vpi_compare_objects({a:?}, {b:?})"), with(0, |s| s.api_compare(a, b)))
    }

    pub unsafe extern "C" fn vpi_flush() -> i32 {
        0
    }
}

/// Defines the VPI's C functions in the crate that calls it, under the
/// names the standard gives them, so that an executable that loads VPI
/// libraries has them for the libraries to find.
#[macro_export]
macro_rules! export_vpi {
    () => {
        $crate::export_vpi!(@fns
            vpi_register_cb(data: *mut $crate::sim::vpi::CbData) -> $crate::sim::vpi::Handle;
            vpi_remove_cb(h: $crate::sim::vpi::Handle) -> i32;
            vpi_register_systf(data: *mut $crate::sim::vpi::SystfData) -> $crate::sim::vpi::Handle;
            vpi_handle(ty: i32, h: $crate::sim::vpi::Handle) -> $crate::sim::vpi::Handle;
            vpi_handle_by_name(name: *mut ::std::ffi::c_char, scope: $crate::sim::vpi::Handle) -> $crate::sim::vpi::Handle;
            vpi_handle_by_index(h: $crate::sim::vpi::Handle, i: i32) -> $crate::sim::vpi::Handle;
            vpi_iterate(ty: i32, h: $crate::sim::vpi::Handle) -> $crate::sim::vpi::Handle;
            vpi_scan(h: $crate::sim::vpi::Handle) -> $crate::sim::vpi::Handle;
            vpi_get(prop: i32, h: $crate::sim::vpi::Handle) -> i32;
            vpi_get64(prop: i32, h: $crate::sim::vpi::Handle) -> i64;
            vpi_get_str(prop: i32, h: $crate::sim::vpi::Handle) -> *mut ::std::ffi::c_char;
            vpi_get_value(h: $crate::sim::vpi::Handle, v: *mut $crate::sim::vpi::VpiValue) -> ();
            vpi_put_value(h: $crate::sim::vpi::Handle, v: *mut $crate::sim::vpi::VpiValue, t: *mut $crate::sim::vpi::VpiTime, flags: i32) -> $crate::sim::vpi::Handle;
            vpi_get_time(h: $crate::sim::vpi::Handle, t: *mut $crate::sim::vpi::VpiTime) -> ();
            vpi_control(op: i32) -> i32;
            vpi_free_object(h: $crate::sim::vpi::Handle) -> i32;
            vpi_release_handle(h: $crate::sim::vpi::Handle) -> i32;
            vpi_chk_error(info: *mut $crate::sim::vpi::ErrorInfo) -> i32;
            vpi_get_vlog_info(info: *mut $crate::sim::vpi::VlogInfo) -> i32;
            vpi_compare_objects(a: $crate::sim::vpi::Handle, b: $crate::sim::vpi::Handle) -> i32;
            vpi_flush() -> i32;
        );
    };
    (@fns $($name:ident($($arg:ident: $ty:ty),*) -> $ret:ty;)*) => {
        $(
            /// # Safety
            /// Called by C code as the standard says.
            #[unsafe(no_mangle)]
            pub unsafe extern "C" fn $name($($arg: $ty),*) -> $ret {
                unsafe { $crate::sim::vpi::c::$name($($arg),*) }
            }
        )*
        /// Keeps the functions in the executable, which nothing in it calls.
        struct VpiExports(&'static [*const ()]);
        unsafe impl Sync for VpiExports {}
        #[used]
        static VPI_EXPORTS: VpiExports = VpiExports(&[$($name as *const ()),*]);
    };
}
