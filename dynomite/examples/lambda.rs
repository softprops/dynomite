use dynomite::{
    dynamodb::{DynamoDb, DynamoDbClient},
    retry::Policy,
    Retries,
};
use lambda::handler_fn;
use serde_json::Value;

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let client = DynamoDbClient::new(Default::default()).with_retries(Policy::default());

    lambda::run(handler_fn(move |event: Value| {
        let client = client.clone();
        async move {
            client.clone().list_tables(Default::default()).await?;
            Ok::<_, Error>(event)
        }
    }))
    .await?;

    Ok(())
}
