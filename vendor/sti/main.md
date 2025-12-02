
- todo:
    - add missing `PhantomData`s for dropck.
    - simple fmt replacement.
        - without result. (impls can set error flag & ignore)
        - with stack buffer for speed.
        - `write!` equiv just generates inline code that does what the fmt string says.
        - would like ability to nest.
            - eg in `s.push_fmt(fmt!("hey {}", ",".join(foo.map(|_| { ...; fmt!(...) }))))`,
              `map` can't return a `fmt!` referencing locals.
            - unclear how you'd make that work.
            - maybe by putting all the logic into the `write` macro.
    - box as ptr.
    - s/boks/boxed.
    - s/raw parts/raw.
    - rc from/into raw, hide inner.
    - rc s/try mut/get mut.
    - custom arc impl.
    - custom lock impls.
        - based on platform futex & spin locks.
        - rwlock `write_with_attitude`, `write_timid`.
        - should we just expose `Futex` instead of `Mutex`?
          not sure we really need 2 locks for condvar use cases.
          i mean, we could expose futex and condvar. then we can still
          support non-futex platforms, in theory.
        - panic abort in guards.
    - remove spin locks.
    - fix rwspinlock sync impl.
    - mimalloc for real this time.
    - nonblocking file io?
    - stdio.
    - docs:
        - give examples for kvec & arena.
        - fix `enclose!` docs.


### backlog:

- sync:
    - improve rwspinlock test.

- cell & unck docs.
    - why we have them, what they're for.
    - how to use them correctly.

- arena:
    - remove max align.
    - vm alloc.
    - large allocations -> globalalloc.
    - stats isize, sub, include total used.
    - test temp arena overflow case.
    - consider returning growing arena for temp overflow case.

- leb128:
    - single byte fast path inline fns.
    - writing.
    - tests.

- kslice index proper panic & track caller.

- str::Repeat.

- mimalloc.

- utf-8 module:
    - ceil/floor.
    - iteration.
    - consider removing `_inline` versions.
      perhaps maybe have short string opt inline versions.

- simd:
    - scalar add/sub (ext trait, "add", "sub", maybe "sadd").
    - scalar into, ext for "v2", "v4".
    - impl aarch64.
    - platform detection.
    - impl x86.
        - sse4 & error.
    - use `core::simd` on nightly.
    - make simd opt-in on x86 and warn if not enabled sse4.
    - simd `x8` vectors using `x4`.
        - should get use avx, if available.

- hash map.
    - test non-allocation for non-inserting funcs.
        - full test for `get_or_insert`, fn to test both versions (k/not).
    - test key mutation.
    - rename `empty` to something less misleading.
    - slot api for ptr eq keys.


