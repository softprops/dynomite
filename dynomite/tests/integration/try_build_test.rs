//! Provides an error message testing framework using https://github.com/dtolnay/trybuild
//! See `dynomite/trybuild-tests/readme.md` for instructions on how to add more tests.

// Try-build tests are run only on stable version of the toolchain. This is because
// error messages in `rustc` change frequent enough to break the tests on beta or nightly
// jobs.
#[rustversion::stable]
#[test]
fn try_build_tests() {
    let t = trybuild::TestCases::new();
    t.compile_fail("trybuild-tests/fail/*.rs");
    t.pass("trybuild-tests/pass/*.rs");
}
