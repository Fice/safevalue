//! safevalue
//! =========
//! 
//! TODO: Links
//! # Purpose
//!
//!
//! # Usage

//!

//!

//make sure we run the code in the readme.md during testing
#![cfg_attr(doctest, doc = include_str!("../README.md"))]
#![doc(issue_tracker_base_url = "https://github.com/Fice/safevalue/issues")]
// We don't need std at all, so we might as well be no_std
// We can still use std in integration tests, so any tests that would require it
// can still do so.
#![no_std]
#![warn(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]

#[derive(Debug)]
#[repr(transparent)]
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
    /// Creates a new SafeHolder that vouches for the given 'data'.
    ///
    /// This is unsafe to make sure the safety requirements are uphold
    ///
    /// SAFETY
    /// - See SAFETY requirements of [`T`]
    #[inline(always)]
    pub const unsafe fn vouch_for(data: T) -> Self {
        Self {
            data,
            sealed: core::marker::PhantomData::<Sealed> {},
        }
    }

    #[inline(always)]
    pub fn take(self) -> T { self.data }

    #[inline(always)]
    pub fn invalidate(self) {}

    #[inline(always)]
    pub fn rely_on(&self) {}
}


impl<const WRITE_ONCE: bool, const READ_ONCE: bool>
    SafeHolder<(), WRITE_ONCE, READ_ONCE>
{
    #[inline(always)]
    pub const unsafe fn vouch() -> Self {
        Self {
            data:   (),
            sealed: core::marker::PhantomData::<Sealed> {},
        }
    }
}
impl<T, const READ_ONCE: bool> SafeHolder<T, false, READ_ONCE> {
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



/// Non-public struct that is used to prevent the use of ['SafeHolder'] that
/// circumvents the creation of the marker without the 'unsafe' functions
#[derive(Debug)]
struct Sealed {}


/// A marker type that you could use to denote that something has happened
/// without any associated data.
///
/// It's best to typedef it
//pub type SafeMarker = SafeHolder<()>;


// Without this, unit testing the macro fails, because safevale::SafeHolder not
// found.
mod safevalue {
    #[allow(unused_imports)]
    pub(crate) use super::SafeHolder;
}

#[macro_export]
macro_rules! unsafe_marker {
    (  $(#[doc = $doc:expr]) * $v:vis $i:ident ) => {
        $(
            #[doc = $doc]
        )*
        $v struct $i(safevalue::SafeHolder<(), true, false>);

        impl $i {
            #[inline(always)]
            pub const unsafe fn vouch() -> Self {
                Self(
                    unsafe { safevalue::SafeHolder::vouch() }
                )
            }
            #[allow(dead_code)]
            #[inline(always)]
            pub const fn trust(&self) -> bool { true }
            #[allow(dead_code)]
            #[inline(always)]
            pub const fn take(self) -> bool { true }
        }


        impl core::ops::Deref for $i
        {
            type Target = safevalue::SafeHolder<(), true, false>;

            // Required method
            fn deref(&self) -> &Self::Target { &self.0 }
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

    #[test]
    pub fn test_marker() {
        let marker = unsafe { Test::vouch() };
        let marker2 = unsafe { Test2::vouch() };
        let marker3 = unsafe { Test3::vouch() };
        let marker4 = unsafe { Test4::vouch() };

        marker.trust();
        marker.take();

        let _cp = marker4.clone();

        if marker2.trust() {
            // we can use this in if
        }

        marker3.rely_on();
    }
}
