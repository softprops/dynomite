# 0.1.4

* add Stream oriented extension interfaces for paginated apis

By default, the `DyanomoDb` apis `list_backups`, `list_tables`, `query`, `scan`
all require application management of pagination using inconsistent api's.
This release brings a consistent interface for each with extension methods prefixed with `stream_`
which return a consistent interface for retrieving a `futures::Stream` of their
respective values.

* pin rusoto crate versioning to minor release `0.34`

In the past this crate was pinned to a major version of rusoto. It will be pinned to a minor
version going forward.

# 0.1.3

* fix examples for rusoto breaking changes in 0.32, async release

# 0.1.2

* fix `dynomite-derive` `dynomite` dependency version

# 0.1.1

* initial release