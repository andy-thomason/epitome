#include <arm_sve.h>
#include <stdint.h>
#include <stdio.h>

svint32_t add_s(svint32_t, svint32_t);
svint8_t add_b(svint8_t, svint8_t);
svint16_t add_h(svint16_t, svint16_t);
svint64_t add_d3(svint64_t, svint64_t, svint64_t);

int main(void) {
  int32_t a[64], b[64];
  int8_t c[256], d[256];
  int16_t e[128], f[128];
  int64_t g[32], h[32], k[32];
  for (int i = 0; i < 64; i++) a[i] = i * 7 - 100, b[i] = 1000 - i * i;
  for (int i = 0; i < 256; i++) c[i] = i * 3, d[i] = 100 - i;
  for (int i = 0; i < 128; i++) e[i] = i * 300, f[i] = -i * 77;
  for (int i = 0; i < 32; i++) g[i] = (int64_t)i << 40, h[i] = -i, k[i] = i * i * i;
  int32_t r[64];
  svst1_s32(svptrue_b32(), r, add_s(svld1_s32(svptrue_b32(), a), svld1_s32(svptrue_b32(), b)));
  for (int i = 0; i < (int)svcntw(); i++) printf("%d ", r[i]);
  printf("\n");
  int8_t rb[256];
  svst1_s8(svptrue_b8(), rb, add_b(svld1_s8(svptrue_b8(), c), svld1_s8(svptrue_b8(), d)));
  for (int i = 0; i < (int)svcntb(); i++) printf("%d ", rb[i]);
  printf("\n");
  int16_t rh[128];
  svst1_s16(svptrue_b16(), rh, add_h(svld1_s16(svptrue_b16(), e), svld1_s16(svptrue_b16(), f)));
  for (int i = 0; i < (int)svcnth(); i++) printf("%d ", rh[i]);
  printf("\n");
  int64_t rd[32];
  svst1_s64(svptrue_b64(), rd, add_d3(svld1_s64(svptrue_b64(), g), svld1_s64(svptrue_b64(), h), svld1_s64(svptrue_b64(), k)));
  for (int i = 0; i < (int)svcntd(); i++) printf("%lld ", (long long)rd[i]);
  printf("\n");
  return 0;
}
