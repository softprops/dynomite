<div align="center">
  🦀🧨
</div>

<h1 align="center">
  dynomite
</h1>

<p align="center">
   dynomite makes DynamoDB fit your types (and visa versa)
</p>

<div align="center">
  <a alt="GitHub Actions" href="https://github.com/softprops/dynomite/actions">
    <img src="https://github.com/softprops/dynomite/workflows/Main/badge.svg"/>
  </a>
  <a alt="crates.io" href="https://crates.io/crates/dynomite">
    <img src="https://img.shields.io/crates/v/dynomite.svg?logo=rust"/>
  </a>
  <a alt="docs.rs" href="http://docs.rs/dynomite">
    <img src="https://docs.rs/dynomite/badge.svg"/>
  </a>
  <a alt="latest docs" href="https://softprops.github.io/dynomite">
   <img src="https://img.shields.io/badge/docs-latest-green.svg"/>
  </a>
  <a alt="license" href="LICENSE">
    <img src="https://img.shields.io/badge/license-MIT-brightgreen.svg"/>
  </a>
</div>

<br />

## Overview

Goals

* ⚡ make writing [dynamodb](https://aws.amazon.com/dynamodb/) applications in [rust](https://www.rust-lang.org/) a productive experience
* 🦀 exploit rust's type safety features
* 👩‍💻 leverage existing work of the [rusoto](https://github.com/rusoto/rusoto) rust project
* ☔ commitment to supporting applications built using stable rust
* 📚 commitment to documentation

Features

* 💌 less boilerplate
* ♻️ automatic async pagination
* 🕶️ client level retry interfaces for [robust error handling](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/Programming.Errors.html)


From this

```rust
use std::collections::HashMap;
use rusoto_dynamodb::AttributeValue;
use uuid::Uuid;

let mut item = HashMap.new();
item.insert(
  "pk".to_string(), AttributeValue {
    s: Some(Uuid::new_v4().to_hyphenated().to_string()),
    ..AttributeValue::default()
  }
);
item.insert(
  // 🤬typos anyone?
  "quanity".to_string(), AttributeValue {
    n: Some("whoops".to_string()),
    ..AttributeValue::default()
  }
);
```

To this

```rust
use dynomite::Item;
use uuid::Uuid;

#[derive(Item)]
struct Order {
  #[dynomite(partition_key)]
  pk: Uuid,
  quantity: u16
}

let item = Order {
  pk: Uuid::new_v4(),
  quantity: 4
}.into();
```

Please see the [API documentation](https://softprops.github.io/dynomite) for how
to get started. Enjoy.

## 📦 Install

In your Cargo.toml file, add the following under the `[dependencies]` heading

```toml
dynomite = "0.10"
```

## 🤸 Examples

You can find some example application code under [dynomite/examples](dynomite/examples)

### DynamoDB local

AWS provides [a convenient way to host a local instance of DynamoDB](https://hub.docker.com/r/amazon/dynamodb-local/) for
testing.

Here is a short example of how to get up a testing locally quickly with both dynomite as well as `rusoto_dynamodb`.

In one terminal spin up a Docker container for [DynamoDB local](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/DynamoDBLocal.UsageNotes.html) listening on port `8000`

```sh
$ docker run --rm -p 8000:8000 amazon/dynamodb-local
```

In another, run a rust binary with a client initialized like you see the the [local.rs example](dynomite/examples/local.rs)

## Resources

* [How DynamoDB works](https://www.slideshare.net/AmazonWebServices/amazon-dynamodb-under-the-hood-how-we-built-a-hyperscale-database-dat321-aws-reinvent-2018)

Doug Tangren (softprops) 2018-2020
