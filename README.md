safevalue
=========

[![Tests](https://github.com/Fice/safevalue/actions/workflows/tests.yml/badge.svg)](https://github.com/Fice/safevalue/actions/workflows/tests.yml)
[![Clippy + rustfmt](https://github.com/Fice/safevalue/actions/workflows/fmt.yml/badge.svg)](https://github.com/Fice/safevalue/actions/workflows/fmt.yml)
[![Env](https://github.com/Fice/safevalue/actions/workflows/env.yml/badge.svg)](https://github.com/Fice/safevalue/actions/workflows/env.yml)
[![Crates.io](https://img.shields.io/crates/v/safevalue.svg)](https://crates.io/crates/safevalue)
[![Documentation](https://docs.rs/safevalue/badge.svg)](https://docs.rs/safevalue)

## Installation

Add safevalue to your project e.g. via ```cargo add safevalue```.

Upgrading from an older version? See [MIGRATE.md](MIGRATE.md).

## Rationale

An `unsafe fn` comes with a safety contract the caller has to uphold, e.g.
"`addr` points to a page nobody else uses". Usually that leaves two options,
and neither is good:

- **Pass the `unsafe` upwards**: every function on the way becomes an
  `unsafe fn` as well, until half the code base is `unsafe` and it no longer
  tells you anything.
- **Wrap the call in an `unsafe` block**: the actual check happened somewhere
  else, maybe in another module, and the `// SAFETY:` comment silently goes
  stale when that code changes.

`safevalue` turns the contract into a type. `unsafe` is needed exactly where a
decision is made: where you check a value, or decide to trust it, and vouch for
it. From there on the guarantee travels with the value, and all the code in
between stays safe:

```rust
use safevalue::unsafe_marker;

unsafe_marker! {
    /// The page at this address is not used by anybody else.
    pub struct FreePage(usize);
}

fn find_free_page() -> FreePage {
    let addr = 0x1000; // ... ask the frame allocator
    // SAFETY: the allocator just handed out this page; nobody else has it.
    unsafe { FreePage::vouch_for(addr) }
}

// A safe function: it can't be called without a `FreePage`, and there is no
// way to get one without `unsafe`.
fn map_into_process(page: FreePage) {
    let addr = page.take(); // the page is used up from here on
    // SAFETY: guaranteed by `FreePage`.
    unsafe { map_page(addr) }
}

unsafe fn map_page(_addr: usize) { /* ... */ }
```

What you get:

- **`unsafe` only where it matters**: reviewing the code means reviewing the
  places that vouch, not every function a value passes through.
- **Guarantees the compiler checks**: a `SafeHolder` or marker can't be
  created without `unsafe`, two markers can't be mixed up, and a guarantee can
  be made read-once, write-once, permanent (copyable) or bound to one thread.
- **Documentation in the type**: a function taking `FreePage` says what it
  relies on, and keeps saying it when the code changes.
- **Zero cost**: a `SafeHolder` is `#[repr(transparent)]` over its data, and a
  marker without data is zero-sized.

## Features

- [x] Zero Cost Abstraction.
- [x] Ergonomic
- [ ] Good Idea. Well, it works quite nicely in my kernel project, let's see how it goes.

## Dependencies

`safevalue` has two dependencies:

- [`safevalue-derive`](https://crates.io/crates/safevalue-derive), our own
  crate for `#[derive(MarkerData)]`. Derive macros have to live in a crate of
  their own; use it through `safevalue`, which re-exports it. It is built
  with [`syn`](https://crates.io/crates/syn),
  [`quote`](https://crates.io/crates/quote) and
  [`proc-macro2`](https://crates.io/crates/proc-macro2), at compile time only.
- [`pastey`](https://crates.io/crates/pastey), used by `unsafe_marker!`. This
  dependency will be removed when/if `concat-idents` is powerful enough and
  stabilised.

It does additionally have dev-dependencies used for testing.

### Rust

`safevalue` works on stable rust and requires rust version `1.85` (2024 Edition).

### No STD

This crate is `#[no_std]` and needs neither `std` nor `alloc`.

Everything a `SafeHolder` holds has to implement `MarkerData` (derive it for
your own types with `#[derive(MarkerData)]`). The crate implements it for the
common `core` types out of the box; two optional features add more:

- `alloc`: `String`, `Vec`, `Box`, `Rc`, `Arc`, the `alloc` collections, ...
- `std` (includes `alloc`): `HashMap`, `HashSet`, `PathBuf`, `OsString`.


## License

Licensed under either of:

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
