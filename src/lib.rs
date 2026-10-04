//! safevalue
//! =========
//!
//! [![Tests](https://github.com/Fice/safevalue/actions/workflows/tests.yml/badge.svg)](https://github.com/Fice/safevalue/actions/workflows/tests.yml)
//! [![Clippy + rustfmt](https://github.com/Fice/safevalue/actions/workflows/fmt.yml/badge.svg)](https://github.com/Fice/safevalue/actions/workflows/fmt.yml)
//! [![Env](https://github.com/Fice/safevalue/actions/workflows/env.yml/badge.svg)](https://github.com/Fice/safevalue/actions/workflows/env.yml)
//! [![Crates.io](https://img.shields.io/crates/v/safevalue.svg)](https://crates.io/crates/safevalue)
//! [![Documentation](https://docs.rs/safevalue/badge.svg)](https://docs.rs/safevalue)
//!
//! # Purpose
//! We have an unsafe functions
//! Passing the stick upwards
//! Everything get unsafe
//!
//!
//! # Usage
//!
//!
//! # Implementation Details
//!
//!
//! # Terminology
//! Marker
//! READ_ONCE,
//! WRITE_ONCE

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

mod holder;
mod marker;

pub use holder::SafeHolder;
pub use marker::NonDataMarker;

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
    T,
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
pub fn take_marker<T, const WRITE_ONCE: bool, const READ_ONCE: bool>(
    #[allow(unused)]
    // Not `PERMANENT` ones: like `take`, this is "I am using this guarantee
    // up".
    marker: SafeHolder<T, WRITE_ONCE, READ_ONCE, false>,
) {
}

// We need to reexport this, so unsafe_marker! works in downstream crates.
#[doc(hidden)]
pub use pastey::*;
