#include <stdio.h>
typedef struct { long a, b; } L2;
L2 bump_second(L2);
L2 swap(L2);
int main(void) {
    L2 v = bump_second((L2){ 3, 40 });
    L2 w = swap((L2){ 5, 6 });
    printf("%ld %ld %ld %ld\n", v.a, v.b, w.a, w.b);
    return 0;
}
