//! What a [SafeHolder](crate::SafeHolder) can hold: [MarkerData].

#[cfg(feature = "alloc")]
mod alloc_types;
mod core_types;
#[cfg(feature = "std")]
mod std_types;

/// Data that can be held - and vouched for - by a
/// [SafeHolder](crate::SafeHolder).
///
/// A [SafeHolder](crate::SafeHolder) doesn't hand out what it stores
/// directly: it goes through this trait.
/// [vouch_for()](crate::SafeHolder::vouch_for),
/// [take()](crate::SafeHolder::take), [set()](crate::SafeHolder::set),
/// [Deref](core::ops::Deref) and the traits it forwards (`Debug`, `Eq`, ...)
/// all work with [Data](MarkerData::Data).
///
/// For almost every type, the data is simply the value itself. That is what
/// `#[derive(MarkerData)]` implements, and what this crate implements for
/// common types: the integers, `bool`, `char`, `()`, references, pointers,
/// arrays, tuples, `Option`, ... With the `alloc` feature also `String`,
/// `Vec`, `Box`, `Rc`, `Arc`, the `alloc` collections, ..., and with the `std`
/// feature (which includes `alloc`) also `HashMap`, `HashSet`, `PathBuf` and
/// `OsString`.
///
/// ```
/// # use safevalue::{MarkerData, SafeHolder};
/// #[derive(MarkerData)]
/// struct FreeMemory(*const u8);
///
/// // SAFETY: an example; the pointer is never used.
/// let free = unsafe {
///     SafeHolder::<FreeMemory>::vouch_for(FreeMemory(core::ptr::null()))
/// };
/// let FreeMemory(pointer) = free.take();
/// ```
///
/// The exception are the types [unsafe_marker](crate::unsafe_marker)
/// generates: they only exist to make every marker a distinct type, and hand
/// out the data they wrap, so they never show up in the marker's API.
pub trait MarkerData: Sized {
    /// What the [SafeHolder](crate::SafeHolder) hands out.
    type Data;

    /// Wraps `data` - used by [vouch_for()](crate::SafeHolder::vouch_for).
    fn from_data(data: Self::Data) -> Self;

    /// The data - used by [Deref](core::ops::Deref) and the forwarded traits.
    fn data(&self) -> &Self::Data;

    /// The data, mutably - used by [set()](crate::SafeHolder::set).
    fn data_mut(&mut self) -> &mut Self::Data;

    /// Unwraps the data - used by [take()](crate::SafeHolder::take).
    fn into_data(self) -> Self::Data;

    /// Replaces the data.
    #[inline(always)]
    fn set_data(&mut self, data: Self::Data) { *self.data_mut() = data; }
}

/// Implements [MarkerData] with `Data = Self`, like `#[derive(MarkerData)]`.
macro_rules! data_is_self {
    ($(
        $(#[$attr:meta])*
        [$($generics:tt)*] $ty:ty
    ),* $(,)?) => {
        $(
            $(#[$attr])*
            impl<$($generics)*> $crate::MarkerData for $ty {
                type Data = Self;

                #[inline(always)]
                fn from_data(data: Self) -> Self { data }

                #[inline(always)]
                fn data(&self) -> &Self { self }

                #[inline(always)]
                fn data_mut(&mut self) -> &mut Self { self }

                #[inline(always)]
                fn into_data(self) -> Self { self }
            }
        )*
    };
}
use data_is_self;

#[cfg(test)]
mod tests {
    use core::ptr::NonNull;

    use crate::{MarkerData, SafeHolder};

    type Readable<T> = SafeHolder<T, true, false>;

    #[test]
    fn common_types_are_their_own_data() {
        let text = unsafe { Readable::<&str>::vouch_for("text") };
        assert_eq!(*text, "text");

        let bytes = unsafe { Readable::<[u8; 2]>::vouch_for([1, 2]) };
        assert_eq!(bytes[1], 2);

        let pair = unsafe { Readable::<(u8, char)>::vouch_for((1, 'a')) };
        assert_eq!(pair.1, 'a');

        let maybe = unsafe { Readable::<Option<u8>>::vouch_for(None) };
        assert!(maybe.is_none());

        let mut value = 7u8;
        let pointer = NonNull::from(&mut value);
        let held = unsafe { SafeHolder::<NonNull<u8>>::vouch_for(pointer) };
        assert_eq!(held.take(), pointer);
    }

    #[derive(Debug, PartialEq, MarkerData)]
    struct Generic<'a, T: Clone>(&'a T)
    where
        T: PartialEq;

    #[test]
    fn derive_works_with_generics() {
        let value = 3;
        let held =
            unsafe { Readable::<Generic<i32>>::vouch_for(Generic(&value)) };
        assert_eq!(*held, Generic(&3));
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn alloc_types_are_their_own_data() {
        extern crate alloc;
        use alloc::string::String;
        use alloc::vec;
        use alloc::vec::Vec;

        let text = unsafe { Readable::<String>::vouch_for(String::from("hi")) };
        assert_eq!(text.len(), 2);

        let list = unsafe { Readable::<Vec<u8>>::vouch_for(vec![1]) };
        assert_eq!(list[0], 1);
    }

    #[cfg(feature = "std")]
    #[test]
    fn std_types_are_their_own_data() {
        extern crate std;
        use std::collections::HashMap;

        let map = unsafe {
            Readable::<HashMap<u8, u8>>::vouch_for(HashMap::from([(1, 2)]))
        };
        assert_eq!(map[&1], 2);
    }
}
