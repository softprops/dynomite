# dynomite [![Build Status](https://travis-ci.org/softprops/dynomite.svg?branch=master)](https://travis-ci.org/softprops/dynomite) [![Coverage Status](https://coveralls.io/repos/softprops/dynomite/badge.svg?branch=master&service=github)](https://coveralls.io/github/softprops/dynomite?branch=master) [![Software License](https://img.shields.io/badge/license-MIT-brightgreen.svg)](LICENSE) [![crates.io](http://meritbadge.herokuapp.com/dynomite)](https://crates.io/crates/dynomite) [![Released API docs](https://docs.rs/dynomite/badge.svg)](http://docs.rs/dynomite) [![Master API docs](https://img.shields.io/badge/docs-master-green.svg)](https://softprops.github.io/dynomite)

> dynomite makes DynamoDB fit your types (and visa versa)

## Overview

Goals

* ⚡ make writing [dynamodb](https://aws.amazon.com/dynamodb/) applications in [rust](https://www.rust-lang.org/) a productive experience
* 🦀 exploit rust's type safety features
* 👩‍💻 leverage existing work of the [rusoto](https://github.com/rusoto/rusoto) rust project
* ☔ commitment to supporting applications built using stable rust
* 📚 commitment to documentation

Please see [API documentation](https://softprops.github.io/dynomite) for how
to get started

## 📦 Install

In your Cargo.toml file, add the following under the `[dependencies]` heading

```toml
dynomite = "0.2"
```

## 🤸 Examples

You can find some example application code under [dynomite/examples](dynomite/examples)

### Dynamodb local

AWS provides [a convient way to host a local instance of DynamoDB](https://hub.docker.com/r/amazon/dynamodb-local/) for
testing.

Here is a short example of how to get up a testing locally quickly with both dynomite as well as rusoto_dynamodb.

In one terminal spin up a Docker container for [DynamoDB local](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/DynamoDBLocal.UsageNotes.html) listening on port `8000`

```sh
$ docker run --rm -p 8000:8000 amazon/dynamodb-local
```

In another, run a rust binary with a client initialized like you see the the [local.rs example](dynomite-derive/examples/local.rs)


## 🚧 Planned work

* retryable extensions to rusto's client based on [retryable errors](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/Programming.Errors.html#Programming.Errors.MessagesAndCodes)

Doug Tangren (softprops) 2018