#include <stdbool.h>
#include <stdint.h>

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

// C Oracle declarations and definitions for signature comparisons

int64_t oracle_add_integers(int64_t a, int64_t b) {
    return a + b;
}

bool oracle_flip_bool(bool b) {
    return !b;
}

int64_t oracle_pair_sum(struct Pair p) {
    return p.a + p.b;
}

double oracle_mixed_eval(struct Mixed m) {
    return (double)m.a + m.b;
}

double oracle_hfa_sum(struct HFA2 h) {
    return h.a + h.b;
}

struct Large oracle_large_eval(struct Large l) {
    struct Large res;
    res.a = l.a + 1;
    res.b = l.b + 2;
    res.c = l.c + 3;
    return res;
}

void oracle_mutate_inout(int64_t *val) {
    *val += 42;
}
