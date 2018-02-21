extern crate dynomite;
#[macro_use]
extern crate dynomite_derive;
extern crate rusoto_dynamodb;

use std::collections::HashSet;

#[derive(Item, Default)]
pub struct User {
  #[hash]
  name: String,
}

#[derive(Item, Default)]
pub struct Book {
  #[hash]
  title: String,
  authors: Option<Vec<User>>,
}

#[cfg(test)]
mod tests {
  use super::{Book, User};
  use dynomite::Attribute;
  use rusoto_dynamodb::AttributeValue;
  use std::collections::HashMap;

  #[test]
  fn it_works() {
    let attr: HashMap<String, AttributeValue> = Book {
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
