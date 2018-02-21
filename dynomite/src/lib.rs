//! Dynomite provides a set of convenience types for working with
//! [rusoto_dynamodb](https://rusoto.github.io/rusoto/rusoto_dynamodb/index.html)
//!
#[macro_use]
extern crate maplit;
extern crate rusoto_core;
extern crate rusoto_dynamodb;
extern crate uuid;

use std::collections::{HashMap, HashSet};

use rusoto_core::{default_tls_client, DefaultCredentialsProvider, Region};
use rusoto_dynamodb::*;

/// A type which can be represented as a set of string keys and
/// `AttributeValues` and may also be coersed from the same set
///
/// # Examples
///
/// ```
/// extern crate rusoto_dynamodb;
/// extern crate dynomite;
///
/// use dynomite::{Item, Attribute, FromAttributeValues};
/// use rusoto_dynamodb::AttributeValue;
///
/// struct User {
///   id: String
/// }
///
/// impl Item for Person {
///   fn key(&self) -> HashMap<String, AttributeValue> {
///     let mut attrs = HashMap::new();
///     attrs.insert("id".into(), "123".to_string().into_attr());
///     attrs
///   }
/// }
///
/// impl FromAttributeValues for Person {
///    fn from_attrs(
///      attrs: HashMap<String, AttributeValue>
///    ) -> Result<Self, String> {
///      Self {
///        id: attrs.get("id")
///          .and_then(|val| val.s)
///          .ok_or("missing id".to_string())?
///      }
///    }
/// }
///
/// impl Into<HashMap<String, AttributeValue>> for Person {
///   fn into(self: Self) -> HashMap<String, AttributeValue> {
///     let mut attrs = HashMap::new();
///     attrs.insert("id".into(), "123".to_string().into_attr());
///     attrs
///   }
/// }
/// ```
pub trait Item: Into<HashMap<String, AttributeValue>> + FromAttributeValues {
    /// Returns the set of attributes which make up this items key
    fn key(&self) -> HashMap<String, AttributeValue>;
}

/// A type capable of being converted into an attrbute value or converted from
/// an `AttributeValue`
///
/// Implementations of this is provided for each type of `AttributeValue` field
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
/// assert_eq!(
///   "test".to_string().into_attr(),
///    AttributeValue {
///      s: Some("test".to_string()),
///      ..Default::default()
///    }
///  );
/// ```
pub trait Attribute: Sized {
    /// Returns a conversion into an `AttributeValue`
    fn into_attr(self: Self) -> AttributeValue;
    /// Returns a fallible conversion from an `AttributeValue`
    fn from_attr(value: AttributeValue) -> Result<Self, String>;
}

/// A type capable of being produced from
/// a set of string keys and `AttributeValues`
pub trait FromAttributeValues: Sized {
    fn from_attrs(values: HashMap<String, AttributeValue>) -> Result<Self, String>;
}

impl<T: Item> Attribute for T {
    fn into_attr(self: Self) -> AttributeValue {
        AttributeValue {
            m: Some(self.into()),
            ..Default::default()
        }
    }
    fn from_attr(value: AttributeValue) -> Result<Self, String> {
        value
            .m
            .ok_or("missing".into())
            .and_then(|attrs| T::from_attrs(attrs))
    }
}

impl Attribute for String {
    fn into_attr(self: Self) -> AttributeValue {
        AttributeValue {
            s: Some(self),
            ..Default::default()
        }
    }
    fn from_attr(value: AttributeValue) -> Result<Self, String> {
        value.s.ok_or("missing".into())
    }
}

impl Attribute for HashSet<String> {
    fn into_attr(mut self: Self) -> AttributeValue {
        AttributeValue {
            ss: Some(self.drain().collect()),
            ..Default::default()
        }
    }
    fn from_attr(value: AttributeValue) -> Result<Self, String> {
        value
            .ss
            .ok_or("missing".into())
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
    fn from_attr(value: AttributeValue) -> Result<Self, String> {
        value
            .bs
            .ok_or("missing".into())
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
    fn from_attr(value: AttributeValue) -> Result<Self, String> {
        value.bool.ok_or("missing".into())
    }
}

impl Attribute for Vec<u8> {
    fn into_attr(self: Self) -> AttributeValue {
        AttributeValue {
            b: Some(self),
            ..Default::default()
        }
    }
    fn from_attr(value: AttributeValue) -> Result<Self, String> {
        value.b.ok_or("missing".into())
    }
}

impl<T: Item> Attribute for Vec<T> {
    fn into_attr(mut self: Self) -> AttributeValue {
        AttributeValue {
            l: Some(self.drain(..).map(|s| s.into_attr()).collect()),
            ..Default::default()
        }
    }
    fn from_attr(value: AttributeValue) -> Result<Self, String> {
        value
            .l
            .ok_or("missing".to_string())?
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
    fn from_attr(value: AttributeValue) -> Result<Self, String> {
        match Attribute::from_attr(value) {
            Ok(value) => Ok(Some(value)),
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
            fn from_attr(value: AttributeValue) -> Result<Self, String> {
                value.n
                    .ok_or("missing".into())
                    .and_then(|num| {
                        num.parse()
                            .map_err(|_| "invalid value".into())
                    })
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
            fn from_attr(value: AttributeValue) -> Result<Self, String> {
                let mut nums = value.ns
                    .ok_or("missing".to_string())?;
                let mut results: Vec<Result<$type, String>> =
                    nums.drain(..).map(|ns| ns.parse().map_err(|_| "invalid type".to_string())).collect();
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
