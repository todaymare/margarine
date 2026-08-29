#include <stddef.h>
#include <stdint.h>

extern unsigned char __heap_base[];
_Noreturn void margarineAbort(int32_t code);

static unsigned char *wasm_brk;

static void *wasm_sbrk(ptrdiff_t increment) {
    uintptr_t current;
    uintptr_t next;
    size_t pages;
    size_t old_pages;

    if (wasm_brk == NULL) {
        current = (uintptr_t)__heap_base;
        current = (current + 15) & ~(uintptr_t)15;
        wasm_brk = (unsigned char *)current;
    }

    current = (uintptr_t)wasm_brk;
    if (increment == 0) {
        return wasm_brk;
    }
    if (increment < 0 || (uintptr_t)increment > UINTPTR_MAX - current) {
        return (void *)-1;
    }

    next = current + (uintptr_t)increment;
    old_pages = __builtin_wasm_memory_size(0);
    if (next > (uintptr_t)old_pages * 65536) {
        pages = (next - ((uintptr_t)old_pages * 65536) + 65535) / 65536;
        if (__builtin_wasm_memory_grow(0, pages) == (size_t)-1) {
            return (void *)-1;
        }
    }

    wasm_brk = (unsigned char *)next;
    return (void *)current;
}

#define DLMALLOC_EXPORT static
#define HAVE_MMAP 0
#define HAVE_MORECORE 1
#define MORECORE wasm_sbrk
#define UNSIGNED_MORECORE 1
#define MORECORE_CANNOT_TRIM 1
#define LACKS_UNISTD_H
#define LACKS_SYS_PARAM_H
#define LACKS_SYS_MMAN_H
#define LACKS_SYS_TYPES_H
#define LACKS_ERRNO_H
#define LACKS_SCHED_H
#define LACKS_TIME_H
#define LACKS_STDLIB_H
#define LACKS_STRING_H
#define LACKS_STRINGS_H
#define NO_MALLOC_STATS 1
#define NO_MALLINFO 1
#define USE_LOCKS 0
#define MALLOC_ALIGNMENT 16
#define EINVAL 22
#define ENOMEM 12
#define ABORT margarineAbort(1)
#define MALLOC_FAILURE_ACTION margarineAbort(1)

void *memset(void *destination, int value, size_t len) {
    unsigned char *bytes = destination;
    while (len-- != 0) {
        *bytes++ = (unsigned char)value;
    }
    return destination;
}

void *memcpy(void *destination, const void *source, size_t len) {
    unsigned char *to = destination;
    const unsigned char *from = source;
    while (len-- != 0) {
        *to++ = *from++;
    }
    return destination;
}

#include "dlmalloc.c"

void *malloc(size_t size) { return dlmalloc(size); }
void free(void *ptr) { dlfree(ptr); }
void *calloc(size_t count, size_t size) { return dlcalloc(count, size); }
void *realloc(void *ptr, size_t size) { return dlrealloc(ptr, size); }
