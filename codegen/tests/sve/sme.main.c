#include <stdint.h>
#include <stdio.h>

void outer(float *, const float *, const float *, int);
void transpose(int32_t *, const int32_t *);
void smopa_bytes(int32_t *, const int8_t *, const int8_t *, int);
uint64_t words_from_outside(void);
int64_t sum_from_inside(const int64_t *, int);

static float a[64 * 8], b[64 * 8], c[64 * 64];
static int32_t t[64 * 64], u[64 * 64], w[64 * 64];
static int8_t p[256 * 4], q[256 * 4];
static int64_t g[100];

int main(void) {
  for (int i = 0; i < 64 * 8; i++) a[i] = (i % 13) * 0.5f - 3, b[i] = 1.25f - (i % 7) * 0.75f;
  for (int i = 0; i < 64 * 64; i++) t[i] = i * 3 - 1000;
  for (int i = 0; i < 256 * 4; i++) p[i] = i * 7, q[i] = 100 - i;
  for (int i = 0; i < 100; i++) g[i] = (int64_t)i * i * 1001;
  outer(c, a, b, 8);
  transpose(u, t);
  smopa_bytes(w, p, q, 4);
  unsigned long vl = words_from_outside() / 10;
  printf("words %lu\n", vl);
  for (unsigned long i = 0; i < vl * vl; i++) printf("%a ", c[i]);
  printf("\n");
  for (unsigned long i = 0; i < vl * vl; i++) printf("%d ", u[i]);
  printf("\n");
  for (unsigned long i = 0; i < vl * vl; i++) printf("%d ", w[i]);
  printf("\n%lld\n", (long long)sum_from_inside(g, 100));
  return 0;
}
