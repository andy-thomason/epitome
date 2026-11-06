#include <stdint.h>
#include <stdio.h>

struct pair { float re, im; };
int32_t sum_u8(const uint8_t *, int);
void narrow(int8_t *restrict, const int32_t *restrict, int);
void cond_load(int32_t *restrict, const int32_t *restrict, const int32_t *restrict, int);
void gather(float *restrict, const float *restrict, const int32_t *restrict, int);
void cmul(struct pair *restrict, const struct pair *restrict, const struct pair *restrict, int);
float fmax_red(const float *, int);
double fmin_red(const double *, int);
void i16_to_f(float *restrict, const int16_t *restrict, int);
void stride2(int32_t *restrict, const int32_t *restrict, int);
int count_eq(const int32_t *, int32_t, int);

#define N 211

int main(void) {
  static uint8_t u[N];
  static int32_t a[2 * N], c[N], y[N], idx[N];
  static int8_t b[N];
  static float x[N], fy[N];
  static double d[N];
  static int16_t h[N];
  static struct pair p[N], q[N], r[N];
  for (int i = 0; i < N; i++) {
    u[i] = i * 37;
    a[i] = (i * 7919) % 6001 - 3000, a[N + i] = i * 3;
    c[i] = (i % 5) - 2;
    idx[i] = (i * 13) % N;
    x[i] = (i * 31 % 97) * 0.5f - 20;
    d[i] = (i * 17 % 89) * 0.25 - 3;
    h[i] = i * 151 - 16000;
    p[i].re = i * 0.5f, p[i].im = 1 - i * 0.25f;
    q[i].re = 2 - i * 0.125f, q[i].im = i * 0.75f;
  }
  for (int n = 0; n <= N; n += 37)
    printf("%d: %d %a %a %d\n", n, sum_u8(u, n), fmax_red(x, n), fmin_red(d, n), count_eq(c, 1, n));
  narrow(b, a, N);
  for (int i = 0; i < N; i++) printf("%d ", b[i]);
  printf("\n");
  cond_load(y, a, c, N);
  for (int i = 0; i < N; i++) printf("%d ", y[i]);
  printf("\n");
  gather(fy, x, idx, N);
  for (int i = 0; i < N; i++) printf("%a ", fy[i]);
  printf("\n");
  cmul(r, p, q, N);
  for (int i = 0; i < N; i++) printf("%a,%a ", r[i].re, r[i].im);
  printf("\n");
  i16_to_f(fy, h, N);
  for (int i = 0; i < N; i++) printf("%a ", fy[i]);
  printf("\n");
  stride2(y, a, N);
  for (int i = 0; i < N; i++) printf("%d ", y[i]);
  printf("\n");
  return 0;
}
