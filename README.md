# dynomite [![Build Status](https://travis-ci.org/softprops/dynomite.svg?branch=master)](https://travis-ci.org/softprops/dynomite) [![Coverage Status](https://coveralls.io/repos/softprops/dynomite/badge.svg?branch=master&service=github)](https://coveralls.io/github/softprops/dynomite?branch=master) [![Software License](https://img.shields.io/badge/license-MIT-brightgreen.svg)](LICENSE) [![crates.io](http://meritbadge.herokuapp.com/dynomite)](https://crates.io/crates/dynomite) [![Released API docs](https://docs.rs/dynomite/badge.svg)](http://docs.rs/dynomite) [![Master API docs](https://img.shields.io/badge/docs-master-green.svg)](https://softprops.github.io/dynomite)

> dynomite makes rusoto_dynamodb fit your types (and visa versa)

## Overview

Goals

* make interacting with [dynamodb](https://aws.amazon.com/dynamodb/) in [rust](https://www.rust-lang.org/) a productive experience
* exploit rust's type safety features
* commitment to supporting applications build using stable rust
* commitment to documentation

Please see [API documentation](https://softprops.github.io/dynomite) for how
to get started

## Install

In your Cargo.toml file, add the following under the `[dependencies]` heading

```toml
dynomite = "0.0.0"
```

Optionally, you can install a companion crate which allows you to derive
dynomite types for your structs at compile time

```toml
dynomite-derive = "0.0.0"
```

## Examples

You can find some example application code under [dynomite-derive/examples](dynomite-derive/examples)

### Alternatives

The [korat](https://crates.io/crates/korat) crate was the the original inspiration for this crate. It's focus is very similar but fell short on a few
accounts. It does not work on stable rust and it's api is not documented. Dynomite intends to build on similar features as well as build out others.

Doug Tangren (softprops) 2018