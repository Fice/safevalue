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
  `unsafe_marker!(pub struct CoreCount(pub u32): permanent + !Sync);`. See the
  docs of `unsafe_marker!` for what each one does.
- **Attributes** are now accepted: doc comments go on the marker, `#[derive]`
  and any other attributes go on the hidden struct behind it, and the marker
  gets those traits through `SafeHolder`'s implementations listed below.

The following traits are only implemented for a `SafeHolder` (and so for a
marker) with `READ_ONCE = false`, i.e. `readable` or `permanent` markers. They
also need the contained type (the marker's hidden struct) to implement them:

- `Debug`: **changed** - it used to be implemented for every `SafeHolder`, which
  printed read-once data.
- `Display`: **new** in 0.4.
- `Hash`: **new** in 0.4.
- `Deref` and `AsRef`
- `PartialEq` and `Eq`
- `PartialOrd` and `Ord`
- `UpperHex`, `LowerHex` and `Binary`
