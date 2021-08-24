# 0.11.0

* Introduce new `#[dynomite(skip_serializing_if = "...")]` field attribute that
  allows for skipping the value from serializing into a map according to the
  given condition.

# 0.10.0

* Bump rusoto dependencies to version from `0.44` to `0.45`
* fixed issue with dynomite renamed `partition_key` fields which copied unrelated attributes into the generated KeyStruct. These unrelated attributes are now omitted. [#130](https://github.com/softprops/dynomite/pull/130)

# 0.9.0

* Introduce new `#[dynomite(default)]` field attribute which permits the absence of field values in DynamoDB. These will be replaced with their default value when deserializing item data [#113](https://github.com/softprops/dynomite/pull/113)
* Introduce new `#[derive(Attributes)]` attribute for structs for deriving a subsets of attributes for projections [#115](https://github.com/softprops/dynomite/pull/115)

  > This is similar to `#[derive(Item)]` except that
  it does not require a `#[dynomoite(partition_key)]`
* `Items` will now fail at compile time when they don't have a single `#[dynomoite(partition_key)]` field
  > All DynamoDB items require a uniquely identifiable attribute. This enforces that fact
* Derive compilation errors are now more helpful! More errors will now indicate where in source in context where problems occur.
* ItemKey structs now honor all Item field attributes.

  Previously if you had declared a renamed partition key named `foo`

  ```rust
  #[dynomite(partition_key, rename = "Foo")]
  foo: String,
  ```

  you would end out with a ItemKey struct with a field named `Foo`. This was not intended.
  These ItemKey struct fields will now be properly
  named `foo` but deserialized as `Foo`.

# 0.8.2

* Bump rusoto dependencies to version `0.44`

# 0.8.1

* Add `Attribute` support for time types including `std::time::SystemTime`, `chrono::DateTime<{Utc,Local,FixedOffset}>` [#101](https://github.com/softprops/dynomite/pull/101) [#102](https://github.com/softprops/dynomite/pull/102)

# 0.8.0

* Breaking change. upgrade to rusoto@0.43.0 which itself is contains a number of breaking changes, albeit very useful ones. Dynomite is now based on standard libraries futures which means that async/await style programming are supported out of the box. This also impacted the dependency of `futures` upgraded to `0.3` which included breaking changes in streams apis which impacted autopaginating interfaces. See the `examples/` directory in this repo for up to date examples of current usage
* Breaking change. Dropped `failure` crate support. This wasn't adding any value over `std::error::Error` and was removed as an unnecessary dependency and replaced with an impl of `std::error::Error`

# 0.7.0

* Breaking change. Improved support for optional attribute values [#84](https://github.com/softprops/dynomite/pull/84)

Previously Dynomite's support for `Option` types did not map correctly to DynamoDB's notion of [null value types](https://docs.aws.amazon.com/amazondynamodb/latest/APIReference/API_AttributeValue.html): in serialized form `{ "NULL": true }`. Instead, Dynomite would not serialize the field at all which in some cases would not actually nullify the field in DynamoDB. Kudos to [@elslooo](https://github.com/elslooo) for discovering and fixing the bug. Because some applications may have relied on this previous behavior, we're bumping the version.

# 0.6.0

* Breaking change. Rename Item attributes to align with current aws docs [#76](https://github.com/softprops/dynomite/pull/76)


`#[hash]` and `#[range]` are now more closely aligned with the [AWS docs](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/HowItWorks.CoreComponents.html#HowItWorks.CoreComponents.PrimaryKey) vocabulary

`#[hash]` is now  `#[dynomite(partition_key)]`
`#[range]` is now `#[dynomite(sort_key)]`

This was a breaking change but one we think was worth it.

# 0.5.2

* Item fields now support renaming [#68](https://github.com/softprops/dynomite/pull/68)

Those familiar with `#[serde(rename = "actualName")]` will feel at home with `#[dynomite(rename = "actualName)]`. This feature brings a welcome ergnomic improvement when interacting with DynamoDB tables with attributes that don't follow [Rust's naming conventions](https://rust-lang-nursery.github.io/api-guidelines/naming.html).

# 0.5.1

* Upgrade to the latest rusoto version [`0.40.0`](https://github.com/rusoto/rusoto/blob/master/CHANGELOG.md#0400---2019-06-28)

# 0.5.0

* Upgrade to latest rusoto version [`0.39.0`](https://github.com/rusoto/rusoto/blob/master/CHANGELOG.md#0390---2019-05-19)

This introduces a change to Rusoto DynamoDB where the representation of the DynamoDB value type `binary` types changed from `Vec<u8>` to `bytes::Bytes`. This should not break existing applications but dynomite users now get transparent support for Items which declare fields of type `byte::Bytes`, which will be interpreted the same opaque binary blob of bytes, for free.

# 0.4.1

* added a new `rustls` feature flag which when enabled replaces openssl with `rustls` [#54](https://github.com/softprops/dynomite/pull/55)

# 0.4.0

* Upgrade to latest rusoto version [`0.38.0`](https://github.com/rusoto/rusoto/blob/master/CHANGELOG.md#0380---2019-04-17)

# 0.3.0

* Upgrade to latest rusoto version ([`0.37.0`](https://github.com/rusoto/rusoto/blob/master/CHANGELOG.md#0370---2019-03-12)) with added support for new DynamoDB methods `describe_endpoints`, `transact_get_items`, and `transact_write_items`.
* Upgrading to the latest rusoto means that clients are Cloneable. As such, `Arc` restrictions are removed on stream-based auto-pagination interfaces.

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
