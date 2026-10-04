#![allow(unused)]
extern crate safevalue;
use safevalue::unsafe_marker;

unsafe_marker! {
    #[derive(Debug, PartialEq)]
    pub struct Readable(u32): readable;
}
unsafe_marker! {
    #[derive(Debug, PartialEq)]
    pub struct ReadOnce(u32);
}

fn debug<T: core::fmt::Debug>(_: &T) {}

pub fn main() {
    let readable = unsafe { Readable::vouch_for(ReadableNDM::new(1)) };
    let read_once = unsafe { ReadOnce::vouch_for(ReadOnceNDM::new(1)) };

    // these work: `readable` passes the derives on
    debug(&readable);
    let _ = *readable;

    // these fail: without `readable`, the data can only be `take`n
    debug(&read_once);
    let _ = *read_once;

    // and without `writable`, it can't be changed
    unsafe { read_once.set(ReadOnceNDM::new(2)) };
}
