use dynomite::{
    attr_map,
    dynamodb::{
        AttributeDefinition, CreateTableInput, DynamoDb, DynamoDbClient, GetItemInput,
        KeySchemaElement, ProvisionedThroughput, PutItemInput, ScanInput,
    },
    DynamoDbExt, FromAttributes, Item,
};
use futures::{Future, Stream};
use std::sync::Arc;
use tokio::runtime::Runtime;
use uuid::Uuid;

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
        ..CreateTableInput::default()
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
            item: book.clone().into(), // <= convert book into it's attribute map representation
            ..PutItemInput::default()
        }))
    );

    println!(
        "put_item() result {:#?}",
        rt.block_on(
            client.put_item(PutItemInput {
                table_name: table_name.clone(),
                // convert book into it's attribute map representation
                item: Book {
                    id: Uuid::new_v4(),
                    title: "rust and beyond".into(),
                }
                .into(),
                ..PutItemInput::default()
            })
        )
    );

    // scan through all pages of results in the books table for books who's title is "rust"
    println!(
        "scan result {:#?}",
        rt.block_on(
            client
                .clone()
                .scan_pages(ScanInput {
                    limit: Some(1), // to demonstrate we're getting through more than one page
                    table_name: table_name.clone(),
                    filter_expression: Some("title = :title".into()),
                    expression_attribute_values: Some(attr_map!(
                        ":title" => "rust".to_string()
                    )),
                    ..ScanInput::default()
                })
                .for_each(|item| {
                    println!("stream_scan() item {:#?}", Book::from_attrs(item));
                    Ok(())
                }) // attempt to convert a attribute map to a book type
        ),
    );

    // get the "rust' book by the Book type's generated key
    println!(
        "get_item() result {:#?}",
        rt.block_on(
            client
                .get_item(GetItemInput {
                    table_name: table_name.clone(),
                    key: book.clone().key(), // get a book by key
                    ..GetItemInput::default()
                })
                .map(|result| result.item.map(Book::from_attrs)) // attempt to convert a attribute map to a book type
        )
    );
}
