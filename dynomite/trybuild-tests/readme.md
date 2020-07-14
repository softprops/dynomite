## How to add more tests

- Create a test file in `test` folder, e.g. `item-not-on-struct-fail.rs`.
- Write the least code needed to pass or fail. A fail will stop the crate from compiling.
- Make sure you get the desired error message.
- Move `item-not-on-struct-fail.rs` to `trybuild-tests` folder.
- Add `t.compile_fail("trybuild-tests/item-not-on-struct-fail.rs");` (with your file name) to `dynomite/tests/try_build_test.rs`
- Run `cargo test try_build_test`
- You should see the following output:
```
test trybuild-tests/item-not-on-struct-fail.rs ... wip

NOTE: writing the following output to `wip/item-not-on-struct-fail.stderr`.
Move this file to `trybuild-tests/item-not-on-struct-fail.stderr` to accept it as correct.
┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈
error: `derive` may only be applied to structs, enums and unions
 --> $DIR/item-not-on-struct-fail.rs:9:1
  |
9 | #[derive(Item)]
  | ^^^^^^^^^^^^^^^
┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈

test try_build_test ... ok
```
- Check if the contents of this file contain the correct error message and no irrelevant info: `dynomite/wip/item-not-on-struct-fail.stderr`
- Move `dynomite/wip/item-not-on-struct-fail.stderr` to `dynomite\trybuild-tests` folder, next to `item-not-on-struct-fail.rs` file
- Re-run `cargo test try_build_test`
- The test should pass OK with no error message, e.g. 
```
test trybuild-tests/item-not-on-struct-fail.rs ... ok

test try_build_test ... ok
```

`trybuild` will now check the contents of the compiler error message against the the `.stderr` and flag it as OK as long as they match.

#### t.compile_fail vs t.pass
Use one or the other depending on the intent. See https://github.com/dtolnay/trybuild for more info.

#### Test file location
Test files that fail compilation should be placed outside the main project tree to avoid test/build compilation failures outside of _trybuild_ framework.
E.g. this test snippet would prevent the project from building.
```rust
#[derive(Item)]
fn fail() {
  println!("This should fail");
}
```
That was the sole reason for creating `dynomite/trybuild-tests` folder.

#### dev-deps
`dynomite-derive` has to be added to `[dev-dependencies]` for _trybuild_ to work

#### trybuild ignores warnings
It's either compile fail or pass. Compiler warnings cannot be checked for correctness with _trybuild_.