#include <arm_sve.h>
#include <stdint.h>
#include <stdio.h>

void add_s(int32_t *, const int32_t *, const int32_t *);
void fma_f(float *, const float *, const float *);
void logic_b(uint8_t *, const uint8_t *, const uint8_t *);
void copy_d(double *, const double *);

int main(void) {
  int32_t a[64], b[64], r[64];
  float fa[64], fb[64], fr[64];
  uint8_t ba[256], bb[256], br[256];
  double da[32], dr[32];
  for (int i = 0; i < 64; i++) a[i] = i * i, b[i] = -3 * i, fa[i] = i * 0.25f, fb[i] = 1.5f - i;
  for (int i = 0; i < 256; i++) ba[i] = i * 7, bb[i] = 255 - i;
  for (int i = 0; i < 32; i++) da[i] = i / 3.0;
  add_s(r, a, b);
  for (int i = 0; i < (int)svcntw(); i++) printf("%d ", r[i]);
  printf("\n");
  fma_f(fr, fa, fb);
  for (int i = 0; i < (int)svcntw(); i++) printf("%g ", fr[i]);
  printf("\n");
  logic_b(br, ba, bb);
  for (int i = 0; i < (int)svcntb(); i++) printf("%d ", br[i]);
  printf("\n");
  copy_d(dr, da);
  for (int i = 0; i < (int)svcntd(); i++) printf("%g ", dr[i]);
  printf("\n");
  return 0;
}
