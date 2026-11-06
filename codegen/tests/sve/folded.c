// clang: -mllvm -prefer-predicate-over-epilogue=predicate-dont-vectorize
// Loops vectorised with their tails folded in: each step's lanes under
// the predicate of those still to do (llvm.get.active.lane.mask, whilelo).
#include <stdint.h>

void saxpy(float *restrict y, const float *restrict x, float a, int n) {
  for (int i = 0; i < n; i++) y[i] = a * x[i] + y[i];
}

int64_t sum_i64(const int64_t *x, int n) {
  int64_t s = 0;
  for (int i = 0; i < n; i++) s += x[i];
  return s;
}

void add_u8(uint8_t *restrict y, const uint8_t *restrict a, int n) {
  for (int i = 0; i < n; i++) y[i] = a[i] + 3;
}

void abs_i16(int16_t *restrict y, const int16_t *restrict x, int n) {
  for (int i = 0; i < n; i++) y[i] = x[i] < 0 ? -x[i] : x[i];
}
