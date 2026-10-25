#include <assert.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>

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
struct CallbackBox {
    int64_t (*callback)(int64_t, int64_t);
};


struct Large {
    int64_t a;
    int64_t b;
    int64_t c;
};

struct Bytes12 {
    uint8_t data[12];
};

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

struct RecursiveCallback {
    int64_t (*callback)(struct RecursiveCallback);
};

// Prototypes for Margarine exports
extern int64_t mar_test_scalar(int64_t a, int64_t b);
extern int64_t mar_test_duplicate_import(int64_t a, int64_t b);
extern int64_t mar_test_imported_definition(int64_t a, int64_t b);
extern int64_t mar_test_defined_symbol(int64_t a, int64_t b);
extern int64_t mar_test_indirect_call(int64_t a, int64_t b);
extern int64_t mar_test_c_callback(int64_t a, int64_t b);
extern int64_t mar_test_c_callback_inout(int64_t a, int64_t b);
extern int64_t mar_test_nested_c_callback(int64_t a, int64_t b);
extern double mar_test_float(double value);
extern bool mar_test_bool(bool b);
extern uint8_t mar_test_byte(uint8_t value);
extern int64_t mar_test_tiny(struct Tiny t);
extern int64_t mar_test_pair(struct Pair p);
extern struct Pair mar_test_pair_return(struct Pair p);
extern double mar_test_mixed(struct Mixed m);
extern double mar_test_hfa(struct HFA2 h);
extern struct Large mar_test_large(struct Large l);
extern struct Large mar_test_large_explicit_return(struct Large l);
extern struct Bytes12 mar_test_bytes12(struct Bytes12 value);
extern void mar_test_inout(int64_t *val);
extern void mar_test_bool_inout(bool *value);
extern bool mar_test_bool_inout_import(bool value);
extern struct Flags mar_test_flags(struct Flags value);
extern void mar_test_flags_inout(struct Flags *value);
extern struct Flags mar_test_bool_tuple(struct Flags value);
extern struct LargeFlags mar_test_large_flags(struct LargeFlags value);
extern int64_t mar_test_recursive_callback(struct RecursiveCallback value);

static int64_t recursive_callback(struct RecursiveCallback value) {
    return value.callback == recursive_callback ? 73 : -1;
}


int main(void) {
    // 1. Scalar
    int64_t sum = mar_test_scalar(15, 27);
    assert(sum == 42);
    assert(mar_test_duplicate_import(15, 27) == 84);
    assert(mar_test_imported_definition(15, 27) == 57);
    assert(mar_test_defined_symbol(15, 27) == 57);
    assert(mar_test_indirect_call(15, 27) == 42);
    assert(mar_test_c_callback(15, 27) == 42);
    assert(mar_test_c_callback_inout(15, 27) == -12);
    assert(mar_test_nested_c_callback(15, 27) == 30);

    // 2. Bool
    bool b = mar_test_bool(true);
    assert(b == false);
    bool b2 = mar_test_bool(false);
    assert(b2 == true);
    assert(mar_test_byte(254) == 255);

    struct { bool value; uint8_t canary[7]; } bool_slot =
        { false, { 0xa5, 0xa5, 0xa5, 0xa5, 0xa5, 0xa5, 0xa5 } };
    mar_test_bool_inout(&bool_slot.value);
    assert(bool_slot.value);
    for (int index = 0; index < 7; index++) {
        assert(bool_slot.canary[index] == 0xa5);
    }
    assert(mar_test_bool_inout_import(false));
    assert(!mar_test_bool_inout_import(true));

    struct Flags flipped = mar_test_flags((struct Flags){ false, true });
    assert(flipped.a && !flipped.b);
    struct Flags tuple = mar_test_bool_tuple((struct Flags){ false, true });
    assert(tuple.a && !tuple.b);
    struct { struct Flags value; uint8_t canary[6]; } flags_slot =
        { { false, true }, { 0xa5, 0xa5, 0xa5, 0xa5, 0xa5, 0xa5 } };
    mar_test_flags_inout(&flags_slot.value);
    assert(flags_slot.value.a && !flags_slot.value.b);
    for (int index = 0; index < 6; index++) {
        assert(flags_slot.canary[index] == 0xa5);
    }

    struct LargeFlags large_flags = { 11, { 0x31, { false, true }, 0x72 }, 29 };
    struct LargeFlags large_flags_result = mar_test_large_flags(large_flags);
    assert(large_flags_result.first == 11 && large_flags_result.last == 29);
    assert(large_flags_result.nested.prefix == 0x31);
    assert(large_flags_result.nested.flags.a && !large_flags_result.nested.flags.b);
    assert(large_flags_result.nested.suffix == 0x72);
    assert(mar_test_recursive_callback((struct RecursiveCallback){ recursive_callback }) == 73);

    // 3. Small aggregate (<=8 bytes)
    struct Tiny tiny = { 73 };
    int64_t tiny_value = mar_test_tiny(tiny);
    assert(tiny_value == 73);

    // 4. Scalar f64
    double scaled = mar_test_float(2.5);
    assert(scaled == 5.0);

    // 5. Pair
    struct Pair p = { 100, 200 };
    int64_t p_sum = mar_test_pair(p);
    assert(p_sum == 300);
    struct Pair p_return = mar_test_pair_return(p);
    assert(p_return.a == 101);
    assert(p_return.b == 202);

    // 6. Mixed
    struct Mixed m = { 50, 2.5 };
    double m_res = mar_test_mixed(m);
    assert(m_res == 52.5);

    // 7. HFA
    struct HFA2 h = { 3.14, 2.71 };
    double h_res = mar_test_hfa(h);
    assert(h_res > 5.84 && h_res < 5.86);

    // 8. Large aggregate (>16 bytes)
    struct Large l = { 1, 2, 3 };
    struct Large l_res = mar_test_large(l);
    assert(l_res.a == 10);
    assert(l_res.b == 20);
    assert(l_res.c == 30);
    struct Large l_explicit_res = mar_test_large_explicit_return(l);
    assert(l_explicit_res.a == 1);
    assert(l_explicit_res.b == 2);
    assert(l_explicit_res.c == 3);
    struct Bytes12 bytes12 = { { 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11 } };
    struct Bytes12 bytes12_result = mar_test_bytes12(bytes12);
    for (int index = 0; index < 12; index++) {
        assert(bytes12_result.data[index] == index + 1);
    }

    // 9. Inout pointer
    int64_t val = 50;
    mar_test_inout(&val);
    assert(val == 150);
    puts("All C ABI bidirectional tests passed successfully!");


    return 0;
}
