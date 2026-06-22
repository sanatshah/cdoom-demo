#include "cdoom_rust.h"

#include <stdio.h>
#include <stdlib.h>

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

    printf("cdoom-rust probe OK: %s\n", version);
    return 0;
}
