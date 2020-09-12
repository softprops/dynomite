use dynomite::{Attribute, Attributes, Item};
use serde::{Deserialize, Serialize};

#[derive(Item, Default, PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct Author {
    #[dynomite(partition_key)]
    // Test that the serde attr is not propagated to the generated key
    // Issue: https://github.com/softprops/dynomite/issues/121
    #[serde(rename = "Name")]
    name: String,
}

#[derive(Attribute, PartialEq, Debug, Clone)]
pub enum Category {
    Foo,
}

impl Default for Category {
    fn default() -> Self {
        Category::Foo
    }
}

#[derive(Item, Default, PartialEq, Debug, Clone)]
pub struct Book {
    #[dynomite(partition_key)]
    title: String,
    category: Category,
    authors: Option<Vec<Author>>,
}

#[derive(Item, PartialEq, Debug, Clone)]
struct Recipe {
    #[dynomite(partition_key, rename = "RecipeId")]
    id: String,
    servings: u64,
}

#[derive(Item, PartialEq, Debug, Clone)]
struct FlattenRoot {
    #[dynomite(partition_key)]
    id: String,
    #[dynomite(flatten)]
    flat: Flattened,
}

#[derive(Attributes, PartialEq, Debug, Clone)]
struct Flattened {
    a: bool,
    #[dynomite(flatten)]
    flat_nested: FlattenedNested,
}

#[derive(Attributes, PartialEq, Debug, Clone)]
struct FlattenedNested {
    b: u64,
    c: bool,
}

#[derive(Attributes)]
struct RemainingPropsInMap {
    a: bool,
    b: u32,

    #[dynomite(flatten)]
    original_c_collector: HasC,

    #[dynomite(flatten)]
    remainder: Attributes,
}

#[derive(Attributes)]
struct HasC {
    c: u32,
}

#[derive(Attributes, Clone)]
struct AdditionalPropsVerbatim {
    a: bool,
    b: u32,
    c: u32,
    d: String,
    e: u32,
}

#[derive(Attributes, Clone, Debug, PartialEq)]
#[dynomite(tag = "kind")]
enum MyEnum {
    Foo(Foo),
    Bar(Bar),
    Nested(Nested),
}

#[derive(Attributes, Clone, Debug, PartialEq)]
struct Foo {
    a: String,
    b: u32,
}

#[derive(Attributes, Clone, Debug, PartialEq)]
struct Bar {
    a: String,
    c: bool,
}

#[derive(Attributes, Clone, Debug, PartialEq)]
#[dynomite(tag = "nested_kind")]
enum Nested {
    #[dynomite(rename = "renamed_nested_variant")]
    NestedVariant(NestedVariant),
}

#[derive(Attributes, Clone, Debug, PartialEq)]
struct NestedVariant {
    a: String,
}

#[cfg(test)]
mod tests {

    use super::*;
    use dynomite::{Attribute, Attributes, FromAttributes, Item};

    #[test]
    fn derived_key() {
        let value = Recipe {
            id: "test".into(),
            servings: 1,
        };
        assert_eq!(value.key(), RecipeKey { id: "test".into() }.into());
    }

    #[test]
    fn to_and_from_book() {
        let value = Book {
            title: "rust".into(),
            ..Default::default()
        };
        let attrs: Attributes = value.clone().into();
        assert_eq!(value, Book::from_attrs(attrs).unwrap())
    }

    #[test]
    fn derive_attr() {
        #[derive(Attribute, Debug, PartialEq)]
        enum Foo {
            Bar,
        };
        assert_eq!(Foo::Bar, Foo::from_attr(Foo::Bar.into_attr()).unwrap());
    }

    #[test]
    fn field_rename() {
        let value = Recipe {
            id: "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee".into(),
            servings: 2,
        };

        let attrs: Attributes = value.clone().into();
        assert!(attrs.contains_key("RecipeId"));
        assert!(!attrs.contains_key("id"));

        assert_eq!(value, Recipe::from_attrs(attrs).unwrap());
    }

    #[test]
    fn flatten() {
        let value = FlattenRoot {
            id: "foo".into(),
            flat: Flattened {
                a: true,
                flat_nested: FlattenedNested { b: 42, c: false },
            },
        };

        let attrs: Attributes = value.clone().into();
        assert!(!attrs.contains_key("flat"));
        assert!(!attrs.contains_key("flat_nested"));
        assert!(attrs.contains_key("id"));
        assert!(attrs.contains_key("a"));
        assert!(attrs.contains_key("b"));
        assert!(attrs.contains_key("c"));

        assert_eq!(value, FlattenRoot::from_attrs(attrs).unwrap());
    }

    #[test]
    fn additional_props() {
        let original = AdditionalPropsVerbatim {
            a: true,
            b: 42,
            c: 43,
            d: "foo".to_owned(),
            e: 44,
        };
        let attrs: Attributes = original.clone().into();
        let collected = RemainingPropsInMap::from_attrs(attrs).unwrap();

        assert_eq!(collected.a, original.a);
        assert_eq!(collected.b, original.b);
        assert_eq!(collected.original_c_collector.c, original.c);
        assert!(
            !collected.remainder.contains_key("c"),
            "prev flattened field has collected field `c` due to the order of declaration and eval"
        );
        assert!(collected.remainder.contains_key("d"));
        assert!(collected.remainder.contains_key("e"));
    }

    #[test]
    fn flat_single_item_tuple_enum() {
        let original = MyEnum::Foo(Foo {
            a: "Hello".to_owned(),
            b: 42,
        });
        let attrs: Attributes = original.clone().into();
        assert_eq!(attrs.len(), 3);
        assert!(attrs.contains_key("kind"));
        assert!(!attrs.contains_key("nested_kind"));
        assert!(attrs.contains_key("a"));
        assert!(attrs.contains_key("b"));
        assert!(!attrs.contains_key("c"));

        assert_eq!(MyEnum::from_attrs(attrs).unwrap(), original);
    }

    #[test]
    fn nested_single_item_tuple_enum() {
        let original = MyEnum::Nested(Nested::NestedVariant(NestedVariant { a: "hello".into() }));

        let attrs: Attributes = original.into();

        assert_eq!(attrs.len(), 3);

        let kind = String::from_attr(attrs.get("nested_kind").unwrap().clone()).unwrap();
        assert_eq!(kind, "renamed_nested_variant");

        assert!(attrs.contains_key("kind"));
        assert!(attrs.contains_key("a"));
    }
}
