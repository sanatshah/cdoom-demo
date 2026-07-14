#include "cdoom_rust.h"

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct {
    uint32_t h0, h1, h2, h3, h4;
    uint32_t nblocks;
    unsigned char buf[64];
    int count;
} probe_sha1_context_t;

int main(void)
{
    if (cdoom_rust_init() != 0) {
        fprintf(stderr, "cdoom_rust_init failed\n");
        return 1;
    }

    const char *version = cdoom_rust_version();
    if (version == NULL) {
        fprintf(stderr, "cdoom_rust_version returned NULL\n");
        return 1;
    }

    probe_sha1_context_t sha1;
    unsigned char digest[20];
    cdoom_rust_sha1_init(&sha1);
    cdoom_rust_sha1_update(&sha1, "abc", 3);
    cdoom_rust_sha1_final(digest, &sha1);
    if (digest[0] != 0xa9 || digest[19] != 0x9d) {
        fprintf(stderr, "cdoom_rust_sha1 returned unexpected digest\n");
        return 1;
    }

    unsigned char seed[16] = {0};
    cdoom_rust_prng_start(seed);
    if (cdoom_rust_prng_random() != 0xe53e64e8u) {
        fprintf(stderr, "cdoom_rust_prng returned unexpected value\n");
        return 1;
    }
    cdoom_rust_prng_stop();

    char input[] = "abcdef";
    char output[8] = {0};
    void *stream = cdoom_rust_mem_fopen_read(input, strlen(input));
    if (cdoom_rust_mem_fread(output, 2, 4, stream) != 3
     || memcmp(output, "abcdef", 6) != 0) {
        fprintf(stderr, "cdoom_rust_mem_fread returned unexpected data\n");
        return 1;
    }
    cdoom_rust_mem_fclose(stream);

    printf("cdoom-rust probe OK: %s\n", version);
    return 0;
}
