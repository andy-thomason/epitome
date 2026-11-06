#include <arm_sve.h>
#include <stdint.h>
#include <stdio.h>

svint32_t keep2(svint32_t, svint32_t);
svint64_t keep3(svint64_t, svint64_t, svint64_t, int64_t);
svint32_t stack_arg(svint32_t, int64_t, int64_t, int64_t, int64_t, int64_t, int64_t, int64_t, int64_t, int64_t);

static int64_t seen;
int64_t twice(int64_t x) { seen += x; return 2 * x; }

/* Writes every vector register. */
void clobber(void) {
  __asm__ volatile("mov z0.d, #-1\n mov z1.d, #-1\n mov z2.d, #-1\n mov z3.d, #-1\n mov z4.d, #-1\n mov z5.d, #-1\n"
                   "mov z6.d, #-1\n mov z7.d, #-1\n mov z16.d, #-1\n mov z17.d, #-1\n mov z18.d, #-1\n mov z19.d, #-1\n"
                   "mov z20.d, #-1\n mov z21.d, #-1\n mov z22.d, #-1\n mov z23.d, #-1\n mov z24.d, #-1\n mov z25.d, #-1\n"
                   "mov z26.d, #-1\n mov z27.d, #-1\n mov z28.d, #-1\n mov z29.d, #-1\n mov z30.d, #-1\n mov z31.d, #-1\n"
                   ::: "z0", "z1", "z2", "z3", "z4", "z5", "z6", "z7", "z16", "z17", "z18", "z19", "z20", "z21",
                   "z22", "z23", "z24", "z25", "z26", "z27", "z28", "z29", "z30", "z31");
}

int main(void) {
  int32_t a[64], b[64], r[64];
  int64_t c[32], d[32], e[32], rd[32];
  for (int i = 0; i < 64; i++) a[i] = i * 11 - 7, b[i] = 300 - i;
  for (int i = 0; i < 32; i++) c[i] = i * 1000003, d[i] = -i * 77, e[i] = (int64_t)i << 33;
  svbool_t p32 = svptrue_b32(), p64 = svptrue_b64();
  svst1_s32(p32, r, keep2(svld1_s32(p32, a), svld1_s32(p32, b)));
  for (int i = 0; i < (int)svcntw(); i++) printf("%d ", r[i]);
  printf("\n");
  svst1_s64(p64, rd, keep3(svld1_s64(p64, c), svld1_s64(p64, d), svld1_s64(p64, e), 21));
  for (int i = 0; i < (int)svcntd(); i++) printf("%lld ", (long long)rd[i]);
  printf("\n");
  svst1_s32(p32, r, stack_arg(svld1_s32(p32, a), 1, 2, 3, 4, 5, 6, 7, 8, 1000));
  for (int i = 0; i < (int)svcntw(); i++) printf("%d ", r[i]);
  printf("\nseen %lld\n", (long long)seen);
  return 0;
}
