#![allow(unused)]
extern crate safevalue;
use safevalue::unsafe_marker;

pub trait Seed {}
impl Seed for u64 {}

// these work
unsafe_marker!(pub struct SecretSeed<T: Seed>(T));
unsafe_marker!(pub struct WithDefault<T = u8>(T): readable);

// these fail: not supported (yet)
unsafe_marker!(pub struct Borrowed<'a>(&'a u8));
unsafe_marker!(pub struct Sized<const N: usize>([u8; N]));
// these fail: the parameters have to be used by the data
unsafe_marker!(pub struct NoData<T>);
unsafe_marker!(pub struct NoDataWithBounds<T>: readable);

pub fn main() {
    let seed = unsafe { SecretSeed::vouch_for(1u64) };

    // fails: the bound is checked wherever the marker is used
    let seed = unsafe { SecretSeed::vouch_for(1u8) };
}
