#![allow(unused)]
extern crate safevalue;
use safevalue::unsafe_marker;

// these work
unsafe_marker!(pub struct Fine: readable + !Send + !Sync);
unsafe_marker!(pub struct AlsoFine(u32): writable + !Sync;);

// these fail
unsafe_marker!(pub struct Unknown: sendable);
unsafe_marker!(pub struct UnknownNegative: !Copy);
unsafe_marker!(pub struct Twice: readable + readable);
unsafe_marker!(pub struct NoPlus: readable writable);
unsafe_marker!(pub struct TrailingPlus: readable +);
unsafe_marker!(pub struct PermanentWritable(u32): permanent + writable);
unsafe_marker!(pub struct OnlyNotSend: !Send);
unsafe_marker!(pub struct WritableWithoutData: writable);
unsafe_marker!(pub struct Named { data: u32 });
unsafe_marker!(pub struct Generic<T>(T));

pub fn main() {}
