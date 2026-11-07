// Drives fp128_narrow.ll on values in each integer's range, and on
// integers at the ends of each.
#include <stdio.h>
#include <stdint.h>

int8_t fptosi_i8(__float128);
uint8_t fptoui_i8(__float128);
int16_t fptosi_i16(__float128);
uint16_t fptoui_i16(__float128);
__float128 sitofp_i8(int8_t);
__float128 uitofp_i8(uint8_t);
__float128 sitofp_i16(int16_t);
__float128 uitofp_i16(uint16_t);

int main(void) {
  double s8[] = {0, -0.0, 1.5, -1.5, 127.9, -128.9, 99};
  for (int i = 0; i < 7; i++) printf("%d\n", fptosi_i8((__float128)s8[i]));
  double u8[] = {0, 1.5, 255.9, 128, 7};
  for (int i = 0; i < 5; i++) printf("%u\n", fptoui_i8((__float128)u8[i]));
  double s16[] = {0, -1.25, 32767.5, -32768.5, 1000};
  for (int i = 0; i < 5; i++) printf("%d\n", fptosi_i16((__float128)s16[i]));
  double u16[] = {0, 1.25, 65535.5, 32768, 40000};
  for (int i = 0; i < 5; i++) printf("%u\n", fptoui_i16((__float128)u16[i]));
  int8_t a[] = {0, 1, -1, 127, -128};
  for (int i = 0; i < 5; i++) printf("%g\n", (double)sitofp_i8(a[i]));
  uint8_t b[] = {0, 1, 128, 255};
  for (int i = 0; i < 4; i++) printf("%g\n", (double)uitofp_i8(b[i]));
  int16_t c[] = {0, -1, 32767, -32768};
  for (int i = 0; i < 4; i++) printf("%g\n", (double)sitofp_i16(c[i]));
  uint16_t d[] = {0, 1, 32768, 65535};
  for (int i = 0; i < 4; i++) printf("%g\n", (double)uitofp_i16(d[i]));
  return 0;
}
