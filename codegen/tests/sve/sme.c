// SME: streaming mode and ZA from functions' attributes, and SME's
// intrinsics on ZA's tiles.
#include <arm_sme.h>
#include <stdint.h>

// C = A^T B for a tile: the sum of the outer products of A's and B's rows,
// in ZA of its own, in streaming mode of its own.
__arm_new("za") __arm_locally_streaming
void outer(float *c, const float *a, const float *b, int n) {
  uint64_t vl = svcntw();
  svzero_za();
  for (int k = 0; k < n; k++) {
    svfloat32_t x = svld1(svptrue_b32(), a + k * vl);
    svfloat32_t y = svld1(svptrue_b32(), b + k * vl);
    svmopa_za32_m(0, svptrue_b32(), svptrue_b32(), x, y);
  }
  for (uint32_t i = 0; i < vl; i++)
    svst1_hor_za32(0, i, svptrue_b32(), c + i * vl);
}

// A tile loaded by rows, read back by columns: the transpose.
__arm_new("za") __arm_locally_streaming
void transpose(int32_t *out, const int32_t *in) {
  uint64_t vl = svcntw();
  for (uint32_t i = 0; i < vl; i++)
    svld1_hor_za32(1, i, svptrue_b32(), in + i * vl);
  for (uint32_t i = 0; i < vl; i++) {
    svint32_t col = svread_ver_za32_s32_m(svdup_s32(0), svptrue_b32(), 1, i);
    svst1(svptrue_b32(), out + i * vl, col);
  }
}

// Integer outer products, of bytes into words (smopa: four at a time).
__arm_new("za") __arm_locally_streaming
void smopa_bytes(int32_t *c, const int8_t *a, const int8_t *b, int n) {
  uint64_t vb = svcntb(), vw = svcntw();
  svzero_za();
  for (int k = 0; k < n; k++)
    svmopa_za32_s8_m(2, svptrue_b8(), svptrue_b8(), svld1(svptrue_b8(), a + k * vb), svld1(svptrue_b8(), b + k * vb));
  for (uint32_t i = 0; i < vw; i++)
    svst1_hor_za32(2, i, svptrue_b32(), c + i * vw);
}

uint64_t streaming_words(void) __arm_streaming { return svcntw(); }

// Not streaming: a call of a streaming function switches around it.
uint64_t words_from_outside(void) { return streaming_words() * 10; }

__attribute__((noinline)) static int64_t plain_sum(const int64_t *x, int n) {
  int64_t s = 0;
  for (int i = 0; i < n; i++) s += x[i] ^ i;
  return s;
}

// Streaming: a call of one that is not switches out and back.
__arm_locally_streaming
int64_t sum_from_inside(const int64_t *x, int n) {
  return plain_sum(x, n) + (int64_t)svcntd();
}
