extern crate dynomite;
#[macro_use]
extern crate dynomite_derive;
extern crate rusoto_core;
extern crate rusoto_dynamodb;
extern crate uuid;

use uuid::Uuid;
use rusoto_core::{default_tls_client, DefaultCredentialsProvider, Region};
use rusoto_dynamodb::*;

// for Item trait interface resolution
use dynomite::Item;

#[derive(Item, Debug, Clone)]
pub struct Book {
  #[hash]
  id: Uuid,
  title: String,
}

// this will create a rust book shelf in your aws account!
fn main() {
  // create rusoto client
  let client = DynamoDbClient::new(
    default_tls_client().unwrap(),
    DefaultCredentialsProvider::new().unwrap(),
    Region::UsEast1,
  );

  // create a book table with a single string (S) primary key.
  // if this table does not already exists
  // this may take a second or two to provision.
  // it will fail if this table already exists but that's okay,
  // this is just an example :)
  let table_name = "books".to_string();
  let _ = client.create_table(&CreateTableInput {
    table_name: table_name.clone(),
    key_schema: vec![
      KeySchemaElement {
        attribute_name: "id".into(),
        key_type: "HASH".into(),
      },
    ],
    attribute_definitions: vec![
      AttributeDefinition {
        attribute_name: "id".into(),
        attribute_type: "S".into(),
      },
    ],
    provisioned_throughput: ProvisionedThroughput {
      read_capacity_units: 1,
      write_capacity_units: 1,
    },
    ..Default::default()
  });

  let book = Book {
    id: Uuid::new_v4(),
    title: "rust".into(),
  };

  // print the key for this book
  // requires bringing `dynomite::Item` into scope
  println!("key {:#?}", book.key());

  // add a book to the shelf
  println!(
    "{:#?}",
    client.put_item(&PutItemInput {
      table_name: table_name.clone(),
      item: book.clone().into(), // convert book into it's attribute representation
      ..Default::default()
    })
  );

  // get the book by it's application generated key
  println!(
    "{:#?}",
    client.get_item(&GetItemInput {
      table_name: table_name.clone(),
      key: book.clone().key(), // get a book by key
      ..Default::default()
    })
  );
}
