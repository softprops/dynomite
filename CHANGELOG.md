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