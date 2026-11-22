// The C side of structs.rs: functions that take and give structs of
// different shapes, and that call back into Rust's `extern "C" fn`s.
#include <stdint.h>

typedef struct { int32_t a, b; } I2;
typedef struct { int64_t a, b; } L2;
typedef struct { double x, y; } D2;
typedef struct { float x, y, z; } F3;
typedef struct { int64_t a; double b; } LD;
typedef struct { double b; int64_t a; } DL;
typedef struct { int8_t c; int16_t s; int32_t i; int64_t l; } Mix;
typedef struct { int64_t a, b, c; } L3;
typedef struct { double a, b, c, d; } D4;
typedef struct { float a; int32_t b; } FI;
typedef struct { uint8_t a[3]; } C3;
typedef struct { double d; float f; } DF;
typedef struct { float a, b, c, d, e; } F5;
typedef struct { uint8_t a; } U8;
typedef struct { float a, b; } F2;
typedef struct { int32_t a, b, c; } I3;

I2 c_i2(I2 v) { v.a += 1; v.b *= 2; return v; }
L2 c_l2(L2 v) { v.a += 1; v.b *= 2; return v; }
D2 c_d2(D2 v) { v.x += 1; v.y *= 2; return v; }
F3 c_f3(F3 v) { v.x += 1; v.y *= 2; v.z -= 3; return v; }
LD c_ld(LD v) { v.a += 1; v.b *= 2; return v; }
DL c_dl(DL v) { v.a += 1; v.b *= 2; return v; }
Mix c_mix(Mix v) { v.c += 1; v.s *= 2; v.i -= 3; v.l *= 5; return v; }
L3 c_l3(L3 v) { v.a += 1; v.b *= 2; v.c -= 3; return v; }
D4 c_d4(D4 v) { v.a += 1; v.b *= 2; v.c -= 3; v.d /= 4; return v; }
FI c_fi(FI v) { v.a += 1; v.b *= 2; return v; }
C3 c_c3(C3 v) { v.a[0] += 1; v.a[1] += 2; v.a[2] += 3; return v; }
DF c_df(DF v) { v.d += 1; v.f *= 2; return v; }
F5 c_f5(F5 v) { v.a += 1; v.e *= 2; return v; }
U8 c_u8(U8 v) { v.a += 1; return v; }
F2 c_f2(F2 v) { float t = v.a; v.a = v.b; v.b = t; return v; }
I3 c_i3(I3 v) { v.a += v.b + v.c; return v; }

// Several at once, and more than the registers hold: the last ones go on
// the stack.
int64_t c_many(L2 a, D2 b, LD c, L2 d, L2 e, D2 f, D2 g, D2 h, L2 i, F3 j, int64_t k, double l) {
    return a.a + a.b + (int64_t)b.x + (int64_t)b.y + c.a + (int64_t)c.b + d.a + d.b + e.a + e.b
        + (int64_t)f.x + (int64_t)g.y + (int64_t)h.x + i.a + i.b + (int64_t)j.z + k + (int64_t)l;
}
double c_floats(double a, double b, double c, double d, double e, double f, double g, D2 h, double i, F3 j) {
    return a + b + c + d + e + f + g + h.x + h.y + i + j.x + j.y + j.z;
}

// Calls back into Rust: through a function pointer passed in, and through
// functions Rust defines with `#[no_mangle]`.
typedef L3 (*L3Fn)(L3, D2, int32_t);
L3 c_call_back(L3Fn f, L3 v) { return f(v, (D2){ 1.5, 2.5 }, 7); }

extern Mix rust_mix(Mix v, F3 w);
extern D4 rust_d4(D4 v, LD w, I2 x);
extern C3 rust_c3(C3 v, U8 w);
extern DF rust_df(DF v, F5 w);
int64_t c_calls_rust(void) {
    Mix m = rust_mix((Mix){ 1, 2, 3, 4 }, (F3){ 5, 6, 7 });
    D4 d = rust_d4((D4){ 1, 2, 3, 4 }, (LD){ 5, 6.5 }, (I2){ 7, 8 });
    C3 c = rust_c3((C3){ { 9, 10, 11 } }, (U8){ 12 });
    DF f = rust_df((DF){ 13.5, 14.5f }, (F5){ 1, 2, 3, 4, 5 });
    return m.c + m.s + m.i + m.l + (int64_t)(d.a + d.b + d.c + d.d) + c.a[0] + c.a[1] + c.a[2]
        + (int64_t)(f.d * 10) + (int64_t)(f.f * 10);
}
