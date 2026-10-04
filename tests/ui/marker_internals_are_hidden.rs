#![allow(unused)]
extern crate safevalue;

mod markers {
    use safevalue::unsafe_marker;

    unsafe_marker!(pub struct Secret(u32): readable);
}

pub fn main() {
    // works: the data is reachable through the marker's API
    let secret = unsafe { markers::Secret::vouch_for(1) };
    let _: u32 = *secret;

    // fails: outside of where the marker is defined, the hidden struct can't
    // even be named (for where it is defined, see
    // marker_internals_are_hidden_where_defined.rs)
    let _ = markers::__secret_ndm::SecretNDM { data: 1 };
}
