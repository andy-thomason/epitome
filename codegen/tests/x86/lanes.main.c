// Calls the functions in lanes.c and prints their results.
#include <stdio.h>
typedef unsigned char u8 __attribute__((vector_size(16)));
typedef unsigned char u8x32 __attribute__((vector_size(32)));
typedef unsigned short u16 __attribute__((vector_size(16)));
typedef unsigned u32 __attribute__((vector_size(16)));
typedef unsigned u32x8 __attribute__((vector_size(32)));
int mask16(u8, u8);
int any16(u8, u8);
int mask32(const u8x32 *, const u8x32 *);
int any32(const u8x32 *, const u8x32 *);
u32 rot32(u32);
u16 swap16(u16);
u16 lohi16(u16);
u8 rotb(u8);
void rotb32(u8x32 *, const u8x32 *);
void rot32x8(u32x8 *, const u32x8 *);
u8 shl4(u8);
u8 shr3(u8);
u8 window5(u8, u8);
int tz(unsigned);
int lz(unsigned long);
int tz0(unsigned);
int lz0(unsigned);

static void bytes(const char *name, const void *p, int n) {
    printf("%s", name);
    for (int k = 0; k < n; k++) printf(" %02x", ((const unsigned char *)p)[k]);
    printf("\n");
}

int main(void) {
    u8 a, b, c;
    u8x32 x, y, o;
    for (int k = 0; k < 16; k++) {
        a[k] = (unsigned char)(k * 37 + 11);
        b[k] = k % 3 ? a[k] : (unsigned char)(a[k] + 1);
        c[k] = (unsigned char)(200 - k * 7);
    }
    for (int k = 0; k < 32; k++) {
        x[k] = (unsigned char)(k * 13 + 5);
        y[k] = k % 5 ? x[k] : 0;
    }
    printf("mask16 %x any16 %d %d\n", mask16(a, b), any16(a, b), any16(a, c));
    printf("mask32 %x any32 %d\n", mask32(&x, &y), any32(&x, &y));
    u32 r = rot32((u32)a);
    bytes("rot32", &r, 16);
    u16 s = swap16((u16)a);
    bytes("swap16", &s, 16);
    s = lohi16((u16)c);
    bytes("lohi16", &s, 16);
    u8 t = rotb(a);
    bytes("rotb", &t, 16);
    rotb32(&o, &x);
    bytes("rotb32", &o, 32);
    u32x8 w;
    rot32x8(&w, (const u32x8 *)&y);
    bytes("rot32x8", &w, 32);
    t = shl4(a);
    bytes("shl4", &t, 16);
    t = shr3(c);
    bytes("shr3", &t, 16);
    t = window5(a, c);
    bytes("window5", &t, 16);
    for (unsigned v = 1; v; v <<= 7) printf("tz %u %d tz0 %d lz0 %d\n", v, tz(v), tz0(v), lz0(v));
    printf("tz0 0 %d lz0 0 %d\n", tz0(0), lz0(0));
    for (unsigned long v = 3; v; v <<= 9) printf("lz %lu %d\n", v, lz(v));
    return 0;
}
