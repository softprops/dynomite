extern crate dynomite;
#[macro_use]
extern crate dynomite_derive;
extern crate rusoto_dynamodb;

#[derive(Item, Default, PartialEq, Debug, Clone)]
pub struct Author {
  #[hash]
  name: String,
}

#[derive(Item, Default, PartialEq, Debug, Clone)]
pub struct Book {
  #[hash]
  title: String,
  authors: Option<Vec<Author>>,
}

#[cfg(test)]
mod tests {

  use super::Book;
  use super::dynomite::{Attributes, FromAttributes};

  #[test]
  fn to_and_from_book() {
    let value = Book {
      title: "rust".into(),
      ..Default::default()
    };
    let attrs: Attributes = value.clone().into();
    assert_eq!(value, Book::from_attrs(attrs).unwrap())
  }
}
