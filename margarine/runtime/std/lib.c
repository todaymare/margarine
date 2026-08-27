#include "margarine.h"

#if !defined(__wasm__)
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <errno.h>
#include <sys/wait.h>
#include <time.h>

#if defined(__APPLE__)
#include <crt_externs.h>
#endif
#endif

#if defined(__wasm__)
__attribute__((import_module("env")))
__attribute__((import_name("host_print")))
void host_print(const uint8_t *ptr, size_t len);

/* Writes the shortest useful decimal representation of value as UTF-8 into
 * out. Returns the byte length, or -1 when capacity is insufficient. */
__attribute__((import_module("env")))
__attribute__((import_name("host_float_to_str")))
int32_t host_float_to_str(double value, uint8_t *out, int32_t capacity);

__attribute__((import_module("env")))
__attribute__((import_name("host_random_int")))
int64_t host_random_int(void);

__attribute__((import_module("env")))
__attribute__((import_name("host_random_float")))
double host_random_float(void);

__attribute__((import_module("env")))
__attribute__((import_name("host_now_secs")))
int64_t host_now_secs(void);

__attribute__((import_module("env")))
__attribute__((import_name("host_now_nanos")))
int64_t host_now_nanos(void);
#endif

MARGARINE_DEFINE_OPTION(int64_t);
MARGARINE_DEFINE_OPTION(MargarineStr);

static MargarineStr string_from_bytes(const uint8_t *bytes, size_t len) {
    return margarineStringFromUtf8(bytes, len);
}

#if !defined(__wasm__)
static MargarineStr string_from_cstr(const char *value) {
    return string_from_bytes((const uint8_t *)value, strlen(value));
}

static MARGARINE_OPTION(MargarineStr) result_ok(MargarineStr value) {
    return (MARGARINE_OPTION(MargarineStr)){
        .tag = MARGARINE_SOME,
        .data = value,
    };
}

static MARGARINE_OPTION(MargarineStr) result_err(const char *message) {
    return (MARGARINE_OPTION(MargarineStr)){
        .tag = MARGARINE_NONE,
        .data = string_from_cstr(message),
    };
}
#endif

void print_byte(uint8_t value) {
#if defined(__wasm__)
    host_print(&value, 1);
#else
    fputc((int)value, stdout);
    fflush(stdout);
#endif
}

void eprint_byte(uint8_t value) {
#if defined(__wasm__)
    host_print(&value, 1);
#else
    fputc((int)value, stderr);
    fflush(stderr);
#endif
}

MargarineStr int_to_str(int64_t value) {
    uint8_t bytes[20];
    size_t start = sizeof(bytes);
    uint64_t magnitude = value < 0 ? 0 - (uint64_t)value : (uint64_t)value;

    do {
        bytes[--start] = (uint8_t)('0' + magnitude % 10);
        magnitude /= 10;
    } while (magnitude != 0);

    if (value < 0) bytes[--start] = '-';
    return string_from_bytes(bytes + start, sizeof(bytes) - start);
}

#if defined(__wasm__)
MargarineStr float_to_str(double value) {
    uint8_t bytes[64];
    int32_t len = host_float_to_str(value, bytes, (int32_t)sizeof(bytes));
    if (len < 0 || (size_t)len > sizeof(bytes)) {
        static const uint8_t message[] = "<float format error>";
        return string_from_bytes(message, sizeof(message) - 1);
    }
    return string_from_bytes(bytes, (size_t)len);
}
#else
MargarineStr float_to_str(double value) {
    char bytes[64];
    int len = snprintf(bytes, sizeof(bytes), "%.17g", value);
    if (len < 0) return string_from_cstr("<float format error>");
    if ((size_t)len >= sizeof(bytes)) return string_from_cstr("<float format error>");
    return string_from_bytes((const uint8_t *)bytes, (size_t)len);
}
#endif

#if !defined(__wasm__)
static size_t checked_length(int64_t len) {
    if (len < 0 || (uint64_t)len > SIZE_MAX - 1) margarineAbort(1);
    return (size_t)len;
}

static char *cstring_from_bytes(const uint8_t *bytes, int64_t len) {
    size_t length = checked_length(len);
    char *result = margarineAlloc(length + 1);
    if (length != 0) memcpy(result, bytes, length);
    result[length] = '\0';
    return result;
}

int64_t margarineSpawn(const uint8_t *bytes, int64_t len) {
    char *command = cstring_from_bytes(bytes, len);
    int status = system(command);
    margarineDealloc((uint8_t *)command, 0);

    if (status == -1) return -1;
    if (WIFEXITED(status)) return (int64_t)WEXITSTATUS(status);
    if (WIFSIGNALED(status)) return -(int64_t)WTERMSIG(status);
    return -1;
}

MARGARINE_OPTION(MargarineStr) margarineEnvVariable(
    const uint8_t *bytes,
    int64_t len
) {
    char *name = cstring_from_bytes(bytes, len);
    const char *value = getenv(name);
    margarineDealloc((uint8_t *)name, 0);
    return value == NULL ? result_err("environment variable not found")
                         : result_ok(string_from_cstr(value));
}

MargarineCollection margarineEnvArgs(void) {
    char **args;
#if defined(__APPLE__)
    args = *_NSGetArgv();
#else
    args = NULL;
#endif
    size_t len = 0;
    while (args != NULL && args[len] != NULL) ++len;

    size_t total_size = sizeof(size_t) + len * sizeof(MargarineStr);
    uint8_t *allocation = margarineRcAlloc(total_size);
    MargarineStr *values = (MargarineStr *)(allocation + sizeof(size_t));
    for (size_t index = 0; index < len; ++index) {
        values[index] = string_from_cstr(args[index]);
    }

    return (MargarineCollection){
        .ptr = allocation,
        .len = len,
    };
}

MARGARINE_OPTION(MargarineStr) io_read_file(
    const uint8_t *path_bytes,
    int64_t path_len
) {
    char *file_path = cstring_from_bytes(path_bytes, path_len);
    FILE *file = fopen(file_path, "rb");
    margarineDealloc((uint8_t *)file_path, 0);
    if (file == NULL) return result_err(strerror(errno));

    if (fseek(file, 0, SEEK_END) != 0) {
        const char *message = strerror(errno);
        fclose(file);
        return result_err(message);
    }
    long size = ftell(file);
    if (size < 0 || fseek(file, 0, SEEK_SET) != 0) {
        const char *message = strerror(errno);
        fclose(file);
        return result_err(message);
    }

    uint8_t *bytes = margarineAlloc((size_t)size);
    size_t read = fread(bytes, 1, (size_t)size, file);
    if (read != (size_t)size && ferror(file)) {
        const char *message = strerror(errno);
        fclose(file);
        margarineDealloc(bytes, (size_t)size);
        return result_err(message);
    }
    fclose(file);

    MargarineStr result = string_from_bytes(bytes, read);
    margarineDealloc(bytes, (size_t)size);
    return result_ok(result);
}

static MARGARINE_OPTION(MargarineStr) result_ok_unit(void) {
    return (MARGARINE_OPTION(MargarineStr)){
        .tag = MARGARINE_SOME,
        .data = { .value = { .ptr = NULL, .len = 0 } },
    };
}

MARGARINE_OPTION(MargarineStr) io_write_file(
    const uint8_t *path_bytes,
    int64_t path_len,
    const uint8_t *bytes,
    int64_t len
) {
    size_t write_len = checked_length(len);
    char *file_path = cstring_from_bytes(path_bytes, path_len);
    FILE *file = fopen(file_path, "wb");
    margarineDealloc((uint8_t *)file_path, 0);
    if (file == NULL) return result_err(strerror(errno));

    if (write_len != 0 && fwrite(bytes, 1, write_len, file) != write_len) {
        const char *message = ferror(file) ? strerror(errno) : "short write";
        MARGARINE_OPTION(MargarineStr) result = result_err(message);
        fclose(file);
        return result;
    }
    if (fclose(file) != 0) return result_err(strerror(errno));
    return result_ok_unit();
}

MARGARINE_OPTION(MargarineStr) io_read_line(void) {
    size_t capacity = 128;
    size_t len = 0;
    uint8_t *bytes = margarineAlloc(capacity);
    int byte;

    while ((byte = fgetc(stdin)) != EOF) {
        if (len == capacity) {
            size_t next_capacity = capacity * 2;
            uint8_t *next = margarineAlloc(next_capacity);
            memcpy(next, bytes, len);
            margarineDealloc(bytes, capacity);
            bytes = next;
            capacity = next_capacity;
        }
        bytes[len++] = (uint8_t)byte;
        if (byte == '\n') break;
    }

    if (ferror(stdin)) {
        const char *message = strerror(errno);
        margarineDealloc(bytes, capacity);
        return result_err(message);
    }
    if (len == 0 && byte == EOF) {
        margarineDealloc(bytes, capacity);
        return result_err("EOF");
    }

    MargarineStr result = string_from_bytes(bytes, len);
    margarineDealloc(bytes, capacity);
    return result_ok(result);
}
#endif

#if defined(__wasm__)
int64_t random_int(void) {
    return host_random_int();
}

double random_float(void) {
    return host_random_float();
}

int64_t now_secs(void) {
    return host_now_secs();
}

int64_t now_nanos(void) {
    return host_now_nanos();
}
#else
static uint64_t native_random_u64(void) {
    uint64_t value;
    arc4random_buf(&value, sizeof(value));
    return value;
}

int64_t random_int(void) {
    return (int64_t)native_random_u64();
}

double random_float(void) {
    return (double)(native_random_u64() >> 11) * (1.0 / 9007199254740992.0);
}

int64_t now_secs(void) {
    struct timespec now;
    if (clock_gettime(CLOCK_REALTIME, &now) != 0) return 0;
    return (int64_t)now.tv_sec;
}

int64_t now_nanos(void) {
    struct timespec now;
    if (clock_gettime(CLOCK_REALTIME, &now) != 0) return 0;
    return (int64_t)now.tv_nsec;
}
#endif



MARGARINE_OPTION(MargarineStr) str_from_codepoint(int64_t codepoint) {
    uint8_t bytes[4];
    size_t len;

    if (codepoint < 0 || codepoint > 0x10ffff ||
        (codepoint >= 0xd800 && codepoint <= 0xdfff)) {
        return (MARGARINE_OPTION(MargarineStr)){
            .tag = MARGARINE_NONE,
            .data = { .value = { .ptr = NULL, .len = 0 } },
        };
    }

    if (codepoint <= 0x7f) {
        bytes[0] = (uint8_t)codepoint;
        len = 1;
    } else if (codepoint <= 0x7ff) {
        bytes[0] = 0xc0 | (uint8_t)(codepoint >> 6);
        bytes[1] = 0x80 | (uint8_t)(codepoint & 0x3f);
        len = 2;
    } else if (codepoint <= 0xffff) {
        bytes[0] = 0xe0 | (uint8_t)(codepoint >> 12);
        bytes[1] = 0x80 | (uint8_t)((codepoint >> 6) & 0x3f);
        bytes[2] = 0x80 | (uint8_t)(codepoint & 0x3f);
        len = 3;
    } else {
        bytes[0] = 0xf0 | (uint8_t)(codepoint >> 18);
        bytes[1] = 0x80 | (uint8_t)((codepoint >> 12) & 0x3f);
        bytes[2] = 0x80 | (uint8_t)((codepoint >> 6) & 0x3f);
        bytes[3] = 0x80 | (uint8_t)(codepoint & 0x3f);
        len = 4;
    }

    return (MARGARINE_OPTION(MargarineStr)){
        .tag = MARGARINE_SOME,
        .data = margarineStringFromUtf8(bytes, len),
    };
}
