// Drives rem_by_min.ll: x % MIN is x, except MIN % MIN; x / MIN is 0 or 1.
#include <limits.h>
#include <stdio.h>
long rem64(long), div64(long);
int rem32(int), div32(int);
int main(void) {
  long xs[] = {0, 1, -1, 5, -5, LONG_MAX, LONG_MIN, LONG_MIN + 1};
  for (int i = 0; i < 8; i++)
    printf("%ld %ld %d %d\n", rem64(xs[i]), div64(xs[i]), rem32((int)xs[i]), div32((int)xs[i]));
  printf("%d %d\n", rem32(INT_MIN), div32(INT_MIN));
  return 0;
}
