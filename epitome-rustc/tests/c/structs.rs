//! Structs of different shapes passed to and returned from C (`structs.c`,
//! linked as `libcstructs`), C calling back into `extern "C" fn`s, and
//! calls through `extern "C" fn` pointers.

#[repr(C)] #[derive(Clone, Copy, Debug)] struct I2 { a: i32, b: i32 }
#[repr(C)] #[derive(Clone, Copy, Debug)] struct L2 { a: i64, b: i64 }
#[repr(C)] #[derive(Clone, Copy, Debug)] struct D2 { x: f64, y: f64 }
#[repr(C)] #[derive(Clone, Copy, Debug)] struct F3 { x: f32, y: f32, z: f32 }
#[repr(C)] #[derive(Clone, Copy, Debug)] struct LD { a: i64, b: f64 }
#[repr(C)] #[derive(Clone, Copy, Debug)] struct DL { b: f64, a: i64 }
#[repr(C)] #[derive(Clone, Copy, Debug)] struct Mix { c: i8, s: i16, i: i32, l: i64 }
#[repr(C)] #[derive(Clone, Copy, Debug)] struct L3 { a: i64, b: i64, c: i64 }
#[repr(C)] #[derive(Clone, Copy, Debug)] struct D4 { a: f64, b: f64, c: f64, d: f64 }
#[repr(C)] #[derive(Clone, Copy, Debug)] struct FI { a: f32, b: i32 }
#[repr(C)] #[derive(Clone, Copy, Debug)] struct C3 { a: [u8; 3] }
#[repr(C)] #[derive(Clone, Copy, Debug)] struct DF { d: f64, f: f32 }
#[repr(C)] #[derive(Clone, Copy, Debug)] struct F5 { a: f32, b: f32, c: f32, d: f32, e: f32 }
#[repr(C)] #[derive(Clone, Copy, Debug)] struct U8 { a: u8 }
#[repr(C)] #[derive(Clone, Copy, Debug)] struct F2 { a: f32, b: f32 }
#[repr(C)] #[derive(Clone, Copy, Debug)] struct I3 { a: i32, b: i32, c: i32 }

#[link(name = "cstructs")]
unsafe extern "C" {
    fn c_i2(v: I2) -> I2;
    fn c_l2(v: L2) -> L2;
    fn c_d2(v: D2) -> D2;
    fn c_f3(v: F3) -> F3;
    fn c_ld(v: LD) -> LD;
    fn c_dl(v: DL) -> DL;
    fn c_mix(v: Mix) -> Mix;
    fn c_l3(v: L3) -> L3;
    fn c_d4(v: D4) -> D4;
    fn c_fi(v: FI) -> FI;
    fn c_c3(v: C3) -> C3;
    fn c_df(v: DF) -> DF;
    fn c_f5(v: F5) -> F5;
    fn c_u8(v: U8) -> U8;
    fn c_f2(v: F2) -> F2;
    fn c_i3(v: I3) -> I3;
    fn c_many(a: L2, b: D2, c: LD, d: L2, e: L2, f: D2, g: D2, h: D2, i: L2, j: F3, k: i64, l: f64) -> i64;
    fn c_floats(a: f64, b: f64, c: f64, d: f64, e: f64, f: f64, g: f64, h: D2, i: f64, j: F3) -> f64;
    fn c_call_back(f: extern "C" fn(L3, D2, i32) -> L3, v: L3) -> L3;
    fn c_calls_rust() -> i64;
}

extern "C" fn rust_l3(v: L3, w: D2, k: i32) -> L3 {
    L3 { a: v.a * 10, b: v.b + w.x as i64 + w.y as i64, c: v.c + k as i64 }
}

#[unsafe(no_mangle)]
extern "C" fn rust_mix(v: Mix, w: F3) -> Mix {
    Mix { c: v.c + 1, s: v.s * 2, i: v.i + w.x as i32, l: v.l + (w.y * w.z) as i64 }
}

#[unsafe(no_mangle)]
extern "C" fn rust_d4(v: D4, w: LD, x: I2) -> D4 {
    D4 { a: v.a + w.a as f64, b: v.b * w.b, c: v.c - x.a as f64, d: v.d + x.b as f64 }
}

#[unsafe(no_mangle)]
extern "C" fn rust_c3(v: C3, w: U8) -> C3 {
    C3 { a: [v.a[2] + w.a, v.a[1], v.a[0]] }
}

#[unsafe(no_mangle)]
extern "C" fn rust_df(v: DF, w: F5) -> DF {
    DF { d: v.d + w.a as f64 + w.e as f64, f: v.f * w.c }
}

/// Through a pointer, as C would call it.
fn via(f: extern "C" fn(Mix, F3) -> Mix) -> Mix {
    f(Mix { c: -1, s: -2, i: -3, l: -4 }, F3 { x: 0.5, y: 1.5, z: 2.5 })
}

fn main() {
    unsafe {
        println!("{:?}", c_i2(I2 { a: 1, b: -2 }));
        println!("{:?}", c_l2(L2 { a: 3, b: 1 << 40 }));
        println!("{:?}", c_d2(D2 { x: 5.5, y: 6.25 }));
        println!("{:?}", c_f3(F3 { x: 7.0, y: 8.5, z: 9.0 }));
        println!("{:?}", c_ld(LD { a: -10, b: 11.5 }));
        println!("{:?}", c_dl(DL { b: 12.5, a: 13 }));
        println!("{:?}", c_mix(Mix { c: 97, s: 14, i: 15, l: 16 }));
        println!("{:?}", c_l3(L3 { a: 17, b: 18, c: 19 }));
        println!("{:?}", c_d4(D4 { a: 20.0, b: 21.0, c: 22.0, d: 23.0 }));
        println!("{:?}", c_fi(FI { a: 24.5, b: 25 }));
        println!("{:?}", c_c3(C3 { a: [120, 121, 122] }));
        println!("{:?}", c_df(DF { d: 26.0, f: 27.5 }));
        println!("{:?}", c_f5(F5 { a: 28.0, b: 29.0, c: 30.0, d: 31.0, e: 32.0 }));
        println!("{:?}", c_u8(U8 { a: 33 }));
        println!("{:?}", c_f2(F2 { a: 34.0, b: 35.0 }));
        println!("{:?}", c_i3(I3 { a: 36, b: 37, c: 38 }));
        let l2 = |a, b| L2 { a, b };
        let d2 = |x, y| D2 { x, y };
        println!(
            "{}",
            c_many(l2(1, 2), d2(3.0, 4.0), LD { a: 5, b: 6.0 }, l2(7, 8), l2(9, 10), d2(11.0, 12.0), d2(13.0, 14.0), d2(15.0, 16.0), l2(17, 18), F3 { x: 19.0, y: 20.0, z: 21.0 }, 22, 23.0)
        );
        println!("{}", c_floats(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, d2(8.0, 9.0), 10.0, F3 { x: 11.0, y: 12.0, z: 13.0 }));
        println!("{:?}", c_call_back(rust_l3, L3 { a: 1, b: 2, c: 3 }));
        println!("{}", c_calls_rust());
    }
    println!("{:?}", via(rust_mix));
    let f: extern "C" fn(D4, LD, I2) -> D4 = rust_d4;
    println!("{:?}", f(D4 { a: 1.0, b: 2.0, c: 3.0, d: 4.0 }, LD { a: 5, b: 6.0 }, I2 { a: 7, b: 8 }));
}
