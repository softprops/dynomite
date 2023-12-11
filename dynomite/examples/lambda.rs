use dynomite::{
    dynamodb::{DynamoDb, DynamoDbClient},
    retry::Policy,
    Retries,
};
use lambda_http::service_fn;

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let client = DynamoDbClient::new(Default::default()).with_retries(Policy::default());

    lambda_http::run(service_fn(move |_| {
        let client = client.clone();
        async move {
            let tables = client
                .list_tables(Default::default())
                .await?
                .table_names
                .unwrap_or_default();
            Ok::<_, Error>(tables.join("\n"))
        }
    }))
    .await?;

    Ok(())
}
