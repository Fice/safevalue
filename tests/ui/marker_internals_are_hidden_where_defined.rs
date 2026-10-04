#![allow(unused)]
extern crate safevalue;
use safevalue::unsafe_marker;

unsafe_marker!(pub struct Secret(u32): readable);

pub fn main() {
    // fails: even where the marker is defined, the hidden struct can be named,
    // but neither built nor read - its field is private to its own module
    let _ = __secret_ndm::SecretNDM { data: 1 };
}
