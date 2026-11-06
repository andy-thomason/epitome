// Harder loops for SVE: widening by two steps, narrowing, conditional
// loads, gathers, interleaved structures and float min/max.
#include <stdint.h>

int32_t sum_u8(const uint8_t *x, int n) {
  int32_t s = 0;
  for (int i = 0; i < n; i++) s += x[i];
  return s;
}

void narrow(int8_t *restrict y, const int32_t *restrict x, int n) {
  for (int i = 0; i < n; i++) y[i] = (int8_t)(x[i] >> 4);
}

void cond_load(int32_t *restrict y, const int32_t *restrict x, const int32_t *restrict c, int n) {
  for (int i = 0; i < n; i++) y[i] = c[i] > 0 ? x[i] : 7;
}

void gather(float *restrict y, const float *restrict x, const int32_t *restrict idx, int n) {
  for (int i = 0; i < n; i++) y[i] = x[idx[i]];
}

struct pair { float re, im; };

void cmul(struct pair *restrict y, const struct pair *restrict a, const struct pair *restrict b, int n) {
  for (int i = 0; i < n; i++) {
    y[i].re = a[i].re * b[i].re - a[i].im * b[i].im;
    y[i].im = a[i].re * b[i].im + a[i].im * b[i].re;
  }
}

float fmax_red(const float *x, int n) {
  float m = -1e30f;
  for (int i = 0; i < n; i++) m = __builtin_fmaxf(m, x[i]);
  return m;
}

double fmin_red(const double *x, int n) {
  double m = 1e300;
  for (int i = 0; i < n; i++) m = x[i] < m ? x[i] : m;
  return m;
}

void i16_to_f(float *restrict y, const int16_t *restrict x, int n) {
  for (int i = 0; i < n; i++) y[i] = x[i];
}

void stride2(int32_t *restrict y, const int32_t *restrict x, int n) {
  for (int i = 0; i < n; i++) y[i] = x[2 * i] + x[2 * i + 1];
}

int count_eq(const int32_t *x, int32_t k, int n) {
  int c = 0;
  for (int i = 0; i < n; i++) c += x[i] == k;
  return c;
}
