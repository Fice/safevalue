#![allow(unused)]
extern crate safevalue;
use safevalue::{unsafe_marker, unsafe_marker_no_send};

unsafe_marker!(pub AnyThread);
unsafe_marker_no_send!(pub ThisThreadOnly);

fn send<T: Send>(_: T) {}
fn sync<T: Sync>() {}

pub fn main() {
    // these work: an ordinary marker is about the whole program
    send(unsafe { AnyThread::vouch() });
    sync::<AnyThread>();

    // these fail: a no-send marker must stay on the thread that vouched for it
    send(unsafe { ThisThreadOnly::vouch() });
    sync::<ThisThreadOnly>();
}
