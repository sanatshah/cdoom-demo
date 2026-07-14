#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>

int myargc = 0;
char **myargv = NULL;

FILE *M_fopen(const char *filename, const char *mode)
{
    return fopen(filename, mode);
}

long M_FileLength(FILE *handle)
{
    long saved_position;
    long length;

    saved_position = ftell(handle);
    fseek(handle, 0, SEEK_END);
    length = ftell(handle);
    fseek(handle, saved_position, SEEK_SET);

    return length;
}

void I_Error(const char *error, ...)
{
    va_list args;

    va_start(args, error);
    vfprintf(stderr, error, args);
    va_end(args);
    fputc('\n', stderr);
    exit(1);
}
