// Only for the intra-doc links.
#[cfg(doc)]
use crate::SafeHolder;

/// A [MarkerData](crate::MarkerData) type without any data, so a [SafeHolder]
/// of it can be created out of nothing, with [vouch()](SafeHolder::vouch).
///
/// [unsafe_marker](crate::unsafe_marker) implements it for the markers it
/// creates without data - use that rather than implementing it by hand.
///
/// > Only implement this trait for types that are distinct (e.g. not `()`), so
/// > that different markers can't be mixed up.
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
/// hidden struct for every marker, which only wraps the marker's data, and the
/// marker is a [SafeHolder] of it:
/// ```ignore
/// #[doc(hidden)]
/// mod __paging_enabled_ndm {
///     pub struct PagingEnabledNDM<D> { data: D }
///     impl<D> safevalue::MarkerData for PagingEnabledNDM<D> {
///         type Data = D;
///         // ...
///     }
///     impl safevalue::NonDataMarker for PagingEnabledNDM<()> { /* ... */ }
/// }
///
/// /// The page tables are set up.
/// pub type PagingEnabled = safevalue::SafeHolder<
///     __paging_enabled_ndm::PagingEnabledNDM<()>,
///     true,
///     true,
///     false,
/// >;
/// ```
/// The struct is an implementation detail: outside of where the marker is
/// defined it can't be named, and nowhere can it be built or read, since its
/// field is private to its own module. It hands out the data it wraps (see
/// [MarkerData](crate::MarkerData)), so it never shows up in the marker's API
/// either: [vouch_for()](SafeHolder::vouch_for), [take()](SafeHolder::take),
/// [set()](SafeHolder::set) and [Deref](core::ops::Deref) all work with the
/// data itself.
///
/// # Bounds
///
/// By default a marker is as restrictive as a [SafeHolder] can be
/// (`WRITE_ONCE = true`, `READ_ONCE = true`, `PERMANENT = false`). Bounds after
/// a colon open it up - or keep it on one thread:
///
/// | Bound       | Effect |
/// |-------------|--------|
/// | `readable`  | `READ_ONCE = false`: [Deref](core::ops::Deref), [Eq], [Ord], [Hash](core::hash::Hash), [Debug](core::fmt::Debug), ... - for those the data implements |
/// | `writable`  | `WRITE_ONCE = false`: [set()](SafeHolder::set). Only for markers with data. |
/// | `permanent` | `PERMANENT = true`: implies `readable`, is [Copy]/[Clone] if the data is, has no [take()](SafeHolder::take). Can't be `writable`. |
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
/// A marker can be a tuple struct, vouching for data. With one field, the data
/// is of that field's type; with several, it is a tuple of them:
/// ```
/// # use safevalue::unsafe_marker;
/// unsafe_marker! {
///     /// The number of cores found at boot. It never changes afterwards.
///     pub struct CoreCount(u32): permanent;
/// }
/// unsafe_marker! {
///     /// Physical memory nobody else uses: start and length.
///     pub struct FreeRegion(usize, usize);
/// }
///
/// // SAFETY: an example; pretend we counted them.
/// let cores = unsafe { CoreCount::vouch_for(4) };
/// let also_cores = cores; // permanent: copied, not moved
/// assert_eq!(*cores, 4);
/// assert_eq!(cores, also_cores);
///
/// // SAFETY: an example; pretend we looked it up.
/// let region = unsafe { FreeRegion::vouch_for((0x1000, 0x2000)) };
/// let (start, length) = region.take();
/// ```
/// Markers with data are created with [vouch_for()](SafeHolder::vouch_for)
/// rather than [vouch()](SafeHolder::vouch). Like for every [SafeHolder],
/// traits such as `Debug`, `Eq` or `Hash` come from the data - most of them
/// only when the marker is `readable`. For your own types,
/// `#[derive(MarkerData)]` them (see [MarkerData](crate::MarkerData)).
///
/// # Generics
///
/// A marker with data can be generic over it - with bounds and defaults, as
/// usual. The marker is then a generic type alias:
/// ```
/// # use safevalue::unsafe_marker;
/// pub trait Seed {
///     fn seed(&self) -> u64;
/// }
/// impl Seed for u64 {
///     fn seed(&self) -> u64 { *self }
/// }
///
/// unsafe_marker! {
///     /// Data to seed an RNG with: secret, so it can only be read once.
///     pub struct SecretSeed<T: Seed = u64>(T);
/// }
///
/// // SAFETY: an example; this is no secret at all.
/// let secret: SecretSeed = unsafe { SecretSeed::vouch_for(42) };
/// let seed = secret.take().seed();
/// ```
/// Rust ignores bounds on type aliases, so the bounds go on the hidden
/// struct's [MarkerData](crate::MarkerData) implementation instead, which
/// every [SafeHolder] requires: they are checked wherever the marker is used,
/// e.g. `SecretSeed::vouch_for(1u8)` doesn't compile, since `u8` isn't a
/// `Seed`.
///
/// Only type parameters are supported, and they have to be used by the data:
/// no lifetimes, no const generics, no `where` clauses (yet).
///
/// # Attributes
///
/// - Doc comments, `#[doc(...)]` and `#[deprecated]` go on the marker.
/// - `#[allow]`, `#[warn]`, `#[deny]`, `#[forbid]` and `#[cfg]` go on both.
/// - Everything else - `#[cfg_attr(...)]`, custom attributes - goes on the
///   hidden struct. It already has `Clone` and `Copy` (when the data has them),
///   and the marker's other traits come from the data, so derives aren't needed
///   there.
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

    // The struct itself, described as {marker-attrs both-attrs struct-attrs
    // [vis] name [fields] [generics]}. Defaults: not readable, not writable,
    // not permanent, Send, Sync.
    (@attrs $m:tt $b:tt $h:tt $v:vis struct $name:ident $(;)?) => {
        $crate::unsafe_marker! {
            @bounds {$m $b $h [$v] $name [] []} [false false false true true]
        }
    };
    (@attrs $m:tt $b:tt $h:tt $v:vis struct $name:ident : $($bounds:tt)+) => {
        $crate::unsafe_marker! {
            @bounds {$m $b $h [$v] $name [] []} [false false false true true]
            $($bounds)+
        }
    };
    (@attrs $m:tt $b:tt $h:tt
        $v:vis struct $name:ident ($($fields:tt)*) $(;)?) => {
        $crate::unsafe_marker! {
            @bounds {$m $b $h [$v] $name [$($fields)*] []}
            [false false false true true]
        }
    };
    (@attrs $m:tt $b:tt $h:tt
        $v:vis struct $name:ident ($($fields:tt)*) : $($bounds:tt)+) => {
        $crate::unsafe_marker! {
            @bounds {$m $b $h [$v] $name [$($fields)*] []}
            [false false false true true] $($bounds)+
        }
    };
    (@attrs $m:tt $b:tt $h:tt $v:vis struct $name:ident < $($rest:tt)*) => {
        $crate::unsafe_marker! { @generics {$m $b $h [$v] $name} [] $($rest)* }
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
            @data {$m $b $h [$v] $name [] []} [] [true false false]
        }
    };
    (@attrs $m:tt $b:tt $h:tt $($rest:tt)*) => {
        ::core::compile_error!(
            "expected a marker like `pub struct Name;` or \
             `pub struct Name(Type);`, optionally followed by bounds, like \
             `pub struct Name: readable + !Sync;`"
        );
    };

    // Generic parameters, up to the closing `>`, collected as
    // [{name [trait bounds] [= default]} ...]. Only type parameters are
    // supported. Bounds and defaults can hold `<...>` themselves, so their
    // nesting is counted, as [x x ...].
    (@generics $d:tt [$($g:tt)+] > $($rest:tt)*) => {
        $crate::unsafe_marker! { @after_generics $d [$($g)+] $($rest)* }
    };
    (@generics $d:tt $g:tt $lt:lifetime $($rest:tt)*) => {
        ::core::compile_error!(
            "lifetime parameters aren't supported by `unsafe_marker!` yet"
        );
    };
    (@generics $d:tt $g:tt const $($rest:tt)*) => {
        ::core::compile_error!(
            "const generic parameters aren't supported by `unsafe_marker!` yet"
        );
    };
    (@generics $d:tt $g:tt $p:ident : $($rest:tt)*) => {
        $crate::unsafe_marker! { @bound $d $g $p [] [] $($rest)* }
    };
    (@generics $d:tt $g:tt $p:ident $($rest:tt)*) => {
        $crate::unsafe_marker! { @bound $d $g $p [] [] $($rest)* }
    };
    (@generics $d:tt $g:tt $($rest:tt)*) => {
        ::core::compile_error!(
            "expected a generic type parameter, like `T`, `T: Trait` or \
             `T = u8`"
        );
    };

    // The bounds of a parameter: [bounds so far] [nesting].
    (@bound $d:tt [$($g:tt)*] $p:ident [$($b:tt)*] [] , $($rest:tt)*) => {
        $crate::unsafe_marker! {
            @generics $d [$($g)* {$p [$($b)*] []}] $($rest)*
        }
    };
    (@bound $d:tt [$($g:tt)*] $p:ident [$($b:tt)*] [] > $($rest:tt)*) => {
        $crate::unsafe_marker! {
            @after_generics $d [$($g)* {$p [$($b)*] []}] $($rest)*
        }
    };
    (@bound $d:tt $g:tt $p:ident $b:tt [] = $($rest:tt)*) => {
        $crate::unsafe_marker! { @default $d $g $p $b [] [] $($rest)* }
    };
    (@bound $d:tt $g:tt $p:ident [$($b:tt)*] [$($x:tt)*] < $($rest:tt)*) => {
        $crate::unsafe_marker! { @bound $d $g $p [$($b)* <] [x $($x)*] $($rest)* }
    };
    (@bound $d:tt $g:tt $p:ident [$($b:tt)*] [x $($x:tt)*] > $($rest:tt)*) => {
        $crate::unsafe_marker! { @bound $d $g $p [$($b)* >] [$($x)*] $($rest)* }
    };
    // `>>` is a single token: closing two levels, or one and the parameters.
    (@bound $d:tt $g:tt $p:ident [$($b:tt)*] [x x $($x:tt)*]
        >> $($rest:tt)*) => {
        $crate::unsafe_marker! { @bound $d $g $p [$($b)* >>] [$($x)*] $($rest)* }
    };
    (@bound $d:tt [$($g:tt)*] $p:ident [$($b:tt)*] [x] >> $($rest:tt)*) => {
        $crate::unsafe_marker! {
            @after_generics $d [$($g)* {$p [$($b)* >] []}] $($rest)*
        }
    };
    (@bound $d:tt $g:tt $p:ident [$($b:tt)*] $x:tt $t:tt $($rest:tt)*) => {
        $crate::unsafe_marker! { @bound $d $g $p [$($b)* $t] $x $($rest)* }
    };
    (@bound $($rest:tt)*) => {
        ::core::compile_error!("expected `>` after the generic parameters");
    };

    // The default of a parameter: [default so far] [nesting].
    (@default $d:tt [$($g:tt)*] $p:ident $b:tt [$($v:tt)*] []
        , $($rest:tt)*) => {
        $crate::unsafe_marker! {
            @generics $d [$($g)* {$p $b [= $($v)*]}] $($rest)*
        }
    };
    (@default $d:tt [$($g:tt)*] $p:ident $b:tt [$($v:tt)*] []
        > $($rest:tt)*) => {
        $crate::unsafe_marker! {
            @after_generics $d [$($g)* {$p $b [= $($v)*]}] $($rest)*
        }
    };
    (@default $d:tt $g:tt $p:ident $b:tt [$($v:tt)*] [$($x:tt)*]
        < $($rest:tt)*) => {
        $crate::unsafe_marker! {
            @default $d $g $p $b [$($v)* <] [x $($x)*] $($rest)*
        }
    };
    (@default $d:tt $g:tt $p:ident $b:tt [$($v:tt)*] [x $($x:tt)*]
        > $($rest:tt)*) => {
        $crate::unsafe_marker! {
            @default $d $g $p $b [$($v)* >] [$($x)*] $($rest)*
        }
    };
    (@default $d:tt $g:tt $p:ident $b:tt [$($v:tt)*] [x x $($x:tt)*]
        >> $($rest:tt)*) => {
        $crate::unsafe_marker! {
            @default $d $g $p $b [$($v)* >>] [$($x)*] $($rest)*
        }
    };
    (@default $d:tt [$($g:tt)*] $p:ident $b:tt [$($v:tt)*] [x]
        >> $($rest:tt)*) => {
        $crate::unsafe_marker! {
            @after_generics $d [$($g)* {$p $b [= $($v)* >]}] $($rest)*
        }
    };
    (@default $d:tt $g:tt $p:ident $b:tt [$($v:tt)*] $x:tt
        $t:tt $($rest:tt)*) => {
        $crate::unsafe_marker! { @default $d $g $p $b [$($v)* $t] $x $($rest)* }
    };
    (@default $($rest:tt)*) => {
        ::core::compile_error!("expected `>` after the generic parameters");
    };

    // After the generics: a generic marker needs data using its parameters.
    (@after_generics {$m:tt $b:tt $h:tt $v:tt $name:ident} $g:tt
        ($($fields:tt)+) $(;)?) => {
        $crate::unsafe_marker! {
            @bounds {$m $b $h $v $name [$($fields)+] $g}
            [false false false true true]
        }
    };
    (@after_generics {$m:tt $b:tt $h:tt $v:tt $name:ident} $g:tt
        ($($fields:tt)+) : $($bounds:tt)+) => {
        $crate::unsafe_marker! {
            @bounds {$m $b $h $v $name [$($fields)+] $g}
            [false false false true true] $($bounds)+
        }
    };
    (@after_generics $d:tt $g:tt $($rest:tt)*) => {
        ::core::compile_error!(
            "a generic marker needs data that uses its parameters, like \
             `pub struct Secret<T>(T);`"
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
    (@check {$m:tt $b:tt $h:tt $v:tt $name:ident [] $g:tt}
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
        $crate::unsafe_marker! { @data $d $t [true false true] }
    };
    (@params $d:tt $t:tt [true false false]) => {
        $crate::unsafe_marker! { @data $d $t [true false false] }
    };
    (@params $d:tt $t:tt [true true false]) => {
        $crate::unsafe_marker! { @data $d $t [false false false] }
    };
    (@params $d:tt $t:tt [false false false]) => {
        $crate::unsafe_marker! { @data $d $t [true true false] }
    };
    (@params $d:tt $t:tt [false true false]) => {
        $crate::unsafe_marker! { @data $d $t [false true false] }
    };

    // The type of the data: `()`, the one field's type, or a tuple of them.
    (@data {$m:tt $b:tt $h:tt $v:tt $name:ident [] $g:tt} $t:tt $p:tt) => {
        $crate::unsafe_marker! { @emit {$m $b $h $v $name $g} [()] $t $p }
    };
    (@data {$m:tt $b:tt $h:tt $v:tt $name:ident [pub $($f:tt)*] $g:tt}
        $t:tt $p:tt) => {
        ::core::compile_error!(
            "the fields of a marker are just types, like \
             `pub struct Region(usize, usize);` - they aren't accessed by \
             name, so they have no visibility"
        );
    };
    (@data {$m:tt $b:tt $h:tt $v:tt $name:ident [$f:ty $(,)?] $g:tt}
        $t:tt $p:tt) => {
        $crate::unsafe_marker! { @emit {$m $b $h $v $name $g} [$f] $t $p }
    };
    (@data {$m:tt $b:tt $h:tt $v:tt $name:ident [$f0:ty, $($f:ty),+ $(,)?]
        $g:tt} $t:tt $p:tt) => {
        $crate::unsafe_marker! {
            @emit {$m $b $h $v $name $g} [($f0, $($f),+)] $t $p
        }
    };

    // The hidden struct only makes the marker a distinct type, and hands out
    // the data through MarkerData. It lives in a private module of its own:
    // - outside of where the marker is defined, it can't even be named;
    // - where it is defined, it can be named, but neither built nor read,
    //   since its field is private to the inner module.
    // The struct itself is `pub`: other crates can't use values of a private
    // type, not even through a public alias. It is generic over the data, so
    // the data's type is resolved where the user wrote it, and `Clone`/`Copy`
    // follow the data's.
    (@emit {[$($m:tt)*] [$($b:tt)*] [$($h:tt)*] [$v:vis] $name:ident []}
        [$data:ty] [$($thread:ty)?] [$wo:tt $ro:tt $pe:tt]) => {
        $crate::paste! {
            $($b)*
            #[doc(hidden)]
            mod [<__ $name:snake _ndm>] {
                #[derive(Clone, Copy)]
                $($h)*
                pub struct [<$name NDM>]<D> {
                    data: D,
                    $(_thread: ::core::marker::PhantomData<$thread>,)?
                }

                impl<D> $crate::MarkerData for [<$name NDM>]<D> {
                    type Data = D;

                    #[inline(always)]
                    fn from_data(data: D) -> Self {
                        Self {
                            data,
                            $(_thread: ::core::marker::PhantomData::<$thread>,)?
                        }
                    }

                    #[inline(always)]
                    fn data(&self) -> &D { &self.data }

                    #[inline(always)]
                    fn data_mut(&mut self) -> &mut D { &mut self.data }

                    #[inline(always)]
                    fn into_data(self) -> D { self.data }
                }

                // Only used by markers without data, for `vouch()`.
                impl $crate::NonDataMarker for [<$name NDM>]<()> {
                    const NEW_MARKER: Self = Self {
                        data: (),
                        $(_thread: ::core::marker::PhantomData::<$thread>,)?
                    };
                }
            }

            $($b)*
            $($m)*
            $v type $name = $crate::SafeHolder<
                [<__ $name:snake _ndm>]::[<$name NDM>]<$data>,
                $wo,
                $ro,
                $pe,
            >;
        }
    };
    // A generic marker. Rust ignores bounds on type aliases, so the alias
    // only gets the parameters (and their defaults). The bounds go on the
    // `Bounds` impl instead, which MarkerData - and so `SafeHolder` - requires:
    // they are checked wherever the marker is used. `Bounds` is implemented
    // outside of the module, where the marker is written, so the bounds and
    // the data's type mean what they mean there (even inside a function).
    (@emit {[$($m:tt)*] [$($b:tt)*] [$($h:tt)*] [$v:vis] $name:ident
        [$({$p:ident [$($pb:tt)*] [$($pd:tt)*]})+]}
        [$data:ty] [$($thread:ty)?] [$wo:tt $ro:tt $pe:tt]) => {
        $crate::paste! {
            $($b)*
            #[doc(hidden)]
            mod [<__ $name:snake _ndm>] {
                #[derive(Clone, Copy)]
                $($h)*
                pub struct [<$name NDM>]<D> {
                    data: D,
                    $(_thread: ::core::marker::PhantomData<$thread>,)?
                }

                /// Implemented for the data the marker's generic bounds
                /// allow.
                pub trait Bounds {}

                impl<D> $crate::MarkerData for [<$name NDM>]<D>
                where
                    Self: Bounds,
                {
                    type Data = D;

                    #[inline(always)]
                    fn from_data(data: D) -> Self {
                        Self {
                            data,
                            $(_thread: ::core::marker::PhantomData::<$thread>,)?
                        }
                    }

                    #[inline(always)]
                    fn data(&self) -> &D { &self.data }

                    #[inline(always)]
                    fn data_mut(&mut self) -> &mut D { &mut self.data }

                    #[inline(always)]
                    fn into_data(self) -> D { self.data }
                }
            }

            $($b)*
            impl<$($p: $($pb)*),+> [<__ $name:snake _ndm>]::Bounds
                for [<__ $name:snake _ndm>]::[<$name NDM>]<$data>
            {
            }

            $($b)*
            $($m)*
            $v type $name<$($p $($pd)*),+> = $crate::SafeHolder<
                [<__ $name:snake _ndm>]::[<$name NDM>]<$data>,
                $wo,
                $ro,
                $pe,
            >;
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
/// Deprecated: use [unsafe_marker](crate::unsafe_marker) with `!Send + !Sync`
/// instead.
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
    use crate::{SafeHolder, assert_marker, take_marker};

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
        marker.invalidate();

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
        marker3.invalidate();
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
        T: crate::MarkerData,
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
        #[repr(C)]
        pub struct CoreCount(u32): permanent;
    }
    unsafe_marker! {
        struct Region(usize, usize,): readable + writable + !Send + !Sync;
    }
    unsafe_marker! {
        struct Secret(u64);
    }

    #[test]
    pub fn markers_hand_out_the_raw_data() {
        let cores = unsafe { CoreCount::vouch_for(4) };
        let copied = cores;
        assert_eq!(cores, copied);
        assert_eq!(*cores, 4);
        assert_eq!(*cores + 1, 5);
        assert_eq!(core::mem::size_of::<CoreCount>(), 4);

        let mut region = unsafe { Region::vouch_for((0x1000, 0x2000)) };
        unsafe { region.set((0x3000, 0x4000)) };
        assert_eq!((region.0, region.1), (0x3000, 0x4000));
        assert_eq!(*region, (0x3000, 0x4000));
        assert_eq!(core::mem::size_of::<Region>(), 2 * 8);

        let secret: u64 = unsafe { Secret::vouch_for(42) }.take();
        assert_eq!(secret, 42);
    }

    #[test]
    pub fn readable_markers_forward_the_datas_traits() {
        extern crate std;
        use core::hash::BuildHasher;

        let cores = unsafe { CoreCount::vouch_for(2) };
        assert_eq!(std::format!("{cores:?}"), "SafeHolder { data: 2 }");
        assert_eq!(std::format!("{cores} {cores:x}"), "2 2");
        let hasher = std::hash::RandomState::new();
        assert_eq!(hasher.hash_one(cores), hasher.hash_one(2u32));
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

    pub trait Seed {
        fn seed(&self) -> u64;
    }
    impl Seed for u64 {
        fn seed(&self) -> u64 { *self }
    }

    unsafe_marker! {
        /// Data to seed an RNG with - only to be read once.
        pub struct SecretSeed<T: Seed>(T);
    }
    unsafe_marker!(struct Unbounded<T>(T): readable);
    unsafe_marker!(struct WithDefault<T = u8>(T): readable);
    // Several parameters, `<...>` inside bounds and defaults, and `>>`/`>>>`
    // tokens closing bounds, defaults and the parameters themselves.
    unsafe_marker! {
        struct Nested<U: Into<u64>, T: Into<Option<u8>> + Copy = Option<u8>>(
            T,
            U,
        ): readable + !Sync;
    }
    unsafe_marker!(struct ShiftClose<T: Into<u8>>(T): readable);
    unsafe_marker!(struct ShiftCloseDefault<T = Option<u8>>(T));
    unsafe_marker!(struct DeepDefault<T = Option<Option<u8>>>(T));

    #[test]
    pub fn generic_markers_hold_their_parameters() {
        let seed = unsafe { SecretSeed::vouch_for(42u64) };
        assert_eq!(seed.take().seed(), 42);

        let text = unsafe { Unbounded::vouch_for("text") };
        assert_eq!(*text, "text");

        let byte: WithDefault = unsafe { WithDefault::vouch_for(7) };
        assert_eq!(*byte, 7u8);
        let wide = unsafe { WithDefault::<u32>::vouch_for(7) };
        assert_eq!(*wide, 7u32);

        let nested: Nested<u32> = unsafe { Nested::vouch_for((Some(1), 2)) };
        assert_eq!(*nested, (Some(1), 2));

        let shift = unsafe { ShiftClose::vouch_for(3u8) };
        assert_eq!(*shift, 3);
        let shift: ShiftCloseDefault =
            unsafe { ShiftCloseDefault::vouch_for(None) };
        assert_eq!(shift.take(), None);
        let deep: DeepDefault = unsafe { DeepDefault::vouch_for(Some(None)) };
        assert_eq!(deep.take(), Some(None));

        assert_eq!(params(None::<SecretSeed<u64>>), (true, true, false));
        assert_eq!(params(None::<Nested<u32>>), (true, false, false));
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
