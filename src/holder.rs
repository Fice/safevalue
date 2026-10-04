use crate::NonDataMarker;

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
    /// Especially useful for [Marker](crate::unsafe_marker) types where we
    /// don't have actual data to use.
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
    /// requirements of the [Marker](crate::unsafe_marker).
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
}
