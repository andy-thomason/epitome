#include <stdint.h>
#include <stdio.h>

void saxpy(float *restrict, const float *restrict, float, int);
int64_t sum_i32(const int32_t *, int);
int32_t max_i32(const int32_t *, int);
void clamp(int32_t *restrict, const int32_t *restrict, int);
void add_u8(uint8_t *restrict, const uint8_t *restrict, const uint8_t *restrict, int);
double dot(const double *, const double *, int);
void to_float(float *restrict, const int32_t *restrict, int);
void cond_store(int32_t *restrict, const int32_t *restrict, int);
void widen(int32_t *restrict, const int16_t *restrict, int);
void scale64(int64_t *restrict, const int64_t *restrict, int64_t, int);

#define N 203

int main(void) {
  static float x[N], y[N], fy[N];
  static int32_t a[N], b[N];
  static uint8_t c[N], d[N], e[N];
  static double u[N], v[N];
  static int16_t h[N];
  static int64_t g[N], gy[N];
  for (int i = 0; i < N; i++) {
    x[i] = i * 0.25f - 7, y[i] = 3 - i * 0.125f;
    a[i] = (i * 7919) % 601 - 300, b[i] = -1;
    c[i] = i * 3, d[i] = 200 - i;
    u[i] = i * 0.5 - 10, v[i] = 1.0 / (i + 1);
    h[i] = i * 300 - 20000;
    g[i] = (int64_t)i * 123456789 - 5000000000;
  }
  for (int n = 0; n <= N; n += 29) {
    saxpy(y, x, 1.5f, n);
    printf("%d: %lld %d %a\n", n, (long long)sum_i32(a, n), max_i32(a, n), dot(u, v, n));
  }
  for (int i = 0; i < N; i++) printf("%a ", y[i]);
  printf("\n");
  clamp(b, a, N);
  for (int i = 0; i < N; i++) printf("%d ", b[i]);
  printf("\n");
  add_u8(e, c, d, N);
  for (int i = 0; i < N; i++) printf("%d ", e[i]);
  printf("\n");
  to_float(fy, a, N);
  for (int i = 0; i < N; i++) printf("%a ", fy[i]);
  printf("\n");
  cond_store(b, a, N);
  for (int i = 0; i < N; i++) printf("%d ", b[i]);
  printf("\n");
  widen(b, h, N);
  for (int i = 0; i < N; i++) printf("%d ", b[i]);
  printf("\n");
  scale64(gy, g, -77, N);
  for (int i = 0; i < N; i++) printf("%lld ", (long long)gy[i]);
  printf("\n");
  return 0;
}
