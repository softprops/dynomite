//! Provides an error message testing framework using https://github.com/dtolnay/trybuild
//! See `dynomite/trybuild-tests/readme.md` for instructions on how to add more tests.

#[test]
fn try_build_test() {
    println!("try-build");
    let t = trybuild::TestCases::new();
    t.compile_fail("trybuild-tests/item-not-on-struct-fail.rs");
}
