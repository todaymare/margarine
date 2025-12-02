# STd Improved

common algorithms & data structures for rust.

highlights:
- [allocator api](https://docs.rs/sti/latest/sti/alloc/trait.Alloc.html) and
  various data structures that support it (including `Vec`, `HashMap`, `Box`, `Rc`).
- [arena allocation](https://docs.rs/sti/latest/sti/arena/struct.Arena.html) with thread
  local temp arenas.
- [typed indices](https://docs.rs/sti/latest/sti/macro.define_key.html) and
  [index typed vecs](https://docs.rs/sti/latest/sti/keyed/struct.KVec.html).
- [a slice reader](https://docs.rs/sti/latest/sti/reader/struct.Reader.html) for parsing.


disclaimer:
- even though most things are tested, you may still run into bugs.
- the library is under active development and breaking changes are likely.


[change-log](changelog.md)

