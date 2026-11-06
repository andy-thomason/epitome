// Drives mask_memory.ll: each mask stored over 0xaa bytes (a store must
// write only its own), then masks of known bits loaded to select bytes.
#include <stdio.h>
#include <string.h>
void st4(const int *, unsigned char *);
void st16(const char *, const char *, unsigned char *);
void st32(const char *, unsigned char *);
void st64(const char *, unsigned char *);
void ld4(const unsigned char *, const int *, int *);
void ld16(const unsigned char *, const char *, char *);
void ld64(const unsigned char *, const char *, char *);
static void bytes(const unsigned char *b, int n) { for (int i = 0; i < n; i++) printf("%02x", b[i]); printf("\n"); }
int main(void) {
  const char *t = "the quick brown fox jumps over the lazy dog, THE QUICK BROWN FOX!";
  const char *u = "the quack brawn fix jumps ovxr the lazy cat, THE QUICK BROWN FOX?";
  unsigned char o[16];
  int a[4] = {-1, 2, -3, 4};
  memset(o, 0xaa, 16); st4(a, o); o[0] &= 0x0f; bytes(o, 3);
  memset(o, 0xaa, 16); st16(t, u, o); bytes(o, 4);
  memset(o, 0xaa, 16); st16(t + 16, u + 16, o); bytes(o, 4);
  memset(o, 0xaa, 16); st32(t, o); bytes(o, 6);
  memset(o, 0xaa, 16); st64(t, o); bytes(o, 10);
  unsigned char m[8] = {0x5a, 0xc3, 0x0f, 0xf0, 0x81, 0x7e, 0x01, 0x80};
  int r[4]; int x[4] = {10, 20, 30, 40};
  for (int k = 0; k < 4; k++) { unsigned char mm = k * 5; ld4(&mm, x, r); printf("%d %d %d %d\n", r[0], r[1], r[2], r[3]); }
  char s[65];
  ld16(m, t, s); s[16] = 0; printf("%s\n", s);
  ld16(m + 3, t, s); s[16] = 0; printf("%s\n", s);
  ld64(m, t, s); s[64] = 0; printf("%s\n", s);
  return 0;
}
