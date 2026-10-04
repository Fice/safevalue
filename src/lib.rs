//! safevalue
//! =========
//!
//! [![Tests](https://github.com/Fice/safevalue/actions/workflows/tests.yml/badge.svg)](https://github.com/Fice/safevalue/actions/workflows/tests.yml)
//! [![Clippy + rustfmt](https://github.com/Fice/safevalue/actions/workflows/fmt.yml/badge.svg)](https://github.com/Fice/safevalue/actions/workflows/fmt.yml)
//! [![Env](https://github.com/Fice/safevalue/actions/workflows/env.yml/badge.svg)](https://github.com/Fice/safevalue/actions/workflows/env.yml)
//! [![Crates.io](https://img.shields.io/crates/v/safevalue.svg)](https://crates.io/crates/safevalue)
//! [![Documentation](https://docs.rs/safevalue/badge.svg)](https://docs.rs/safevalue)
//!
//! ## Rationale
//!
//! An `unsafe fn` comes with a safety contract the caller has to uphold, e.g.
//! "`addr` points to a page nobody else uses". Usually that leaves two options,
//! and neither is good:
//!
//! - **Pass the `unsafe` upwards**: every function on the way becomes an
//!   `unsafe fn` as well, until half the code base is `unsafe` and it no longer
//!   tells you anything.
//! - **Wrap the call in an `unsafe` block**: the actual check happened
//!   somewhere else, maybe in another module, and the `// SAFETY:` comment
//!   silently goes stale when that code changes.
//!
//! `safevalue` turns the contract into a type. `unsafe` is needed exactly where
//! a decision is made: where you check a value, or decide to trust it, and
//! vouch for it. From there on the guarantee travels with the value, and all
//! the code in between stays safe:
//!
//! ```rust
//! use safevalue::unsafe_marker;
//!
//! unsafe_marker! {
//!     /// The page at this address is not used by anybody else.
//!     pub struct FreePage(usize);
//! }
//!
//! fn find_free_page() -> FreePage {
//!     let addr = 0x1000; // ... ask the frame allocator
//!     // SAFETY: the allocator just handed out this page; nobody else has it.
//!     unsafe { FreePage::vouch_for(addr) }
//! }
//!
//! // A safe function: it can't be called without a `FreePage`, and there is no
//! // way to get one without `unsafe`.
//! fn map_into_process(page: FreePage) {
//!     let addr = page.take(); // the page is used up from here on
//!     // SAFETY: guaranteed by `FreePage`.
//!     unsafe { map_page(addr) }
//! }
//!
//! unsafe fn map_page(_addr: usize) {
//!     // ...
//! }
//! ```
//!
//!
//! ## Terminology
//!
//! - **Guarantee**: a fact that `unsafe` code relies on, e.g. "this page is not
//!   used by anybody else" - the safety contract of an `unsafe fn`, turned into
//!   a value.
//! - **[SafeHolder]**: a value that carries a guarantee about its data. It can
//!   only be created with `unsafe`, so holding one means someone vouched for
//!   the guarantee.
//! - **Vouch**: creating a [SafeHolder], i.e. stating that the guarantee holds
//!   with [vouch_for()](SafeHolder::vouch_for) for data, or
//!   [vouch()](SafeHolder::vouch) for a marker without data. This is where the
//!   `unsafe` (and its `// SAFETY:` comment) belongs.
//! - **Marker**: a [SafeHolder] type of its own for one guarantee, created with
//!   [unsafe_marker]. Two markers can't be mixed up, even if they hold the same
//!   kind of data - or none at all, like "interrupts are disabled".
//! - **Data**: what a [SafeHolder] hands out, described by [MarkerData] (derive
//!   it for your own types with `#[derive(MarkerData)]`).
//! - **Rely on**: using a guarantee without using it up -
//!   [rely_on()](SafeHolder::rely_on) or [assert_marker].
//! - **Take / invalidate**: using a guarantee up, because what was vouched for
//!   is no longer true afterwards - [take()](SafeHolder::take) (to get the
//!   data), [invalidate()](SafeHolder::invalidate) or [take_marker].
//! - **`READ_ONCE`**: the data can only be read by
//!   [take()](SafeHolder::take)ing it, not through [Deref](core::ops::Deref) or
//!   traits like `Debug` - a marker's default; `readable` lifts it.
//! - **`WRITE_ONCE`**: the data can't be changed after vouching for it; without
//!   it, it can be replaced with [set()](SafeHolder::set) - a marker's default;
//!   `writable` lifts it.
//! - **`PERMANENT`**: the guarantee can never expire, so the [SafeHolder] is
//!   `Copy`/`Clone` and can't be taken - a marker's `permanent` bound.
//! - **Bounds**: `permanent + !Sync` and the like, after the colon in
//!   [unsafe_marker]. See the 3 definitions above.

//make sure we run the code in the readme.md during testing
#![cfg_attr(doctest, doc = include_str!("../README.md"))]
#![doc(issue_tracker_base_url = "https://github.com/Fice/safevalue/issues")]
// We don't need std at all, so we might as well be no_std
// We can still use std in integration tests, so any tests that would require it
// can still do so.
#![no_std]
#![warn(missing_docs)]
#![warn(rustdoc::broken_intra_doc_links)]
#![deny(unsafe_op_in_unsafe_fn)]

// So `::safevalue::...` paths, as #[derive(MarkerData)] generates them, also
// work inside this crate.
extern crate self as safevalue;

mod data;
mod holder;
mod marker;

pub use data::MarkerData;
pub use holder::SafeHolder;
pub use marker::NonDataMarker;
/// Implements [MarkerData] with the value itself as its
/// [Data](MarkerData::Data). See [MarkerData].
pub use safevalue_derive::MarkerData;

/// Especially with empty ```()``` [SafeHolder] you don't really interact with
/// it directly.
///
/// This leads to situations where such a [SafeHolder] is part of a function
/// signature but not really used. Instead of using the ```_``` prefix
/// You can just use this function.
///
/// Example:
/// ```
/// # use safevalue::{assert_marker, SafeHolder, unsafe_marker};
/// unsafe_marker! {
///     pub struct SomePrecondition;
/// }
///
/// pub fn example_func(some_precondition: &SomePrecondition) {
///     assert_marker(some_precondition);
///
///     unsafe {
///         // Do something unsafe, that is only safe when some_precondition is uphold
///         // ...
///     }
/// }
/// ```
/// We will not get a ```unused variable``` warning for the above code.
///
/// See also:
/// If you want to invalidate the marker as well: [take_marker]
pub const fn assert_marker<
    T: MarkerData,
    const WRITE_ONCE: bool,
    const READ_ONCE: bool,
    const PERMANENT: bool,
>(
    #[allow(unused)] marker: &SafeHolder<T, WRITE_ONCE, READ_ONCE, PERMANENT>,
) {
}

/// Especially with empty ```()``` [SafeHolder] you don't really interact with
/// it directly.
///
/// This leads to situations where such a [SafeHolder] is part of a function
/// signature but not really used. In those cases you can use ```take_marker```
/// if you will invalidate the marker
///
/// Example:
/// ```
/// # use safevalue::{take_marker, SafeHolder, unsafe_marker};
/// unsafe_marker! {
///     pub struct SomePrecondition;
/// }
///
/// pub fn example_func(some_precondition: SomePrecondition) {
///     take_marker(some_precondition); // You cannot use 'some_precondition' after this line.
///
///     unsafe {
///         // Do something unsafe, that is only safe when some_precondition is uphold and that
///         // will lead to the precondition to no longer be true, afterwards
///         // ...
///     }
/// }
/// ```
/// We will not get a ```unused variable``` warning for the above code.
///
/// See also:
/// If you don't want to invalidate the marker, use: [assert_marker]
pub fn take_marker<
    T: MarkerData,
    const WRITE_ONCE: bool,
    const READ_ONCE: bool,
>(
    #[allow(unused)]
    // Not `PERMANENT` ones: like `take`, this is "I am using this guarantee
    // up".
    marker: SafeHolder<T, WRITE_ONCE, READ_ONCE, false>,
) {
}

// We need to reexport this, so unsafe_marker! works in downstream crates.
#[doc(hidden)]
pub use pastey::*;
