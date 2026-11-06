#include <stdint.h>
#include <stdio.h>

int64_t lea3(int64_t a, int64_t b);
int64_t add_shift(int64_t a);
int64_t load8(const int64_t *p);
uint32_t zext_byte(const uint8_t *p);
int64_t mul3(int64_t a);
void store_word(uint16_t *p, uint16_t v);
uint32_t vendor_ebx(void);

int main(void) {
  int64_t xs[2] = {11, -42};
  uint8_t b[1] = {200};
  uint16_t w[3] = {1, 2, 3};
  store_word(w, 0xbeef);
  printf("%lld %lld %lld %u %lld\n", (long long)lea3(5, 7), (long long)add_shift(9), (long long)load8(xs), zext_byte(b),
         (long long)mul3(-4));
  printf("%x %x %x %x\n", w[0], w[1], w[2], vendor_ebx() != 0);
  return 0;
}
