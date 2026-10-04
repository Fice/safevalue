//! safevalue
//! =========
//!
//! [![Tests](https://github.com/Fice/safevalue/actions/workflows/tests.yml/badge.svg)](https://github.com/Fice/safevalue/actions/workflows/tests.yml)
//! [![Code format](https://github.com/Fice/safevalue/actions/workflows/fmt.yml/badge.svg)](https://github.com/Fice/safevalue/actions/workflows/fmt.yml)
//! [![Env](https://github.com/Fice/safevalue/actions/workflows/env.yml/badge.svg)](https://github.com/Fice/safevalue/actions/workflows/env.yml)
//! [![Docs build](https://github.com/Fice/safevalue/actions/workflows/docs.yml/badge.svg)](https://github.com/Fice/safevalue/actions/workflows/docs.yml)
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
///     const NEW_MARKER: Self = Self {};
/// }
///
///
/// pub type FooMarker =
///     safevalue::SafeHolder<FooMarkerUniqueData, true, true, false>;
/// ```
/// Note: Try to use [unsafe_marker] macro over manually implementing this.
///
/// > Only implement this trait for Types that are distinct (e.g. not `()`) and
/// > don't hold any
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
///     // This compiles, even though we are mixing up our SAFETY requirements
///     requires_foo(unsafe { BarMarker::vouch_for(()) });
/// }
/// ```
/// With a distinct type per marker, the same mix-up doesn't compile:
/// ```compile_fail,E0308
/// # use safevalue::unsafe_marker;
/// unsafe_marker!(pub struct FooMarker);
/// unsafe_marker!(pub struct BarMarker);
///
/// pub fn requires_foo(foo_marker: FooMarker) {
///     foo_marker.take()
///     // ...
/// }
///
/// pub fn provides_bar() {
///     requires_foo(unsafe { BarMarker::vouch() });
/// }
/// ```
pub trait NonDataMarker {
    /// The value used by [SafeHolder::vouch] to create the [SafeHolder].
    const NEW_MARKER: Self;
}

#[repr(transparent)]
/// SafeHolder is a struct that vouches for the data within it.
///
/// Although it is common to have a SafeHolder vouching for () with a different
/// meaning like 'Interrupts for the processor are currently disabled'
///
/// # Permanent guarantees
///
/// Normally a guarantee can be *used up*: "this address points to unused
/// memory" stops being true the moment someone uses that memory (see
/// [take()](SafeHolder::take)), which is why a holder can't simply be copied -
/// every copy would go on vouching for something that is no longer true.
///
/// Some facts never expire, though: "the bootloader mapped all of physical
/// memory at this offset" is true for the whole run. Vouching for one of those
/// with `PERMANENT = true` flips what the holder can do:
/// - it is [Clone] and [Copy], so everything that needs the fact can just have
///   its own copy;
/// - it has no [take()](SafeHolder::take) and no
///   [invalidate()](SafeHolder::invalidate) - a permanent guarantee can't be
///   revoked, and offering those next to `Copy` would let one copy claim to
///   consume a fact the others still hold. (Read it through
///   [Deref](core::ops::Deref) and copy the data out of it instead.)
///
/// Only `WRITE_ONCE = true`, `READ_ONCE = false` holders can be permanent: a
/// `READ_ONCE` value is a secret that must be consumed exactly once, which
/// copies would defeat, and the copies of a writable holder could each be `set`
/// to a different value. Any other combination fails to compile when it is
/// first constructed.
///
/// ```
/// # use safevalue::SafeHolder;
/// #[derive(Clone, Copy)]
/// struct CpuCount(u32);
///
/// // SAFETY: the number of cores found at boot never changes afterwards - and that is
/// // exactly what `PERMANENT = true` claims.
/// let cores = unsafe { SafeHolder::<CpuCount, true, false, true>::vouch_for(CpuCount(4)) };
/// let also_cores = cores; // copied, not moved
/// assert_eq!(cores.0, also_cores.0);
/// ```
///
/// A permanent holder can't be consumed:
/// ```compile_fail
/// # use safevalue::SafeHolder;
/// let permanent = unsafe { SafeHolder::<u32, true, false, true>::vouch_for(4) };
/// let _ = permanent.take();
/// ```
/// and an ordinary one can't be copied:
/// ```compile_fail
/// # use safevalue::SafeHolder;
/// let ordinary = unsafe { SafeHolder::<u32, true, false>::vouch_for(4) };
/// let first = ordinary;
/// let second = ordinary; // moved above
/// ```
/// and the combinations that would make no sense are rejected:
/// ```compile_fail
/// # use safevalue::SafeHolder;
/// // a permanent, but write-many holder
/// let _ = unsafe { SafeHolder::<u32, false, false, true>::vouch_for(4) };
/// ```
pub struct SafeHolder<
    T,
    const WRITE_ONCE: bool = true,
    const READ_ONCE: bool = true,
    const PERMANENT: bool = false,
> {
    /// Holds the actual data we are vouching for.
    data: T,
}

impl<T, const WRITE_ONCE: bool, const READ_ONCE: bool, const PERMANENT: bool>
    SafeHolder<T, WRITE_ONCE, READ_ONCE, PERMANENT>
{
    /// Evaluated (at compile time) whenever a holder is constructed, so a
    /// combination of parameters that doesn't make sense can never exist -
    /// see the type's docs.
    const VALID: () = assert!(
        !PERMANENT || (WRITE_ONCE && !READ_ONCE),
        "a PERMANENT SafeHolder must be WRITE_ONCE = true and READ_ONCE = \
         false",
    );

    /// Creates a new [SafeHolder] that vouches for the given data.
    /// If `WRITE_ONCE` is true, we cannot write to or change the contained data
    /// in any way. We can only mut the data after we [take()](SafeHolder::take)
    /// it and thereby remove the SAFETY guarantee.
    ///
    /// See also [vouch()](SafeHolder::vouch) and [set()](SafeHolder::set)
    ///
    /// # SAFETY
    /// - It is up to the caller to uphold the SAFETY requirements for the type
    ///   T.
    /// - If `PERMANENT` is true, the caller also promises the guarantee can
    ///   never expire, whatever happens to any copy of the holder afterwards.
    #[inline(always)]
    #[must_use]
    pub const unsafe fn vouch_for(data: T) -> Self {
        let () = Self::VALID;
        Self { data }
    }
}

impl<T, const WRITE_ONCE: bool, const READ_ONCE: bool>
    SafeHolder<T, WRITE_ONCE, READ_ONCE, false>
{
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
    /// contained data, otherwise [Deref](core::ops::Deref) and [AsRef] are
    /// implemented.
    ///
    /// See also [invalidate()](`Self::invalidate`) if you want to remove the
    /// SAFETY guarantee without the need to access the data.
    #[inline(always)]
    #[must_use]
    pub fn take(self) -> T { self.data }

    /// If you perform an operations that invalidates the SAFETY guarantee you
    /// should invalidate.
    ///
    /// See also [take()](`Self::take`) if you actually need access to the
    /// contained data.
    ///
    /// NOTE: Only exists for holders that aren't `PERMANENT`. We could not have
    /// this function on a [Copy] or [Clone] holder, because there might
    /// still be instances out there - and a permanent guarantee can't be
    /// invalidated anyway.
    #[inline(always)]
    pub fn invalidate(self) {}
}

impl<T, const WRITE_ONCE: bool, const READ_ONCE: bool, const PERMANENT: bool>
    SafeHolder<T, WRITE_ONCE, READ_ONCE, PERMANENT>
{
    /// A function indicating that the current code piece is relying on the
    /// given SAFETY guarantees.
    ///
    /// Especially useful for [Marker](unsafe_marker) types where we don't have
    /// actual data to use.
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
    /// The above example is perfectly fine, however we get a unused warning
    /// because we rely on `guarantee` conceptually, we don't actually use
    /// it in code.
    ///
    /// We could markt it unused via `_guarante`, but we might miss it in future
    /// refactors, where it no longer requires/relies on this guarantee.
    ///
    /// # The Solution
    ///
    /// This is where [rely_on()](SafeHolder::rely_on) comes in:
    /// ```
    /// # use safevalue::SafeHolder;
    /// # struct Foo {}
    /// pub fn foo(guarantee: SafeHolder<Foo, true, true>) {
    ///     guarantee.rely_on();
    ///     // SAFETY
    ///     // We know this is safe, because of `guarantee`
    ///     unsafe {
    ///         // ...
    ///     }
    /// }
    /// ```
    /// This not only makes use of the guarantee, it also provides great
    /// locality when documenting `//SAFETY` sections of `unsafe` code.
    ///
    /// > This functions is zerocost in that it doesn't actually do anything
    ///
    /// If the `unsafe` code invalidates the safety guarantees (e.g. memory
    /// pointed to by a pointer is no longer unused) consider using
    /// [invalidate()](SafeHolder::invalidate) or [take()](SafeHolder::take)
    ///
    /// See also [trust()](SafeHolder::trust).
    #[inline(always)]
    pub const fn rely_on(&self) {}

    /// TODO: not sure I want to keep this.
    /// If anyone uses this, please open an Issue and tell me.
    #[inline(always)]
    #[must_use]
    pub const fn trust(&self) -> bool { true }
}

impl<T: Clone, const WRITE_ONCE: bool> SafeHolder<T, WRITE_ONCE, false, false> {
    /// Creates a copy of the SafeHolder and its data
    ///
    /// Not called `clone`: an inherent method with that name would shadow
    /// [Clone::clone], so `holder.clone()` would keep demanding `unsafe`
    /// even for the holders where copying is safe (the `PERMANENT` ones - see
    /// [SafeHolder]).
    ///
    /// # SAFETY
    ///
    /// make sure that the safety requirements of T still hold, when there
    /// are two instances of this around. Functions like [take()](Self::take) or
    /// [invalidate()](Self::invalidate) will only consum one of the copies.
    pub unsafe fn clone_unchecked(&self) -> Self {
        Self {
            data: self.data.clone(),
        }
    }
}

/// Copies of a permanent guarantee - see [SafeHolder]'s docs. There is no
/// `take` or `invalidate` on these, so nothing can claim to have used one of
/// the copies up.
impl<T: Clone> Clone for SafeHolder<T, true, false, true> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
        }
    }
}

impl<T: Copy> Copy for SafeHolder<T, true, false, true> {}

impl<
    T: NonDataMarker,
    const WRITE_ONCE: bool,
    const READ_ONCE: bool,
    const PERMANENT: bool,
> SafeHolder<T, WRITE_ONCE, READ_ONCE, PERMANENT>
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
    #[must_use]
    pub const unsafe fn vouch() -> Self {
        let () = Self::VALID;
        Self {
            data: T::NEW_MARKER,
        }
    }
}
impl<T, const READ_ONCE: bool> SafeHolder<T, false, READ_ONCE, false> {
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

impl<T, const WRITE_ONCE: bool, const PERMANENT: bool> AsRef<T>
    for SafeHolder<T, WRITE_ONCE, false, PERMANENT>
{
    fn as_ref(&self) -> &T {
        use core::ops::Deref;
        self.deref()
    }
}

impl<T, const WRITE_ONCE: bool, const PERMANENT: bool> core::ops::Deref
    for SafeHolder<T, WRITE_ONCE, false, PERMANENT>
{
    type Target = T;

    fn deref(&self) -> &Self::Target { &self.data }
}

impl<T: Eq, const WRITE_ONCE: bool, const PERMANENT: bool> Eq
    for SafeHolder<T, WRITE_ONCE, false, PERMANENT>
{
}
impl<T: PartialEq, const WRITE_ONCE: bool, const PERMANENT: bool> PartialEq
    for SafeHolder<T, WRITE_ONCE, false, PERMANENT>
{
    fn eq(&self, other: &Self) -> bool { self.data == other.data }
}

impl<T: Ord, const WRITE_ONCE: bool, const PERMANENT: bool> Ord
    for SafeHolder<T, WRITE_ONCE, false, PERMANENT>
{
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.data.cmp(&other.data)
    }
}
impl<T: PartialOrd, const WRITE_ONCE: bool, const PERMANENT: bool> PartialOrd
    for SafeHolder<T, WRITE_ONCE, false, PERMANENT>
{
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.data.partial_cmp(&other.data)
    }
}

impl<T: core::fmt::UpperHex, const WRITE_ONCE: bool, const PERMANENT: bool>
    core::fmt::UpperHex for SafeHolder<T, WRITE_ONCE, false, PERMANENT>
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::UpperHex::fmt(&self.data, f)
    }
}
impl<T: core::fmt::LowerHex, const WRITE_ONCE: bool, const PERMANENT: bool>
    core::fmt::LowerHex for SafeHolder<T, WRITE_ONCE, false, PERMANENT>
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::LowerHex::fmt(&self.data, f)
    }
}
impl<T: core::fmt::Binary, const WRITE_ONCE: bool, const PERMANENT: bool>
    core::fmt::Binary for SafeHolder<T, WRITE_ONCE, false, PERMANENT>
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Binary::fmt(&self.data, f)
    }
}
/// Prints the same as `#[derive(Debug)]` would - but not for `READ_ONCE`
/// holders, whose data may only be read once through
/// [take()](SafeHolder::take).
impl<T: core::fmt::Debug, const WRITE_ONCE: bool, const PERMANENT: bool>
    core::fmt::Debug for SafeHolder<T, WRITE_ONCE, false, PERMANENT>
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SafeHolder")
            .field("data", &self.data)
            .finish()
    }
}
impl<T: core::fmt::Display, const WRITE_ONCE: bool, const PERMANENT: bool>
    core::fmt::Display for SafeHolder<T, WRITE_ONCE, false, PERMANENT>
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Display::fmt(&self.data, f)
    }
}

/// Hashes exactly like the contained data, so it agrees with [PartialEq].
impl<T: core::hash::Hash, const WRITE_ONCE: bool, const PERMANENT: bool>
    core::hash::Hash for SafeHolder<T, WRITE_ONCE, false, PERMANENT>
{
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.data.hash(state)
    }
}

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
    #[allow(unused)] marker: &safevalue::SafeHolder<
        T,
        WRITE_ONCE,
        READ_ONCE,
        PERMANENT,
    >,
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
    marker: safevalue::SafeHolder<T, WRITE_ONCE, READ_ONCE, false>,
) {
}

// We need to reexport this, so unsafe_marker! works in downstream crates.
#[doc(hidden)]
pub use pastey::*;

// Without this, unit testing the macro fails, because safevalue::SafeHolder not
// found.
mod safevalue {
    #[allow(unused_imports)]
    pub(crate) use super::*;
}

#[doc(alias = "Marker")]
#[macro_export]
/// Defines a Marker: a [SafeHolder] type that vouches for a fact (and maybe
/// carries data along with it), and that can't be mixed up with any other
/// marker.
///
/// ```
/// # use safevalue::{assert_marker, unsafe_marker};
/// unsafe_marker! {
///     /// The page tables are set up.
///     pub struct PagingEnabled;
/// }
///
/// pub fn map_page(paging_enabled: &PagingEnabled) {
///     assert_marker(paging_enabled);
///     // ...
/// }
///
/// // SAFETY: an example; nothing here needs page tables.
/// let paging_enabled = unsafe { PagingEnabled::vouch() };
/// map_page(&paging_enabled);
/// ```
///
/// # Why not just `type PagingEnabled = SafeHolder<()>`?
///
/// All `SafeHolder<()>` are the same type, so a function asking for one marker
/// would accept any other (see [NonDataMarker]). Instead, the macro creates a
/// hidden struct for every marker, and the marker is a [SafeHolder] of it:
/// ```ignore
/// #[doc(hidden)]
/// pub struct PagingEnabledNDM {}
/// impl safevalue::NonDataMarker for PagingEnabledNDM { /* ... */ }
///
/// /// The page tables are set up.
/// pub type PagingEnabled =
///     safevalue::SafeHolder<PagingEnabledNDM, true, true, false>;
/// ```
///
/// # Bounds
///
/// By default a marker is as restrictive as a [SafeHolder] can be
/// (`WRITE_ONCE = true`, `READ_ONCE = true`, `PERMANENT = false`). Bounds after
/// a colon open it up - or keep it on one thread:
///
/// | Bound       | Effect |
/// |-------------|--------|
/// | `readable`  | `READ_ONCE = false`: [Deref](core::ops::Deref), [Eq], [Ord], [Hash](core::hash::Hash), [Debug](core::fmt::Debug), ... - for those the hidden struct implements |
/// | `writable`  | `WRITE_ONCE = false`: [set()](SafeHolder::set). Only for markers with data. |
/// | `permanent` | `PERMANENT = true`: implies `readable`, is [Copy]/[Clone] if the hidden struct is, has no [take()](SafeHolder::take). Can't be `writable`. |
/// | `!Send + !Sync` | The guarantee only holds on the thread (or CPU core) that vouched for it. |
/// | `!Sync`     | The marker may be moved to another thread, but not shared with one. |
///
/// `!Send` alone is rejected: the marker would still be [Sync], so a shared
/// reference to it could still reach another thread.
///
/// ```
/// # use safevalue::unsafe_marker;
/// unsafe_marker! {
///     /// Interrupts are disabled on this core.
///     pub struct InterruptsDisabled: !Send + !Sync;
/// }
/// ```
///
/// Moving such a marker to another thread doesn't compile:
/// ```compile_fail,E0277
/// # use safevalue::unsafe_marker;
/// unsafe_marker!(pub struct ThisThreadOnly: !Send + !Sync);
///
/// fn send<T: Send>(_: T) {}
/// send(unsafe { ThisThreadOnly::vouch() });
/// ```
/// and neither does sharing it with one:
/// ```compile_fail,E0277
/// # use safevalue::unsafe_marker;
/// unsafe_marker!(pub struct ThisThreadOnly: !Send + !Sync);
///
/// fn sync<T: Sync>() {}
/// sync::<ThisThreadOnly>();
/// ```
///
/// # Data
///
/// A marker can be a tuple struct, vouching for the data in it:
/// ```
/// # use safevalue::unsafe_marker;
/// unsafe_marker! {
///     /// The number of cores found at boot. It never changes afterwards.
///     #[derive(Debug, Clone, Copy, PartialEq)]
///     pub struct CoreCount(pub u32): permanent;
/// }
///
/// // SAFETY: an example; pretend we counted them.
/// let cores = unsafe { CoreCount::vouch_for(CoreCountNDM::new(4)) };
/// let also_cores = cores; // permanent: copied, not moved
/// assert_eq!(cores.0, 4);
/// assert_eq!(cores, also_cores);
/// ```
/// Markers with data are created with [vouch_for()](SafeHolder::vouch_for)
/// rather than [vouch()](SafeHolder::vouch). The hidden struct gets a
/// `const fn new`, taking its fields - use it, since `!Send`/`!Sync` add a
/// field of their own.
///
/// # Attributes
///
/// - Doc comments, `#[doc(...)]` and `#[deprecated]` go on the marker.
/// - `#[allow]`, `#[warn]`, `#[deny]`, `#[forbid]` and `#[cfg]` go on both.
/// - Everything else - `#[derive(...)]`, `#[repr(...)]`, `#[cfg_attr(...)]`,
///   custom attributes - goes on the hidden struct. The marker gets the derived
///   traits through [SafeHolder], which implements them for the data it holds
///   (most of them only when the marker is `readable`).
///
/// # Old syntax
///
/// `unsafe_marker!(pub Name)`, without `struct`, still works but is
/// deprecated. It creates a `readable` marker, as it always has - so when
/// switching, write `pub struct Name: readable;` if you rely on that.
macro_rules! unsafe_marker {
    // The public entry point is the last rule; these are internal.

    // Sort the attributes into [marker] [both] [hidden struct].
    (@attrs [$($m:tt)*] [$($b:tt)*] [$($h:tt)*]
        #[doc $($a:tt)*] $($rest:tt)*) => {
        $crate::unsafe_marker! {
            @attrs [$($m)* #[doc $($a)*]] [$($b)*] [$($h)*] $($rest)*
        }
    };
    (@attrs [$($m:tt)*] [$($b:tt)*] [$($h:tt)*]
        #[deprecated $($a:tt)*] $($rest:tt)*) => {
        $crate::unsafe_marker! {
            @attrs [$($m)* #[deprecated $($a)*]] [$($b)*] [$($h)*] $($rest)*
        }
    };
    (@attrs [$($m:tt)*] [$($b:tt)*] [$($h:tt)*]
        #[allow $($a:tt)*] $($rest:tt)*) => {
        $crate::unsafe_marker! {
            @attrs [$($m)*] [$($b)* #[allow $($a)*]] [$($h)*] $($rest)*
        }
    };
    (@attrs [$($m:tt)*] [$($b:tt)*] [$($h:tt)*]
        #[warn $($a:tt)*] $($rest:tt)*) => {
        $crate::unsafe_marker! {
            @attrs [$($m)*] [$($b)* #[warn $($a)*]] [$($h)*] $($rest)*
        }
    };
    (@attrs [$($m:tt)*] [$($b:tt)*] [$($h:tt)*]
        #[deny $($a:tt)*] $($rest:tt)*) => {
        $crate::unsafe_marker! {
            @attrs [$($m)*] [$($b)* #[deny $($a)*]] [$($h)*] $($rest)*
        }
    };
    (@attrs [$($m:tt)*] [$($b:tt)*] [$($h:tt)*]
        #[forbid $($a:tt)*] $($rest:tt)*) => {
        $crate::unsafe_marker! {
            @attrs [$($m)*] [$($b)* #[forbid $($a)*]] [$($h)*] $($rest)*
        }
    };
    (@attrs [$($m:tt)*] [$($b:tt)*] [$($h:tt)*]
        #[cfg $($a:tt)*] $($rest:tt)*) => {
        $crate::unsafe_marker! {
            @attrs [$($m)*] [$($b)* #[cfg $($a)*]] [$($h)*] $($rest)*
        }
    };
    (@attrs [$($m:tt)*] [$($b:tt)*] [$($h:tt)*]
        #[$($a:tt)*] $($rest:tt)*) => {
        $crate::unsafe_marker! {
            @attrs [$($m)*] [$($b)*] [$($h)* #[$($a)*]] $($rest)*
        }
    };

    // The struct itself. Defaults: not readable, not writable, not
    // permanent, Send, Sync.
    (@attrs $m:tt $b:tt $h:tt $v:vis struct $name:ident $(;)?) => {
        $crate::unsafe_marker! {
            @bounds {$m $b $h [$v] $name []} [false false false true true]
        }
    };
    (@attrs $m:tt $b:tt $h:tt $v:vis struct $name:ident : $($bounds:tt)+) => {
        $crate::unsafe_marker! {
            @bounds {$m $b $h [$v] $name []} [false false false true true]
            $($bounds)+
        }
    };
    (@attrs $m:tt $b:tt $h:tt
        $v:vis struct $name:ident ($($fields:tt)*) $(;)?) => {
        $crate::unsafe_marker! {
            @bounds {$m $b $h [$v] $name [$($fields)*]}
            [false false false true true]
        }
    };
    (@attrs $m:tt $b:tt $h:tt
        $v:vis struct $name:ident ($($fields:tt)*) : $($bounds:tt)+) => {
        $crate::unsafe_marker! {
            @bounds {$m $b $h [$v] $name [$($fields)*]}
            [false false false true true] $($bounds)+
        }
    };
    // The deprecated form without `struct`, as readable as it always was.
    (@attrs $m:tt $b:tt $h:tt $v:vis $name:ident) => {
        const _: () = {
            #[deprecated(note = "write `struct` before the marker's name, \
                e.g. `unsafe_marker!(pub struct Name: readable);` - `readable` \
                keeps what this form did, new markers are read-once by \
                default")]
            #[allow(non_upper_case_globals)]
            const $name: () = ();
            $name
        };
        $crate::unsafe_marker! {
            @emit {$m $b $h [$v] $name []} [] [true false false]
        }
    };
    (@attrs $m:tt $b:tt $h:tt $($rest:tt)*) => {
        ::core::compile_error!(
            "expected a marker like `pub struct Name;` or \
             `pub struct Name(Type);`, optionally followed by bounds, like \
             `pub struct Name: readable + !Sync;`"
        );
    };

    // Bounds, as [readable writable permanent Send Sync].
    (@bounds $d:tt $f:tt $(;)?) => {
        $crate::unsafe_marker! { @check $d $f }
    };
    (@bounds $d:tt [false $w:tt $p:tt $s:tt $y:tt]
        readable $($rest:tt)*) => {
        $crate::unsafe_marker! { @sep $d [true $w $p $s $y] $($rest)* }
    };
    (@bounds $d:tt [$r:tt false $p:tt $s:tt $y:tt]
        writable $($rest:tt)*) => {
        $crate::unsafe_marker! { @sep $d [$r true $p $s $y] $($rest)* }
    };
    (@bounds $d:tt [$r:tt $w:tt false $s:tt $y:tt]
        permanent $($rest:tt)*) => {
        $crate::unsafe_marker! { @sep $d [$r $w true $s $y] $($rest)* }
    };
    (@bounds $d:tt [$r:tt $w:tt $p:tt true $y:tt] !Send $($rest:tt)*) => {
        $crate::unsafe_marker! { @sep $d [$r $w $p false $y] $($rest)* }
    };
    (@bounds $d:tt [$r:tt $w:tt $p:tt $s:tt true] !Sync $($rest:tt)*) => {
        $crate::unsafe_marker! { @sep $d [$r $w $p $s false] $($rest)* }
    };
    (@bounds $d:tt $f:tt readable $($rest:tt)*) => {
        ::core::compile_error!("`readable` is listed twice");
    };
    (@bounds $d:tt $f:tt writable $($rest:tt)*) => {
        ::core::compile_error!("`writable` is listed twice");
    };
    (@bounds $d:tt $f:tt permanent $($rest:tt)*) => {
        ::core::compile_error!("`permanent` is listed twice");
    };
    (@bounds $d:tt $f:tt !Send $($rest:tt)*) => {
        ::core::compile_error!("`!Send` is listed twice");
    };
    (@bounds $d:tt $f:tt !Sync $($rest:tt)*) => {
        ::core::compile_error!("`!Sync` is listed twice");
    };
    (@bounds $d:tt $f:tt ! $other:tt $($rest:tt)*) => {
        ::core::compile_error!(::core::concat!(
            "unknown marker bound `!", ::core::stringify!($other), "`, \
             expected `readable`, `writable`, `permanent`, `!Send` or `!Sync`"
        ));
    };
    (@bounds $d:tt $f:tt $other:tt $($rest:tt)*) => {
        ::core::compile_error!(::core::concat!(
            "unknown marker bound `", ::core::stringify!($other), "`, \
             expected `readable`, `writable`, `permanent`, `!Send` or `!Sync`"
        ));
    };
    (@sep $d:tt $f:tt $(;)?) => {
        $crate::unsafe_marker! { @check $d $f }
    };
    (@sep $d:tt $f:tt + $($rest:tt)+) => {
        $crate::unsafe_marker! { @bounds $d $f $($rest)+ }
    };
    (@sep $d:tt $f:tt $($rest:tt)*) => {
        ::core::compile_error!(
            "expected `+` between the bounds of a marker, or `;` after them"
        );
    };

    // Reject combinations that make no sense, and turn `!Send`/`!Sync` into
    // the type of a zero-sized field that takes them away.
    (@check $d:tt [$r:tt true true $s:tt $y:tt]) => {
        ::core::compile_error!(
            "a marker can't be both `permanent` and `writable`: each copy of \
             it could be `set` to something different"
        );
    };
    (@check $d:tt [$r:tt $w:tt $p:tt false true]) => {
        ::core::compile_error!(
            "`!Send` needs `!Sync` as well: a `Sync` marker can still reach \
             another thread, through a shared reference"
        );
    };
    (@check {$m:tt $b:tt $h:tt $v:tt $name:ident []}
        [$r:tt true $p:tt $s:tt $y:tt]) => {
        ::core::compile_error!(
            "a marker without data can't be `writable`: there is nothing to \
             `set`"
        );
    };
    (@check $d:tt [$r:tt $w:tt $p:tt true true]) => {
        $crate::unsafe_marker! { @params $d [] [$r $w $p] }
    };
    (@check $d:tt [$r:tt $w:tt $p:tt false false]) => {
        // `*const ()` is neither `Send` nor `Sync`.
        $crate::unsafe_marker! { @params $d [*const ()] [$r $w $p] }
    };
    (@check $d:tt [$r:tt $w:tt $p:tt true false]) => {
        // `Cell` is `Send`, but not `Sync`.
        $crate::unsafe_marker! {
            @params $d [::core::cell::Cell<()>] [$r $w $p]
        }
    };

    // [readable writable permanent] to the SafeHolder's
    // [WRITE_ONCE READ_ONCE PERMANENT].
    (@params $d:tt $t:tt [$r:tt false true]) => {
        $crate::unsafe_marker! { @emit $d $t [true false true] }
    };
    (@params $d:tt $t:tt [true false false]) => {
        $crate::unsafe_marker! { @emit $d $t [true false false] }
    };
    (@params $d:tt $t:tt [true true false]) => {
        $crate::unsafe_marker! { @emit $d $t [false false false] }
    };
    (@params $d:tt $t:tt [false false false]) => {
        $crate::unsafe_marker! { @emit $d $t [true true false] }
    };
    (@params $d:tt $t:tt [false true false]) => {
        $crate::unsafe_marker! { @emit $d $t [false true false] }
    };

    // A marker without data.
    (@emit {[$($m:tt)*] [$($b:tt)*] [$($h:tt)*] [$v:vis] $name:ident []}
        [$($thread:ty)?] [$wo:tt $ro:tt $pe:tt]) => {
        $crate::paste! {
            $($b)*
            #[doc(hidden)]
            $($h)*
            $v struct [<$name NDM>] {
                $(_thread: ::core::marker::PhantomData<$thread>,)?
            }

            $($b)*
            impl $crate::NonDataMarker for [<$name NDM>] {
                const NEW_MARKER: Self = Self {
                    $(_thread: ::core::marker::PhantomData::<$thread>,)?
                };
            }

            $($b)*
            $($m)*
            #[allow(private_interfaces)]
            $v type $name = $crate::SafeHolder<[<$name NDM>], $wo, $ro, $pe>;
        }
    };
    // A marker with data.
    (@emit {[$($m:tt)*] [$($b:tt)*] [$($h:tt)*] [$v:vis] $name:ident
        [$($(#[$fa:meta])* $fv:vis $ft:ty),+ $(,)?]}
        [$($thread:ty)?] [$wo:tt $ro:tt $pe:tt]) => {
        $crate::paste! {
            $($b)*
            #[doc(hidden)]
            $($h)*
            $v struct [<$name NDM>](
                $($(#[$fa])* $fv $ft,)+
                $(::core::marker::PhantomData<$thread>,)?
            );

            $($b)*
            impl [<$name NDM>] {
                $crate::unsafe_marker! {
                    @new [$v] [] [] [$($ft),+]
                    [$(::core::marker::PhantomData::<$thread>)?]
                }
            }

            $($b)*
            $($m)*
            #[allow(private_interfaces)]
            $v type $name = $crate::SafeHolder<[<$name NDM>], $wo, $ro, $pe>;
        }
    };

    // `new` for a marker with data: one parameter per field. Each `field`
    // comes from a different expansion of this rule, so hygiene keeps them
    // apart.
    (@new [$v:vis] [$($p:tt)*] [$($a:tt)*] [] [$($thread:tt)*]) => {
        /// Creates the data for the marker. Vouching for it is up to
        /// `SafeHolder::vouch_for`.
        #[allow(dead_code, clippy::too_many_arguments)]
        $v const fn new($($p)*) -> Self { Self($($a)* $($thread)*) }
    };
    (@new $v:tt [$($p:tt)*] [$($a:tt)*] [$t:ty $(, $($rest:tt)*)?]
        $thread:tt) => {
        $crate::unsafe_marker! {
            @new $v [$($p)* field: $t,] [$($a)* field,] [$($($rest)*)?]
            $thread
        }
    };

    ($($input:tt)*) => {
        $crate::unsafe_marker! { @attrs [] [] [] $($input)* }
    };
}

#[doc(alias = "Marker")]
#[macro_export]
#[deprecated(
    note = "use `unsafe_marker!(pub struct Name: readable + !Send + !Sync)`"
)]
/// Deprecated: use [unsafe_marker] with `!Send + !Sync` instead.
///
/// `unsafe_marker_no_send!(pub Name)` is the same as
/// `unsafe_marker!(pub struct Name: readable + !Send + !Sync)`.
macro_rules! unsafe_marker_no_send {
    ($(#[$($attr:tt)*])* $v:vis $i:ident) => {
        $crate::unsafe_marker! {
            $(#[$($attr)*])*
            $v struct $i: readable + !Send + !Sync
        }
    };
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

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct Forever(u64);
    type SafeForever = SafeHolder<Forever, true, false, true>;

    #[test]
    fn permanent_guarantees_are_copy_and_clone_without_unsafe() {
        // SAFETY: only a test.
        let original = unsafe { SafeForever::vouch_for(Forever(7)) };

        let copied = original; // Copy: `original` is still usable
        #[allow(clippy::clone_on_copy)]
        let cloned = original.clone(); // resolves to Clone::clone - no `unsafe` needed

        // All three read the same fact, through Deref, as often as they like.
        assert_eq!(*original, Forever(7));
        assert_eq!(*copied, Forever(7));
        assert_eq!(*cloned.as_ref(), Forever(7));
        assert_eq!(original, copied);
    }

    #[test]
    fn unchecked_clone_is_still_available_for_other_types() {
        let safe_u64 = unsafe { SafeU64::vouch_for(5u64) };
        let copy = unsafe { safe_u64.clone_unchecked() };
        assert_eq!(copy.take(), 5u64);
        assert_eq!(safe_u64.take(), 5u64);
    }

    #[test]
    fn display_and_hash_forward_to_the_data() {
        extern crate std;
        use core::hash::BuildHasher;

        let safe_u64 = unsafe { SafeU64::vouch_for(42u64) };
        assert_eq!(std::format!("{safe_u64}"), "42");

        let hasher = std::hash::RandomState::new();
        assert_eq!(hasher.hash_one(&safe_u64), hasher.hash_one(42u64));
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

    unsafe_marker!(struct Test);
    unsafe_marker!(
        /// do we have documentation?
        pub struct Test2;
    );

    unsafe_marker!(
        /// do we have documentation?
        pub struct Test3();
    );
    unsafe_marker!(
        pub struct Test4;
    );

    unsafe_marker! {
        pub struct SafeMarker
    }

    #[test]
    pub fn test_marker() {
        let mut _marker = unsafe { SafeMarker::vouch() };

        let marker = unsafe { Test::vouch() };
        let marker2 = unsafe { Test2::vouch() };
        let marker3 = unsafe { Test3::vouch() };
        let _marker4 = unsafe { Test4::vouch() };

        let _ = marker.trust();
        let _ = marker.take();

        if marker2.trust() {
            // we can use this in if
        }

        marker3.rely_on();
    }

    unsafe_marker!(struct NoSend: !Send + !Sync);
    unsafe_marker!(
        /// do we have documentation?
        pub struct NoSend2: !Sync + !Send;
    );
    unsafe_marker!(pub struct NoSend3: !Send + !Sync + readable;);

    #[test]
    pub fn test_no_send_marker() {
        let marker = unsafe { NoSend::vouch() };
        let marker2 = unsafe { NoSend2::vouch() };
        let marker3 = unsafe { NoSend3::vouch() };

        assert_marker(&marker);
        let _ = marker.trust();
        take_marker(marker);

        if marker2.trust() {
            // we can use this in if
        }

        marker3.rely_on();
        let _ = marker3.take();
    }

    #[test]
    pub fn no_send_markers_are_zero_sized() {
        assert_eq!(core::mem::size_of::<NoSend>(), 0);
        assert_eq!(core::mem::size_of::<SafeMarker>(), 0);
    }

    #[test]
    pub fn only_ordinary_markers_are_send_and_sync() {
        fn send_sync<T: Send + Sync>() {}
        // The counterpart - `NoSend` being neither - is checked by the
        // compile-fail tests (tests/ui/no_send_marker_*.rs).
        send_sync::<SafeMarker>();
    }

    unsafe_marker!(struct NoShare: !Sync);

    #[test]
    pub fn not_sync_markers_may_still_move() {
        fn send<T: Send>() {}
        // ... but not be shared, see tests/ui/marker_bounds_restrict_threads.rs
        send::<NoShare>();
        assert_eq!(core::mem::size_of::<NoShare>(), 0);
    }

    /// The parameters of the SafeHolder a marker is, as
    /// (WRITE_ONCE, READ_ONCE, PERMANENT).
    fn params<
        T,
        const WRITE_ONCE: bool,
        const READ_ONCE: bool,
        const PERMANENT: bool,
    >(
        _: Option<SafeHolder<T, WRITE_ONCE, READ_ONCE, PERMANENT>>,
    ) -> (bool, bool, bool) {
        (WRITE_ONCE, READ_ONCE, PERMANENT)
    }

    unsafe_marker!(struct Restrictive);
    unsafe_marker!(struct Readable: readable);
    unsafe_marker!(struct Permanent: permanent);
    unsafe_marker!(struct ReadablePermanent: readable + permanent);
    unsafe_marker!(#[allow(dead_code)] struct Writable(u8): writable);
    unsafe_marker!(#[allow(dead_code)] struct ReadWrite(u8): writable + readable);
    unsafe_marker! {
        #[allow(dead_code)]
        struct Everything(u8): readable + writable + !Send + !Sync;
    }

    #[test]
    pub fn bounds_open_up_the_most_restrictive_default() {
        assert_eq!(params(None::<Restrictive>), (true, true, false));
        assert_eq!(params(None::<Readable>), (true, false, false));
        assert_eq!(params(None::<Permanent>), (true, false, true));
        assert_eq!(params(None::<ReadablePermanent>), (true, false, true));
        assert_eq!(params(None::<Writable>), (false, true, false));
        assert_eq!(params(None::<ReadWrite>), (false, false, false));
        assert_eq!(params(None::<Everything>), (false, false, false));
    }

    unsafe_marker! {
        /// A marker with data.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #[repr(C)]
        pub struct CoreCount(pub u32): permanent;
    }
    unsafe_marker! {
        #[derive(Debug, PartialEq)]
        struct Region(
            /// start
            pub usize,
            pub(crate) usize,
        ): readable + writable + !Send + !Sync;
    }

    #[test]
    pub fn markers_can_hold_data() {
        let cores = unsafe { CoreCount::vouch_for(CoreCountNDM::new(4)) };
        let copied = cores;
        assert_eq!(cores, copied);
        assert_eq!(cores.0, 4);
        assert_eq!(core::mem::size_of::<CoreCount>(), 4);

        let mut region =
            unsafe { Region::vouch_for(RegionNDM::new(0x1000, 0x2000)) };
        unsafe { region.set(RegionNDM::new(0x3000, 0x4000)) };
        assert_eq!((region.0, region.1), (0x3000, 0x4000));
        assert_eq!(*region, RegionNDM::new(0x3000, 0x4000));
        assert_eq!(core::mem::size_of::<Region>(), 2 * 8);
    }

    #[test]
    pub fn derives_reach_readable_markers() {
        extern crate std;
        let cores = unsafe { CoreCount::vouch_for(CoreCountNDM::new(2)) };
        assert_eq!(
            std::format!("{cores:?}"),
            "SafeHolder { data: CoreCountNDM(2) }"
        );
    }

    // `cfg` has to reach the generated impls too, or this wouldn't compile.
    unsafe_marker! {
        #[cfg(any())]
        #[derive(Debug)]
        pub struct ConfiguredAway(u32): readable;
    }
    unsafe_marker! {
        #[allow(dead_code)]
        #[cfg(all())]
        pub struct ConfiguredIn;
    }

    #[test]
    pub fn cfg_applies_to_the_whole_marker() {
        let _ = unsafe { ConfiguredIn::vouch() };
    }

    #[allow(deprecated)]
    mod old_syntax {
        unsafe_marker!(OldMarker);
        unsafe_marker!(
            /// do we have documentation?
            pub OldMarker2
        );
        unsafe_marker_no_send!(
            /// do we have documentation?
            pub OldNoSend
        );

        #[test]
        pub fn still_works_and_is_as_readable_as_before() {
            use super::params;
            assert_eq!(params(None::<OldMarker>), (true, false, false));
            assert_eq!(params(None::<OldMarker2>), (true, false, false));
            assert_eq!(params(None::<OldNoSend>), (true, false, false));
            let _ = unsafe { OldMarker::vouch() };
            let _ = unsafe { OldNoSend::vouch() };
        }
    }
}
