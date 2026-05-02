//! safevalue
//! =========
//!
//! [![Build & Test Status](https://github.com/Fice/safevalue/actions/workflows/rust.yml/badge.svg)](https://github.com/Fice/safevalue/actions/)
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


/// Trait implemented for Data Types, that do not actually hold any data.
///
/// It is used by the [unsafe_marker] macro. It will expand to something like:
/// ```
/// struct FooMarkerUniqueData {}
///
/// impl safevalue::NonDataMarker for FooMarkerUniqueData {
///    const NEW_MARKER: Self = Self {};
/// }
///
///
/// pub type FooMarker = safevalue::SafeHolder<FooMarkerUniqueData, true, false>;
/// ```
/// Note: Try to use [unsafe_marker] macro over manually implementing this.
///
/// > Only implement this trait for Types that are distinct (e.g. not `()`) and don't hold any
/// > runtime data, e.g. `struct InterruptsDisabled {}`
///
/// ## Why don't we just use ```()```?
/// If we did, we couldn't distinguish between different Markers.
/// ```
/// pub type FooMarker = safevalue::SafeHolder<(), true, false>;
/// pub type BarMarker = safevalue::SafeHolder<(), true, false>;
///
/// pub fn requires_foo(foo_marker: FooMarker) {
///     foo_marker.take()
///     // ...
/// }
///
/// pub fn provides_bar() {
///     // This works, even though we are mixing up our SAFETY requirements
///     requires_foo(
///         unsafe{ BarMarker::vouch()}
///     );
/// }
/// ```
///n
pub trait NonDataMarker {
    /// The value used by [SafeHolder::vouch] to create the [SafeHolder].
    const NEW_MARKER: Self;
}
impl NonDataMarker for () {
    const NEW_MARKER: Self = ();
}


#[derive(Debug)]
#[repr(transparent)]
/// SafeHolder is a struct that vouches for the data within it.
///
/// Although it is common to have a SafeHolder vouching for () with a different meaning like 'Interrupts for the processor are currently disabled'
pub struct SafeHolder<
    T,
    const WRITE_ONCE: bool = true,
    const READ_ONCE: bool = true,
> {
    /// Holds the actual data we are vouching for.
    data:   T,
    /// Sealed is non public, so only functions within this crate can create a
    /// SafeHolder. All constructors are 'unsafe' making it impossible to
    /// create a SafeHolder without unsafe
    #[doc(hidden)]
    sealed: core::marker::PhantomData<Sealed>,
}

impl<T, const WRITE_ONCE: bool, const READ_ONCE: bool>
    SafeHolder<T, WRITE_ONCE, READ_ONCE>
{
    /// Creates a new [SafeHolder] that vouches for the given data.
    /// If `WRITE_ONCE` is true, we cannot write to or change the contained data
    /// in any way. We can only mut the data after we [take()](SafeHolder::take) it and thereby remove the SAFETY
    /// guarantee.
    ///
    /// See also [vouch()](SafeHolder::vouch) and [set()](SafeHolder::set)
    ///
    /// # SAFETY
    /// - It is up to the caller to uphold the SAFETY requirements for the type T.
    #[inline(always)]
    pub const unsafe fn vouch_for(data: T) -> Self {
        Self {
            data,
            sealed: core::marker::PhantomData::<Sealed> {},
        }
    }

    /// Consumes the `SafeHolder` and returns the value contained in it.
    ///
    /// In some Situations using the value removes the SAFETY guarantee.
    /// For Example in an Operating System we find an unused Physical Memory
    /// address. Once we found it, we assign that memory address to a specif
    /// process. Obviously, now that memory is no longer free and available.
    /// ```
    /// # use safevalue::SafeHolder;
    ///
    /// struct FreeMemoryPointer(*const u8);
    /// // This should be a doccomment, but rustdoc dos not allow it in here.
    /// // # SAFETY
    /// // By creating this structure, the creator guarantees that the memory adres points to unused physical memory.
    /// type SafeFreeMemoryPointer = SafeHolder<FreeMemoryPointer>;
    ///
    /// pub fn assign_memory_to_process(free_memory: SafeFreeMemoryPointer) {
    ///
    ///     let memory_address = free_memory.take();
    ///     // ... implement the actual memory stuff
    /// }
    /// ```
    ///
    /// When the SafeHolder is `READ_ONCE`, this is the only way to get the
    /// contained data, otherwise [Deref](core::ops::Deref) and [AsRef] are implemented.
    ///
    /// See also [invalidate()](`Self::invalidate`) if you want to remove the SAFETY guarantee
    /// without the need to access the data.
    #[inline(always)]
    pub fn take(self) -> T { self.data }

    /// If you perform an operations that invalidates the SAFETY guarantee you
    /// should invalidate.
    ///
    /// See also [take()](`Self::take`) if you actually need access to the contained
    /// data.
    ///
    /// NOTE: We could not have this function if SafeHolder is [Copy] or
    /// [Clone], because there might still be instances out there.
    #[inline(always)]
    pub fn invalidate(self) {}

    /// A function indicating that the current code piece is relying on the given SAFETY guarantees.
    ///
    /// Especially useful for [Marker](unsafe_marker) types where we don't have actual data to use.
    ///
    /// # The Problem
    /// ```
    /// # use safevalue::SafeHolder;
    /// # struct Foo {}
    /// pub fn foo(guarantee: SafeHolder<Foo, true, true>) {
    ///
    ///     unsafe {
    ///         // Lets do something unsafe, that we now is ok, because of the guarantee.
    ///         // ...
    ///     }
    /// }
    /// ```
    ///
    /// The above example is perfectly fine, however we get a unused warning because we rely on `guarantee` conceptually,
    /// we don't actually use it in code.
    ///
    /// We could markt it unused via `_guarante`, but we might miss it in future refactors, where it no longer
    /// requires/relies on this guarantee.
    ///
    /// # The Solution
    ///
    /// This is where [rely_on()](SafeHolder::rely_on) comes in:
    /// ```
    /// # use safevalue::SafeHolder;
    /// # struct Foo {}
    /// pub fn foo(guarantee: SafeHolder<Foo, true, true>) {
    ///
    ///     guarantee.rely_on();
    ///     // SAFETY
    ///     // We know this is safe, because of `guarantee`
    ///     unsafe {
    ///         // ...
    ///     }
    /// }
    /// ```
    /// This not only makes use of the guarantee, it also provides great locality when documenting
    /// `//SAFETY` sections of `unsafe` code.
    ///
    /// > This functions is zerocost in that it doesn't actually do anything
    ///
    /// If the `unsafe` code invalidates the safety guarantees (e.g. memory pointed to by a pointer is no longer unused)
    /// consider using [invalidate()](SafeHolder::invalidate) or [take()](SafeHolder::take)
    ///
    /// See also [trust()](SafeHolder::trust).
    #[inline(always)]
    pub const fn rely_on(&self) {}

    /// TODO: not sure I want to keep this.
    /// If anyone uses this, please open an Issue and tell me.
    #[inline(always)]
    pub const fn trust(&self) -> bool { true }
}

impl<T: Clone, const WRITE_ONCE: bool> SafeHolder<T, WRITE_ONCE, false> {
    /// Creates a copy of the SafeHolder and its data
    ///
    /// # SAFETY
    ///
    /// make sure that the safety requirements of T still hold, when there
    /// are two instances of this around. Functions like [take()](Self::take) or [invalidate()](Self::invalidate)
    /// will only consum one of the copies.
    pub unsafe fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            sealed: core::marker::PhantomData::<Sealed> {},
        }
    }
}

impl<T: NonDataMarker, const WRITE_ONCE: bool, const READ_ONCE: bool>
    SafeHolder<T, WRITE_ONCE, READ_ONCE>
{
    /// Creates a new SafeHolder that vouches for a certain fact.
    ///
    /// This is unsafe, it is up to the caller to uphold the SAFETY
    /// requirements of the [Marker](unsafe_marker).
    ///
    /// See also [vouch_for()](SafeHolder::vouch_for)
    ///
    /// # SAFETY
    /// - See SAFETY requirements of `T`
    #[inline(always)]
    pub const unsafe fn vouch() -> Self {
        Self {
            data:   T::NEW_MARKER,
            sealed: core::marker::PhantomData::<Sealed> {},
        }
    }
}
impl<T, const READ_ONCE: bool> SafeHolder<T, false, READ_ONCE> {
    /// When `WRITE_ONCE` is false, you can use set to change the data vouched
    /// for.
    ///
    /// This is unsafe, it is up to the caller to uphold the SAFETY requirements
    /// of type `T`.
    ///
    /// See also [SafeHolder::vouch_for]
    ///
    /// # SAFETY
    /// - See SAFETY requirements of `T`
    #[inline(always)]
    pub unsafe fn set(&mut self, data: T) { self.data = data; }
}

impl<T, const WRITE_ONCE: bool> AsRef<T> for SafeHolder<T, WRITE_ONCE, false> {
    fn as_ref(&self) -> &T {
        use core::ops::Deref;
        self.deref()
    }
}

impl<T, const WRITE_ONCE: bool> core::ops::Deref
    for SafeHolder<T, WRITE_ONCE, false>
{
    type Target = T;

    fn deref(&self) -> &Self::Target { &self.data }
}

impl<T: Eq, const WRITE_ONCE: bool> Eq for SafeHolder<T, WRITE_ONCE, false> {}
impl<T: PartialEq, const WRITE_ONCE: bool> PartialEq
    for SafeHolder<T, WRITE_ONCE, false>
{
    fn eq(&self, other: &Self) -> bool { self.data == other.data }
}

impl<T: Ord, const WRITE_ONCE: bool> Ord for SafeHolder<T, WRITE_ONCE, false> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.data.cmp(&other.data)
    }
}
impl<T: PartialOrd, const WRITE_ONCE: bool> PartialOrd
    for SafeHolder<T, WRITE_ONCE, false>
{
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.data.partial_cmp(&other.data)
    }
}

impl<T: core::fmt::UpperHex, const WRITE_ONCE: bool> core::fmt::UpperHex
    for SafeHolder<T, WRITE_ONCE, false>
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::UpperHex::fmt(&self.data, f)
    }
}
impl<T: core::fmt::LowerHex, const WRITE_ONCE: bool> core::fmt::LowerHex
    for SafeHolder<T, WRITE_ONCE, false>
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::LowerHex::fmt(&self.data, f)
    }
}
impl<T: core::fmt::Binary, const WRITE_ONCE: bool> core::fmt::Binary
    for SafeHolder<T, WRITE_ONCE, false>
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Binary::fmt(&self.data, f)
    }
}


/// Especially with empty ```()``` [SafeHolder] you don't really interact with it directly.
/// 
/// This leads to situations where such a [SafeHolder] is part of a function signature but not really used. Instead of using the ```_``` prefix
/// You can just use this function.
/// 
/// Example:
/// ```
/// # use safevalue::{assert_marker, SafeHolder, unsafe_marker};
/// unsafe_marker! {
///     pub SomePrecondition
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
pub const fn assert_marker<T, const WRITE_ONCE: bool, const READ_ONCE: bool>(
    #[allow(unused)]
    marker: &safevalue::SafeHolder<T, WRITE_ONCE, READ_ONCE>
) {}

/// Especially with empty ```()``` [SafeHolder] you don't really interact with it directly.
/// 
/// This leads to situations where such a [SafeHolder] is part of a function signature but not really used. In those cases you can use ```take_marker``` if you will invalidate the marker
/// 
/// Example:
/// ```
/// # use safevalue::{take_marker, SafeHolder, unsafe_marker};
/// unsafe_marker! {
///     pub SomePrecondition
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
    marker: safevalue::SafeHolder<T, WRITE_ONCE, READ_ONCE>
) {}


/// Non-public struct that is used to prevent the use of ['SafeHolder'] that
/// circumvents the creation of the SafeHolder without the 'unsafe' functions
#[derive(Debug)]
struct Sealed {}

// We need to reexport this, so unsafe_marker! works in downstream crates.
#[doc(hidden)]
pub use paste::*;

// Without this, unit testing the macro fails, because safevalue::SafeHolder not
// found.
mod safevalue {
    #[allow(unused_imports)]
    pub(crate) use super::*;
}


#[doc(alias = "Marker")]
#[macro_export]
/// A macro to easily create a Marker
/// 
/// It supports doc expressions and visibility for the marker.
/// 
/// Use this over ´´´pub MarkerType = SafeHolder<()>´´´
/// 
/// The reason is, that all SafeHolder<()> are interchangable. this macro will create a hidden type, so each marker definition is unique.
macro_rules! unsafe_marker {
    (  $(#[doc = $doc:expr]) * $v:vis $i:ident ) => {
        safevalue::paste! {
            #[doc(hidden)]
            $v struct [<$i NDM >] {}

            impl safevalue::NonDataMarker for [<$i NDM >] {
                const NEW_MARKER: Self = Self {};
            }

            $(
                #[doc = $doc]
            )*
            #[allow(private_interfaces)]
            $v type $i = safevalue::SafeHolder<[<$i NDM >], true, false>;
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;


    #[derive(Debug, PartialEq, Eq)]
    struct Custom {
        c: char,
        b: bool,
    }

    type SafeU64 = SafeHolder<u64, false, false>;
    type SafeF32 = SafeHolder<f32, true, false>;
    type SafeArray = SafeHolder<[bool; 4], false, true>;
    type SafeCustom = SafeHolder<Custom, true, true>;


    #[test]
    fn new_and_take_works() {
        let safe_u64 = unsafe { SafeU64::vouch_for(12u64) };
        let safe_f32 = unsafe { SafeF32::vouch_for(0.5) };
        let safe_array =
            unsafe { SafeArray::vouch_for([true, false, false, true]) };
        let safe_custom =
            unsafe { SafeCustom::vouch_for(Custom { c: 'a', b: false }) };



        safe_u64.rely_on();
        safe_f32.rely_on();
        safe_array.rely_on();
        safe_custom.rely_on();

        //Taking is always safe
        assert_eq!(safe_u64.take(), 12u64);
        assert_eq!(safe_f32.take(), 0.5);
        assert_eq!(safe_array.take(), [true, false, false, true]);
        assert_eq!(safe_custom.take(), Custom { c: 'a', b: false });
    }

    #[test]
    fn as_ref_when_readable() {
        let safe_u64 = unsafe { SafeU64::vouch_for(0u64) };
        let safe_f32 = unsafe { SafeF32::vouch_for(0.5) };
        let _safe_array =
            unsafe { SafeArray::vouch_for([true, false, false, true]) };
        let _safe_custom =
            unsafe { SafeCustom::vouch_for(Custom { c: 'a', b: false }) };


        //ref is available and works when READ_ONCE is false
        assert_eq!(*safe_u64.as_ref(), 0u64);
        assert_eq!(*safe_f32.as_ref(), 0.5);
    }

    #[test]
    fn settable() {
        let mut safe_u64 = unsafe { SafeU64::vouch_for(12u64) };
        let mut safe_array =
            unsafe { SafeArray::vouch_for([true, false, false, true]) };

        unsafe {
            safe_u64.set(23u64);
        }
        unsafe {
            safe_array.set([true, true, false, false]);
        }

        assert_eq!(safe_u64.take(), 23u64);
        assert_eq!(safe_array.take(), [true, true, false, false]);
    }

    #[test]
    fn test_deref() {
        let safe_u64 = unsafe { SafeU64::vouch_for(97u64) };
        let safe_f32 = unsafe { SafeF32::vouch_for(0.5) };

        //Make sure deref fails to compile for READ_ONCE types
        assert_eq!(*safe_u64, 97u64);
        assert_eq!(*safe_f32, 0.5);
    }

    unsafe_marker!(Test);
    unsafe_marker!(
        /// do we have documentation?
        pub Test2
    );


    unsafe_marker!(
        /// do we have documentation?
        pub Test3
    );
    unsafe_marker!(pub Test4);

    unsafe_marker!(pub SafeMarker);

    #[test]
    pub fn test_marker() {
        let mut _marker = unsafe { SafeMarker::vouch() };

        let marker = unsafe { Test::vouch() };
        let marker2 = unsafe { Test2::vouch() };
        let marker3 = unsafe { Test3::vouch() };
        let _marker4 = unsafe { Test4::vouch() };

        marker.trust();
        marker.take();

        if marker2.trust() {
            // we can use this in if
        }

        marker3.rely_on();
    }
}
