#![allow(unused)]
extern crate safevalue;
use safevalue::unsafe_marker;

unsafe_marker!(pub struct NotShared: !Sync);
unsafe_marker!(pub struct NotSharedWithData(u32): !Sync);

fn send<T: Send>() {}
fn sync<T: Sync>() {}

pub fn main() {
    // these work: a `!Sync` marker may move to another thread
    send::<NotShared>();
    send::<NotSharedWithData>();

    // these fail: but it may not be shared with one
    sync::<NotShared>();
    sync::<NotSharedWithData>();
}
