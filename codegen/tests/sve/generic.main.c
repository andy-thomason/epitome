#include <arm_sve.h>
#include <stdint.h>
#include <stdio.h>

svint32_t int_s(svint32_t, svint32_t);
svint64_t mul_d(svint64_t, svint64_t);
svuint16_t mul_h(svuint16_t, svuint16_t);
svint32_t select_s(svint32_t, svint32_t);
svint8_t select_b(svint8_t, svint8_t);
svint64_t splat_d(int64_t, svint64_t);
svint32_t splat_s(int32_t, svint32_t);
int32_t reduce_s(svint32_t);
int8_t reduce_b(svint8_t);
svfloat32_t float_s(svfloat32_t, svfloat32_t, svfloat32_t);
svfloat64_t float_d(svfloat64_t, svfloat64_t, svfloat64_t);
float fsum_s(float, svfloat32_t);
svint32_t convert_s(svint32_t, svfloat32_t);
int64_t lanes(void);

static void ps(svint32_t v) {
  int32_t r[64];
  svst1_s32(svptrue_b32(), r, v);
  for (int i = 0; i < (int)svcntw(); i++) printf("%d ", r[i]);
  printf("\n");
}

static void pf(svfloat32_t v) {
  float r[64];
  svst1_f32(svptrue_b32(), r, v);
  for (int i = 0; i < (int)svcntw(); i++) printf("%a ", r[i]);
  printf("\n");
}

int main(void) {
  int32_t a[64], b[64];
  int8_t c[256], d[256];
  uint16_t e[128], f[128];
  int64_t g[32], h[32];
  float x[64], y[64], z[64];
  double u[32], v[32], w[32];
  for (int i = 0; i < 64; i++) {
    a[i] = i * 7 - 100 + (i == 5);
    b[i] = 1000 - i * i + (i % 3 == 0 ? 0 : 1);
    if (b[i] == 0) b[i] = 3;
    if (i % 9 == 4) b[i] = a[i];
    x[i] = i * 0.75f - 11.0f;
    y[i] = 3.5f - i * 0.25f + (i == 14 ? 0.125f : 0.0f);
    z[i] = i * 1.5f - 20.0f;
  }
  for (int i = 0; i < 256; i++) c[i] = i * 3, d[i] = 100 - i;
  for (int i = 0; i < 128; i++) e[i] = i * 300, f[i] = 65000 - i * 77;
  for (int i = 0; i < 32; i++) {
    g[i] = (int64_t)i << 40 | i, h[i] = -i * 12345;
    u[i] = i * 1.25 - 7, v[i] = 2.0 - i * 0.5 + 0.1, w[i] = i * i * 0.01;
  }
  svbool_t p32 = svptrue_b32(), p64 = svptrue_b64();
  svint32_t sa = svld1_s32(p32, a), sb = svld1_s32(p32, b);
  svfloat32_t fx = svld1_f32(p32, x), fy = svld1_f32(p32, y), fz = svld1_f32(p32, z);
  ps(int_s(sa, sb));
  ps(select_s(sa, sb));
  ps(splat_s(-3, sa));
  ps(convert_s(sa, fy));
  int64_t rd[32];
  svst1_s64(p64, rd, mul_d(svld1_s64(p64, g), svld1_s64(p64, h)));
  for (int i = 0; i < (int)svcntd(); i++) printf("%lld ", (long long)rd[i]);
  printf("\n");
  svst1_s64(p64, rd, splat_d(1234567890123, svld1_s64(p64, h)));
  for (int i = 0; i < (int)svcntd(); i++) printf("%lld ", (long long)rd[i]);
  printf("\n");
  uint16_t rh[128];
  svst1_u16(svptrue_b16(), rh, mul_h(svld1_u16(svptrue_b16(), e), svld1_u16(svptrue_b16(), f)));
  for (int i = 0; i < (int)svcnth(); i++) printf("%u ", rh[i]);
  printf("\n");
  int8_t rb[256];
  svst1_s8(svptrue_b8(), rb, select_b(svld1_s8(svptrue_b8(), c), svld1_s8(svptrue_b8(), d)));
  for (int i = 0; i < (int)svcntb(); i++) printf("%d ", rb[i]);
  printf("\n");
  printf("%d %d\n", reduce_s(sa), reduce_b(svld1_s8(svptrue_b8(), c)));
  pf(float_s(fx, fy, fz));
  double rf[32];
  svst1_f64(p64, rf, float_d(svld1_f64(p64, u), svld1_f64(p64, v), svld1_f64(p64, w)));
  for (int i = 0; i < (int)svcntd(); i++) printf("%a ", rf[i]);
  printf("\n");
  printf("%a %lld\n", fsum_s(0.5f, fx), (long long)lanes());
  return 0;
}
