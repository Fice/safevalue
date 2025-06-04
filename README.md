[![Build & Test Status](https://github.com/Fice/safevalue/actions/workflows/rust.yml/badge.svg)](https://github.com/Fice/safevalue/actions/)
[![Crates.io](https://img.shields.io/crates/v/safevalue.svg)](https://crates.io/crates/safevalue)
[![Documentation](https://docs.rs/safevalue/badge.svg)](https://docs.rs/safevalue)



## Usage

Add safevalue to your project e.g. via ```cargo add safevalue```.

Then:

```rust
use hashbrown::HashMap;

let mut map = HashMap::new();
map.insert(1, "one");
```

### Rust

safevalue works on stable rust and requires rust version `1.85` (2024 Edition).

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
