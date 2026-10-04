//! [MarkerData](crate::MarkerData) for types from `core`.

use core::marker::PhantomData;
use core::num::{
    NonZeroI8,
    NonZeroI16,
    NonZeroI32,
    NonZeroI64,
    NonZeroI128,
    NonZeroIsize,
    NonZeroU8,
    NonZeroU16,
    NonZeroU32,
    NonZeroU64,
    NonZeroU128,
    NonZeroUsize,
    Saturating,
    Wrapping,
};
use core::ptr::NonNull;
use core::time::Duration;

use super::data_is_self;

data_is_self! {
    [] (),
    [] bool,
    [] char,
    [] u8, [] u16, [] u32, [] u64, [] u128, [] usize,
    [] i8, [] i16, [] i32, [] i64, [] i128, [] isize,
    [] f32, [] f64,
    [] NonZeroU8, [] NonZeroU16, [] NonZeroU32, [] NonZeroU64,
    [] NonZeroU128, [] NonZeroUsize,
    [] NonZeroI8, [] NonZeroI16, [] NonZeroI32, [] NonZeroI64,
    [] NonZeroI128, [] NonZeroIsize,
    [] Duration,

    ['a, T: ?Sized] &'a T,
    ['a, T: ?Sized] &'a mut T,
    [T: ?Sized] *const T,
    [T: ?Sized] *mut T,
    [T: ?Sized] NonNull<T>,
    [T: ?Sized] PhantomData<T>,
    [T, const N: usize] [T; N],
    [T] Option<T>,
    [T, E] Result<T, E>,
    [T] Wrapping<T>,
    [T] Saturating<T>,

    [A] (A,),
    [A, B] (A, B),
    [A, B, C] (A, B, C),
    [A, B, C, D] (A, B, C, D),
    [A, B, C, D, E] (A, B, C, D, E),
    [A, B, C, D, E, F] (A, B, C, D, E, F),
    [A, B, C, D, E, F, G] (A, B, C, D, E, F, G),
    [A, B, C, D, E, F, G, H] (A, B, C, D, E, F, G, H),
    [A, B, C, D, E, F, G, H, I] (A, B, C, D, E, F, G, H, I),
    [A, B, C, D, E, F, G, H, I, J] (A, B, C, D, E, F, G, H, I, J),
    [A, B, C, D, E, F, G, H, I, J, K] (A, B, C, D, E, F, G, H, I, J, K),
    [A, B, C, D, E, F, G, H, I, J, K, L] (A, B, C, D, E, F, G, H, I, J, K, L),
}
