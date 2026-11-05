#include <stdint.h>
#include <stdio.h>

void saxpy(float *restrict, const float *restrict, float, int);
int64_t sum_i64(const int64_t *, int);
void add_u8(uint8_t *restrict, const uint8_t *restrict, int);
void abs_i16(int16_t *restrict, const int16_t *restrict, int);

#define N 197

int main(void) {
  static float x[N], y[N];
  static int64_t g[N];
  static uint8_t u[N], v[N];
  static int16_t h[N], k[N];
  for (int i = 0; i < N; i++) {
    x[i] = i * 0.25f - 9, y[i] = 1 - i * 0.5f;
    g[i] = (int64_t)i * 987654321 - 77;
    u[i] = i * 41;
    h[i] = i * 333 - 30000;
  }
  for (int n = 0; n <= N; n += 13) {
    saxpy(y, x, -0.75f, n);
    add_u8(v, u, n);
    abs_i16(k, h, n);
    printf("%d: %lld %a %d %d\n", n, (long long)sum_i64(g, n), y[n ? n - 1 : 0], v[n ? n - 1 : 0], k[n ? n - 1 : 0]);
  }
  return 0;
}
