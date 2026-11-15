#include <stdint.h>
#include <stdio.h>
#include <string.h>

void sv_eval_1(void *, void *, void *, void *);

int main(void) {
    static uint8_t s[4096] __attribute__((aligned(64))), in[4096] __attribute__((aligned(64))),
        out[4096] __attribute__((aligned(64))), p2[4096] __attribute__((aligned(64)));
    uint32_t x = 0x2545f491u;
    for (int i = 0; i < 4096; i++) {
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        in[i] = (uint8_t)(x >> (x % 13));
    }
    sv_eval_1(s, in, out, p2);
    for (int i = 0; i < 512; i++) printf("%02x%s", out[i], i % 32 == 31 ? "\n" : "");
    return 0;
}
