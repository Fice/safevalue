extern crate safevalue;
use safevalue::SafeHolder;


type TF = SafeHolder<f32, true, false>;
type TT = SafeHolder<u32, true, true>;
type FF = SafeHolder<bool, false, false>;
type FT = SafeHolder<char, false, true>;


pub fn main() {
    let mut ff = unsafe { FF::vouch_for(true) };
    let mut ft = unsafe { FT::vouch_for('a') };
    let mut tf = unsafe { TF::vouch_for(12.0) };
    let mut tt = unsafe { TT::vouch_for(37u32) };
    let mut marker = unsafe { safevalue::SafeMarker::vouch() };

    // Un-Unsafe vouch should fail
    let mut ff2 = FF::vouch_for(true);
    //~^ 19:19: 19:38: call to unsafe function `SafeHolder::<T, WRITE_ONCE, READ_ONCE>::vouch_for` is unsafe and requires unsafe function or block [E0133]
    let mut ft2 = FT::vouch_for('a');    
    //~^ 21:19: 21:37: call to unsafe function `SafeHolder::<T, WRITE_ONCE, READ_ONCE>::vouch_for` is unsafe and requires unsafe function or block [E0133]
    let mut tf2 = TF::vouch_for(12.0);
    //~^ 23:19: 23:38: call to unsafe function `SafeHolder::<T, WRITE_ONCE, READ_ONCE>::vouch_for` is unsafe and requires unsafe function or block [E0133]
    let mut tt2 = TT::vouch_for(37u32);
    //~^ 25:19: 25:39: call to unsafe function `SafeHolder::<T, WRITE_ONCE, READ_ONCE>::vouch_for` is unsafe and requires unsafe function or block [E0133]
    let mut marker2 = safevalue::SafeMarker::vouch();
    //~^ 27:23: 27:53: call to unsafe function `SafeHolder::<(), WRITE_ONCE, READ_ONCE>::vouch` is unsafe and requires unsafe function or block [E0133]

    // works with unsafe
    unsafe { ff.set(false) };
    unsafe { ft.set('b') };

    // does not work without unsafe
    ff.set(false);
    //~^ 35:5: 35:18: call to unsafe function `SafeHolder::<T, false, READ_ONCE>::set` is unsafe and requires unsafe function or block [E0133]
    ft.set('b');
    //~^ 37:5: 37:16: call to unsafe function `SafeHolder::<T, false, READ_ONCE>::set` is unsafe and requires unsafe function or block [E0133]

}