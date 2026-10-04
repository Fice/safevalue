#![allow(unused)]
extern crate safevalue;
use safevalue::SafeHolder;

type Readable = SafeHolder<u32, true, false>;
type ReadOnce = SafeHolder<u32, true, true>;

fn hash<T: core::hash::Hash>(_: &T) {}
fn display<T: core::fmt::Display>(_: &T) {}

pub fn main() {
    let readable = unsafe { Readable::vouch_for(1) };
    let read_once = unsafe { ReadOnce::vouch_for(2) };

    // these work: the data may be looked at as often as you like
    hash(&readable);
    display(&readable);

    // these fail: hashing or printing would reveal data that may only be
    // read once, through `take`
    hash(&read_once);
    display(&read_once);
}
