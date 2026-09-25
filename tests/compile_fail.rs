//! Given the nature of this crate, we don't atually have much logic to test,
//! instead we need to test the proper usage of it, like you cannot build a
//! SafeHolder or marker, without using unsafe.
//!
//! In order to do this, we use trybuild to make sure certain usages are
//! invalid and generate a compiler error. The expected diagnostics live in the
//! `.stderr` files next to each test in `tests/ui`. To (re)generate them after
//! an intended change, run with `TRYBUILD=overwrite`.

#[test]
/// Main test that checks for proper compilation errors on inproper usage
fn compile_test() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
}
