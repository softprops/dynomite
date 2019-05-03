//! Dynomite is set of high-level interfaces built on top of
//! [rusoto_dynamodb](https://rusoto.github.io/rusoto/rusoto_dynamodb/index.html)
//! which make interacting with [AWS DynamoDB](https://aws.amazon.com/dynamodb/) more productive.
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
//! AWS typed values feel more natural and ergonomic in Rust. You can implement `Attribute` for your own
//! types to leverage higher level functionality.
//!
//! The [Item](trait.Item.html) type
//! provides conversion interfaces for complex types which represent
//! DynamoDB's notion of "items".
//!
//! 💡 A cargo feature named "derive" makes it easy to derive `Item` instances for your custom types. This feature is enabled by default.
//!
//! ## Rusoto extensions
//!
//! By importing the [dynomite::DynamoDbExt](trait.DynamoDbExt.html) trait, dynomite
//! adds client interfaces for creating async Stream-based auto pagination interfaces
//!
//! ## Robust retries
//!
//! By importing the [dynomite::Retries](retry/trait.Retries.html) trait, dynomite
//! provides an interface for adding configuration retry policies so your
//! rusoto DynamoDb clients.
//!
//! # Errors
//!
//! Some operations which require coercion from AWS to Rust types may fail result in an
//! [AttributeError](error/enum.AttributeError.html). These errors were
//! designed to work with the [failure](https://crates.io/crates/failure)
//! crate ecosystem.
//!
//! # Cargo Features
//!
//! This crate as two features
//!
//! ## uuid
//!
//! Enabled by default, the `uuid` features adds support for implementing `Attribute` for
//! the [uuid](https://crates.io/crates/uuid) crate type `Uuid`, a useful
//! type for producing and representing
//! unique identifiers for items that satisfy [effective characteristcs for partition keys](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/bp-partition-key-design.html)
//!
//! ## derive
//!
//! Enabled by default, the `derive` feature enables the use of the dynomite derive feature which
//! allows you simple add `#[derive(Item)]` to your structs.
//!
//! ## rustls
//!
//! Disabled by default, the `rustls` feature changes overrides rusoto's default tls
//! dependency on openssl replacing it with `rustls`-based tls implementation
//!
//! To disable any of these features
//!
//! ```toml
//! [dependencies.dynomite]
//! version = "xxx"
//! default-features = false
//! features = ["feature-you-want"]
//! ```

#![deny(missing_docs)]
// reexported
// note: this is used inside the attr_map! macro
#[cfg(feature = "default")]
pub use rusoto_dynamodb_default as dynamodb;

#[cfg(feature = "rustls")]
pub use rusoto_dynamodb_rustls as dynamodb;

use dynamodb::AttributeValue;
use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
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
/// Below is an example of doing this manually for demonstration. You can also do
/// this automatically using `#[derive(Item)]` on your structs (the recommended approach).
///
/// ```
/// use std::collections::HashMap;
/// use dynomite::{AttributeError, Item, Attribute, FromAttributes, Attributes};
/// use dynomite::dynamodb::AttributeValue;
///
/// #[derive(PartialEq,Debug, Clone)]
/// struct Person {
///   id: String
/// }
///
/// impl Item for Person {
///   fn key(&self) -> Attributes {
///     let mut attrs = HashMap::new();
///     attrs.insert("id".into(), "123".to_string().into_attr());
///     attrs
///   }
/// }
///
/// impl FromAttributes for Person {
///    fn from_attrs(
///      attrs: Attributes
///    ) -> Result<Self, AttributeError> {
///      Ok(Self {
///        id: attrs.get("id")
///          .and_then(|val| val.s.clone())
///          .ok_or(AttributeError::MissingField { name: "id".into() })?
///      })
///    }
/// }
///
/// impl Into<Attributes> for Person {
///   fn into(self: Self) -> Attributes {
///     let mut attrs = HashMap::new();
///     attrs.insert("id".into(), "123".to_string().into_attr());
///     attrs
///   }
/// }
/// fn main() {
///   let person = Person { id: "123".into() };
///   let attrs: Attributes = person.clone().into();
///   assert_eq!(Ok(person), FromAttributes::from_attrs(attrs))
/// }
/// ```
pub trait Item: Into<Attributes> + FromAttributes {
    /// Returns the set of attributes which make up this item's primary key
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
/// use dynomite::Attribute;
/// use dynomite::dynamodb::AttributeValue;
///
/// fn main() {
///   assert_eq!(
///     "test".to_string().into_attr().s,
///      AttributeValue {
///        s: Some("test".to_string()),
///        ..AttributeValue::default()
///      }.s
///    );
/// }
/// ```
pub trait Attribute: Sized {
    /// Returns a conversion into an `AttributeValue`
    fn into_attr(self: Self) -> AttributeValue;
    /// Returns a fallible conversion from an `AttributeValue`
    fn from_attr(value: AttributeValue) -> Result<Self, AttributeError>;
}

/// A type capable of being produced from
/// a set of string keys and `AttributeValues`
pub trait FromAttributes: Sized {
    /// Returns an instance of of a type resolved at runtime from a collection
    /// of a `String` keys and `AttributeValues`. If
    /// a instance can not be resolved and `AttributeError` will be returned.
    fn from_attrs(attrs: Attributes) -> Result<Self, AttributeError>;
}

/// Coerces a homogenious HashMap of attribute values into a homogeneous Map of types
/// that implement Attribute
#[allow(clippy::implicit_hasher)]
impl<A: Attribute> FromAttributes for HashMap<String, A> {
    fn from_attrs(attrs: Attributes) -> Result<Self, AttributeError> {
        attrs
            .into_iter()
            .try_fold(HashMap::new(), |mut result, (k, v)| {
                result.insert(k, A::from_attr(v)?);
                Ok(result)
            })
    }
}

/// Coerces a homogenious Map of attribute values into a homogeneous BTreeMap of types
/// that implement Attribute
impl<A: Attribute> FromAttributes for BTreeMap<String, A> {
    fn from_attrs(attrs: Attributes) -> Result<Self, AttributeError> {
        attrs
            .into_iter()
            .try_fold(BTreeMap::new(), |mut result, (k, v)| {
                result.insert(k, A::from_attr(v)?);
                Ok(result)
            })
    }
}

/// A Map type for Items, represented as the `M` AttributeValue type
impl<T: Item> Attribute for T {
    fn into_attr(self: Self) -> AttributeValue {
        AttributeValue {
            m: Some(self.into()),
            ..AttributeValue::default()
        }
    }
    fn from_attr(value: AttributeValue) -> Result<Self, AttributeError> {
        value
            .m
            .ok_or(AttributeError::InvalidType)
            .and_then(T::from_attrs)
    }
}

/// A Map type for Items for HashMaps, represented as the `M` AttributeValue type
#[allow(clippy::implicit_hasher)]
impl<A: Attribute> Attribute for HashMap<String, A> {
    fn into_attr(self: Self) -> AttributeValue {
        AttributeValue {
            m: Some(self.into_iter().map(|(k, v)| (k, v.into_attr())).collect()),
            ..AttributeValue::default()
        }
    }
    fn from_attr(value: AttributeValue) -> Result<Self, AttributeError> {
        value
            .m
            .ok_or(AttributeError::InvalidType)
            .and_then(Self::from_attrs) // because FromAttributes is impl by all HashMap<String, A>
    }
}

/// A Map type for `Items` for `BTreeMaps`, represented as the `M` AttributeValue type
impl<A: Attribute> Attribute for BTreeMap<String, A> {
    fn into_attr(self: Self) -> AttributeValue {
        AttributeValue {
            m: Some(self.into_iter().map(|(k, v)| (k, v.into_attr())).collect()),
            ..AttributeValue::default()
        }
    }
    fn from_attr(value: AttributeValue) -> Result<Self, AttributeError> {
        value
            .m
            .ok_or(AttributeError::InvalidType)
            .and_then(Self::from_attrs) // because FromAttributes is impl by all BTreeMap<String, A>
    }
}

// A `String` type for `Uuids`, represented by the `S` AttributeValue type
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
            bs: Some(self.drain().collect()),
            ..AttributeValue::default()
        }
    }
    fn from_attr(value: AttributeValue) -> Result<Self, AttributeError> {
        value
            .bs
            .ok_or(AttributeError::InvalidType)
            .map(|mut value| value.drain(..).collect())
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
impl Attribute for Vec<u8> {
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
            _ => AttributeValue::default(),
        }
    }
    fn from_attr(value: AttributeValue) -> Result<Self, AttributeError> {
        match Attribute::from_attr(value) {
            Ok(value) => Ok(Some(value)),
            Err(AttributeError::InvalidType) => Ok(None),
            Err(err) => Err(err),
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
/// # fn main() {
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
/// # }
macro_rules! attr_map {
    (@single $($x:tt)*) => (());
    (@count $($rest:expr),*) => (<[()]>::len(&[$(attr_map!(@single $rest)),*]));
    ($($key:expr => $value:expr,)+) => { attr_map!($($key => $value),+) };
    ($($key:expr => $value:expr),*) => {
        {
            let _cap = attr_map!(@count $($key),*);
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
    fn option_some_attr() {
        let value = Some(1);
        assert_eq!(Ok(value), Attribute::from_attr(value.into_attr()));
    }

    #[test]
    fn option_none_attr() {
        let value: Option<u32> = Default::default();
        assert_eq!(Ok(value), Attribute::from_attr(value.into_attr()));
    }

    #[test]
    fn option_invalid_attr() {
        assert_eq!(
            Ok(None),
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
        assert_eq!(
            Ok(value.clone()),
            String::from_attr(value.clone().into_attr())
        );
    }

    #[test]
    fn byte_vec_attr_from_attr() {
        let value = b"test".to_vec();
        assert_eq!(
            Ok(value.clone()),
            Vec::<u8>::from_attr(value.clone().into_attr())
        );
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
