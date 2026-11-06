// Loops clang's vectoriser makes scalable vectors of, for SVE.
#include <stdint.h>

void saxpy(float *restrict y, const float *restrict x, float a, int n) {
  for (int i = 0; i < n; i++) y[i] = a * x[i] + y[i];
}

int64_t sum_i32(const int32_t *x, int n) {
  int64_t s = 0;
  for (int i = 0; i < n; i++) s += x[i];
  return s;
}

int32_t max_i32(const int32_t *x, int n) {
  int32_t m = INT32_MIN;
  for (int i = 0; i < n; i++) m = x[i] > m ? x[i] : m;
  return m;
}

void clamp(int32_t *restrict y, const int32_t *restrict x, int n) {
  for (int i = 0; i < n; i++) y[i] = x[i] < 0 ? 0 : x[i] > 255 ? 255 : x[i];
}

void add_u8(uint8_t *restrict y, const uint8_t *restrict a, const uint8_t *restrict b, int n) {
  for (int i = 0; i < n; i++) y[i] = a[i] + b[i];
}

double dot(const double *x, const double *y, int n) {
  double s = 0;
  for (int i = 0; i < n; i++) s += x[i] * y[i];
  return s;
}

void to_float(float *restrict y, const int32_t *restrict x, int n) {
  for (int i = 0; i < n; i++) y[i] = (float)x[i] * 0.5f;
}

void cond_store(int32_t *restrict y, const int32_t *restrict x, int n) {
  for (int i = 0; i < n; i++)
    if (x[i] & 1) y[i] = x[i] * 3;
}

void widen(int32_t *restrict y, const int16_t *restrict x, int n) {
  for (int i = 0; i < n; i++) y[i] = x[i] * 2;
}

void scale64(int64_t *restrict y, const int64_t *restrict x, int64_t k, int n) {
  for (int i = 0; i < n; i++) y[i] = x[i] * k + (x[i] >> 3);
}
