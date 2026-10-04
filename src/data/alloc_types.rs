//! [MarkerData](crate::MarkerData) for types from `alloc`, behind the `alloc`
//! feature (also enabled by `std`).

extern crate alloc;

use alloc::borrow::{Cow, ToOwned};
use alloc::boxed::Box;
use alloc::collections::{
    BTreeMap,
    BTreeSet,
    BinaryHeap,
    LinkedList,
    VecDeque,
};
use alloc::ffi::CString;
use alloc::rc::Rc;
use alloc::string::String;
#[cfg(target_has_atomic = "ptr")]
use alloc::sync::Arc;
use alloc::vec::Vec;

use super::data_is_self;

data_is_self! {
    [] String,
    [] CString,
    [T] Vec<T>,
    [T] VecDeque<T>,
    [T] LinkedList<T>,
    [T] BinaryHeap<T>,
    [K, V] BTreeMap<K, V>,
    [T] BTreeSet<T>,
    [T: ?Sized] Box<T>,
    [T: ?Sized] Rc<T>,
    #[cfg(target_has_atomic = "ptr")]
    [T: ?Sized] Arc<T>,
    ['a, B: ?Sized + ToOwned] Cow<'a, B>,
}
