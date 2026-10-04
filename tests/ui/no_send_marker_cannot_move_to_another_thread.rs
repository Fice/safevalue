#![allow(unused)]
extern crate safevalue;
use safevalue::unsafe_marker;

unsafe_marker!(pub struct AnyThread);
unsafe_marker!(pub struct ThisThreadOnly: !Send + !Sync);

pub fn main() {
    // works: an ordinary marker can go to another thread
    let any_thread = unsafe { AnyThread::vouch() };
    std::thread::spawn(move || any_thread.rely_on());

    // fails: a no-send marker can't
    let this_thread_only = unsafe { ThisThreadOnly::vouch() };
    std::thread::spawn(move || this_thread_only.rely_on());
}
