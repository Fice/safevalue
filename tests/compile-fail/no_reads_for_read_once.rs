extern crate safevalue;
use safevalue::SafeHolder;


type TF = SafeHolder<f32, true, false>;
type FF = SafeHolder<bool, false, false>;
type TT = SafeHolder<u32, true, true>;
type FT = SafeHolder<char, false, true>;


pub fn main() {
    let mut ff = unsafe { FF::vouch_for(true) };
    let mut tf = unsafe { TF::vouch_for(12.0) };
    let mut ft = unsafe { FT::vouch_for('a') };
    let mut tt = unsafe { TT::vouch_for(37u32) };
    let mut marker = unsafe { safevalue::SafeMarker::vouch() };

    //these reads should work (because types are read_once=false)
    let ff2 = ff.clone();
    let tf2 = tf.clone();
    let ff_ref = ff.as_ref();
    let tf_ref = tf.as_ref();
    let comparison = ff == FF::vouch_for(true);
    let comparison2 = tf == TF::vouch_for(12);
    let ff_deref = *ff;
    let tf_deref = *tf;

    // these reads should fail because types are read_once=true,
    // meaning you can only 'take' to access the values
    let ft2 = ft.clone();
    //~^ 30:18: 30:23: no method named `clone` found for struct `SafeHolder` in the current scope [E0599]
    let tt2 = tt.clone();
    //~^ 32:18: 32:23: no method named `clone` found for struct `SafeHolder` in the current scope [E0599]
    let ft_ref = ft.as_ref();
    //~^ 34:21: 34:27: no method named `as_ref` found for struct `SafeHolder` in the current scope [E0599] 
    let tt_ref = tt.as_ref();
    //~^ 36:21: 36:27: no method named `as_ref` found for struct `SafeHolder` in the current scope [E0599]
    let comparison = ft == FT::vouch_for('b');
    //~^ 38:25: 38:27: binary operation `==` cannot be applied to type `SafeHolder<char, false>` [E0369]
    let comparison = tt == TT::vouch_for(true);
    //~^ 40:25: 40:27: binary operation `==` cannot be applied to type `SafeHolder<u32>` [E0369]
    let ft_deref = *ft;
    //~^ 42:20: 42:23: type `SafeHolder<char, false>` cannot be dereferenced [E0614]
    let tt_deref = *tt;
    //~^ 44:20: 44:23: type `SafeHolder<u32>` cannot be dereferenced [E0614]


    //take always works
    tf.take();
    tf.take();
    ft.take();
    tt.take();

}

