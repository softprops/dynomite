use dynomite_derive::{Attribute, Item};

#[derive(Item, Default, PartialEq, Debug, Clone)]
pub struct Author {
    #[dynomite(partition_key)]
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

#[cfg(test)]
mod tests {

    use super::*;
    use dynomite::{Item, Attribute, Attributes, FromAttributes};

    #[test]
    fn derived_key() {
        let value = Recipe {
            id: "test".into(),
            servings: 1
        };
        //let attrs: Attributes = value.clone().into();
        assert_eq!(value.key(), RecipeKey { RecipeId: "test".into() }.into());
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

        assert!(attrs.contains_key("recipe_id"));
        assert!(!attrs.contains_key("id"));

        assert_eq!(value, Recipe::from_attrs(attrs).unwrap());
    }
}
