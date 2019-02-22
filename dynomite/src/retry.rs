//! Retry functionality
//!
//! Specifcally this implementation focuses on honoring [these documented DynamoDB retryable errors](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/Programming.Errors.html#Programming.Errors.MessagesAndCodes)
//! on top AWS's general recommendations of for [retrying API requests](https://docs.aws.amazon.com/general/latest/gr/api-retries.html).
//!
//! # examples
//! ```rust,no_run
//!  use dynomite::{Retries, retry::Policy};
//!  use dynomite::dynamodb::{DynamoDb, DynamoDbClient};
//!
//!  # fn main() {
//!  let client =
//!     DynamoDbClient::new(Default::default())
//!         .with_retries(Policy::default());
//!
//!  // any client operation will now be retried when
//!  // appropriate
//!  let tables = client.list_tables(Default::default());
//!  # }
//! ```
//!
use crate::dynamodb::*;
use futures_backoff::{Condition, Strategy};
use log::debug;
use rusoto_core::{CredentialsError, HttpDispatchError, RusotoFuture};
use std::{sync::Arc, time::Duration};

/// Preconfigured retry policies for failable operations
///
/// A `Default` impl of retrying 5 times with an exponential backoff of 100 milliseconds
#[derive(Clone)]
pub enum Policy {
    /// Limited number of times to retry
    Limit(usize),
    /// Limited number of times to retry with fixed pause between retries
    Pause(usize, Duration),
    /// Limited number of times to retry with an expoential pause between retries
    Exponential(usize, Duration),
}

impl Default for Policy {
    fn default() -> Self {
        Policy::Exponential(5, Duration::from_millis(100))
    }
}

impl Into<Strategy> for Policy {
    fn into(self) -> Strategy {
        match self {
            Policy::Limit(times) => Strategy::default()
                .with_max_retries(times)
                .with_jitter(true),
            Policy::Pause(times, duration) => Strategy::fixed(duration)
                .with_max_retries(times)
                .with_jitter(true),
            Policy::Exponential(times, duration) => Strategy::exponential(duration)
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

impl<R> Condition<R> for Counter
where
    R: Retry,
{
    fn should_retry(
        &mut self,
        error: &R,
    ) -> bool {
        debug!("retrying operation {}", self.0);
        if let Some(value) = self.0.checked_add(1) {
            self.0 = value;
        }
        error.retryable()
    }
}

// wrapper so we only pay for one arc
struct Inner<D> {
    client: D,
    strategy: Strategy,
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
                strategy: policy.into(),
            }),
        }
    }

    /// Retry and operation based on this clients configured retry policy
    #[inline]
    fn retry<F, T, R>(
        &self,
        operation: F,
    ) -> RusotoFuture<T, R>
    where
        F: FnMut() -> RusotoFuture<T, R> + Send + 'static,
        R: Retry + From<CredentialsError> + From<HttpDispatchError>,
    {
        RusotoFuture::from_future(self.inner.strategy.retry_if(operation, Counter(0)))
    }
}

impl<D> DynamoDb for RetryingDynamoDb<D>
where
    D: DynamoDb + Sync + Send + 'static,
{
    fn batch_get_item(
        &self,
        input: BatchGetItemInput,
    ) -> RusotoFuture<BatchGetItemOutput, BatchGetItemError> {
        let inner = self.inner.clone();
        self.retry(move || inner.client.batch_get_item(input.clone()))
    }

    fn batch_write_item(
        &self,
        input: BatchWriteItemInput,
    ) -> RusotoFuture<BatchWriteItemOutput, BatchWriteItemError> {
        let inner = self.inner.clone();
        self.retry(move || inner.client.batch_write_item(input.clone()))
    }

    fn create_backup(
        &self,
        input: CreateBackupInput,
    ) -> RusotoFuture<CreateBackupOutput, CreateBackupError> {
        let inner = self.inner.clone();
        self.retry(move || inner.client.create_backup(input.clone()))
    }

    fn create_global_table(
        &self,
        input: CreateGlobalTableInput,
    ) -> RusotoFuture<CreateGlobalTableOutput, CreateGlobalTableError> {
        let inner = self.inner.clone();
        self.retry(move || inner.client.create_global_table(input.clone()))
    }

    fn create_table(
        &self,
        input: CreateTableInput,
    ) -> RusotoFuture<CreateTableOutput, CreateTableError> {
        let inner = self.inner.clone();
        self.retry(move || inner.client.create_table(input.clone()))
    }

    fn delete_backup(
        &self,
        input: DeleteBackupInput,
    ) -> RusotoFuture<DeleteBackupOutput, DeleteBackupError> {
        let inner = self.inner.clone();
        self.retry(move || inner.client.delete_backup(input.clone()))
    }

    fn delete_item(
        &self,
        input: DeleteItemInput,
    ) -> RusotoFuture<DeleteItemOutput, DeleteItemError> {
        let inner = self.inner.clone();
        self.retry(move || inner.client.delete_item(input.clone()))
    }

    fn delete_table(
        &self,
        input: DeleteTableInput,
    ) -> RusotoFuture<DeleteTableOutput, DeleteTableError> {
        let inner = self.inner.clone();
        self.retry(move || inner.client.delete_table(input.clone()))
    }

    fn describe_backup(
        &self,
        input: DescribeBackupInput,
    ) -> RusotoFuture<DescribeBackupOutput, DescribeBackupError> {
        let inner = self.inner.clone();
        self.retry(move || inner.client.describe_backup(input.clone()))
    }

    fn describe_continuous_backups(
        &self,
        input: DescribeContinuousBackupsInput,
    ) -> RusotoFuture<DescribeContinuousBackupsOutput, DescribeContinuousBackupsError> {
        let inner = self.inner.clone();
        self.retry(move || inner.client.describe_continuous_backups(input.clone()))
    }

    fn describe_global_table(
        &self,
        input: DescribeGlobalTableInput,
    ) -> RusotoFuture<DescribeGlobalTableOutput, DescribeGlobalTableError> {
        let inner = self.inner.clone();
        self.retry(move || inner.client.describe_global_table(input.clone()))
    }

    fn describe_global_table_settings(
        &self,
        input: DescribeGlobalTableSettingsInput,
    ) -> RusotoFuture<DescribeGlobalTableSettingsOutput, DescribeGlobalTableSettingsError> {
        let inner = self.inner.clone();
        self.retry(move || inner.client.describe_global_table_settings(input.clone()))
    }

    fn describe_limits(&self) -> RusotoFuture<DescribeLimitsOutput, DescribeLimitsError> {
        let inner = self.inner.clone();
        self.retry(move || inner.client.describe_limits())
    }

    fn describe_table(
        &self,
        input: DescribeTableInput,
    ) -> RusotoFuture<DescribeTableOutput, DescribeTableError> {
        let inner = self.inner.clone();
        self.retry(move || inner.client.describe_table(input.clone()))
    }

    fn describe_time_to_live(
        &self,
        input: DescribeTimeToLiveInput,
    ) -> RusotoFuture<DescribeTimeToLiveOutput, DescribeTimeToLiveError> {
        let inner = self.inner.clone();
        self.retry(move || inner.client.describe_time_to_live(input.clone()))
    }

    fn get_item(
        &self,
        input: GetItemInput,
    ) -> RusotoFuture<GetItemOutput, GetItemError> {
        let inner = self.inner.clone();
        self.retry(move || inner.client.get_item(input.clone()))
    }

    fn list_backups(
        &self,
        input: ListBackupsInput,
    ) -> RusotoFuture<ListBackupsOutput, ListBackupsError> {
        let inner = self.inner.clone();
        self.retry(move || inner.client.list_backups(input.clone()))
    }

    fn list_global_tables(
        &self,
        input: ListGlobalTablesInput,
    ) -> RusotoFuture<ListGlobalTablesOutput, ListGlobalTablesError> {
        let inner = self.inner.clone();
        self.retry(move || inner.client.list_global_tables(input.clone()))
    }

    fn list_tables(
        &self,
        input: ListTablesInput,
    ) -> RusotoFuture<ListTablesOutput, ListTablesError> {
        let inner = self.inner.clone();
        self.retry(move || inner.client.list_tables(input.clone()))
    }

    fn list_tags_of_resource(
        &self,
        input: ListTagsOfResourceInput,
    ) -> RusotoFuture<ListTagsOfResourceOutput, ListTagsOfResourceError> {
        let inner = self.inner.clone();
        self.retry(move || inner.client.list_tags_of_resource(input.clone()))
    }

    fn put_item(
        &self,
        input: PutItemInput,
    ) -> RusotoFuture<PutItemOutput, PutItemError> {
        let inner = self.inner.clone();
        self.retry(move || inner.client.put_item(input.clone()))
    }

    fn query(
        &self,
        input: QueryInput,
    ) -> RusotoFuture<QueryOutput, QueryError> {
        let inner = self.inner.clone();
        self.retry(move || inner.client.query(input.clone()))
    }

    fn restore_table_from_backup(
        &self,
        input: RestoreTableFromBackupInput,
    ) -> RusotoFuture<RestoreTableFromBackupOutput, RestoreTableFromBackupError> {
        let inner = self.inner.clone();
        self.retry(move || inner.client.restore_table_from_backup(input.clone()))
    }

    fn restore_table_to_point_in_time(
        &self,
        input: RestoreTableToPointInTimeInput,
    ) -> RusotoFuture<RestoreTableToPointInTimeOutput, RestoreTableToPointInTimeError> {
        let inner = self.inner.clone();
        self.retry(move || inner.client.restore_table_to_point_in_time(input.clone()))
    }

    fn scan(
        &self,
        input: ScanInput,
    ) -> RusotoFuture<ScanOutput, ScanError> {
        let inner = self.inner.clone();
        self.retry(move || inner.client.scan(input.clone()))
    }

    fn tag_resource(
        &self,
        input: TagResourceInput,
    ) -> RusotoFuture<(), TagResourceError> {
        let inner = self.inner.clone();
        self.retry(move || inner.client.tag_resource(input.clone()))
    }

    fn untag_resource(
        &self,
        input: UntagResourceInput,
    ) -> RusotoFuture<(), UntagResourceError> {
        let inner = self.inner.clone();
        self.retry(move || inner.client.untag_resource(input.clone()))
    }

    fn update_continuous_backups(
        &self,
        input: UpdateContinuousBackupsInput,
    ) -> RusotoFuture<UpdateContinuousBackupsOutput, UpdateContinuousBackupsError> {
        let inner = self.inner.clone();
        self.retry(move || inner.client.update_continuous_backups(input.clone()))
    }

    fn update_global_table(
        &self,
        input: UpdateGlobalTableInput,
    ) -> RusotoFuture<UpdateGlobalTableOutput, UpdateGlobalTableError> {
        let inner = self.inner.clone();
        self.retry(move || inner.client.update_global_table(input.clone()))
    }

    fn update_global_table_settings(
        &self,
        input: UpdateGlobalTableSettingsInput,
    ) -> RusotoFuture<UpdateGlobalTableSettingsOutput, UpdateGlobalTableSettingsError> {
        let inner = self.inner.clone();
        self.retry(move || inner.client.update_global_table_settings(input.clone()))
    }

    fn update_item(
        &self,
        input: UpdateItemInput,
    ) -> RusotoFuture<UpdateItemOutput, UpdateItemError> {
        let inner = self.inner.clone();
        self.retry(move || inner.client.update_item(input.clone()))
    }

    fn update_table(
        &self,
        input: UpdateTableInput,
    ) -> RusotoFuture<UpdateTableOutput, UpdateTableError> {
        let inner = self.inner.clone();
        self.retry(move || inner.client.update_table(input.clone()))
    }

    fn update_time_to_live(
        &self,
        input: UpdateTimeToLiveInput,
    ) -> RusotoFuture<UpdateTimeToLiveOutput, UpdateTimeToLiveError> {
        let inner = self.inner.clone();
        self.retry(move || inner.client.update_time_to_live(input.clone()))
    }
}

macro_rules! retry {
    ($e:ty, $($p: pat)+) => {
        impl Retry for $e {
            fn retryable(&self) -> bool {
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
