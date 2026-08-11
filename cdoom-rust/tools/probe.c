#include "cdoom_rust.h"

#include <stdio.h>
#include <stdlib.h>

int main(void)
{
    const char *version;
    unsigned char source_palette[3] = { 1, 2, 255 };
    unsigned char gamma_table[5 * 256];
    unsigned char corrected_palette[3] = { 0, 0, 0 };
    int i;

    if (cdoom_rust_init() != 0) {
        fprintf(stderr, "cdoom_rust_init failed\n");
        return 1;
    }

    version = cdoom_rust_version();
    if (version == NULL) {
        fprintf(stderr, "cdoom_rust_version returned NULL\n");
        return 1;
    }

    for (i = 0; i < 5 * 256; ++i) {
        gamma_table[i] = (unsigned char) i;
    }

    cdoom_rust_v_gamma_correct_palette(corrected_palette,
                                       source_palette,
                                       gamma_table,
                                       1,
                                       1);
    if (corrected_palette[0] != 0
     || corrected_palette[1] != 0
     || corrected_palette[2] != 252) {
        fprintf(stderr, "palette gamma probe failed\n");
        return 1;
    }

    printf("cdoom-rust probe OK: %s\n", version);
    return 0;
}
