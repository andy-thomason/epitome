// Drives half_constants.ll, printing each half's bits.
#include <stdio.h>
#include <string.h>
_Float16 zero(void), one(void), call_pass(void);
void store_minus_two(_Float16 *);
static unsigned bits(_Float16 h) {
  unsigned short u;
  memcpy(&u, &h, 2);
  return u;
}
int main(void) {
  _Float16 h;
  store_minus_two(&h);
  printf("%04x %04x %04x %04x\n", bits(zero()), bits(one()), bits(h), bits(call_pass()));
  return 0;
}
