// With this `macro_use` we are able to use standard `assert_eq/assert_ne`
// macros without an explicit import `use pretty_assertions::{assert_eq, assert_ne}`,
// because they are a full drop-in replacement
#[macro_use]
extern crate pretty_assertions;

mod derive_conflict;
mod derived;
mod skip_serializing_if;
mod try_build_test;
