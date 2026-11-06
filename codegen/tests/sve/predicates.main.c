#include <arm_sve.h>
#include <stdint.h>
#include <stdio.h>

svbool_t second(svbool_t, svbool_t);
svbool_t keep(svbool_t, svbool_t);
svbool_t all_d(void);
svbool_t none_h(void);

/* Writes every predicate register. */
void clobber(void) {
  __asm__ volatile("pfalse p0.b\n pfalse p1.b\n pfalse p2.b\n pfalse p3.b\n pfalse p4.b\n pfalse p5.b\n"
                   "pfalse p6.b\n pfalse p7.b\n ptrue p8.b\n ptrue p9.b\n ptrue p10.b\n ptrue p11.b\n"
                   "ptrue p12.b\n ptrue p13.b\n ptrue p14.b\n ptrue p15.b\n" ::: "p0", "p1", "p2", "p3",
                   "p4", "p5", "p6", "p7", "p8", "p9", "p10", "p11", "p12", "p13", "p14", "p15");
}

int main(void) {
  svbool_t a = svwhilelt_b32(0, 3), b = svwhilelt_b32(0, 5);
  printf("%d\n", (int)svcntp_b32(svptrue_b32(), second(a, b)));
  svbool_t c = svwhilelt_b8(0, 7), d = svwhilelt_b8(0, 2);
  printf("%d\n", (int)svcntp_b8(svptrue_b8(), keep(c, d)));
  printf("%d %d\n", (int)svcntp_b64(svptrue_b64(), all_d()), (int)svcntp_b16(svptrue_b16(), none_h()));
  return 0;
}
