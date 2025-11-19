0.7.0:
- add `fmt!`.
- make `write!` use `core::fmt::Write` directly, so `use`ing `Write` is no longer necessary.

0.6.0:
- add `sync::{arc, mutex, rwlock, condvar}`.
- add `unsize_arc!`.
- add `Box::into_inner`.
- add `future`.
- add re-exports for `core::{slice, ops, fmt, pin, str}`.

0.5.0:
- remove `ManualVec`.
- remove `OrdUtils`.
- remove `mem::Align*`.
- remove `UnwrapDebug` & `ExpectDebug`.
- reexport common unsafe primitives in `mem`.
- deprecate `Arena::new_in`.
- make `enclose` more flexible (`ident = expr`).
- bring back `prelude`.
- add `Box::{into, from}_raw_parts_in`.
- add `unsize_box!`, `unsize_rc!`.
- deprecate `float` module for now.

0.4.0:
- arena pool rework:
    - new api: `Arena::tls_get_temp`, `Arena::tls_get_rec`, `Arena::tls_rec_num_refs`.
    - remove unsafe scoped api.
    - temp arenas are pooled per thread. rec arena is exactly one per thread.
      this is effectively how it worked previously too. but now the impls are separated
      and much simpler.
- `Vec::clone{_in}` now guarantees `clone.len() == clone.cap()`.
- impl `Box, VecDeque: Send + Sync`.
- impl `PackedOption: Eq + Hash`

0.3.0:
- add `borrow::{BorrowFlag, BorrowRef, BorrowRefMut, Ref, RefMut}`.
- add `cell::RefCell` & `unck::cell::RefCellUnck`.
- add `unck::shared_box::{SharedBoxUnck, SharedPtrUnck}`.
- add `ExpectDebug`.
- add `Vec: Hash + Write`.
- add `mod leb128`.

0.2.1:
- now bumping minor for features & changes, patch for fixes.
- `trait Alloc` is now unsafe (`Clone` requirement).
- breaking change: `Vec::from_{value, fn}` now take the `len` before the value/fn.
- add `mem::Align{1, 2, 4, 8, 16, 32, 64}`.
- turn `Into` impls into `From` impls.
- simd `INF` & `NAN` consts.
- simd `Default` impls.
- add `erase`, `inc`, `enclose` macros.
- replace `Rc::cast` with `{from, into}_inner`.
- remove `prelude`.

0.1.11:
- add `UnwrapDebug`.
- add `ManualVec`.
- remove `AllocError`.
- rename `reserve_exact` -> `reserve_exactly` (to disambiguate from `reserve_extra`).
- `PackedOption::take` now returns `PackedOption` instead of `Option`.
- add `KGenVec`.

0.1.10:
- `Arena: (Ref)UnwindSafe`.
- add `RwSpinLock`.
- add `KVec::push_with`.
- `KFreeVec`:
    - now returns values on free.
    - add `retain`.
    - add `next_key`, `alloc_with`.
    - add `get`, `get_mut`.
- `impl PartialEq, Eq for Vec, KVec, KSlice`.
- add `VecDeque`.
- `HashMap`:
    - add `retain`.
    - rename `remove` -> `remove_with_key`.
    - rename `remove_value` -> `remove`.
    - add `reserve`.
    - `impl Default`.
    - add `iter_mut`.
- `Vec`:
    - add `remove_swap`.
    - add `insert/_from_slice`.
    - rename `grow_by` -> `reserve_extra`.
    - remove oom fallible methods.
    - add `vec_extend` macro.


0.1.9:
- `HashMap`: add `clear` and `Debug`.
- improved `ArenaPool`.
- add `StaticVec`.
- add `Box`.
- add `FromIn`, `CopyIt`, `MapIt` traits.
- `Vec`:
    - add `vec/_in` macros.
    - add `From/In<I: Iterator>`.
    - add `Extend`.
- major breaking change: alloc params go first now.


0.1.8:
- (breaking change): `cat_next` now takes a length, not a size in bytes.
- new `hash` module:
    - `fxhash` implementation.
    - `HashFn` & `HashFnSeed` traits.
    - `HashMap`.
- new `hint` module:
    - `cold`, `likely`, `unlikely`.
- `GlobalAlloc` is `Send + Sync` now.
- `Reader` is now `Debug`.


0.1.7:
- arena:
    - add save/restore api.
- add `SpinLock`.
- add `ArenaPool`.
- add `prelude`.


0.1.6:
- arena:
    - rename `GrowingArena` -> `Arena`.
    - arena no longer rounds up allocation sizes to `MAX_ALIGN`.
    - `MAX_ALIGN` is now 32.
    - add `stats()`, which returns usage statistics.
    - add `reset_all()`, which frees all arena blocks.
    - add `alloc_str()`.
- vec:
    - make `leak` a method.
    - add `move_into()`.


0.1.5:
- slice reader:
    - `as_slice`, cause deref doesn't return `&'a [T]`.
    - `consume_while_slice[_from]` now return `(&'a [T], bool)`.
    - `consumed_slice`, opposite of `as_slice`.
    - fix `offset` for types of size greater than 1.


0.1.4:
- slice reader.
    - rewrite to be more useful.
    - support non-`Copy` types.


0.1.3:
- simd module.
    - rewrite to support abstracting over lanes (`Simd<T, N>`).
    - scalar impl for now.
    - stop failing to build on `x86`.


