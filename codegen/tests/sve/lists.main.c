#include <arm_sve.h>
#include <stdint.h>
#include <stdio.h>

void st2_swapped(int32_t *, svint32_t, svint32_t);
void st2_same(int32_t *, svint32_t);
svint64_t st3_mixed(int64_t *, svint64_t, svint64_t, svint64_t);
void st4_twice(int16_t *, int16_t *, svint16_t, svint16_t, svint16_t, svint16_t);
svuint8_t tbl2_both(svuint8_t, svuint8_t, svuint8_t);
void zip_loop(int32_t *, const int32_t *, const int32_t *, uint64_t);
void st4_pressure(int32_t *, svint32_t);

static int32_t w[2048 * 10];

int main(void) {
  svbool_t all = svptrue_b8();
  int n = (int)svcntw(), d = (int)svcntd(), h = (int)svcnth(), b = (int)svcntb();
  svint32_t a = svindex_s32(1, 3), c = svindex_s32(-5, 7);

  st2_swapped(w, a, c);
  for (int i = 0; i < 2 * n; i++) printf("%d ", w[i]);
  printf("\n");

  st2_same(w, c);
  for (int i = 0; i < 2 * n; i++) printf("%d ", w[i]);
  printf("\n");

  int64_t l[3 * 32];
  svint64_t r = st3_mixed(l, svindex_s64(2, 5), svindex_s64(-9, 4), svindex_s64(100, -3));
  for (int i = 0; i < 3 * d; i++) printf("%lld ", (long long)l[i]);
  int64_t rr[32];
  svst1_s64(svptrue_b64(), rr, r);
  for (int i = 0; i < d; i++) printf("%lld ", (long long)rr[i]);
  printf("\n");

  int16_t p[4 * 128], q[4 * 128];
  st4_twice(p, q, svindex_s16(0, 1), svindex_s16(100, 2), svindex_s16(-50, 3), svindex_s16(7, -1));
  for (int i = 0; i < 4 * h; i++) printf("%d/%d ", p[i], q[i]);
  printf("\n");

  uint8_t t[256];
  svuint8_t x = tbl2_both(svindex_u8(10, 1), svindex_u8(200, 3), svindex_u8(0, 5));
  svst1_u8(all, t, x);
  for (int i = 0; i < b; i++) printf("%d ", t[i]);
  printf("\n");

  int32_t ia[4 * 64], ib[4 * 64], out[8 * 64];
  for (int i = 0; i < 4 * n; i++) ia[i] = i * i, ib[i] = -i;
  zip_loop(out, ia, ib, 4);
  for (int i = 0; i < 8 * n; i++) printf("%d ", out[i]);
  printf("\n");

  st4_pressure(w, svindex_s32(3, 2));
  for (int i = 0; i < 40 * n; i++) printf("%d ", w[i]);
  printf("\n");
  return 0;
}
