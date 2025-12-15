// margarine_runtime.c

#include <stdlib.h>
#include <stdint.h>
#include <stdio.h>

struct ErrorTy {
    int32_t count;
    char** errs;
};


extern int32_t fileCount;
extern int32_t semaErrorsLen;
extern char* semaErrors[];
extern struct ErrorTy parserErrors[];
extern struct ErrorTy lexerErrors[];



void *margarineAlloc(int32_t size) {
    // You can make this a plain malloc for now
    void *p = malloc(size);
    if (!p) {
        fprintf(stderr, "margarineAlloc: out of memory\n");
        abort();
    }
    return p;
}

// Crashes immediately. Called by compiler when something is "impossible"
void margarineAbort() {
    printf("margarineAbort called\n");
    abort();
}

// Error reporting hook
void margarineError(int32_t kind, int32_t file, int32_t index) {
    if (kind == 0) {
        struct ErrorTy file_meta = lexerErrors[file];
        char* err = file_meta.errs[index];

        printf("%s", err);
    } else if (kind == 1) {
        struct ErrorTy file_meta = parserErrors[file];
        char* err = file_meta.errs[index];

        printf("%s", err);
    } else if (kind == 2) {
        char* err = semaErrors[index];
        printf("%s", err);
    }

    abort();
}


void print_int(int64_t x) {
    printf("%lld\n", (long long)x);
}
