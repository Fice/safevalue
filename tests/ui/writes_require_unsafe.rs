#![allow(unused)]
extern crate safevalue;
use safevalue::{ unsafe_marker, SafeHolder };


type TF = SafeHolder<f32, true, false>;
type TT = SafeHolder<u32, true, true>;
type FF = SafeHolder<bool, false, false>;
type FT = SafeHolder<char, false, true>;
unsafe_marker!(pub SafeMarker);

pub fn main() {
    let mut ff = unsafe { FF::vouch_for(true) };
    let mut ft = unsafe { FT::vouch_for('a') };
    let mut tf = unsafe { TF::vouch_for(12.0) };
    let mut tt = unsafe { TT::vouch_for(37u32) };
    let mut marker = unsafe { SafeMarker::vouch() };

    // Un-Unsafe vouch should fail
    let mut ff2 = FF::vouch_for(true);
    let mut ft2 = FT::vouch_for('a');    
    let mut tf2 = TF::vouch_for(12.0);
    let mut tt2 = TT::vouch_for(37u32);
    let mut marker2 = SafeMarker::vouch();

    // works with unsafe
    unsafe { ff.set(false) };
    unsafe { ft.set('b') };

    // does not work without unsafe
    ff.set(false);
    ft.set('b');

}