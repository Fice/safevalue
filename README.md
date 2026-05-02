safevalue
=========

[![Build & Test Status](https://github.com/Fice/safevalue/actions/workflows/rust.yml/badge.svg)](https://github.com/Fice/safevalue/actions/)
[![Crates.io](https://img.shields.io/crates/v/safevalue.svg)](https://crates.io/crates/safevalue)
[![Documentation](https://docs.rs/safevalue/badge.svg)](https://docs.rs/safevalue)

## Usage

Add safevalue to your project e.g. via ```cargo add safevalue```.

Then:

```rust


fn main() {
    //TODO!
}
```

## Rationale

## Features

[X] Zero Cost Abstraction.
[X] Ergonomic
[ ] Good Idea. Well, it works quite nicely in my kernel project, let's see how it goes.

## Dependencies

`safevalue`'s only runtime dependency is [`paste`](https://crates.io/crates/paste). This dependency will be removed when/if `concat-idents` is powerful enough and stabilised.

It does additionally have dev-dependencies used for testing.

### Rust

`safevalue` works on stable rust and requires rust version `1.85` (2024 Edition).

### No STD

This crate is `#[no_std]` as well as no `alloc`, because it does not need them. 
If this is ever to change, `alloc` or `std` will be hidden behind appropriate feature flags.

## License

Licensed under either of:

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.


Note: If you are running the tests, sometimes the error 
> multiple candidates for `rlib` dependency `safevalue` found [E0464]'
pops up. It seems some duplicate build artifact between cargo check and cargo build confuse the tests. Run `cargo clean` and it should work again.