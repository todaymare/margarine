#ifndef MARGARINE_STD_NATIVE_H
#define MARGARINE_STD_NATIVE_H

#include <stddef.h>
#include <stdint.h>

/* Core runtime layouts used by native std functions. */
typedef struct {
    uint8_t *ptr;
    size_t len;
} MargarineCollection;

typedef struct {
    MargarineCollection value;
} MargarineStr;

/* Generates a concrete enum layout, allowing C to apply the correct payload
 * padding and alignment. */
#define MARGARINE_ENUM(name, value_type) \
    typedef struct { \
        uint32_t tag; \
        value_type data; \
    } name

#define MARGARINE_JOIN_(left, right) left##right
#define MARGARINE_JOIN(left, right) MARGARINE_JOIN_(left, right)

/* Option<T> is an enum whose tag 0 is some and tag 1 is none. These macros
 * allow MARGARINE_OPTION(T) wherever a named C type is required. */
#define MARGARINE_OPTION(value_type) MARGARINE_JOIN(MargarineOption_, value_type)
#define MARGARINE_DEFINE_OPTION(value_type) \
    MARGARINE_ENUM(MARGARINE_OPTION(value_type), value_type)

#define MARGARINE_SOME 0u
#define MARGARINE_NONE 1u

/* Implemented by core's native layer. */
_Noreturn void margarineAbort(int32_t code);

MargarineStr margarineStringFromUtf8(const uint8_t *bytes, size_t len);
/* Native string inputs use a borrowed byte pointer and explicit length. */
void *margarineAlloc(size_t size);
uint8_t *margarineRcAlloc(size_t total_size);
void margarineDealloc(uint8_t *ptr, size_t size);

#endif
