extern crate dynomite;
#[macro_use]
extern crate dynomite_derive;
extern crate rusoto_dynamodb;

#[derive(Item, Default)]
pub struct Author {
  #[hash]
  name: String,
}

#[derive(Item, Default)]
pub struct Book {
  #[hash]
  title: String,
  authors: Option<Vec<Author>>,
}

#[cfg(test)]
mod tests {

  use std::collections::HashMap;
  use super::{Author, Book};
  use super::rusoto_dynamodb::AttributeValue;
  use super::dynomite::Attributes;

  #[test]
  fn it_works() {
    let attr: Attributes = Book {
      title: "rust".into(),
      ..Default::default()
    }.into();
    let mut expected = HashMap::new();
    expected.insert(
      "name",
      AttributeValue {
        s: Some("rust".to_string()),
        ..Default::default()
      },
    );
    println!("{:#?}", attr);
  }
}
