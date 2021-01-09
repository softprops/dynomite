//! Dynomite is the set of high-level interfaces making interacting with [AWS DynamoDB](https://aws.amazon.com/dynamodb/) more productive.
//!
//! 💡To learn more about DynamoDB, see [this helpful guide](https://www.dynamodbguide.com/).
//!
//! ## Data Modeling
//!
//! Dynomite adapts Rust's native types to
//! DynamoDB's [core components](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/HowItWorks.CoreComponents.html)
//! to form a coherent interface.
//!
//! The [Attribute](trait.Attribute.html) type
//! provides conversion interfaces to and from Rust's native scalar types which represent
//! DynamoDB's notion of "attributes". The goal of this type is to make representing
//! AWS typed values feel more natural and ergonomic in Rust. Where a conversion is not available you can implement `Attribute` for your own
//! types to leverage higher level functionality.
//!
//! The [Item](trait.Item.html) trait
//! provides conversion interfaces for complex types which represent
//! DynamoDB's notion of "items".
//!
//! 💡 A cargo feature named `"derive"` makes it easy to derive `Item` instances for your custom types. This feature is enabled by default.
//!
//!
//! ```rust,no_run
//!  use dynomite::{Item, Attributes};
//!  use uuid::Uuid;
//!
//! #[derive(Item)]
//! struct Order {
//!   #[dynomite(partition_key)]
//!   user: Uuid,
//!   #[dynomite(sort_key)]
//!   order_id: Uuid,
//!   color: Option<String>,
//! }
//! ```
//!
//! ## Attributes
//!
//! ### `#[derive(Item)]`
//! Used to define a top-level DynamoDB item.
//! Generates a `<ItemName>Key` struct with only `partition_key/sort_key`
//! fields to be used for type-safe primary key construction.
//! This automatically derives [`Attributes`](#deriveattributes) too.
//!
//! For the `Order` struct from the example higher this will generate an `OrderKey`
//! struct like this:
//!
//! ```rust
//! # use uuid::Uuid;
//! # use dynomite::Attributes;
//! #[derive(Attributes)]
//! struct OrderKey {
//!     user: Uuid,
//!     order_id: Uuid,
//! }
//! ```
//!
//! Use it to safely and conveniently construct the primary key:
//!
//! ```rust
//! # #[derive(dynomite::Attributes)]
//! # struct Order {}
//! # #[derive(Attributes)]
//! # struct OrderKey {
//! #     user: Uuid,
//! #     order_id: Uuid,
//! # }
//! use dynomite::{
//!     dynamodb::{DynamoDb, GetItemInput},
//!     Attributes, FromAttributes,
//! };
//! use std::{convert::TryFrom, error::Error};
//! use uuid::Uuid;
//!
//! async fn get_order(
//!     client: impl DynamoDb,
//!     user: Uuid,
//!     order_id: Uuid,
//! ) -> Result<Option<Order>, Box<dyn Error>> {
//!     // Use the generated `OrderKey` struct to create a primary key
//!     let key = OrderKey { user, order_id };
//!     // Convert stronly-typed `OrderKey` to a map of `rusoto_dynamodb::AttributeValue`
//!     let key: Attributes = key.into();
//!
//!     let result = client
//!         .get_item(GetItemInput {
//!             table_name: "orders".into(),
//!             key,
//!             ..Default::default()
//!         })
//!         .await?;
//!
//!     Ok(result
//!         .item
//!         .map(|item| Order::try_from(item).expect("Invalid order, db corruption?")))
//! }
//! ```
//!
//! - `#[dynomite(partition_key)]` - required attribute, expected to be applied the target
//!  [partition attribute][partition-key] field with a derivable DynamoDB attribute value
//!  of String, Number or Binary
//!
//! - `#[dynomite(sort_key)]` - optional attribute, may be applied to one target
//!  [sort attribute](sort-key) field with an derivable DynamoDB attribute value
//!  of String, Number or Binary
//!
//! - All other attributes are the same as for [`#[derive(Attributes)]`](#deriveattributes)
//!
//! ### `#[derive(Attributes)]`
//!
//! Used to derive an implementation of `From/IntoAttributes` trait to allow for
//! serializing/deserializing map-like types into [`AttributeValue`].
//! This also generates `TryFrom<Attributes>` and `Into<Attributes>` implementations.
//!
//! - `#[dynomite(rename = "actualName")]` - optional attribute, may be applied to any item
//!   attribute field, useful when the DynamoDB table you're interfacing with has
//!   attributes whose names don't following Rust's naming conventions
//!
//! - `#[dynomite(default)]` - use [`Default::default`] implementation of the field type
//!   if the attribute is absent when deserializing from `Attributes`
//!
//!   ```
//!   use dynomite::Attributes;
//!
//!   #[derive(Attributes)]
//!   struct Todos {
//!       // use Default value of the field if it is absent in DynamoDb (empty vector)
//!       #[dynomite(default)]
//!       items: Vec<String>,
//!       list_name: String,
//!   }
//!   ```
//!
//! - `#[dynomite(flatten)]` - flattens the fields of other struct that also derives `Attributes`
//!   into the current struct.
//!
//!   💡 If this attribute is placed onto a field, no other `dynomite` attributes
//!   are alowed on this field (this restriction may be relaxed in future).
//!
//!   This is reminiscent of [`#[serde(flatten)]`](serde-flatten). The order of
//!   declaration of `flatten`ed fields matters, if the struct has to fields with
//!   `#[dynomite(flatten)]` attribute the one that appears higher in code will
//!   be evaluated before the other one. This is crucial when you want to collect
//!   additional properties into a map:
//!
//!   ```
//!   use dynomite::{Attributes, Item};
//!
//!   #[derive(Item)]
//!   struct ShoppingCart {
//!       #[dynomite(partition_key)]
//!       id: String,
//!       // A separate struct to store data without any id
//!       #[dynomite(flatten)]
//!       data: ShoppingCartData,
//!       // Collect all other additional attributes into a map
//!       // Beware that the order of declaration will affect the order of
//!       // evaluation, so this "wildcard" flatten clause should be the last member
//!       #[dynomite(flatten)]
//!       remaining_props: Attributes,
//!   }
//!
//!   // `Attributes` doesn't require neither of #[dynomite(partition_key/sort_key)]
//!   #[derive(Attributes)]
//!   struct ShoppingCartData {
//!       name: String,
//!       total_price: u32,
//!   }
//!   ```
//!
//! #### Fat enums
//!
//! Fat enums are naturally supported by `#[derive(Attribute)]`.
//! As for now, there is a limitation that the members of the enum must be
//! either unit or one-element tuple variants. This restriction will be relaxed
//! in future versions of `dynomite`.
//!
//! Deriving `Attributes` on fat enums currently uses
//! [internally tagged enum pattern][internally-tagged-enum] (inspired by serde).
//! Thus, you have to explicitly specify the **field name** of enum tag
//! via the `tag` attribute on an enum.
//!
//! For example, the following definition:
//!
//! ```
//! use dynomite::Attributes;
//!
//! #[derive(Attributes)]
//! // Name of the field where to store the discriminant in DynamoDB
//! #[dynomite(tag = "kind")]
//! enum Shape {
//!     Rectangle(Rectangle),
//!     // Use `rename` to change the **value** of the tag for a particular variant
//!     // by default the tag for a particular variant is the name of the variant verbatim
//!     #[dynomite(rename = "my_circle")]
//!     Circle(Circle),
//!     Unknown,
//! }
//!
//! #[derive(Attributes)]
//! struct Circle {
//!     radius: u32,
//! }
//!
//! #[derive(Attributes)]
//! struct Rectangle {
//!     width: u32,
//!     height: u32,
//! }
//! ```
//!
//! corresponds to the following representation in DynamoDB for each enum variant:
//!
//! - `Rectangle`:
//!   ```json
//!   {
//!       "kind": "Rectangle",
//!       "width": 42,
//!       "height": 64
//!   }
//!   ```
//! - `Circle`:
//!   ```json
//!   {
//!       "kind": "my_circle",
//!       "radius": 54
//!   }
//!   ```
//! - `Unknown`:
//!   ```json
//!   {
//!       "kind": "Unknown"
//!   }
//!   ```
//!
//! If you have a plain old enum (without any data fields), you should use
//! [`#[derive(Attribute)]`](#deriveattribute) instead.
//!
//! ### `#[derive(Attribute)]`
//!
//! Derives an implementation of [`Attribute`] for the plain enum.
//! If you want to use a fat enum see [this paragraph](#fat-enums) instead.
//!
//! The enum istelf will be represented as a string with the name of the variant
//! it represents.
//! In contrast, having [`#[derive(Attributes)]`](deriveattributes) on an enum
//! makes it to be represented as an object with a tag field,
//! which implies an additional layer of indirection.
//!
//! ```
//! use dynomite::{Attribute, Item};
//!
//! #[derive(Attribute)]
//! enum UserRole {
//!     Admin,
//!     Moderator,
//!     Regular,
//! }
//!
//! #[derive(Item)]
//! struct User {
//!     #[dynomite(partition_key)]
//!     id: String,
//!     role: UserRole,
//! }
//! ```
//!
//! This data model will have the following representation in DynamoDB:
//!
//! ```json
//! {
//!     "id": "d97de525-c81d-46d4-b945-d01b3a0f9165",
//!     "role": "Admin"
//! }
//! ```
//!
//! `role` field here may be any of `Admin`, `Moderator`, or `Regular` strings.
//!
//! #### Newtype Structs
//!
//! Types that implement the [dynomite::Attribute](trait.Attribute.html) trait can be wrapped in
//! single-field tuple (newtype) structs. For example, a `String` newtype can be used as follows:
//! ```rust
//! use dynomite::{Attribute, Item};
//!
//! #[derive(Attribute)]
//! struct Author(String);
//!
//! #[derive(Item)]
//! struct Book {
//!     #[dynomite(partition_key)]
//!     id: String,
//!     author: Author,
//! }
//! ```
//!
//! ## Rusoto extensions
//!
//! By importing the [dynomite::DynamoDbExt](trait.DynamoDbExt.html) trait, dynomite
//! adds client interfaces for creating async Stream-based auto pagination interfaces.
//!
//! ## Robust retries
//!
//! By importing the [dynomite::Retries](retry/trait.Retries.html) trait, dynomite
//! provides an interface for adding configuration retry policies so your
//! rusoto DynamoDb clients.
//!
//! # Errors
//!
//! Some operations which require coercion from AWS to Rust types may fail which results in an
//! [AttributeError](error/enum.AttributeError.html).
//!
//! # Cargo Features
//!
//! This crate has a few cargo features of note.
//!
//! ## uuid
//!
//! Enabled by default, the `uuid` feature adds support for implementing `Attribute` for
//! the [uuid](https://crates.io/crates/uuid) crate's type `Uuid`, a useful
//! type for producing and representing
//! unique identifiers for items that satisfy [effective characteristics for partition keys](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/bp-partition-key-design.html)
//!
//! ## chrono
//!
//! Enabled by default, the `chrono` feature adds an implementation of `Attribute` for
//! the std's [SystemTime](https://doc.rust-lang.org/std/time/struct.SystemTime.html) and chrono [`DateTime`](https://docs.rs/chrono/0.4.11/chrono/struct.DateTime.html) types which
//! internally use [rfc3339 timestamps](https://www.ietf.org/rfc/rfc3339.txt).
//!
//! ## derive
//!
//! Enabled by default, the `derive` feature enables the use of the dynomite derive feature which
//! allows you to simply add `#[derive(Item)]` to your structs.
//!
//! ## rustls
//!
//! Disabled by default, the `rustls` feature overrides Rusoto's default tls
//! dependency on OpenSSL, replacing it with a [`rustls`](https://crates.io/crates/rustls) based tls implementation. When you
//! enable this feature. It will also enable `uuid` and `derive` by default.
//!
//! To disable any of these features
//!
//! ```toml
//! [dependencies.dynomite]
//! version = "xxx"
//! default-features = false
//! features = ["feature-you-want"]
//! ```
//!
//! [partition-key]: https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/HowItWorks.CoreComponents.html#HowItWorks.CoreComponents.PrimaryKey
//! [sort-key]: https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/HowItWorks.CoreComponents.html#HowItWorks.CoreComponents.SecondaryIndexes
//! [internally-tagged-enum]: https://serde.rs/enum-representations.html#internally-tagged
//! [`Default::default`]: https://doc.rust-lang.org/stable/std/default/trait.Default.html#tymethod.default
//! [`AttributeValue`]: https://docs.rs/rusoto_dynamodb/*/rusoto_dynamodb/struct.AttributeValue.html
//! [`Attribute`]: trait.Attribute.html

#![deny(missing_docs)]
// reexported
// note: this is used inside the attr_map! macro
// #[cfg(feature = "default")]
// pub use rusoto_dynamodb_default as dynamodb;
//
// #[cfg(feature = "rustls")]
// pub use rusoto_dynamodb_rustls as dynamodb;

use bytes::Bytes;
#[cfg(feature = "chrono")]
use chrono::{
    offset::{FixedOffset, Local},
    DateTime, Utc,
};
pub use rusoto_dynamodb as dynamodb;

// we re-export this because we
// refer to it with in derive macros
#[doc(hidden)]
pub use dynamodb::AttributeValue;
use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    time::SystemTime,
};
#[cfg(feature = "uuid")]
use uuid::Uuid;

pub mod error;
mod ext;
pub mod retry;

pub use crate::{ext::DynamoDbExt, retry::Retries};

pub use crate::error::AttributeError;
/// Type alias for map of named attribute values
pub type Attributes = HashMap<String, AttributeValue>;

/// A type which can be converted to and from a set of String keys and
/// `AttributeValues`.
///
/// # Examples
///
/// Below is an example of doing this manually for demonstration.
///
/// ```
/// use dynomite::{
///     dynamodb::AttributeValue, Attribute, AttributeError, Attributes, FromAttributes,
///     IntoAttributes, Item,
/// };
/// use std::{collections::HashMap, convert::TryFrom};
///
/// #[derive(PartialEq, Debug, Clone)]
/// struct Person {
///     id: String,
/// }
///
/// impl Item for Person {
///     fn key(&self) -> Attributes {
///         let mut attrs = HashMap::new();
///         attrs.insert("id".into(), "123".to_string().into_attr());
///         attrs
///     }
/// }
///
/// impl FromAttributes for Person {
///     fn from_attrs(attrs: &mut Attributes) -> Result<Self, AttributeError> {
///         Ok(Self {
///             id: attrs
///                 .remove("id")
///                 .and_then(|val| val.s)
///                 .ok_or_else(|| AttributeError::MissingField { name: "id".into() })?,
///         })
///     }
/// }
///
/// impl IntoAttributes for Person {
///     fn into_attrs(
///         self,
///         attrs: &mut Attributes,
///     ) {
///         attrs.insert("id".into(), "123".to_string().into_attr());
///     }
/// }
///
/// // Unfortunately `dynomite` is not able to provide a blanket impl for std::convert traits
/// // due to orphan rules, but they are generated via the `dynomite_derive` attributes
///
/// impl TryFrom<Attributes> for Person {
///     type Error = AttributeError;
///
///     fn try_from(mut attrs: Attributes) -> Result<Person, AttributeError> {
///         Person::from_attrs(&mut attrs)
///     }
/// }
///
/// impl From<Person> for Attributes {
///     fn from(person: Person) -> Attributes {
///         let mut map = HashMap::new();
///         person.into_attrs(&mut map);
///         map
///     }
/// }
///
/// let person = Person { id: "123".into() };
/// let attrs: Attributes = person.clone().into();
/// assert_eq!(Ok(person), Person::try_from(attrs))
/// ```
///
/// You can get this all for free automatically using `#[derive(Item)]` on your structs. This is the recommended approach.
///
/// ```
/// use dynomite::Item;
/// #[derive(Item)]
/// struct Book {
///     #[dynomite(partition_key)]
///     id: String,
/// }
/// ```
///
/// ## Renaming fields
///
/// In some cases you may be dealing with a DynamoDB table whose
/// fields are named using conventions that do not align with Rust's conventions.
/// You can leverage the `rename` attribute to map Rust's fields back to its source name
/// explicitly
///
/// ```
/// use dynomite::Item;
///
/// #[derive(Item)]
/// struct Book {
///     #[dynomite(partition_key)]
///     id: String,
///     #[dynomite(rename = "notConventional")]
///     not_conventional: String,
/// }
/// ```
///
/// ## Accommodating sparse data
///
/// In some cases you may be dealing with a DynamoDB table whose
/// fields are absent for some records. This is different than fields whose records
/// have `NULL` attribute type values. In these cases you can use the `default` field
/// attribute to communicate that the `std::default::Default::default()` value for the fields
/// type will be used in the absence of data.
///
/// ```
/// use dynomite::Item;
///
/// #[derive(Item)]
/// struct Book {
///     #[dynomite(partition_key)]
///     id: String,
///     #[dynomite(default)]
///     summary: Option<String>,
/// }
/// ```
///
/// ## Item attribute projections
///
/// DynamoDB `Item`s are a set of attributes with a uniquely identifying
/// partition key. At times, you may wish to project over these attributes into a type
/// that does not include a partition_key. For that specific purpose, instead of
/// deriving an `Item` type you'll want to derive `Attributes`
///
/// ```
/// use dynomite::Attributes;
///
/// #[derive(Attributes)]
/// struct BookProjection {
///   author: String,
///   #[dynomite(default)]
///   summary: Option<String>
/// }
pub trait Item: IntoAttributes + FromAttributes {
    /// Returns the set of attributes which make up this item's primary key
    ///
    /// This is often used in item look ups
    fn key(&self) -> Attributes;
}

/// A type capable of being converted into an or from and AWS `AttributeValue`
///
/// Default implementations of this are provided for each type of `AttributeValue` field
/// which map to naturally fitting native Rustlang types.
///
/// # Examples
///
/// ```
/// use dynomite::{dynamodb::AttributeValue, Attribute};
///
/// assert_eq!(
///     "test".to_string().into_attr().s,
///     AttributeValue {
///         s: Some("test".to_string()),
///         ..AttributeValue::default()
///     }
///     .s
/// );
/// ```
pub trait Attribute: Sized {
    /// Returns a conversion into an `AttributeValue`
    fn into_attr(self: Self) -> AttributeValue;
    /// Returns a fallible conversion from an `AttributeValue`
    fn from_attr(value: AttributeValue) -> Result<Self, AttributeError>;
}

impl Attribute for AttributeValue {
    fn into_attr(self: Self) -> AttributeValue {
        self
    }
    fn from_attr(value: AttributeValue) -> Result<Self, AttributeError> {
        Ok(value)
    }
}

/// A type capable of being produced from a set of string keys and [`AttributeValue`]s.
/// Generally, you should not implement this trait manually.
/// Use `#[derive(Attributes/Item)]` to generate the proper implementation instead.
///
/// [`AttributeValue`]: https://docs.rs/rusoto_dynamodb/*/rusoto_dynamodb/struct.AttributeValue.html
pub trait FromAttributes: Sized {
    /// Returns an instance of of a type resolved at runtime from a collection
    /// of a `String` keys and [`AttributeValue`]s.
    /// If an instance can not be resolved and `AttributeError` will be returned.
    /// The implementations of this method should remove the relevant key-value
    /// pairs from the map to consume them.
    ///
    /// This is needed to support `#[dynomite(flatten)]` without creating temporary hash maps.
    ///
    /// [`AttributeValue`]: https://docs.rs/rusoto_dynamodb/*/rusoto_dynamodb/struct.AttributeValue.html
    fn from_attrs(attrs: &mut Attributes) -> Result<Self, AttributeError>;
}

/// Coerces a homogeneous HashMap of attribute values into a homogeneous Map of types
/// that implement `Attribute`
#[allow(clippy::implicit_hasher)]
impl<A: Attribute> FromAttributes for HashMap<String, A> {
    fn from_attrs(attrs: &mut Attributes) -> Result<Self, AttributeError> {
        attrs
            .drain()
            .map(|(k, v)| Ok((k, A::from_attr(v)?)))
            .collect()
    }
}

/// Coerces a homogenious Map of attribute values into a homogeneous BTreeMap of types
/// that implement Attribute
impl<A: Attribute> FromAttributes for BTreeMap<String, A> {
    fn from_attrs(attrs: &mut Attributes) -> Result<Self, AttributeError> {
        attrs
            .drain()
            .map(|(k, v)| Ok((k, A::from_attr(v)?)))
            .collect()
    }
}

/// A type capable of being serialized into a set of string keys and [`AttributeValue`]s
/// Generally, you should not implement this trait manually.
/// Use `#[derive(Attributes/Item)]` to generate the proper implementation instead.
///
/// It also generates `From<T> for Attributes` for your type
/// (there is no blanket impl for `From<T>` here due to orphan rules)
///
/// [`AttributeValue`]: https://docs.rs/rusoto_dynamodb/*/rusoto_dynamodb/struct.AttributeValue.html
pub trait IntoAttributes: Sized {
    /// Converts `self` into `Attributes` by accepting a `sink` argument and
    /// insterting attribute key-value pairs into it.
    /// This is needed to support `#[dynomite(flatten)]` without creating
    /// temporary hash maps.
    fn into_attrs(
        self,
        sink: &mut Attributes,
    );
}

impl<A: Attribute> IntoAttributes for HashMap<String, A> {
    fn into_attrs(
        self,
        sink: &mut Attributes,
    ) {
        sink.extend(self.into_iter().map(|(k, v)| (k, v.into_attr())));
    }
}

impl<A: Attribute> IntoAttributes for BTreeMap<String, A> {
    fn into_attrs(
        self,
        sink: &mut Attributes,
    ) {
        sink.extend(self.into_iter().map(|(k, v)| (k, v.into_attr())));
    }
}

/// A Map type for all hash-map-like values, represented as the `M` AttributeValue type
impl<T: IntoAttributes + FromAttributes> Attribute for T {
    fn into_attr(self: Self) -> AttributeValue {
        let mut map = HashMap::new();
        self.into_attrs(&mut map);
        AttributeValue {
            m: Some(map),
            ..AttributeValue::default()
        }
    }
    fn from_attr(value: AttributeValue) -> Result<Self, AttributeError> {
        T::from_attrs(&mut value.m.ok_or(AttributeError::InvalidType)?)
    }
}

/// A `String` type for `Uuids`, represented by the `S` AttributeValue type
#[cfg(feature = "uuid")]
impl Attribute for Uuid {
    fn into_attr(self: Self) -> AttributeValue {
        AttributeValue {
            s: Some(self.to_hyphenated().to_string()),
            ..AttributeValue::default()
        }
    }
    fn from_attr(value: AttributeValue) -> Result<Self, AttributeError> {
        value
            .s
            .ok_or(AttributeError::InvalidType)
            .and_then(|s| Uuid::parse_str(s.as_str()).map_err(|_| AttributeError::InvalidFormat))
    }
}

/// An `rfc3339` formatted version of `DateTime<Utc>`, represented by the `S` AttributeValue type
#[cfg(feature = "chrono")]
impl Attribute for DateTime<Utc> {
    fn into_attr(self: Self) -> AttributeValue {
        AttributeValue {
            s: Some(self.to_rfc3339()),
            ..Default::default()
        }
    }
    fn from_attr(value: AttributeValue) -> Result<Self, AttributeError> {
        value
            .s
            .ok_or(AttributeError::InvalidType)
            .and_then(
                |s| match DateTime::parse_from_rfc3339(&s).map(|dt| dt.with_timezone(&Utc)) {
                    Ok(date_time) => Ok(date_time),
                    Err(_) => Err(AttributeError::InvalidFormat),
                },
            )
    }
}

/// An `rfc3339` formatted version of `DateTime<Local>`, represented by the `S` AttributeValue type
#[cfg(feature = "chrono")]
impl Attribute for DateTime<Local> {
    fn into_attr(self: Self) -> AttributeValue {
        AttributeValue {
            s: Some(self.to_rfc3339()),
            ..Default::default()
        }
    }
    fn from_attr(value: AttributeValue) -> Result<Self, AttributeError> {
        value
            .s
            .ok_or(AttributeError::InvalidType)
            .and_then(|s| {
                match DateTime::parse_from_rfc3339(&s).map(|dt| dt.with_timezone(&Local)) {
                    Ok(date_time) => Ok(date_time),
                    Err(_) => Err(AttributeError::InvalidFormat),
                }
            })
    }
}

/// An `rfc3339` formatted version of `DateTime<FixedOffset>`, represented by the `S` AttributeValue type
#[cfg(feature = "chrono")]
impl Attribute for DateTime<FixedOffset> {
    fn into_attr(self: Self) -> AttributeValue {
        AttributeValue {
            s: Some(self.to_rfc3339()),
            ..Default::default()
        }
    }
    fn from_attr(value: AttributeValue) -> Result<Self, AttributeError> {
        value
            .s
            .ok_or(AttributeError::InvalidType)
            .and_then(|s| match DateTime::parse_from_rfc3339(&s) {
                Ok(date_time) => Ok(date_time),
                Err(_) => Err(AttributeError::InvalidFormat),
            })
    }
}

/// An `rfc3339` formatted version of `SystemTime`, represented by the `S` AttributeValue type
#[cfg(feature = "chrono")]
impl Attribute for SystemTime {
    fn into_attr(self: Self) -> AttributeValue {
        let dt: DateTime<Utc> = self.into();
        dt.into_attr()
    }
    fn from_attr(value: AttributeValue) -> Result<Self, AttributeError> {
        value
            .s
            .ok_or(AttributeError::InvalidType)
            .and_then(|s| match DateTime::parse_from_rfc3339(&s) {
                Ok(date_time) => Ok(date_time.into()),
                Err(_) => Err(AttributeError::InvalidFormat),
            })
    }
}

/// A `String` type, represented by the S AttributeValue type
impl Attribute for String {
    fn into_attr(self: Self) -> AttributeValue {
        AttributeValue {
            s: Some(self),
            ..AttributeValue::default()
        }
    }
    fn from_attr(value: AttributeValue) -> Result<Self, AttributeError> {
        value.s.ok_or(AttributeError::InvalidType)
    }
}

impl<'a> Attribute for Cow<'a, str> {
    fn into_attr(self: Self) -> AttributeValue {
        AttributeValue {
            s: Some(match self {
                Cow::Owned(o) => o,
                Cow::Borrowed(b) => b.to_owned(),
            }),
            ..AttributeValue::default()
        }
    }
    fn from_attr(value: AttributeValue) -> Result<Self, AttributeError> {
        value.s.map(Cow::Owned).ok_or(AttributeError::InvalidType)
    }
}

/// A String Set type, represented by the SS AttributeValue type
#[allow(clippy::implicit_hasher)]
impl Attribute for HashSet<String> {
    fn into_attr(mut self: Self) -> AttributeValue {
        AttributeValue {
            ss: Some(self.drain().collect()),
            ..AttributeValue::default()
        }
    }
    fn from_attr(value: AttributeValue) -> Result<Self, AttributeError> {
        value
            .ss
            .ok_or(AttributeError::InvalidType)
            .map(|mut value| value.drain(..).collect())
    }
}

impl Attribute for BTreeSet<String> {
    fn into_attr(self: Self) -> AttributeValue {
        AttributeValue {
            ss: Some(self.into_iter().collect()),
            ..AttributeValue::default()
        }
    }
    fn from_attr(value: AttributeValue) -> Result<Self, AttributeError> {
        value
            .ss
            .ok_or(AttributeError::InvalidType)
            .map(|mut value| value.drain(..).collect())
    }
}

/// A Binary Set type, represented by the BS AttributeValue type
#[allow(clippy::implicit_hasher)]
impl Attribute for HashSet<Vec<u8>> {
    fn into_attr(mut self: Self) -> AttributeValue {
        AttributeValue {
            bs: Some(self.drain().map(Bytes::from).collect()),
            ..AttributeValue::default()
        }
    }
    fn from_attr(value: AttributeValue) -> Result<Self, AttributeError> {
        value
            .bs
            .ok_or(AttributeError::InvalidType)
            .map(|mut value| value.drain(..).map(|bs| bs.as_ref().to_vec()).collect())
    }
}

// a Boolean type, represented by the BOOL AttributeValue type
impl Attribute for bool {
    fn into_attr(self: Self) -> AttributeValue {
        AttributeValue {
            bool: Some(self),
            ..AttributeValue::default()
        }
    }
    fn from_attr(value: AttributeValue) -> Result<Self, AttributeError> {
        value.bool.ok_or(AttributeError::InvalidType)
    }
}

// a Binary type, represented by the B AttributeValue type
impl Attribute for bytes::Bytes {
    fn into_attr(self: Self) -> AttributeValue {
        AttributeValue {
            b: Some(self),
            ..AttributeValue::default()
        }
    }
    fn from_attr(value: AttributeValue) -> Result<Self, AttributeError> {
        value.b.ok_or(AttributeError::InvalidType)
    }
}

// a Binary type, represented by the B AttributeValue type
impl Attribute for Vec<u8> {
    fn into_attr(self: Self) -> AttributeValue {
        AttributeValue {
            b: Some(self.into()),
            ..AttributeValue::default()
        }
    }
    fn from_attr(value: AttributeValue) -> Result<Self, AttributeError> {
        value
            .b
            .ok_or(AttributeError::InvalidType)
            .map(|bs| bs.as_ref().to_vec())
    }
}

/// A List type for vectors, represented by the L AttributeValue type
///
/// Note: Vectors support homogenious collection values. This means
/// the default supported scalars do not permit cases where you need
/// to store a list of heterogenus values. To accomplish this you'll need
/// to implement a wrapper type that represents your desired variants
/// and implement `Attribute` for `YourType`. An `Vec<YourType>` implementation
/// will already be provided
impl<A: Attribute> Attribute for Vec<A> {
    fn into_attr(mut self: Self) -> AttributeValue {
        AttributeValue {
            l: Some(self.drain(..).map(|s| s.into_attr()).collect()),
            ..AttributeValue::default()
        }
    }
    fn from_attr(value: AttributeValue) -> Result<Self, AttributeError> {
        value
            .l
            .ok_or(AttributeError::InvalidType)?
            .into_iter()
            .map(Attribute::from_attr)
            .collect()
    }
}

impl<T: Attribute> Attribute for Option<T> {
    fn into_attr(self: Self) -> AttributeValue {
        match self {
            Some(value) => value.into_attr(),
            _ => AttributeValue {
                null: Some(true),
                ..Default::default()
            },
        }
    }
    fn from_attr(value: AttributeValue) -> Result<Self, AttributeError> {
        match value.null {
            Some(true) => Ok(None),
            _ => Ok(Some(Attribute::from_attr(value)?)),
        }
    }
}

macro_rules! numeric_attr {
    ($type:ty) => {
        impl Attribute for $type {
            fn into_attr(self) -> AttributeValue {
                AttributeValue {
                    n: Some(self.to_string()),
                    ..AttributeValue::default()
                }
            }
            fn from_attr(value: AttributeValue) -> Result<Self, AttributeError> {
                value
                    .n
                    .ok_or(AttributeError::InvalidType)
                    .and_then(|num| num.parse().map_err(|_| AttributeError::InvalidFormat))
            }
        }
    };
}

macro_rules! numeric_set_attr {
    ($type:ty => $collection:ty) => {
        /// A Number set type, represented by the NS AttributeValue type
        impl Attribute for $collection {
            fn into_attr(self) -> crate::AttributeValue {
                AttributeValue {
                    ns: Some(self.iter().map(|item| item.to_string()).collect()),
                    ..AttributeValue::default()
                }
            }
            fn from_attr(value: AttributeValue) -> Result<Self, AttributeError> {
                let mut nums = value.ns.ok_or(AttributeError::InvalidType)?;
                let mut results: Vec<Result<$type, AttributeError>> = nums
                    .drain(..)
                    .map(|ns| ns.parse().map_err(|_| AttributeError::InvalidFormat))
                    .collect();
                results.drain(..).collect()
            }
        }
    };
}

// implement Attribute for numeric types
numeric_attr!(u16);
numeric_attr!(i16);
numeric_attr!(u32);
numeric_attr!(i32);
numeric_attr!(u64);
numeric_attr!(i64);
numeric_attr!(f32);
numeric_attr!(f64);

// implement Attribute for numeric collections
numeric_set_attr!(u16 => HashSet<u16>);
numeric_set_attr!(u16 => BTreeSet<u16>);
numeric_set_attr!(i16 => HashSet<i16>);
numeric_set_attr!(i16 => BTreeSet<i16>);

numeric_set_attr!(u32 => HashSet<u32>);
numeric_set_attr!(u32 => BTreeSet<u32>);
numeric_set_attr!(i32 => HashSet<i32>);
numeric_set_attr!(i32 => BTreeSet<i32>);

numeric_set_attr!(i64 => HashSet<i64>);
numeric_set_attr!(i64 => BTreeSet<i64>);
numeric_set_attr!(u64 => HashSet<u64>);
numeric_set_attr!(u64 => BTreeSet<u64>);

// note floats don't implement `Ord` and thus can't
// be used in various XXXSet types
//numeric_set_attr!(f32 => HashSet<f32>);
//numeric_set_attr!(f32 => BTreeSet<f32>);
//numeric_set_attr!(f64 => HashSet<f64>);
//numeric_set_attr!(f64 => BTreeSet<f64>);

#[macro_export]
/// Creates a `HashMap<String, AttributeValue>` from a list of key-value pairs
///
/// This provides some convenience for some interfaces,
///  like [query](../rusoto_dynamodb/struct.QueryInput.html#structfield.expression_attribute_values)
/// where a map of this type is required.
///
/// This syntax for this macro is the same as [maplit](https://crates.io/crates/maplit).
///
/// A avoid using `&str` slices for values when creating a mapping for a `String` `AttributeValue`.
/// Instead use a `String`.
///
/// ## Example
///
/// ```
/// use dynomite::dynamodb::QueryInput;
/// use dynomite::attr_map;
///
/// let query = QueryInput {
///   table_name: "some_table".into(),
///   key_condition_expression: Some(
///     "partitionKeyName = :partitionkeyval".into()
///   ),
///   expression_attribute_values: Some(
///     attr_map! {
///        ":partitionkeyval" => "rust".to_string()
///      }
///    ),
///    ..QueryInput::default()
/// };
macro_rules! attr_map {
    (@single $($x:tt)*) => (());
    (@count $($rest:expr),*) => (<[()]>::len(&[$($crate::attr_map!(@single $rest)),*]));
    ($($key:expr => $value:expr,)+) => { $crate::attr_map!($($key => $value),+) };
    ($($key:expr => $value:expr),*) => {
        {
            let _cap = $crate::attr_map!(@count $($key),*);
            let mut _map: ::std::collections::HashMap<String, ::dynomite::dynamodb::AttributeValue> =
              ::std::collections::HashMap::with_capacity(_cap);
              {
                  use ::dynomite::Attribute;
            $(
                let _ = _map.insert($key.into(), $value.into_attr());
            )*
              }
            _map
        }
    };
}

// Re-export #[derive(Item)]
// work around for 2018 edition issue with needing to
// import but the use dynomite::Item and dynomite_derive::Item
// https://internals.rust-lang.org/t/2018-edition-custom-derives-and-shadowy-import-ux/9097
#[cfg(feature = "derive")]
#[allow(unused_imports)]
#[macro_use]
extern crate dynomite_derive;
#[cfg(feature = "derive")]
#[doc(hidden)]
pub use dynomite_derive::*;

#[cfg(test)]
mod test {
    use super::*;
    use maplit::{btreemap, btreeset, hashmap};

    #[test]
    fn uuid_attr() {
        let value = Uuid::new_v4();
        assert_eq!(Ok(value), Uuid::from_attr(value.into_attr()));
    }

    #[test]
    fn uuid_invalid_attr() {
        assert_eq!(
            Err(AttributeError::InvalidType),
            Uuid::from_attr(AttributeValue {
                bool: Some(true),
                ..AttributeValue::default()
            })
        );
    }

    #[test]
    #[cfg(feature = "chrono")]
    fn chrono_datetime_utc_attr() {
        let value = Utc::now();
        assert_eq!(Ok(value), DateTime::<Utc>::from_attr(value.into_attr()));
    }

    #[test]
    #[cfg(feature = "chrono")]
    fn chrono_datetime_invalid_utc_attr() {
        assert_eq!(
            Err(AttributeError::InvalidType),
            DateTime::<Utc>::from_attr(AttributeValue {
                bool: Some(true),
                ..AttributeValue::default()
            })
        );
    }

    #[test]
    #[cfg(feature = "chrono")]
    fn chrono_datetime_local_attr() {
        let value = Local::now();
        assert_eq!(Ok(value), DateTime::<Local>::from_attr(value.into_attr()));
    }

    #[test]
    #[cfg(feature = "chrono")]
    fn chrono_datetime_invalid_local_attr() {
        assert_eq!(
            Err(AttributeError::InvalidType),
            DateTime::<Local>::from_attr(AttributeValue {
                bool: Some(true),
                ..AttributeValue::default()
            })
        );
    }

    #[test]
    #[cfg(feature = "chrono")]
    fn chrono_datetime_fixedoffset_attr() {
        use chrono::offset::TimeZone;
        let value = FixedOffset::east(5 * 3600)
            .ymd(2015, 2, 18)
            .and_hms(23, 16, 9);
        assert_eq!(
            Ok(value),
            DateTime::<FixedOffset>::from_attr(value.into_attr())
        );
    }

    #[test]
    #[cfg(feature = "chrono")]
    fn chrono_datetime_invalid_fixedoffset_attr() {
        assert_eq!(
            Err(AttributeError::InvalidType),
            DateTime::<FixedOffset>::from_attr(AttributeValue {
                bool: Some(true),
                ..AttributeValue::default()
            })
        );
    }

    #[test]
    #[cfg(feature = "chrono")]
    fn system_time_attr() {
        use std::time::SystemTime;
        let value = SystemTime::now();
        assert_eq!(Ok(value), SystemTime::from_attr(value.into_attr()));
    }

    #[test]
    #[cfg(feature = "chrono")]
    fn system_time_invalid_attr() {
        use std::time::SystemTime;
        assert_eq!(
            Err(AttributeError::InvalidType),
            SystemTime::from_attr(AttributeValue {
                bool: Some(true),
                ..AttributeValue::default()
            })
        );
    }

    #[test]
    fn option_some_attr() {
        let value = Some(1);
        assert_eq!(Ok(value), Attribute::from_attr(value.into_attr()));
    }

    #[test]
    fn option_none_attr() {
        let value: Option<u32> = None;
        assert_eq!(Ok(value), Attribute::from_attr(value.into_attr()));
    }

    #[test]
    fn option_invalid_attr() {
        assert_eq!(
            Err(AttributeError::InvalidType),
            Option::<u32>::from_attr(AttributeValue {
                bool: Some(true),
                ..AttributeValue::default()
            })
        );
    }

    #[test]
    fn bool_attr() {
        let value = true;
        assert_eq!(Ok(value), bool::from_attr(value.into_attr()));
    }

    #[test]
    fn string_attr() {
        let value = "test".to_string();
        assert_eq!(Ok(value.clone()), String::from_attr(value.into_attr()));
    }

    #[test]
    fn bytes_attr_from_attr() {
        let value = Bytes::from("test");
        assert_eq!(Ok(value.clone()), Bytes::from_attr(value.into_attr()));
    }

    #[test]
    fn byte_vec_attr_from_attr() {
        let value = b"test".to_vec();
        assert_eq!(Ok(value.clone()), Vec::<u8>::from_attr(value.into_attr()));
    }

    #[test]
    fn numeric_into_attr() {
        assert_eq!(
            serde_json::to_string(&1.into_attr()).unwrap(),
            r#"{"N":"1"}"#
        );
    }

    #[test]
    fn numeric_from_attr() {
        assert_eq!(
            Attribute::from_attr(serde_json::from_str::<AttributeValue>(r#"{"N":"1"}"#).unwrap()),
            Ok(1)
        );
    }

    #[test]
    fn string_into_attr() {
        assert_eq!(
            serde_json::to_string(&"foo".to_string().into_attr()).unwrap(),
            r#"{"S":"foo"}"#
        );
    }

    #[test]
    fn string_from_attr() {
        assert_eq!(
            Attribute::from_attr(serde_json::from_str::<AttributeValue>(r#"{"S":"foo"}"#).unwrap()),
            Ok("foo".to_string())
        );
    }

    #[test]
    fn cow_str_into_attr() {
        assert_eq!(
            serde_json::to_string(&Cow::Borrowed("foo").into_attr()).unwrap(),
            r#"{"S":"foo"}"#
        );
    }

    #[test]
    fn cow_str_from_attr() {
        assert_eq!(
            Attribute::from_attr(serde_json::from_str::<AttributeValue>(r#"{"S":"foo"}"#).unwrap()),
            Ok(Cow::Borrowed("foo"))
        );
    }

    #[test]
    fn byte_vec_into_attr() {
        assert_eq!(
            serde_json::to_string(&b"foo".to_vec().into_attr()).unwrap(),
            r#"{"B":"Zm9v"}"# // ruosoto converts to base64 for us
        );
    }

    #[test]
    fn byte_vec_from_attr() {
        // ruosoto converts to base64 for us
        assert_eq!(
            Attribute::from_attr(
                serde_json::from_str::<AttributeValue>(r#"{"B":"Zm9v"}"#).unwrap()
            ),
            Ok(b"foo".to_vec())
        );
    }

    #[test]
    fn bytes_into_attr() {
        assert_eq!(
            serde_json::to_string(&Bytes::from("foo").into_attr()).unwrap(),
            r#"{"B":"Zm9v"}"# // ruosoto converts to base64 for us
        );
    }

    #[test]
    fn bytes_from_attr() {
        assert_eq!(
            Attribute::from_attr(
                serde_json::from_str::<AttributeValue>(r#"{"B":"Zm9v"}"#).unwrap()
            ),
            Ok(Bytes::from("foo"))
        );
    }

    #[test]
    fn numeric_set_into_attr() {
        assert_eq!(
            serde_json::to_string(&btreeset! { 1,2,3 }.into_attr()).unwrap(),
            r#"{"NS":["1","2","3"]}"#
        );
    }

    #[test]
    fn numeric_set_from_attr() {
        assert_eq!(
            Attribute::from_attr(
                serde_json::from_str::<AttributeValue>(r#"{"NS":["1","2","3"]}"#).unwrap()
            ),
            Ok(btreeset! { 1,2,3 })
        );
    }

    #[test]
    fn numeric_vec_into_attr() {
        assert_eq!(
            serde_json::to_string(&vec![1, 2, 3, 3].into_attr()).unwrap(),
            r#"{"L":[{"N":"1"},{"N":"2"},{"N":"3"},{"N":"3"}]}"#
        );
    }

    #[test]
    fn numeric_vec_from_attr() {
        assert_eq!(
            Attribute::from_attr(
                serde_json::from_str::<AttributeValue>(
                    r#"{"L":[{"N":"1"},{"N":"2"},{"N":"3"},{"N":"3"}]}"#
                )
                .unwrap()
            ),
            Ok(vec![1, 2, 3, 3])
        );
    }

    #[test]
    fn string_set_into_attr() {
        assert_eq!(
            serde_json::to_string(
                &btreeset! { "a".to_string(), "b".to_string(), "c".to_string() }.into_attr()
            )
            .unwrap(),
            r#"{"SS":["a","b","c"]}"#
        );
    }

    #[test]
    fn string_set_from_attr() {
        assert_eq!(
            Attribute::from_attr(
                serde_json::from_str::<AttributeValue>(r#"{"SS":["a","b","c"]}"#).unwrap()
            ),
            Ok(btreeset! { "a".to_string(), "b".to_string(), "c".to_string() })
        );
    }

    #[test]
    fn string_vec_into_attr() {
        assert_eq!(
            serde_json::to_string(
                &vec! { "a".to_string(), "b".to_string(), "c".to_string() }.into_attr()
            )
            .unwrap(),
            r#"{"L":[{"S":"a"},{"S":"b"},{"S":"c"}]}"#
        );
    }

    #[test]
    fn string_vec_from_attr() {
        assert_eq!(
            Attribute::from_attr(
                serde_json::from_str::<AttributeValue>(r#"{"L":[{"S":"a"},{"S":"b"},{"S":"c"}]}"#)
                    .unwrap()
            ),
            Ok(vec! { "a".to_string(), "b".to_string(), "c".to_string() })
        );
    }

    #[test]
    fn hashmap_into_attr() {
        assert_eq!(
            serde_json::to_string(&hashmap! { "foo".to_string() => 1 }.into_attr()).unwrap(),
            r#"{"M":{"foo":{"N":"1"}}}"#
        );
    }

    #[test]
    fn hashmap_from_attr() {
        assert_eq!(
            Attribute::from_attr(
                serde_json::from_str::<AttributeValue>(r#"{"M":{"foo":{"N":"1"}}}"#).unwrap()
            ),
            Ok(hashmap! { "foo".to_string() => 1 })
        );
    }

    #[test]
    fn btreemap_into_attr() {
        assert_eq!(
            serde_json::to_string(&btreemap! { "foo".to_string() => 1 }.into_attr()).unwrap(),
            r#"{"M":{"foo":{"N":"1"}}}"#
        );
    }

    #[test]
    fn btreemap_from_attr() {
        assert_eq!(
            Attribute::from_attr(
                serde_json::from_str::<AttributeValue>(r#"{"M":{"foo":{"N":"1"}}}"#).unwrap()
            ),
            Ok(btreemap! { "foo".to_string() => 1 })
        );
    }
}
