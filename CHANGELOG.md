# 0.2.1

* Add support for configuring policies for retrying requests [based on DynamoDB recommendations](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/Programming.Errors.html)


```rust
use dynomite::{Retries, retry::Policy};
use dynomite::dynamodb::{DynamoDb, DynamoDbClient};

fn main() {
   let client =
      DynamoDbClient::new(Default::default())
         .with_retries(Policy::default());

   // any client operation will now be retried when
   // appropriate
   let tables = client.list_tables(Default::default());
   // other important work...
}
```

* update documentation to highlight more concisely areas of focus

# 0.2.0

* upgraded to 2018 edition
  * a side effect of this is that an interaction with 2018-style imports caused a name conflict with `dynomite::Item` and now `dynomite_derive::Item`. As a result the dynomite crate now has a
  compiler feature flag called "derive" which is no by default that resolves this. If you do not wish to have the feature enabled by default add the following to your Cargo.toml

  ```toml
  [dependencies.dynomite]
  version = "0.2"
  default-features = false
  features = ["uuid"]
  ```
* updates to supported Attribute type conversions

  * numeric sets (NS) no longer support vec type conversions, only sets types!
  * list types (L) now support  any type that implements `Attribute`, previously this only
     supported lists of types that implemented `Item` (a complex time). This means lists of scalars are now supported by default
  * `Cow<str>` is now supported for String Attributes
  * `FromAttributes` is now implemented for `XXXMap` types of `String` to `Attribute` types.
     This means you now get free, Item-link integration for homogenious maps
  * much needed unit tests now cover the correctness of implementations!
* (breaking change) the `DynamoDbExt.stream_xxx` methods which produced auto-paginating streams have been renamed to `DynamoDbExt.xxx_pages` to be more intention-revealing and inline with naming conventions of other language sdk's methods that implement similar functionality.

# 0.1.5

* updated dependencies

  * `Rusoto-*` 0.34 -> 0.36

# 0.1.4

* add Stream oriented extension interfaces for paginated apis

By default, the `DyanomoDb` apis `list_backups`, `list_tables`, `query`, `scan`
all require application management of pagination using inconsistent api's.
This release brings a consistent interface for each with extension methods prefixed with `stream_`
which return a consistent interface for retrieving a `futures::Stream` of their
respective values.

* add `maplit!` inspired `attr_map!` helper macro useful in query contexts when providing `expression_attribute_values`

* pin rusoto crate versioning to minor release `0.34`

In the past this crate was pinned to a major version of rusoto. It will be pinned to a minor
version going forward.

See the [demo application](https://github.com/softprops/dynomite/blob/5ed3444a46a02bd560644fed35adb553ffb8a0f0/dynomite-derive/examples/demo.rs) for  examples of updated interfaces.

# 0.1.3

* fix examples for rusoto breaking changes in 0.32, async release

# 0.1.2

* fix `dynomite-derive` `dynomite` dependency version

# 0.1.1

* initial release