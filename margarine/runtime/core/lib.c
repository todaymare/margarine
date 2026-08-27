#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#if !defined(__wasm__)
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#else
extern void *malloc(size_t size);
extern void free(void *ptr);
extern void *memcpy(void *destination, const void *source, size_t len);
#endif

#if defined(__wasm__)
__attribute__((import_module("env")))
__attribute__((import_name("host_print")))
void host_print(const uint8_t *ptr, size_t len);

__attribute__((import_module("env")))
__attribute__((import_name("host_abort")))
_Noreturn void host_abort(int32_t code);

__attribute__((import_module("env")))
__attribute__((import_name("consume_fuel")))
void consume_fuel(void);


static int FUEL = 0;

void margarineConsumeFuel(void) {
    FUEL += 1;

    if (FUEL == 5000) {
        consume_fuel();
        FUEL = 0;
    }
}

#endif

typedef struct {
    uint8_t *ptr;
    size_t len;
} MargarineCollection;

typedef struct {
    MargarineCollection value;
} MargarineStr;

typedef struct {
    size_t ref_count;
} MargarineStrHeader;

_Noreturn void margarineAbort(int32_t code);
void margarineAssertNotNull(uint8_t *ptr);

static void write_bytes(const uint8_t *bytes, size_t len) {
#if defined(__wasm__)
    host_print(bytes, len);
#else
    fwrite(bytes, 1, len, stdout);
    fflush(stdout);
#endif
}

void *margarineAlloc(size_t size) {
    void *ptr = malloc(size == 0 ? 1 : size);

    if (ptr == NULL) {
        margarineAbort(1);
    }

    return ptr;
}

void margarineDealloc(uint8_t *ptr, size_t size) {
    (void)size;
    margarineAssertNotNull(ptr);
    free(ptr);
}

uint8_t *margarineRcAlloc(size_t total_size) {
    if (total_size < sizeof(size_t)) {
        margarineAbort(1);
    }

    uint8_t *ptr = margarineAlloc(total_size);
    *(size_t *)ptr = 1;
    return ptr;
}

void margarineAssertNotNull(uint8_t *ptr) {
    if (ptr == NULL) {
        static const uint8_t message[] =
            "panic: null pointer dereference\n";
        write_bytes(message, sizeof(message) - 1);
        margarineAbort(1);
    }
}

MargarineStr margarineStringFromUtf8(const uint8_t *bytes, size_t len) {
    if (len > SIZE_MAX - sizeof(MargarineStrHeader)) {
        margarineAbort(1);
    }
    if (len != 0) {
        margarineAssertNotNull((uint8_t *)bytes);
    }

    uint8_t *buf = margarineRcAlloc(sizeof(MargarineStrHeader) + len);
    if (len != 0) {
        memcpy(buf + sizeof(MargarineStrHeader), bytes, len);
    }

    return (MargarineStr){
        .value = {
            .ptr = buf,
            .len = len,
        },
    };
}

_Noreturn void margarineAbort(int32_t code) {
#if defined(__wasm__)
    host_abort(code);
#else
    fflush(stdout);
    fflush(stderr);
    exit(code);
#endif
}

_Noreturn void margarinePanic(const uint8_t *bytes, int64_t length) {
    static const uint8_t prefix[] = "panic: ";
    static const uint8_t fallback[] = "<invalid panic message>";
    static const uint8_t newline[] = "\n";

    write_bytes(prefix, sizeof(prefix) - 1);

    if (bytes != NULL && length >= 0) {
        if (length != 0) {
            write_bytes(bytes, (size_t)length);
        }
    } else {
        write_bytes(fallback, sizeof(fallback) - 1);
    }

    write_bytes(newline, sizeof(newline) - 1);
    margarineAbort(1);
}
