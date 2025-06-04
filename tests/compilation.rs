//! Given the nature of this crate, we don't atually have much logic to test, instead
//! we need to test the proper usage of it, like you cannot build a SafeHolder or marker, without using unsafe.
//!
//! In order to do this, we use compiletest_rs to make sure certain usages are invalid and generate a compiler error.

extern crate compiletest_rs as compiletest;

use std::path::PathBuf;

fn run_mode(mode: &'static str) {
    let mut config = compiletest::Config::default();

    config.mode = mode.parse().expect("Invalid mode");
    config.src_base = PathBuf::from(format!("tests/{}", mode));
    config.link_deps(); // Populate config.target_rustcflags with dependencies on the path
    config.clean_rmeta(); // If your tests import the parent crate, this helps with E0464

    compiletest::run_tests(&config);
}

#[test]
/// Main test that checks for proper compilation errors on inproper usage
fn compile_test() { 
    run_mode("compile-fail");
}