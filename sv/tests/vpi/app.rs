//! A VPI application, written from IEEE 1800-2023 clauses 36–38, that
//! the `vpi` test builds as a shared library and loads into a simulation
//! of `design.sv`. It walks the design, reads and writes values, drives
//! the clock, uses each kind of callback, forces and releases a net, and
//! registers a system task and function that the design calls. Everything
//! it sees goes to standard output, for the test to compare.

#![allow(non_upper_case_globals, clippy::missing_safety_doc)]

use std::ffi::{CStr, CString, c_char, c_void};
use std::ptr::null_mut;

type Handle = *mut c_void;

#[repr(C)]
#[derive(Clone, Copy)]
struct Time {
    ty: i32,
    high: u32,
    low: u32,
    real: f64,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct VecVal {
    aval: i32,
    bval: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
union ValueUnion {
    str_: *mut c_char,
    scalar: i32,
    integer: i32,
    real: f64,
    time: *mut Time,
    vector: *mut VecVal,
}

#[repr(C)]
struct Value {
    format: i32,
    value: ValueUnion,
}

#[repr(C)]
struct CbData {
    reason: i32,
    cb_rtn: Option<unsafe extern "C" fn(*mut CbData) -> i32>,
    obj: Handle,
    time: *mut Time,
    value: *mut Value,
    index: i32,
    user_data: *mut c_char,
}

#[repr(C)]
struct SystfData {
    ty: i32,
    sysfunctype: i32,
    tfname: *const c_char,
    calltf: Option<unsafe extern "C" fn(*mut c_char) -> i32>,
    compiletf: Option<unsafe extern "C" fn(*mut c_char) -> i32>,
    sizetf: Option<unsafe extern "C" fn(*mut c_char) -> i32>,
    user_data: *mut c_char,
}

#[repr(C)]
struct VlogInfo {
    argc: i32,
    argv: *mut *mut c_char,
    product: *mut c_char,
    version: *mut c_char,
}

#[repr(C)]
struct ErrorInfo {
    state: i32,
    level: i32,
    message: *mut c_char,
    product: *mut c_char,
    code: *mut c_char,
    file: *mut c_char,
    line: i32,
}

unsafe extern "C" {
    fn vpi_register_cb(data: *mut CbData) -> Handle;
    fn vpi_remove_cb(h: Handle) -> i32;
    fn vpi_register_systf(data: *mut SystfData) -> Handle;
    fn vpi_handle(ty: i32, h: Handle) -> Handle;
    fn vpi_handle_by_name(name: *const c_char, scope: Handle) -> Handle;
    fn vpi_handle_by_index(h: Handle, i: i32) -> Handle;
    fn vpi_iterate(ty: i32, h: Handle) -> Handle;
    fn vpi_scan(h: Handle) -> Handle;
    fn vpi_get(prop: i32, h: Handle) -> i32;
    fn vpi_get_str(prop: i32, h: Handle) -> *mut c_char;
    fn vpi_get_value(h: Handle, v: *mut Value);
    fn vpi_put_value(h: Handle, v: *mut Value, t: *mut Time, flags: i32) -> Handle;
    fn vpi_get_time(h: Handle, t: *mut Time);
    fn vpi_get_vlog_info(info: *mut VlogInfo) -> i32;
    fn vpi_control(op: i32, ...) -> i32;
    fn vpi_free_object(h: Handle) -> i32;
    fn vpi_chk_error(info: *mut ErrorInfo) -> i32;
}

// Object types, relations and properties.
const vpiModule: i32 = 32;
const vpiNet: i32 = 36;
const vpiReg: i32 = 48;
const vpiParameter: i32 = 41;
const vpiRegArray: i32 = 116;
const vpiIntVar: i32 = 612;
const vpiRealVar: i32 = 47;
const vpiStringVar: i32 = 616;
const vpiBitVar: i32 = 620;
const vpiInternalScope: i32 = 92;
const vpiArgument: i32 = 89;
const vpiSysTfCall: i32 = 85;
const vpiLeftRange: i32 = 79;
const vpiRightRange: i32 = 83;
const vpiScope: i32 = 84;
const vpiType: i32 = 1;
const vpiName: i32 = 2;
const vpiFullName: i32 = 3;
const vpiSize: i32 = 4;
const vpiDefName: i32 = 9;
const vpiTimeUnit: i32 = 11;
const vpiTimePrecision: i32 = 12;
const vpiVector: i32 = 18;
// Values and delays.
const vpiBinStrVal: i32 = 1;
const vpiHexStrVal: i32 = 4;
const vpiDecStrVal: i32 = 3;
const vpiScalarVal: i32 = 5;
const vpiIntVal: i32 = 6;
const vpiRealVal: i32 = 7;
const vpiStringVal: i32 = 8;
const vpiVectorVal: i32 = 9;
const vpiNoDelay: i32 = 1;
const vpiInertialDelay: i32 = 2;
const vpiForceFlag: i32 = 5;
const vpiReleaseFlag: i32 = 6;
const vpiScaledRealTime: i32 = 1;
const vpiSimTime: i32 = 2;
// Callbacks.
const cbValueChange: i32 = 1;
const cbReadWriteSynch: i32 = 6;
const cbReadOnlySynch: i32 = 7;
const cbNextSimTime: i32 = 8;
const cbAfterDelay: i32 = 9;
const cbStartOfSimulation: i32 = 11;
const cbEndOfSimulation: i32 = 12;
const vpiFinish: i32 = 67;
const vpiSysTask: i32 = 1;
const vpiSysFunc: i32 = 2;
const vpiIntFunc: i32 = 1;

fn cstr(p: *const c_char) -> String {
    if p.is_null() { "(null)".into() } else { unsafe { CStr::from_ptr(p) }.to_string_lossy().into_owned() }
}

fn by_name(name: &str) -> Handle {
    let c = CString::new(name).unwrap();
    unsafe { vpi_handle_by_name(c.as_ptr(), null_mut()) }
}

fn str_of(h: Handle, format: i32) -> String {
    if h.is_null() {
        return "(no handle)".into();
    }
    let mut v = Value { format, value: ValueUnion { integer: 0 } };
    unsafe {
        vpi_get_value(h, &mut v);
        cstr(v.value.str_)
    }
}

fn int_of(h: Handle) -> i32 {
    if h.is_null() {
        return -999;
    }
    let mut v = Value { format: vpiIntVal, value: ValueUnion { integer: 0 } };
    unsafe {
        vpi_get_value(h, &mut v);
        v.value.integer
    }
}

fn put(h: Handle, mut v: Value, delay: Option<f64>, flags: i32) -> Value {
    let mut t = Time { ty: vpiScaledRealTime, high: 0, low: 0, real: delay.unwrap_or(0.0) };
    unsafe { vpi_put_value(h, &mut v, if delay.is_some() { &mut t } else { null_mut() }, flags) };
    v
}

fn int(i: i32) -> Value {
    Value { format: vpiIntVal, value: ValueUnion { integer: i } }
}

fn now() -> String {
    let mut t = Time { ty: vpiSimTime, high: 0, low: 0, real: 0.0 };
    unsafe { vpi_get_time(null_mut(), &mut t) };
    let ticks = (t.high as u64) << 32 | t.low as u64;
    format!("{}ps", ticks)
}

fn name(h: Handle) -> String {
    unsafe { cstr(vpi_get_str(vpiName, h)) }
}

/// Registers a callback for `reason`, `delay` nanoseconds on (in the
/// time unit of `top`) when the reason takes a time.
fn callback(reason: i32, obj: Handle, delay: Option<f64>, format: i32, f: unsafe extern "C" fn(*mut CbData) -> i32, tag: usize) -> Handle {
    let mut t = Time { ty: if delay.is_some() { vpiScaledRealTime } else { vpiSimTime }, high: 0, low: 0, real: delay.unwrap_or(0.0) };
    let mut v = Value { format, value: ValueUnion { integer: 0 } };
    let mut d = CbData { reason, cb_rtn: Some(f), obj, time: &mut t, value: &mut v, index: 0, user_data: tag as *mut c_char };
    unsafe { vpi_register_cb(&mut d) }
}

fn top() -> Handle {
    by_name("top")
}

unsafe extern "C" fn start(_: *mut CbData) -> i32 {
    unsafe {
        let mut info = VlogInfo { argc: 0, argv: null_mut(), product: null_mut(), version: null_mut() };
        vpi_get_vlog_info(&mut info);
        let args: Vec<String> = (0..info.argc).map(|i| cstr(*info.argv.add(i as usize))).collect();
        println!("app: start; +app_flag given: {}", args.iter().any(|a| a == "+app_flag"));
        println!("app: precision {}", vpi_get(vpiTimePrecision, null_mut()));

        // The top modules.
        let it = vpi_iterate(vpiModule, null_mut());
        while let Some(m) = Some(vpi_scan(it)).filter(|h| !h.is_null()) {
            println!("app: top {} ({}), unit {}", name(m), cstr(vpi_get_str(vpiDefName, m)), vpi_get(vpiTimeUnit, m));
        }
        // What is in `top`, by kind, sorted.
        let top = top();
        for (kind, what) in [
            (vpiNet, "net"),
            (vpiReg, "reg"),
            (vpiIntVar, "int"),
            (vpiRealVar, "real"),
            (vpiStringVar, "string"),
            (vpiBitVar, "bit"),
            (vpiRegArray, "array"),
            (vpiParameter, "parameter"),
            (vpiModule, "module"),
            (vpiInternalScope, "scope"),
        ] {
            let it = vpi_iterate(kind, top);
            let mut names = Vec::new();
            if !it.is_null() {
                loop {
                    let h = vpi_scan(it);
                    if h.is_null() {
                        break;
                    }
                    names.push(format!("{}:{}", name(h), vpi_get(vpiSize, h)));
                    vpi_free_object(h);
                }
            }
            names.sort();
            println!("app: {what}: {}", names.join(" "));
        }

        // Parameters in several formats.
        let w = by_name("top.W");
        let mut r = Value { format: vpiRealVal, value: ValueUnion { real: 0.0 } };
        vpi_get_value(by_name("top.R"), &mut r);
        println!("app: W={} NAME={} R={} u.N={}", int_of(w), str_of(by_name("top.NAME"), vpiStringVal), r.value.real, int_of(by_name("top.u.N")));

        // Names, scopes, ranges, bits and elements.
        let v1 = by_name("top.g[1].v");
        println!("app: {} in {} ({}), type {}", cstr(vpi_get_str(vpiFullName, v1)), cstr(vpi_get_str(vpiFullName, vpi_handle(vpiScope, v1))), vpi_get(vpiType, vpi_handle(vpiScope, v1)), vpi_get(vpiType, v1));
        let q = vpi_handle_by_name(c"q".as_ptr(), by_name("top.u"));
        println!("app: {} is a {} output of {} bits, in {}", cstr(vpi_get_str(vpiFullName, q)), vpi_get(vpiType, q), vpi_get(vpiSize, q), name(vpi_handle(vpiModule, q)));
        let nib = by_name("top.nib");
        println!("app: nib={} [{}:{}] vector={}", str_of(nib, vpiBinStrVal), int_of(vpi_handle(vpiLeftRange, nib)), int_of(vpi_handle(vpiRightRange, nib)), vpi_get(vpiVector, nib));
        let mut sv = Value { format: vpiScalarVal, value: ValueUnion { scalar: -1 } };
        vpi_get_value(vpi_handle_by_index(nib, 1), &mut sv);
        println!("app: nib[1] scalar={} nib[3]={}", sv.value.scalar, str_of(vpi_handle_by_index(nib, 3), vpiBinStrVal));
        let pairs = by_name("top.pairs");
        println!("app: pairs[2]={} pairs[2][1]={}", str_of(vpi_handle_by_index(pairs, 2), vpiBinStrVal), str_of(by_name("top.pairs[2][1]"), vpiBinStrVal));
        let mem = by_name("top.mem");
        println!("app: mem has {} words, [{}:{}], type {}", vpi_get(vpiSize, mem), int_of(vpi_handle(vpiLeftRange, mem)), int_of(vpi_handle(vpiRightRange, mem)), vpi_get(vpiType, mem));
        println!("app: label={} ratio={}", str_of(by_name("top.label"), vpiStringVal), str_of(by_name("top.ratio"), vpiDecStrVal));

        // A value change on `top.u.q` (as hex, with the time in its unit),
        // and on `top.edges`; one on `top.label` that is removed at once.
        callback(cbValueChange, q, Some(0.0), vpiHexStrVal, on_change, 1);
        callback(cbValueChange, by_name("top.edges"), None, vpiIntVal, on_change, 2);
        let label = by_name("top.label");
        if !label.is_null() {
            let gone = callback(cbValueChange, label, None, vpiStringVal, on_change, 3);
            println!("app: removed: {}", vpi_remove_cb(gone));
        }

        // The clock, and the steps of the test.
        callback(cbAfterDelay, top, Some(5.0), 0, clock, 0);
        for (at, step) in [(7.0, 1), (12.0, 2), (18.0, 3), (30.0, 4), (40.0, 5)] {
            callback(cbAfterDelay, top, Some(at), 0, step_cb, step);
        }
        0
    }
}

unsafe extern "C" fn clock(_: *mut CbData) -> i32 {
    unsafe {
        let clk = by_name("top.clk");
        let v = int_of(clk);
        put(clk, int(1 - v), None, vpiNoDelay);
        callback(cbAfterDelay, top(), Some(5.0), 0, clock, 0);
    }
    0
}

unsafe extern "C" fn on_change(d: *mut CbData) -> i32 {
    unsafe {
        let d = &*d;
        let v = &*d.value;
        let t = &*d.time;
        match d.user_data as usize {
            1 => println!("app: {} q={} at {}ns", now(), cstr(v.value.str_), t.real),
            2 => println!("app: {} edges={}", now(), v.value.integer),
            _ => println!("app: {} a removed callback ran", now()),
        }
    }
    0
}

unsafe extern "C" fn step_cb(d: *mut CbData) -> i32 {
    unsafe {
        let data = by_name("top.data");
        let inv = by_name("top.inv");
        match (*d).user_data as usize {
            1 => {
                // An inertial write with no delay: it lands in this time
                // slot's NBA region, so reads see it from read-write sync.
                put(data, int(0x41), Some(0.0), vpiInertialDelay);
                println!("app: {} wrote data; it reads {}", now(), str_of(data, vpiHexStrVal));
                callback(cbReadWriteSynch, null_mut(), None, 0, sync, 1);
                callback(cbReadOnlySynch, null_mut(), None, 0, sync, 2);
            }
            2 => {
                // A vector write of a memory word, and a force.
                let w = vpi_handle_by_index(by_name("top.mem"), 1);
                let mut words = [VecVal { aval: 0x99, bval: 0 }];
                put(w, Value { format: vpiVectorVal, value: ValueUnion { vector: words.as_mut_ptr() } }, None, vpiNoDelay);
                put(inv, Value { format: vpiHexStrVal, value: ValueUnion { str_: c"55".as_ptr() as *mut c_char } }, None, vpiForceFlag);
                put(data, int(0x43), None, vpiNoDelay);
                println!("app: {} mem[1]={} forced inv={}", now(), str_of(w, vpiHexStrVal), str_of(inv, vpiHexStrVal));
                callback(cbNextSimTime, null_mut(), None, 0, sync, 3);
                // A string variable written in place; its removed callback
                // must not run.
                let label = by_name("top.label");
                if !label.is_null() {
                    put(label, Value { format: vpiStringVal, value: ValueUnion { str_: c"bye".as_ptr() as *mut c_char } }, None, vpiNoDelay);
                }
            }
            3 => {
                let mut v = Value { format: vpiHexStrVal, value: ValueUnion { integer: 0 } };
                vpi_put_value(inv, &mut v, null_mut(), vpiReleaseFlag);
                println!("app: {} released inv, now {}", now(), cstr(v.value.str_));
                // A write two units from now.
                put(data, int(0x42), Some(2.0), vpiInertialDelay);
            }
            4 => {
                let mut e = ErrorInfo { state: 0, level: 0, message: null_mut(), product: null_mut(), code: null_mut(), file: null_mut(), line: 0 };
                println!("app: {} label is {}, ratio {}", now(), str_of(by_name("top.label"), vpiStringVal), {
                    let mut r = Value { format: vpiRealVal, value: ValueUnion { real: 0.0 } };
                    vpi_get_value(by_name("top.ratio"), &mut r);
                    r.value.real
                });
                let none = vpi_handle(vpiParameter, null_mut());
                let level = vpi_chk_error(&mut e);
                println!("app: no handle: {}, error level after a good call {level}", none.is_null());
                if std::env::var_os("APP_ERRORS").is_some() {
                    // A freed handle is an error, not a crash.
                    let h = by_name("top.data");
                    vpi_free_object(h);
                    let mut v = Value { format: vpiIntVal, value: ValueUnion { integer: 0 } };
                    vpi_get_value(h, &mut v);
                    let level = vpi_chk_error(&mut e);
                    println!("app: a freed handle: error level {level}: {}", cstr(e.message));
                }
            }
            _ => {
                println!("app: {} finishing", now());
                vpi_control(vpiFinish, 0);
            }
        }
    }
    0
}

unsafe extern "C" fn sync(d: *mut CbData) -> i32 {
    unsafe {
        let what = match (*d).user_data as usize {
            1 => "read-write",
            2 => "read-only",
            _ => "next time",
        };
        println!("app: {} {what}: data={} inv={}", now(), str_of(by_name("top.data"), vpiHexStrVal), str_of(by_name("top.inv"), vpiHexStrVal));
    }
    0
}

unsafe extern "C" fn end(_: *mut CbData) -> i32 {
    println!("app: {} end", now());
    0
}

/// `$app_show(...)`: prints its arguments.
unsafe extern "C" fn show(_: *mut c_char) -> i32 {
    unsafe {
        let call = vpi_handle(vpiSysTfCall, null_mut());
        let it = vpi_iterate(vpiArgument, call);
        let mut out = Vec::new();
        loop {
            let a = vpi_scan(it);
            if a.is_null() {
                break;
            }
            out.push(str_of(a, if out.is_empty() { vpiStringVal } else { vpiDecStrVal }));
        }
        println!("app: {} {} {}", now(), name(call), out.join(" "));
    }
    0
}

/// `$app_twice(n)`: 2n.
unsafe extern "C" fn twice(_: *mut c_char) -> i32 {
    unsafe {
        let call = vpi_handle(vpiSysTfCall, null_mut());
        let it = vpi_iterate(vpiArgument, call);
        let n = int_of(vpi_scan(it));
        vpi_free_object(it);
        put(call, int(2 * n), None, vpiNoDelay);
    }
    0
}

unsafe extern "C" fn size32(_: *mut c_char) -> i32 {
    32
}

extern "C" fn register() {
    unsafe {
        let mut d = CbData { reason: cbStartOfSimulation, cb_rtn: Some(start), obj: null_mut(), time: null_mut(), value: null_mut(), index: 0, user_data: null_mut() };
        vpi_register_cb(&mut d);
        d.reason = cbEndOfSimulation;
        d.cb_rtn = Some(end);
        vpi_register_cb(&mut d);
        let mut tf = SystfData { ty: vpiSysTask, sysfunctype: 0, tfname: c"$app_show".as_ptr(), calltf: Some(show), compiletf: None, sizetf: None, user_data: null_mut() };
        vpi_register_systf(&mut tf);
        tf = SystfData { ty: vpiSysFunc, sysfunctype: vpiIntFunc, tfname: c"$app_twice".as_ptr(), calltf: Some(twice), compiletf: None, sizetf: Some(size32), user_data: null_mut() };
        vpi_register_systf(&mut tf);
    }
}

#[unsafe(no_mangle)]
pub static vlog_startup_routines: [Option<extern "C" fn()>; 2] = [Some(register), None];
