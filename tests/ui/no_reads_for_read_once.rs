#![allow(unused)]
extern crate safevalue;
use safevalue::{unsafe_marker, SafeHolder};


type TF = SafeHolder<f32, true, false>;
type FF = SafeHolder<bool, false, false>;
type TT = SafeHolder<u32, true, true>;
type FT = SafeHolder<char, false, true>;
unsafe_marker!(pub SafeMarker);

pub fn main() {
    let mut ff = unsafe { FF::vouch_for(true) };
    let mut tf = unsafe { TF::vouch_for(12.0) };
    let mut ft = unsafe { FT::vouch_for('a') };
    let mut tt = unsafe { TT::vouch_for(37u32) };
    let mut marker = unsafe { SafeMarker::vouch() };

    //these reads should work (because types are read_once=false)
    let ff2 = ff.clone();
    let tf2 = tf.clone();
    let ff_ref = ff.as_ref();
    let tf_ref = tf.as_ref();
    let comparison = ff == unsafe { FF::vouch_for(true) };
    let comparison2 = tf == unsafe { TF::vouch_for(12.0) };
    let ff_deref = *ff;
    let tf_deref = *tf;

    // these reads should fail because types are read_once=true,
    // meaning you can only 'take' to access the values
    let ft2 = ft.clone();
    let tt2 = tt.clone();
    let ft_ref = ft.as_ref();
    let tt_ref = tt.as_ref();
    let comparison = ft == unsafe { FT::vouch_for('b') };
    let comparison = tt == unsafe { TT::vouch_for(26u32) };
    let ft_deref = *ft;
    let tt_deref = *tt;


    //take always works
    tf.take();
    tf.take();
    ft.take();
    tt.take();

}

