#[macro_use]
extern crate dynomite;
#[macro_use]
extern crate dynomite_derive;
extern crate futures;
extern crate rusoto_core;
extern crate tokio;
extern crate uuid;

use dynomite::rusoto_dynamodb;
use futures::stream::Stream;
use rusoto_dynamodb::{
    AttributeDefinition, CreateTableInput, DynamoDb, DynamoDbClient, GetItemInput,
    KeySchemaElement, ProvisionedThroughput, PutItemInput, ScanInput,
};
use std::sync::Arc;
use tokio::runtime::Runtime;
use uuid::Uuid;

// for Item trait interface resolution and extensions
use dynomite::{DynamoDbExt, Item};

#[derive(Item, Debug, Clone)]
pub struct Book {
    #[hash]
    id: Uuid,
    title: String,
}

// this will create a rust book shelf in your aws account!
fn main() {
    let mut rt = Runtime::new().expect("failed to initialize futures runtime");
    // create rusoto client
    let client = Arc::new(DynamoDbClient::new(Default::default()));

    // create a book table with a single string (S) primary key.
    // if this table does not already exists
    // this may take a second or two to provision.
    // it will fail if this table already exists but that's okay,
    // this is just an example :)
    let table_name = "books".to_string();
    let _ = rt.block_on(client.create_table(CreateTableInput {
        table_name: table_name.clone(),
        key_schema: vec![KeySchemaElement {
            attribute_name: "id".into(),
            key_type: "HASH".into(),
        }],
        attribute_definitions: vec![AttributeDefinition {
            attribute_name: "id".into(),
            attribute_type: "S".into(),
        }],
        provisioned_throughput: ProvisionedThroughput {
            read_capacity_units: 1,
            write_capacity_units: 1,
        },
        ..Default::default()
    }));

    let book = Book {
        id: Uuid::new_v4(),
        title: "rust".into(),
    };

    // print the key for this book
    // requires bringing `dynomite::Item` into scope
    println!("book.key() {:#?}", book.key());

    // add a book to the shelf
    println!(
        "put_item() result {:#?}",
        rt.block_on(client.put_item(PutItemInput {
            table_name: table_name.clone(),
            item: book.clone().into(), // convert book into it's attribute representation
            ..PutItemInput::default()
        }))
    );

    // get the book by it's application generated key
    println!(
        "get_item() result {:#?}",
        rt.block_on(client.get_item(GetItemInput {
            table_name: table_name.clone(),
            key: book.clone().key(), // get a book by key
            ..GetItemInput::default()
        }))
    );

    println!(
        "scan result {:#?}",
        rt.block_on(
            client
                .clone()
                .stream_scan(ScanInput {
                    limit: Some(1),
                    table_name: table_name.clone(),
                    filter_expression: Some("title = :title".into()),
                    expression_attribute_values: Some(attr_map!(
                        ":title" => "rust".to_string()
                    )),
                    ..ScanInput::default()
                }).for_each(|item| Ok(println!("stream_scan() item {:#?}", item))),
        ),
    );

    println!(
        "get_item() result {:#?}",
        rt.block_on(client.get_item(GetItemInput {
            table_name: table_name.clone(),
            key: book.clone().key(), // get a book by key
            ..GetItemInput::default()
        }))
    );
}
