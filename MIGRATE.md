# Migrating

## 0.3 -> 0.4

`unsafe_marker!` couldn't set the `SafeHolder` parameters (`WRITE_ONCE`,
`READ_ONCE`, `PERMANENT`), attach derives or other attributes, or carry data,
and keeping a marker on one thread needed a second macro. The new syntax does
all of that in one macro, and a marker without bounds is now as restrictive as
a `SafeHolder` can be: `WRITE_ONCE = true`, `READ_ONCE = true`,
`PERMANENT = false`.

- **`unsafe_marker!(pub Name)`** is deprecated. Write `struct` before the name:
  `unsafe_marker!(pub struct Name: readable);`. The old form created a
  `readable` marker (`READ_ONCE = false`). If you don't need that, leave out
  `: readable` and the marker becomes read-once.
- **`unsafe_marker_no_send!(pub Name)`** is deprecated. Write
  `unsafe_marker!(pub struct Name: readable + !Send + !Sync);` instead.
- **Bounds** go after a colon: `readable`, `writable`, `permanent`,
  `!Send + !Sync` or `!Sync`, e.g.
  `unsafe_marker!(pub struct CoreCount(u32): permanent + !Sync);`. See the
  docs of `unsafe_marker!` for what each one does.
- **Markers can carry data**: `pub struct CoreCount(u32)` vouches for a `u32`,
  `pub struct Region(usize, usize)` for a `(usize, usize)`. They are created
  with `vouch_for(data)`, and `take()`, `set()` and `Deref` work with the data
  itself.
- **Attributes** are now accepted: doc comments go on the marker, lint
  attributes and `#[cfg]` on everything it generates.

### Your own types need `#[derive(MarkerData)]`

**Breaking:** everything a `SafeHolder` holds has to implement the new
`MarkerData` trait. It is what lets markers hand out their data rather than
the hidden struct that keeps them apart.

- For your own types, derive it: `#[derive(MarkerData)]`, with
  `use safevalue::MarkerData;`.
- It is already implemented for the common types from `core`: integers,
  `bool`, `char`, `()`, references, pointers, `NonNull`, arrays, tuples,
  `Option`, `Result`, ...
- For `String`, `Vec`, `Box`, `Rc`, `Arc` and the other types from `alloc`,
  enable the `alloc` feature. For `HashMap`, `HashSet`, `PathBuf` and
  `OsString`, enable the `std` feature (which includes `alloc`).
- Generic code over `SafeHolder<T, ...>` needs a `T: MarkerData` bound.
- `vouch_for()` is no longer a `const fn`, since it goes through the trait.
  `vouch()`, for markers without data, still is.

### Traits that need `READ_ONCE = false`

The following traits are only implemented for a `SafeHolder` (and so for a
marker) with `READ_ONCE = false`, i.e. `readable` or `permanent` markers. They
also need the data (`MarkerData::Data`, for your own types the type itself) to
implement them:

- `Debug`: **changed** - it used to be implemented for every `SafeHolder`, which
  printed read-once data.
- `Display`: **new** in 0.4.
- `Hash`: **new** in 0.4.
- `Deref` and `AsRef`
- `PartialEq` and `Eq`
- `PartialOrd` and `Ord`
- `UpperHex`, `LowerHex` and `Binary`
