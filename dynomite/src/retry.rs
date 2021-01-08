//! Retry functionality
//!
//! Specifically this implementation focuses on honoring [these documented DynamoDB retryable errors](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/Programming.Errors.html#Programming.Errors.MessagesAndCodes)
//! on top AWS's general recommendations of for [retrying API requests](https://docs.aws.amazon.com/general/latest/gr/api-retries.html).
//!
//! # examples
//! ```rust,no_run
//!  use dynomite::{Retries, retry::Policy};
//!  use dynomite::dynamodb::{DynamoDb, DynamoDbClient};
//!
//!  let client =
//!     DynamoDbClient::new(Default::default())
//!         .with_retries(Policy::default());
//!
//!  // any client operation will now be retried when
//!  // appropriate
//!  let tables = client.list_tables(Default::default());
//! ```

use crate::dynamodb::*;
use again::{Condition, RetryPolicy};
use log::debug;
use rusoto_core::RusotoError;
use std::{sync::Arc, time::Duration};

/// Pre-configured retry policies for fallible operations
///
/// A `Default` impl of retrying 5 times with an exponential backoff of 100 milliseconds
#[derive(Clone, PartialEq, Debug)]
pub enum Policy {
    /// Limited number of times to retry
    Limit(usize),
    /// Limited number of times to retry with fixed pause between retries
    Pause(usize, Duration),
    /// Limited number of times to retry with an exponential pause between retries
    Exponential(usize, Duration),
}

impl Default for Policy {
    fn default() -> Self {
        Policy::Exponential(5, Duration::from_millis(100))
    }
}

impl Into<RetryPolicy> for Policy {
    fn into(self) -> RetryPolicy {
        match self {
            Policy::Limit(times) => RetryPolicy::default()
                .with_max_retries(times)
                .with_jitter(true),
            Policy::Pause(times, duration) => RetryPolicy::fixed(duration)
                .with_max_retries(times)
                .with_jitter(true),
            Policy::Exponential(times, duration) => RetryPolicy::exponential(duration)
                .with_max_retries(times)
                .with_jitter(true),
        }
    }
}

/// Predicate trait that determines if an impl
/// type is retryable
trait Retry {
    /// Return true if type is retryable
    fn retryable(&self) -> bool;
}

struct Counter(u16);

impl<R> Condition<RusotoError<R>> for Counter
where
    R: Retry,
{
    fn is_retryable(
        &mut self,
        error: &RusotoError<R>,
    ) -> bool {
        debug!("retrying operation {}", self.0);
        if let Some(value) = self.0.checked_add(1) {
            self.0 = value;
        }
        match error {
            RusotoError::Service(e) => e.retryable(),
            _ => false,
        }
    }
}

// wrapper so we only pay for one arc
struct Inner<D> {
    client: D,
    policy: RetryPolicy,
}

/// A type which implements `DynamoDb` and retries all operations
/// that are retryable
#[derive(Clone)]
pub struct RetryingDynamoDb<D> {
    inner: Arc<Inner<D>>,
}

/// An interface for adapting a `DynamoDb` impl
/// to a `RetryingDynamoDb` impl
pub trait Retries<D>
where
    D: DynamoDb + 'static,
{
    /// Consumes a `DynamoDb` impl and produces
    /// a `DynamoDb` which retries its operations when appropriate
    fn with_retries(
        self,
        policy: Policy,
    ) -> RetryingDynamoDb<D>;
}

impl<D> Retries<D> for D
where
    D: DynamoDb + 'static,
{
    fn with_retries(
        self,
        policy: Policy,
    ) -> RetryingDynamoDb<D> {
        RetryingDynamoDb::new(self, policy)
    }
}

impl<D> RetryingDynamoDb<D>
where
    D: DynamoDb + 'static,
{
    /// Return a new instance with a configured retry policy
    pub fn new(
        client: D,
        policy: Policy,
    ) -> Self {
        Self {
            inner: Arc::new(Inner {
                client,
                policy: policy.into(),
            }),
        }
    }
}

#[async_trait::async_trait]
impl<D> DynamoDb for RetryingDynamoDb<D>
where
    D: DynamoDb + Sync + Send + Clone + 'static,
{
    async fn batch_get_item(
        &self,
        input: BatchGetItemInput,
    ) -> Result<BatchGetItemOutput, RusotoError<BatchGetItemError>> {
        self.inner
            .policy
            .retry_if(
                move || {
                    let client = self.inner.clone().client.clone();
                    let input = input.clone();
                    async move { client.batch_get_item(input).await }
                },
                Counter(0),
            )
            .await
    }

    async fn batch_write_item(
        &self,
        input: BatchWriteItemInput,
    ) -> Result<BatchWriteItemOutput, RusotoError<BatchWriteItemError>> {
        self.inner
            .policy
            .retry_if(
                move || {
                    let client = self.inner.clone().client.clone();
                    let input = input.clone();
                    async move { client.batch_write_item(input).await }
                },
                Counter(0),
            )
            .await
    }

    async fn create_backup(
        &self,
        input: CreateBackupInput,
    ) -> Result<CreateBackupOutput, RusotoError<CreateBackupError>> {
        self.inner
            .policy
            .retry_if(
                move || {
                    let client = self.inner.clone().client.clone();
                    let input = input.clone();
                    async move { client.create_backup(input).await }
                },
                Counter(0),
            )
            .await
    }

    async fn create_global_table(
        &self,
        input: CreateGlobalTableInput,
    ) -> Result<CreateGlobalTableOutput, RusotoError<CreateGlobalTableError>> {
        self.inner
            .policy
            .retry_if(
                move || {
                    let client = self.inner.clone().client.clone();
                    let input = input.clone();
                    async move { client.create_global_table(input).await }
                },
                Counter(0),
            )
            .await
    }

    async fn create_table(
        &self,
        input: CreateTableInput,
    ) -> Result<CreateTableOutput, RusotoError<CreateTableError>> {
        self.inner
            .policy
            .retry_if(
                move || {
                    let client = self.inner.clone().client.clone();
                    let input = input.clone();
                    async move { client.create_table(input).await }
                },
                Counter(0),
            )
            .await
    }

    async fn delete_backup(
        &self,
        input: DeleteBackupInput,
    ) -> Result<DeleteBackupOutput, RusotoError<DeleteBackupError>> {
        self.inner
            .policy
            .retry_if(
                move || {
                    let client = self.inner.clone().client.clone();
                    let input = input.clone();
                    async move { client.delete_backup(input).await }
                },
                Counter(0),
            )
            .await
    }

    async fn delete_item(
        &self,
        input: DeleteItemInput,
    ) -> Result<DeleteItemOutput, RusotoError<DeleteItemError>> {
        self.inner
            .policy
            .retry_if(
                move || {
                    let client = self.inner.clone().client.clone();
                    let input = input.clone();
                    async move { client.delete_item(input).await }
                },
                Counter(0),
            )
            .await
    }

    async fn delete_table(
        &self,
        input: DeleteTableInput,
    ) -> Result<DeleteTableOutput, RusotoError<DeleteTableError>> {
        self.inner
            .policy
            .retry_if(
                move || {
                    let client = self.inner.clone().client.clone();
                    let input = input.clone();
                    async move { client.delete_table(input).await }
                },
                Counter(0),
            )
            .await
    }

    async fn describe_backup(
        &self,
        input: DescribeBackupInput,
    ) -> Result<DescribeBackupOutput, RusotoError<DescribeBackupError>> {
        self.inner
            .policy
            .retry_if(
                move || {
                    let client = self.inner.clone().client.clone();
                    let input = input.clone();
                    async move { client.describe_backup(input).await }
                },
                Counter(0),
            )
            .await
    }

    async fn describe_export(
        &self,
        input: DescribeExportInput,
    ) -> Result<DescribeExportOutput, RusotoError<DescribeExportError>> {
        self.inner.client.describe_export(input).await
    }

    async fn describe_continuous_backups(
        &self,
        input: DescribeContinuousBackupsInput,
    ) -> Result<DescribeContinuousBackupsOutput, RusotoError<DescribeContinuousBackupsError>> {
        self.inner
            .policy
            .retry_if(
                move || {
                    let client = self.inner.clone().client.clone();
                    let input = input.clone();
                    async move { client.describe_continuous_backups(input).await }
                },
                Counter(0),
            )
            .await
    }

    async fn describe_contributor_insights(
        &self,
        input: DescribeContributorInsightsInput,
    ) -> Result<DescribeContributorInsightsOutput, RusotoError<DescribeContributorInsightsError>>
    {
        self.inner.client.describe_contributor_insights(input).await
    }

    async fn describe_global_table(
        &self,
        input: DescribeGlobalTableInput,
    ) -> Result<DescribeGlobalTableOutput, RusotoError<DescribeGlobalTableError>> {
        self.inner
            .policy
            .retry_if(
                move || {
                    let client = self.inner.clone().client.clone();
                    let input = input.clone();
                    async move { client.describe_global_table(input).await }
                },
                Counter(0),
            )
            .await
    }

    async fn describe_global_table_settings(
        &self,
        input: DescribeGlobalTableSettingsInput,
    ) -> Result<DescribeGlobalTableSettingsOutput, RusotoError<DescribeGlobalTableSettingsError>>
    {
        self.inner
            .policy
            .retry_if(
                move || {
                    let client = self.inner.clone().client.clone();
                    let input = input.clone();
                    async move { client.describe_global_table_settings(input).await }
                },
                Counter(0),
            )
            .await
    }

    async fn describe_limits(
        &self
    ) -> Result<DescribeLimitsOutput, RusotoError<DescribeLimitsError>> {
        self.inner
            .policy
            .retry_if(
                move || {
                    let client = self.inner.clone().client.clone();
                    async move { client.describe_limits().await }
                },
                Counter(0),
            )
            .await
    }

    async fn describe_table(
        &self,
        input: DescribeTableInput,
    ) -> Result<DescribeTableOutput, RusotoError<DescribeTableError>> {
        self.inner
            .policy
            .retry_if(
                move || {
                    let client = self.inner.clone().client.clone();
                    let input = input.clone();
                    async move { client.describe_table(input).await }
                },
                Counter(0),
            )
            .await
    }

    async fn describe_table_replica_auto_scaling(
        &self,
        input: DescribeTableReplicaAutoScalingInput,
    ) -> Result<
        DescribeTableReplicaAutoScalingOutput,
        RusotoError<DescribeTableReplicaAutoScalingError>,
    > {
        self.inner
            .client
            .describe_table_replica_auto_scaling(input)
            .await
    }

    async fn describe_time_to_live(
        &self,
        input: DescribeTimeToLiveInput,
    ) -> Result<DescribeTimeToLiveOutput, RusotoError<DescribeTimeToLiveError>> {
        self.inner
            .policy
            .retry_if(
                move || {
                    let client = self.inner.clone().client.clone();
                    let input = input.clone();
                    async move { client.describe_time_to_live(input).await }
                },
                Counter(0),
            )
            .await
    }

    async fn get_item(
        &self,
        input: GetItemInput,
    ) -> Result<GetItemOutput, RusotoError<GetItemError>> {
        self.inner
            .policy
            .retry_if(
                move || {
                    let client = self.inner.clone().client.clone();
                    let input = input.clone();
                    async move { client.get_item(input).await }
                },
                Counter(0),
            )
            .await
    }

    async fn list_backups(
        &self,
        input: ListBackupsInput,
    ) -> Result<ListBackupsOutput, RusotoError<ListBackupsError>> {
        self.inner
            .policy
            .retry_if(
                move || {
                    let client = self.inner.clone().client.clone();
                    let input = input.clone();
                    async move { client.list_backups(input).await }
                },
                Counter(0),
            )
            .await
    }

    async fn list_exports(
        &self,
        input: ListExportsInput,
    ) -> Result<ListExportsOutput, RusotoError<ListExportsError>> {
        self.inner.client.list_exports(input).await
    }

    async fn list_contributor_insights(
        &self,
        input: ListContributorInsightsInput,
    ) -> Result<ListContributorInsightsOutput, RusotoError<ListContributorInsightsError>> {
        self.inner.client.list_contributor_insights(input).await
    }

    async fn list_global_tables(
        &self,
        input: ListGlobalTablesInput,
    ) -> Result<ListGlobalTablesOutput, RusotoError<ListGlobalTablesError>> {
        self.inner
            .policy
            .retry_if(
                move || {
                    let client = self.inner.clone().client.clone();
                    let input = input.clone();
                    async move { client.list_global_tables(input).await }
                },
                Counter(0),
            )
            .await
    }

    async fn list_tables(
        &self,
        input: ListTablesInput,
    ) -> Result<ListTablesOutput, RusotoError<ListTablesError>> {
        self.inner
            .policy
            .retry_if(
                move || {
                    let client = self.inner.clone().client.clone();
                    let input = input.clone();
                    async move { client.list_tables(input).await }
                },
                Counter(0),
            )
            .await
    }

    async fn list_tags_of_resource(
        &self,
        input: ListTagsOfResourceInput,
    ) -> Result<ListTagsOfResourceOutput, RusotoError<ListTagsOfResourceError>> {
        self.inner
            .policy
            .retry_if(
                move || {
                    let client = self.inner.clone().client.clone();
                    let input = input.clone();
                    async move { client.list_tags_of_resource(input).await }
                },
                Counter(0),
            )
            .await
    }

    async fn put_item(
        &self,
        input: PutItemInput,
    ) -> Result<PutItemOutput, RusotoError<PutItemError>> {
        self.inner
            .policy
            .retry_if(
                move || {
                    let client = self.inner.clone().client.clone();
                    let input = input.clone();
                    async move { client.put_item(input).await }
                },
                Counter(0),
            )
            .await
    }

    async fn query(
        &self,
        input: QueryInput,
    ) -> Result<QueryOutput, RusotoError<QueryError>> {
        self.inner
            .policy
            .retry_if(
                move || {
                    let client = self.inner.clone().client.clone();
                    let input = input.clone();
                    async move { client.query(input).await }
                },
                Counter(0),
            )
            .await
    }

    async fn restore_table_from_backup(
        &self,
        input: RestoreTableFromBackupInput,
    ) -> Result<RestoreTableFromBackupOutput, RusotoError<RestoreTableFromBackupError>> {
        self.inner
            .policy
            .retry_if(
                move || {
                    let client = self.inner.clone().client.clone();
                    let input = input.clone();
                    async move { client.restore_table_from_backup(input).await }
                },
                Counter(0),
            )
            .await
    }

    async fn restore_table_to_point_in_time(
        &self,
        input: RestoreTableToPointInTimeInput,
    ) -> Result<RestoreTableToPointInTimeOutput, RusotoError<RestoreTableToPointInTimeError>> {
        self.inner
            .policy
            .retry_if(
                move || {
                    let client = self.inner.clone().client.clone();
                    let input = input.clone();
                    async move { client.restore_table_to_point_in_time(input).await }
                },
                Counter(0),
            )
            .await
    }

    async fn scan(
        &self,
        input: ScanInput,
    ) -> Result<ScanOutput, RusotoError<ScanError>> {
        self.inner
            .policy
            .retry_if(
                move || {
                    let client = self.inner.clone().client.clone();
                    let input = input.clone();
                    async move { client.scan(input).await }
                },
                Counter(0),
            )
            .await
    }

    async fn tag_resource(
        &self,
        input: TagResourceInput,
    ) -> Result<(), RusotoError<TagResourceError>> {
        self.inner
            .policy
            .retry_if(
                move || {
                    let client = self.inner.clone().client.clone();
                    let input = input.clone();
                    async move { client.tag_resource(input).await }
                },
                Counter(0),
            )
            .await
    }

    async fn untag_resource(
        &self,
        input: UntagResourceInput,
    ) -> Result<(), RusotoError<UntagResourceError>> {
        self.inner
            .policy
            .retry_if(
                move || {
                    let client = self.inner.clone().client.clone();
                    let input = input.clone();
                    async move { client.untag_resource(input).await }
                },
                Counter(0),
            )
            .await
    }

    async fn update_continuous_backups(
        &self,
        input: UpdateContinuousBackupsInput,
    ) -> Result<UpdateContinuousBackupsOutput, RusotoError<UpdateContinuousBackupsError>> {
        self.inner
            .policy
            .retry_if(
                move || {
                    let client = self.inner.clone().client.clone();
                    let input = input.clone();
                    async move { client.update_continuous_backups(input).await }
                },
                Counter(0),
            )
            .await
    }

    async fn update_contributor_insights(
        &self,
        input: UpdateContributorInsightsInput,
    ) -> Result<UpdateContributorInsightsOutput, RusotoError<UpdateContributorInsightsError>> {
        // todo: retry
        self.inner
            .clone()
            .client
            .update_contributor_insights(input)
            .await
    }

    async fn update_global_table(
        &self,
        input: UpdateGlobalTableInput,
    ) -> Result<UpdateGlobalTableOutput, RusotoError<UpdateGlobalTableError>> {
        self.inner
            .policy
            .retry_if(
                move || {
                    let client = self.inner.clone().client.clone();
                    let input = input.clone();
                    async move { client.update_global_table(input).await }
                },
                Counter(0),
            )
            .await
    }

    async fn update_global_table_settings(
        &self,
        input: UpdateGlobalTableSettingsInput,
    ) -> Result<UpdateGlobalTableSettingsOutput, RusotoError<UpdateGlobalTableSettingsError>> {
        self.inner
            .policy
            .retry_if(
                move || {
                    let client = self.inner.clone().client.clone();
                    let input = input.clone();
                    async move { client.update_global_table_settings(input).await }
                },
                Counter(0),
            )
            .await
    }

    async fn update_item(
        &self,
        input: UpdateItemInput,
    ) -> Result<UpdateItemOutput, RusotoError<UpdateItemError>> {
        self.inner
            .policy
            .retry_if(
                move || {
                    let client = self.inner.clone().client.clone();
                    let input = input.clone();
                    async move { client.update_item(input).await }
                },
                Counter(0),
            )
            .await
    }

    async fn update_table(
        &self,
        input: UpdateTableInput,
    ) -> Result<UpdateTableOutput, RusotoError<UpdateTableError>> {
        self.inner
            .policy
            .retry_if(
                move || {
                    let client = self.inner.clone().client.clone();
                    let input = input.clone();
                    async move { client.update_table(input).await }
                },
                Counter(0),
            )
            .await
    }

    async fn update_table_replica_auto_scaling(
        &self,
        input: UpdateTableReplicaAutoScalingInput,
    ) -> Result<UpdateTableReplicaAutoScalingOutput, RusotoError<UpdateTableReplicaAutoScalingError>>
    {
        self.inner
            .client
            .update_table_replica_auto_scaling(input)
            .await
    }

    async fn update_time_to_live(
        &self,
        input: UpdateTimeToLiveInput,
    ) -> Result<UpdateTimeToLiveOutput, RusotoError<UpdateTimeToLiveError>> {
        self.inner
            .policy
            .retry_if(
                move || {
                    let client = self.inner.clone().client.clone();
                    let input = input.clone();
                    async move { client.update_time_to_live(input).await }
                },
                Counter(0),
            )
            .await
    }

    async fn describe_endpoints(
        &self
    ) -> Result<DescribeEndpointsResponse, RusotoError<DescribeEndpointsError>> {
        // no apparent retryable errors
        self.inner.client.describe_endpoints().await
    }

    async fn transact_get_items(
        &self,
        input: TransactGetItemsInput,
    ) -> Result<TransactGetItemsOutput, RusotoError<TransactGetItemsError>> {
        self.inner
            .policy
            .retry_if(
                move || {
                    let client = self.inner.clone().client.clone();
                    let input = input.clone();
                    async move { client.transact_get_items(input).await }
                },
                Counter(0),
            )
            .await
    }

    async fn transact_write_items(
        &self,
        input: TransactWriteItemsInput,
    ) -> Result<TransactWriteItemsOutput, RusotoError<TransactWriteItemsError>> {
        self.inner
            .policy
            .retry_if(
                move || {
                    let client = self.inner.clone().client.clone();
                    let input = input.clone();
                    async move { client.transact_write_items(input).await }
                },
                Counter(0),
            )
            .await
    }

    async fn batch_execute_statement(
        &self,
        input: BatchExecuteStatementInput,
    ) -> Result<BatchExecuteStatementOutput, RusotoError<BatchExecuteStatementError>> {
        self.inner.client.batch_execute_statement(input).await
    }

    async fn execute_statement(
        &self,
        input: ExecuteStatementInput,
    ) -> Result<ExecuteStatementOutput, RusotoError<ExecuteStatementError>> {
        self.inner.client.execute_statement(input).await
    }

    async fn execute_transaction(
        &self,
        input: ExecuteTransactionInput,
    ) -> Result<ExecuteTransactionOutput, RusotoError<ExecuteTransactionError>> {
        self.inner.client.execute_transaction(input).await
    }

    async fn describe_kinesis_streaming_destination(
        &self,
        input: DescribeKinesisStreamingDestinationInput,
    ) -> Result<
        DescribeKinesisStreamingDestinationOutput,
        RusotoError<DescribeKinesisStreamingDestinationError>,
    > {
        self.inner
            .client
            .describe_kinesis_streaming_destination(input)
            .await
    }

    async fn enable_kinesis_streaming_destination(
        &self,
        input: KinesisStreamingDestinationInput,
    ) -> Result<
        KinesisStreamingDestinationOutput,
        RusotoError<EnableKinesisStreamingDestinationError>,
    > {
        self.inner
            .client
            .enable_kinesis_streaming_destination(input)
            .await
    }

    async fn disable_kinesis_streaming_destination(
        &self,
        input: KinesisStreamingDestinationInput,
    ) -> Result<
        KinesisStreamingDestinationOutput,
        RusotoError<DisableKinesisStreamingDestinationError>,
    > {
        self.inner
            .disable_kinesis_streaming_destination(input)
            .await
    }

    async fn export_table_to_point_in_time(
        &self,
        input: ExportTableToPointInTimeInput,
    ) -> Result<ExportTableToPointInTimeOutput, RusotoError<ExportTableToPointInTimeError>> {
        self.inner.client.export_table_to_point_in_time(input).await
    }
}

/// retry impl for Service error types
macro_rules! retry {
    ($e:ty, $($p: pat)+) => {
        impl Retry for $e {
            fn retryable(&self) -> bool {
                // we allow unreachable_patterns because
                // _ => false because in some cases
                // all variants are retryable
                // in other cases, only a subset, hence
                // this type matching
                #[allow(unreachable_patterns)]
                match self {
                   $($p)|+ => true,
                    _ => false
                }
            }
        }
    }
}

retry!(
    BatchGetItemError,
    BatchGetItemError::InternalServerError(_) BatchGetItemError::ProvisionedThroughputExceeded(_)
);

retry!(
    BatchWriteItemError,
    BatchWriteItemError::InternalServerError(_) BatchWriteItemError::ProvisionedThroughputExceeded(_)
);

retry!(
    CreateBackupError,
    CreateBackupError::InternalServerError(_) CreateBackupError::LimitExceeded(_)
);

retry!(
    CreateGlobalTableError,
    CreateGlobalTableError::InternalServerError(_) CreateGlobalTableError::LimitExceeded(_)
);

retry!(
    CreateTableError,
    CreateTableError::InternalServerError(_) CreateTableError::LimitExceeded(_)
);

retry!(
    DeleteBackupError,
    DeleteBackupError::InternalServerError(_) DeleteBackupError::LimitExceeded(_)
);

retry!(
    DeleteItemError,
    DeleteItemError::InternalServerError(_) DeleteItemError::ProvisionedThroughputExceeded(_)
);

retry!(
    DeleteTableError,
    DeleteTableError::InternalServerError(_) DeleteTableError::LimitExceeded(_)
);

retry!(
    DescribeBackupError,
    DescribeBackupError::InternalServerError(_)
);

retry!(
    DescribeContinuousBackupsError,
    DescribeContinuousBackupsError::InternalServerError(_)
);

retry!(
    DescribeGlobalTableError,
    DescribeGlobalTableError::InternalServerError(_)
);

retry!(
    DescribeGlobalTableSettingsError,
    DescribeGlobalTableSettingsError::InternalServerError(_)
);

retry!(
    DescribeLimitsError,
    DescribeLimitsError::InternalServerError(_)
);

retry!(
    DescribeTableError,
    DescribeTableError::InternalServerError(_)
);

retry!(
    GetItemError,
    GetItemError::InternalServerError(_) GetItemError::ProvisionedThroughputExceeded(_)
);

retry!(ListBackupsError, ListBackupsError::InternalServerError(_));

retry!(ListTablesError, ListTablesError::InternalServerError(_));

retry!(
    ListTagsOfResourceError,
    ListTagsOfResourceError::InternalServerError(_)
);

retry!(
    PutItemError,
    PutItemError::InternalServerError(_) PutItemError::ProvisionedThroughputExceeded(_)
);

retry!(
    QueryError,
    QueryError::InternalServerError(_) QueryError::ProvisionedThroughputExceeded(_)
);

retry!(
    RestoreTableFromBackupError,
    RestoreTableFromBackupError::InternalServerError(_)
);

retry!(
    RestoreTableToPointInTimeError,
    RestoreTableToPointInTimeError::InternalServerError(_)
);

retry!(
    ScanError,
    ScanError::InternalServerError(_) ScanError::ProvisionedThroughputExceeded(_)
);

retry!(
    TagResourceError,
    TagResourceError::InternalServerError(_) TagResourceError::LimitExceeded(_)
);

retry!(
    UntagResourceError,
    UntagResourceError::InternalServerError(_) UntagResourceError::LimitExceeded(_)
);

retry!(
    UpdateContinuousBackupsError,
    UpdateContinuousBackupsError::InternalServerError(_)
);

retry!(
    UpdateGlobalTableError,
    UpdateGlobalTableError::InternalServerError(_)
);

retry!(
    UpdateGlobalTableSettingsError,
    UpdateGlobalTableSettingsError::InternalServerError(_)
);

retry!(
    UpdateItemError,
    UpdateItemError::InternalServerError(_) UpdateItemError::ProvisionedThroughputExceeded(_)
);

retry!(
    UpdateTableError,
    UpdateTableError::InternalServerError(_) UpdateTableError::LimitExceeded(_)
);

retry!(
    UpdateTimeToLiveError,
    UpdateTimeToLiveError::InternalServerError(_) UpdateTimeToLiveError::LimitExceeded(_)
);

retry!(
    ListGlobalTablesError,
    ListGlobalTablesError::InternalServerError(_)
);

retry!(
    DescribeTimeToLiveError,
    DescribeTimeToLiveError::InternalServerError(_)
);

retry!(
    TransactGetItemsError,
    TransactGetItemsError::InternalServerError(_) TransactGetItemsError::ProvisionedThroughputExceeded(_)
);

retry!(
    TransactWriteItemsError,
    TransactWriteItemsError::InternalServerError(_) TransactWriteItemsError::ProvisionedThroughputExceeded(_)
);

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn policy_has_default() {
        assert_eq!(
            Policy::default(),
            Policy::Exponential(5, Duration::from_millis(100))
        );
    }

    #[test]
    fn policy_impl_into_for_retry_policy() {
        fn test(_: impl Into<RetryPolicy>) {}
        test(Policy::default())
    }
}
