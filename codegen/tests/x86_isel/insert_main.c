#include <stdint.h>
#include <stdio.h>
#include <string.h>

void insert_q(uint64_t, void *);
void insert_d(uint32_t, void *);

int main(void) {
    uint8_t out[16];
    uint64_t xs[] = {0, 1, 0x8000000000000000ull, 0x0123456789abcdefull, ~0ull};
    for (int i = 0; i < 5; i++) {
        memset(out, 0xa5, sizeof out);
        insert_q(xs[i], out);
        for (int k = 0; k < 16; k++) printf("%02x", out[k]);
        printf(" ");
        memset(out, 0xa5, sizeof out);
        insert_d((uint32_t)(xs[i] >> 16), out);
        for (int k = 0; k < 16; k++) printf("%02x", out[k]);
        printf("\n");
    }
    return 0;
}
