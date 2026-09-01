#ifndef MARGARINE_RUNTIME_H
#define MARGARINE_RUNTIME_H

#include <stddef.h>
#include <stdint.h>

/* Public aggregate layouts mirror the compiler's collectionType/strType
 * definitions. Lengths stay fixed-width so the C ABI is target-independent. */
typedef struct {
    uint8_t *ptr;
    int64_t len;
} MargarineCollection;

typedef struct {
    MargarineCollection value;
} MargarineString;

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

/* Implemented by core. */
_Noreturn void margarineAbort(int32_t code);
MargarineString margarineStringFromUtf8(const uint8_t *bytes, size_t len);
void *margarineAlloc(size_t size);
uint8_t *margarineRcAlloc(size_t total_size);
void margarineDealloc(uint8_t *ptr, size_t size);

/* Implemented by std and called by generated native entry points. */
void margarineSetEnvArgs(int32_t argc, char **argv);

#endif
