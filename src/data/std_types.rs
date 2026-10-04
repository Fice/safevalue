//! [MarkerData](crate::MarkerData) for types that are only in `std`, behind
//! the `std` feature. The ones `alloc` has too are in `alloc_types`.

extern crate std;

use std::collections::{HashMap, HashSet};
use std::ffi::OsString;
use std::path::PathBuf;

use super::data_is_self;

data_is_self! {
    [K, V, S] HashMap<K, V, S>,
    [T, S] HashSet<T, S>,
    [] PathBuf,
    [] OsString,
}
