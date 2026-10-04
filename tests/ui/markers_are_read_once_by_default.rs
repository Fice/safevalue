#![allow(unused)]
extern crate safevalue;

mod markers {
    use safevalue::unsafe_marker;

    unsafe_marker!(pub struct Readable(u32): readable);
    unsafe_marker!(pub struct ReadOnce(u32));
}
use markers::{ReadOnce, Readable};

fn debug<T: core::fmt::Debug>(_: &T) {}

pub fn main() {
    let readable = unsafe { Readable::vouch_for(1) };
    let mut read_once = unsafe { ReadOnce::vouch_for(1) };

    // these work: `readable` passes the data's traits on, and everything
    // works with the data itself
    debug(&readable);
    let _: u32 = *readable;
    let _: u32 = read_once.take();
    let mut read_once = unsafe { ReadOnce::vouch_for(1) };

    // these fail: without `readable`, the data can only be `take`n
    debug(&read_once);
    let _ = *read_once;

    // and without `writable`, it can't be changed
    unsafe { read_once.set(2) };
}
