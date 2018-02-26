extern crate dynomite;
#[macro_use]
extern crate dynomite_derive;
extern crate rusoto_dynamodb;

#[derive(Item, Default, PartialEq, Debug, Clone)]
pub struct Author {
  #[hash]
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
  #[hash]
  title: String,
  category: Category,
  authors: Option<Vec<Author>>,
}

#[cfg(test)]
mod tests {

  use super::Book;
  use super::dynomite::{Attribute, Attributes, FromAttributes};

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
}
