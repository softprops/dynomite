//! Dynomite provides a set of interfaces built on top of
//! [rusoto_dynamodb](https://rusoto.github.io/rusoto/rusoto_dynamodb/index.html)
//! which make working with aws Dynamodb more productive in rust.
//!
//! [Dynamodb](https://aws.amazon.com/dynamodb/) is a nosql database aws offers
//! as a managed service. It's abstractions include a table comprised of a collection
//!  of items which are a composed of a collection of named attributes which
//! can be one of a finite set of types. You can learn more about its core components
//! [here](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/HowItWorks.CoreComponents.html)
//!
//! [Rusoto](https://github.com/rusoto/rusoto) provides an excellent set of
//! interfaces for interacting with the dynamodb API. It's representation
//! of Items is essentially a `HashMap` of `String`
//! to [AttributeValue](https://rusoto.github.io/rusoto/rusoto_dynamodb/struct.AttributeValue.html)
//! types which fits dynamodb's nosql contract well.
//! AttributeValues are able to represent multiple types of values in a
//! single container type.
//!
//! However, when programming in rust we're afforded stricter, more concise typing
//! tools than HashMaps when working with data. Dynomite is intended to make those types
//! interface more transparently with rusoto item type apis.
//!
//! Dynomite provides a set of building blocks for making interactions with
//! dynamodb feel more natural for rust's native types.
//!
//! At a low level, [Attribute](dynomite/trait.Attribute.html) type implementations
//! provide conversion interfaces to and from native rust types which represent
//! dynamodb's notion of "attributes".
//!
//! At a higher level, [Item](dynomite/trait.Item.html) type implementations
//! provide converstion interfaces for complex types which represent
//! dynamodb's notion of "items".
//!
//! You can optionally opt into having `Item` types derived for you by using
//! the [dynomite-derive](../dynomite_derive/index.html) crate,
//! which utilizes a technique you may be familiar
//! with if you've ever worked with [serde](https://github.com/serde-rs/serde).
//!
//! # Errors
//!
//! Operations that may fail typically result in an
//! [AttributeError](error/enum.AttributeError.html). These errors were
//! designed to work well with the [failure](https://crates.io/crates/failure)
//! crate ecosystem.
//!
//! # Cargo Features
//!
//! This crate has one Cargo feature, "uuid",
//! which adds support for implementing `Attribute` for
//! the [uuid](https://crates.io/crates/uuid) crate type `Uuid`, a useful
//! type for producing and representing
//! unique identifiers for items. This feature is enabled by default.
//!

#[deny(missing_docs)]
#[macro_use]
extern crate failure;
extern crate rusoto_core;
extern crate rusoto_dynamodb;
#[cfg(feature = "uuid")]
extern crate uuid;

use std::collections::{HashMap, HashSet};

use rusoto_dynamodb::AttributeValue;
#[cfg(feature = "uuid")]
use uuid::Uuid;

pub mod error;

pub use error::AttributeError;

/// type alias for map of named attribute values
pub type Attributes = HashMap<String, AttributeValue>;

/// A type which can be represented as a set of string keys and
/// `AttributeValues` and may also be coersed from the same set of values
///
/// # Examples
///
/// ```
/// extern crate rusoto_dynamodb;
/// extern crate dynomite;
///
/// use std::collections::HashMap;
/// use dynomite::{AttributeError, Item, Attribute, FromAttributes, Attributes};
/// use rusoto_dynamodb::AttributeValue;
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

/// A type capable of being converted into an attrbute value or converted from
/// an `AttributeValue`
///
/// Implementations of this are provided for each type of `AttributeValue` field
/// which maps to a native rustlang type
///
/// # Examples
///
/// ```
/// extern crate rusoto_dynamodb;
/// extern crate dynomite;
///
/// use dynomite::Attribute;
/// use rusoto_dynamodb::AttributeValue;
///
/// fn main() {
///   assert_eq!(
///     "test".to_string().into_attr().s,
///      AttributeValue {
///        s: Some("test".to_string()),
///        ..Default::default()
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
    fn from_attrs(attrs: Attributes) -> Result<Self, AttributeError>;
}

impl<T: Item> Attribute for T {
    fn into_attr(self: Self) -> AttributeValue {
        AttributeValue {
            m: Some(self.into()),
            ..Default::default()
        }
    }
    fn from_attr(value: AttributeValue) -> Result<Self, AttributeError> {
        value
            .m
            .ok_or(AttributeError::InvalidType)
            .and_then(|attrs| T::from_attrs(attrs))
    }
}

#[cfg(feature = "uuid")]
impl Attribute for Uuid {
    fn into_attr(self: Self) -> AttributeValue {
        AttributeValue {
            s: Some(self.hyphenated().to_string()),
            ..Default::default()
        }
    }
    fn from_attr(value: AttributeValue) -> Result<Self, AttributeError> {
        value
            .s
            .ok_or(AttributeError::InvalidType)
            .and_then(|s| Uuid::parse_str(s.as_str()).map_err(|_| AttributeError::InvalidFormat))
    }
}

impl Attribute for String {
    fn into_attr(self: Self) -> AttributeValue {
        AttributeValue {
            s: Some(self),
            ..Default::default()
        }
    }
    fn from_attr(value: AttributeValue) -> Result<Self, AttributeError> {
        value.s.ok_or(AttributeError::InvalidType)
    }
}

impl Attribute for HashSet<String> {
    fn into_attr(mut self: Self) -> AttributeValue {
        AttributeValue {
            ss: Some(self.drain().collect()),
            ..Default::default()
        }
    }
    fn from_attr(value: AttributeValue) -> Result<Self, AttributeError> {
        value
            .ss
            .ok_or(AttributeError::InvalidType)
            .map(|mut value| value.drain(..).collect())
    }
}

impl Attribute for HashSet<Vec<u8>> {
    fn into_attr(mut self: Self) -> AttributeValue {
        AttributeValue {
            bs: Some(self.drain().collect()),
            ..Default::default()
        }
    }
    fn from_attr(value: AttributeValue) -> Result<Self, AttributeError> {
        value
            .bs
            .ok_or(AttributeError::InvalidType)
            .map(|mut value| value.drain(..).collect())
    }
}

impl Attribute for bool {
    fn into_attr(self: Self) -> AttributeValue {
        AttributeValue {
            bool: Some(self),
            ..Default::default()
        }
    }
    fn from_attr(value: AttributeValue) -> Result<Self, AttributeError> {
        value.bool.ok_or(AttributeError::InvalidType)
    }
}

impl Attribute for Vec<u8> {
    fn into_attr(self: Self) -> AttributeValue {
        AttributeValue {
            b: Some(self),
            ..Default::default()
        }
    }
    fn from_attr(value: AttributeValue) -> Result<Self, AttributeError> {
        value.b.ok_or(AttributeError::InvalidType)
    }
}

impl<T: Item> Attribute for Vec<T> {
    fn into_attr(mut self: Self) -> AttributeValue {
        AttributeValue {
            l: Some(self.drain(..).map(|s| s.into_attr()).collect()),
            ..Default::default()
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
            _ => Default::default(),
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
                    ..Default::default()
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

macro_rules! numeric_collection_attr {
    ($type:ty => $collection:ty) => {
        impl Attribute for $collection {
            fn into_attr(self) -> AttributeValue {
                AttributeValue {
                    ns: Some(self.iter().map(|item| item.to_string()).collect()),
                    ..Default::default()
                }
            }
            fn from_attr(value: AttributeValue) -> Result<Self, AttributeError> {
                let mut nums = value.ns.ok_or(AttributeError::InvalidType)?;
                let mut results: Vec<Result<$type, AttributeError>> = nums
                    .drain(..)
                    .map(|ns| ns.parse().map_err(|_| AttributeError::InvalidFormat))
                    .collect();
                let collected = results.drain(..).collect();
                collected
            }
        }
    };
}

// implement Attribute for numeric types
numeric_attr!(u16);
numeric_attr!(u32);
numeric_attr!(i32);
numeric_attr!(i64);
numeric_attr!(f32);
numeric_attr!(f64);

// implement Attribute for numeric collections
numeric_collection_attr!(u16 => HashSet<u16>);
numeric_collection_attr!(u16 => Vec<u16>);
numeric_collection_attr!(u32 => HashSet<u32>);
numeric_collection_attr!(u32 => Vec<u32>);
numeric_collection_attr!(i32 => HashSet<i32>);
numeric_collection_attr!(i32 => Vec<i32>);
numeric_collection_attr!(i64 => HashSet<i64>);
numeric_collection_attr!(i64 => Vec<i64>);
numeric_collection_attr!(f32 => Vec<f32>);
numeric_collection_attr!(f64 => Vec<f64>);

#[cfg(test)]
mod test {
    use super::*;

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
                ..Default::default()
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
                ..Default::default()
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
    fn byte_vec_attr() {
        let value = "test".as_bytes().to_vec();
        assert_eq!(
            Ok(value.clone()),
            Vec::<u8>::from_attr(value.clone().into_attr())
        );
    }
}
