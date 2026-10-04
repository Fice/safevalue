#![allow(unused)]
extern crate safevalue;
use safevalue::unsafe_marker;

unsafe_marker!(pub struct AnyThread);
unsafe_marker!(pub struct ThisThreadOnly: !Send + !Sync);

pub fn main() {
    // works: another thread can borrow an ordinary marker
    let any_thread = unsafe { AnyThread::vouch() };
    let any_thread = &any_thread;
    std::thread::scope(|scope| {
        scope.spawn(move || any_thread.rely_on());
    });

    // fails: a reference to a no-send marker can't leave its thread either
    let this_thread_only = unsafe { ThisThreadOnly::vouch() };
    let this_thread_only = &this_thread_only;
    std::thread::scope(|scope| {
        scope.spawn(move || this_thread_only.rely_on());
    });
}
