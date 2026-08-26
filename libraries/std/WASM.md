# WASM standard-library host ABI

The WASM standard library implements integer formatting locally. It imports
the following functions from the `env` module:

## `host_float_to_str`

Signature: `(f64 value, i32 out_ptr, i32 capacity) -> i32`

Write the shortest useful UTF-8 decimal representation of `value` into the
WASM linear-memory range `[out_ptr, out_ptr + capacity)`. Return the number of
bytes written. Return `-1` if the representation does not fit. The caller
copies the bytes into a managed string immediately, so the buffer is only
needed for the duration of the call. NaN and infinities should be represented
consistently with JavaScript's normal numeric string conversion.

## `host_random_int`

Signature: `() -> i64`

Return a uniformly distributed 64-bit signed integer. It may be seeded from
the host's normal secure or pseudo-random source, but must not trap.

## `host_random_float`

Signature: `() -> f64`

Return a finite uniformly distributed value in the half-open interval
`[0.0, 1.0)`. This is used by `random_range` and must not return `1.0`.

Environment access, filesystem I/O, and command spawning are currently omitted
from the WASM stdlib surface and are cfg-gated out.

## `host_now_secs` and `host_now_nanos`

Signatures: `() -> i64`

Return the current Unix wall-clock time as seconds and nanoseconds. The
nanosecond result is the sub-second component in `[0, 1_000_000_000)`. Both
values should represent the same instant closely enough for `Duration::now()`;
the standard library uses them to implement `Duration::elapsed()`.
