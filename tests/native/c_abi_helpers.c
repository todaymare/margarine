#include <stdbool.h>
#include <stdint.h>

struct Tiny {
    int64_t value;
};

struct Pair {
    int64_t a;
    int64_t b;
};

struct Mixed {
    int64_t a;
    double b;
};

struct HFA2 {
    double a;
    double b;
};

struct Large {
    int64_t a;
    int64_t b;
    int64_t c;
};

struct Bytes12 {
    uint8_t data[12];
};

typedef int64_t (*BinaryFn)(int64_t, int64_t);
struct CallbackBox {
    BinaryFn callback;
};



int64_t c_helper_add_integers(int64_t a, int64_t b) {
    return a + b;
}

int64_t c_helper_subtract_integers(int64_t a, int64_t b) {
    return a - b;
}

int64_t c_helper_call_binary(BinaryFn callback, int64_t a, int64_t b) {
    return callback(a, b);
}

BinaryFn c_helper_identity_binary(BinaryFn callback) {
    return callback;
}

void c_helper_replace_binary(BinaryFn *callback) {
    *callback = c_helper_subtract_integers;
}

int64_t c_helper_call_box(struct CallbackBox callbacks, int64_t a, int64_t b) {
    return callbacks.callback(a, b);
}

struct CallbackBox c_helper_make_box(BinaryFn callback) {
    return (struct CallbackBox){ callback };
}

void c_helper_replace_box(struct CallbackBox *callbacks) {
    callbacks->callback = c_helper_subtract_integers;
}

double c_helper_scale_float(double value) {
    return value * 2.0;
}

uint8_t c_helper_increment_byte(uint8_t value) {
    return value + 1;
}

int64_t c_helper_tiny_value(struct Tiny t) {
    return t.value;
}

bool c_helper_flip_bool(bool b) {
    return !b;
}

int64_t c_helper_pair_sum(struct Pair p) {
    return p.a + p.b;
}

struct Pair c_helper_pair_shift(struct Pair p) {
    struct Pair result = { p.a + 1, p.b + 2 };
    return result;
}

struct Bytes12 c_helper_bytes12_increment(struct Bytes12 value) {
    for (int index = 0; index < 12; index++) {
        value.data[index] += 1;
    }
    return value;
}

double c_helper_mixed_eval(struct Mixed m) {
    return (double)m.a + m.b;
}

double c_helper_hfa_sum(struct HFA2 h) {
    return h.a + h.b;
}

struct Large c_helper_large_eval(struct Large l) {
    struct Large res;
    res.a = l.a * 10;
    res.b = l.b * 10;
    res.c = l.c * 10;
    return res;
}

void c_helper_mutate_inout(int64_t *val) {
    *val += 100;
}

struct Flags {
    bool a;
    bool b;
};

struct FlagsEnvelope {
    uint8_t prefix;
    struct Flags flags;
    uint8_t suffix;
};

struct LargeFlags {
    int64_t first;
    struct FlagsEnvelope nested;
    int64_t last;
};

void c_helper_mutate_bool(bool *value) {
    *value = !*value;
}

struct Flags c_helper_flags(struct Flags value) {
    return (struct Flags){ !value.a, !value.b };
}

void c_helper_flags_inout(struct Flags *value) {
    value->b = !value->b;
}

struct LargeFlags c_helper_large_flags(struct LargeFlags value) {
    value.nested.flags = c_helper_flags(value.nested.flags);
    return value;
}
