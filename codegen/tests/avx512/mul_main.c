#include <stdint.h>
#include <stdio.h>

void mul4x64(const uint64_t *, const uint64_t *, uint64_t *);
void mul2x64(const uint64_t *, const uint64_t *, uint64_t *);
void mul4x32(const uint32_t *, const uint32_t *, uint32_t *);

static uint64_t s = 0x9e3779b97f4a7c15u;

static uint64_t next(void) {
    s ^= s << 13;
    s ^= s >> 7;
    s ^= s << 17;
    return s;
}

int main(void) {
    static uint64_t a[4] __attribute__((aligned(64))), b[4] __attribute__((aligned(64))), r[4] __attribute__((aligned(64)));
    static uint32_t c[4] __attribute__((aligned(64))), d[4] __attribute__((aligned(64))), q[4] __attribute__((aligned(64)));
    for (int round = 0; round < 8; round++) {
        for (int i = 0; i < 4; i++) {
            a[i] = next();
            b[i] = next();
            c[i] = (uint32_t)next();
            d[i] = (uint32_t)next();
        }
        mul4x64(a, b, r);
        for (int i = 0; i < 4; i++) printf("%llx ", (unsigned long long)r[i]);
        mul2x64(a, b, r);
        for (int i = 0; i < 2; i++) printf("%llx ", (unsigned long long)r[i]);
        mul4x32(c, d, q);
        for (int i = 0; i < 4; i++) printf("%x ", q[i]);
        printf("\n");
    }
    return 0;
}
