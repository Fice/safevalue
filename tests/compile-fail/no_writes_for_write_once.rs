extern crate safevalue;
use safevalue::SafeHolder;


type TF = SafeHolder<f32, true, false>;
type TT = SafeHolder<u32, true, true>;
type FF = SafeHolder<bool, false, false>;
type FT = SafeHolder<char, false, true>;

pub fn main() {
    working();
    not_working();
}

//These should compile with no problem, because they are writable more than once
pub fn working() {
    let mut ff = unsafe { FF::vouch_for(true) };
    let mut ft = unsafe { FT::vouch_for('a') };

    unsafe { ff.set(false); }
    unsafe { ft.set('b'); }
}

pub fn not_working() {
    let tf = unsafe { TF::vouch_for(12.0) };
    let tt = unsafe { TT::vouch_for(37) };

    tf.set(18.0);
    //~^ 28:8: 28:11: no method named `set` found for struct `SafeHolder<f32, true, false>` in the current scope [E0599]

    tt.set(9);
    //~^ 31:8: 31:11: no method named `set` found for struct `SafeHolder<u32>` in the current scope [E0599]

}