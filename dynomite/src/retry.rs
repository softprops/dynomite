//! Adds retry functionality to dynamodb operations to fullfil [AWS robustness](https://docs.aws.amazon.com/general/latest/gr/api-retries.html)
//! recommendations
//!
//! Specficically this implementation focus on honoring [these documented retryable errors](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/Programming.Errors.html#Programming.Errors.MessagesAndCodes)

use crate::dynamodb::*;
use futures_backoff::Strategy;
use rusoto_core::{CredentialsError, HttpDispatchError, RusotoFuture};
use std::{sync::Arc, time::Duration};

/// Preconfigured retry policies for failable operations
#[derive(Clone)]
pub enum Policy {
    /// Limited number of times to retry
    Limit(usize),
    /// Limited number of times to retry with fixed pause between retries
    Pause(usize, Duration),
    /// Limited number of times to retry with an expoential pause between retries
    Exponential(usize, Duration),
}

/// A type which implements `DynamoDb` and retries all operations
/// that are retryable
#[derive(Clone)]
pub struct RetryingDynamoDb<D> {
    inner: Arc<D>,
    strategy: Arc<Strategy>,
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

trait Retry {
    fn retryable(&self) -> bool;
}

// todo macro_rules! for generating these

impl Retry for BatchGetItemError {
    fn retryable(&self) -> bool {
        match self {
            BatchGetItemError::InternalServerError(_)
            | BatchGetItemError::ProvisionedThroughputExceeded(_) => true,
            _ => false,
        }
    }
}

impl Retry for BatchWriteItemError {
    fn retryable(&self) -> bool {
        match self {
            BatchWriteItemError::InternalServerError(_)
            | BatchWriteItemError::ProvisionedThroughputExceeded(_) => true,
            _ => false,
        }
    }
}

impl Retry for CreateBackupError {
    fn retryable(&self) -> bool {
        match self {
            CreateBackupError::InternalServerError(_) | CreateBackupError::LimitExceeded(_) => true,
            _ => false,
        }
    }
}

impl Retry for CreateGlobalTableError {
    fn retryable(&self) -> bool {
        match self {
            CreateGlobalTableError::InternalServerError(_)
            | CreateGlobalTableError::LimitExceeded(_) => true,
            _ => false,
        }
    }
}

impl Retry for CreateTableError {
    fn retryable(&self) -> bool {
        match self {
            CreateTableError::InternalServerError(_) | CreateTableError::LimitExceeded(_) => true,
            _ => false,
        }
    }
}

impl Retry for DeleteBackupError {
    fn retryable(&self) -> bool {
        match self {
            DeleteBackupError::InternalServerError(_) | DeleteBackupError::LimitExceeded(_) => true,
            _ => false,
        }
    }
}

impl Retry for DeleteItemError {
    fn retryable(&self) -> bool {
        match self {
            DeleteItemError::InternalServerError(_)
            | DeleteItemError::ProvisionedThroughputExceeded(_) => true,
            _ => false,
        }
    }
}

impl Retry for DeleteTableError {
    fn retryable(&self) -> bool {
        match self {
            DeleteTableError::InternalServerError(_) | DeleteTableError::LimitExceeded(_) => true,
            _ => false,
        }
    }
}

impl Retry for DescribeBackupError {
    fn retryable(&self) -> bool {
        match self {
            DescribeBackupError::InternalServerError(_) => true,
            _ => false,
        }
    }
}

impl Retry for DescribeContinuousBackupsError {
    fn retryable(&self) -> bool {
        match self {
            DescribeContinuousBackupsError::InternalServerError(_) => true,
            _ => false,
        }
    }
}

impl Retry for DescribeGlobalTableError {
    fn retryable(&self) -> bool {
        match self {
            DescribeGlobalTableError::InternalServerError(_) => true,
            _ => false,
        }
    }
}

impl Retry for DescribeGlobalTableSettingsError {
    fn retryable(&self) -> bool {
        match self {
            DescribeGlobalTableSettingsError::InternalServerError(_) => true,
            _ => false,
        }
    }
}

impl Retry for DescribeLimitsError {
    fn retryable(&self) -> bool {
        match self {
            DescribeLimitsError::InternalServerError(_) => true,
            _ => false,
        }
    }
}

impl Retry for DescribeTableError {
    fn retryable(&self) -> bool {
        match self {
            DescribeTableError::InternalServerError(_) => true,
            _ => false,
        }
    }
}

impl Retry for GetItemError {
    fn retryable(&self) -> bool {
        match self {
            GetItemError::InternalServerError(_)
            | GetItemError::ProvisionedThroughputExceeded(_) => true,
            _ => false,
        }
    }
}

impl Retry for ListBackupsError {
    fn retryable(&self) -> bool {
        match self {
            ListBackupsError::InternalServerError(_) => true,
            _ => false,
        }
    }
}

impl Retry for ListTablesError {
    fn retryable(&self) -> bool {
        match self {
            ListTablesError::InternalServerError(_) => true,
            _ => false,
        }
    }
}

impl Retry for ListTagsOfResourceError {
    fn retryable(&self) -> bool {
        match self {
            ListTagsOfResourceError::InternalServerError(_) => true,
            _ => false,
        }
    }
}

impl Retry for PutItemError {
    fn retryable(&self) -> bool {
        match self {
            PutItemError::InternalServerError(_)
            | PutItemError::ProvisionedThroughputExceeded(_) => true,
            _ => false,
        }
    }
}

impl Retry for QueryError {
    fn retryable(&self) -> bool {
        match self {
            QueryError::InternalServerError(_) | QueryError::ProvisionedThroughputExceeded(_) => {
                true
            }
            _ => false,
        }
    }
}

impl<D> RetryingDynamoDb<D>
where
    D: DynamoDb + 'static,
{
    /// Return a new instance with a configured retry policy
    pub fn new(
        inner: Arc<D>,
        policy: Policy,
    ) -> Self {
        Self {
            inner,
            strategy: Arc::new(policy.into()),
        }
    }

    /// Retry and operation based on this clients configured retry policy
    fn retry<F, T, R>(
        &self,
        operation: F,
    ) -> RusotoFuture<T, R>
    where
        F: FnMut() -> RusotoFuture<T, R> + Send + 'static,
        R: Retry + From<CredentialsError> + From<HttpDispatchError>,
    {
        RusotoFuture::from_future(self.strategy.retry_if(operation, |err: &R| err.retryable()))
    }
}

impl<D> DynamoDb for RetryingDynamoDb<D>
where
    D: DynamoDb + Clone + Sync + Send + 'static,
{
    fn batch_get_item(
        &self,
        input: BatchGetItemInput,
    ) -> RusotoFuture<BatchGetItemOutput, BatchGetItemError> {
        let inner = self.inner.clone();
        self.retry(move || inner.batch_get_item(input.clone()))
    }

    fn batch_write_item(
        &self,
        input: BatchWriteItemInput,
    ) -> RusotoFuture<BatchWriteItemOutput, BatchWriteItemError> {
        let inner = self.inner.clone();
        self.retry(move || inner.batch_write_item(input.clone()))
    }

    fn create_backup(
        &self,
        input: CreateBackupInput,
    ) -> RusotoFuture<CreateBackupOutput, CreateBackupError> {
        let inner = self.inner.clone();
        self.retry(move || inner.create_backup(input.clone()))
    }

    fn create_global_table(
        &self,
        input: CreateGlobalTableInput,
    ) -> RusotoFuture<CreateGlobalTableOutput, CreateGlobalTableError> {
        let inner = self.inner.clone();
        self.retry(move || inner.create_global_table(input.clone()))
    }

    fn create_table(
        &self,
        input: CreateTableInput,
    ) -> RusotoFuture<CreateTableOutput, CreateTableError> {
        let inner = self.inner.clone();
        self.retry(move || inner.create_table(input.clone()))
    }

    /// <p>Deletes an existing backup of a table.</p> <p>You can call <code>DeleteBackup</code> at a maximum rate of 10 times per second.</p>
    fn delete_backup(
        &self,
        input: DeleteBackupInput,
    ) -> RusotoFuture<DeleteBackupOutput, DeleteBackupError> {
        let inner = self.inner.clone();
        self.retry(move || inner.delete_backup(input.clone()))
    }

    fn delete_item(
        &self,
        input: DeleteItemInput,
    ) -> RusotoFuture<DeleteItemOutput, DeleteItemError> {
        let inner = self.inner.clone();
        self.retry(move || inner.delete_item(input.clone()))
    }

    fn delete_table(
        &self,
        input: DeleteTableInput,
    ) -> RusotoFuture<DeleteTableOutput, DeleteTableError> {
        let inner = self.inner.clone();
        self.retry(move || inner.delete_table(input.clone()))
    }

    fn describe_backup(
        &self,
        input: DescribeBackupInput,
    ) -> RusotoFuture<DescribeBackupOutput, DescribeBackupError> {
        let inner = self.inner.clone();
        self.retry(move || inner.describe_backup(input.clone()))
    }

    fn describe_continuous_backups(
        &self,
        input: DescribeContinuousBackupsInput,
    ) -> RusotoFuture<DescribeContinuousBackupsOutput, DescribeContinuousBackupsError> {
        let inner = self.inner.clone();
        self.retry(move || inner.describe_continuous_backups(input.clone()))
    }

    /// <p>Returns information about the specified global table.</p>
    fn describe_global_table(
        &self,
        input: DescribeGlobalTableInput,
    ) -> RusotoFuture<DescribeGlobalTableOutput, DescribeGlobalTableError> {
        let inner = self.inner.clone();
        self.retry(move || inner.describe_global_table(input.clone()))
    }

    fn describe_global_table_settings(
        &self,
        input: DescribeGlobalTableSettingsInput,
    ) -> RusotoFuture<DescribeGlobalTableSettingsOutput, DescribeGlobalTableSettingsError> {
        let inner = self.inner.clone();
        self.retry(move || inner.describe_global_table_settings(input.clone()))
    }

    fn describe_limits(&self) -> RusotoFuture<DescribeLimitsOutput, DescribeLimitsError> {
        let inner = self.inner.clone();
        self.retry(move || inner.describe_limits())
    }

    fn describe_table(
        &self,
        input: DescribeTableInput,
    ) -> RusotoFuture<DescribeTableOutput, DescribeTableError> {
        let inner = self.inner.clone();
        self.retry(move || inner.describe_table(input.clone()))
    }

    fn describe_time_to_live(
        &self,
        input: DescribeTimeToLiveInput,
    ) -> RusotoFuture<DescribeTimeToLiveOutput, DescribeTimeToLiveError> {
        let inner = self.inner.clone();
        inner.describe_time_to_live(input.clone())
        //self.retry(move || inner.describe_time_to_live(input.clone()))
    }

    fn get_item(
        &self,
        input: GetItemInput,
    ) -> RusotoFuture<GetItemOutput, GetItemError> {
        let inner = self.inner.clone();
        self.retry(move || inner.get_item(input.clone()))
    }

    fn list_backups(
        &self,
        input: ListBackupsInput,
    ) -> RusotoFuture<ListBackupsOutput, ListBackupsError> {
        let inner = self.inner.clone();
        self.retry(move || inner.list_backups(input.clone()))
    }

    fn list_global_tables(
        &self,
        input: ListGlobalTablesInput,
    ) -> RusotoFuture<ListGlobalTablesOutput, ListGlobalTablesError> {
        let inner = self.inner.clone();
        inner.list_global_tables(input.clone())
        //self.retry(move || inner.list_global_tables(input.clone()))
    }

    fn list_tables(
        &self,
        input: ListTablesInput,
    ) -> RusotoFuture<ListTablesOutput, ListTablesError> {
        let inner = self.inner.clone();
        self.retry(move || inner.list_tables(input.clone()))
    }

    fn list_tags_of_resource(
        &self,
        input: ListTagsOfResourceInput,
    ) -> RusotoFuture<ListTagsOfResourceOutput, ListTagsOfResourceError> {
        let inner = self.inner.clone();
        self.retry(move || inner.list_tags_of_resource(input.clone()))
    }

    fn put_item(
        &self,
        input: PutItemInput,
    ) -> RusotoFuture<PutItemOutput, PutItemError> {
        let inner = self.inner.clone();
        self.retry(move || inner.put_item(input.clone()))
    }

    fn query(
        &self,
        input: QueryInput,
    ) -> RusotoFuture<QueryOutput, QueryError> {
        let inner = self.inner.clone();
        self.retry(move || inner.query(input.clone()))
    }

    fn restore_table_from_backup(
        &self,
        input: RestoreTableFromBackupInput,
    ) -> RusotoFuture<RestoreTableFromBackupOutput, RestoreTableFromBackupError> {
        let inner = self.inner.clone();
        //self.retry(move || inner.restore_table_from_backup(input.clone()))
        inner.restore_table_from_backup(input.clone())
    }

    fn restore_table_to_point_in_time(
        &self,
        input: RestoreTableToPointInTimeInput,
    ) -> RusotoFuture<RestoreTableToPointInTimeOutput, RestoreTableToPointInTimeError> {
        let inner = self.inner.clone();
        // self.retry(move || inner.restore_table_to_point_in_time(input.clone()))
        inner.restore_table_to_point_in_time(input.clone())
    }

    fn scan(
        &self,
        input: ScanInput,
    ) -> RusotoFuture<ScanOutput, ScanError> {
        let inner = self.inner.clone();
        inner.scan(input.clone())
        // self.retry(move || inner.scan(input.clone()))
    }

    fn tag_resource(
        &self,
        input: TagResourceInput,
    ) -> RusotoFuture<(), TagResourceError> {
        let inner = self.inner.clone();
        inner.tag_resource(input.clone())
        // self.retry(move || inner.tag_resource(input.clone()))
    }

    fn untag_resource(
        &self,
        input: UntagResourceInput,
    ) -> RusotoFuture<(), UntagResourceError> {
        let inner = self.inner.clone();
        inner.untag_resource(input.clone())
        // self.retry(move || inner.untag_resource(input.clone()))
    }

    fn update_continuous_backups(
        &self,
        input: UpdateContinuousBackupsInput,
    ) -> RusotoFuture<UpdateContinuousBackupsOutput, UpdateContinuousBackupsError> {
        let inner = self.inner.clone();
        inner.update_continuous_backups(input.clone())
        // self.retry(move || inner.update_continuous_backups(input.clone()))
    }

    fn update_global_table(
        &self,
        input: UpdateGlobalTableInput,
    ) -> RusotoFuture<UpdateGlobalTableOutput, UpdateGlobalTableError> {
        let inner = self.inner.clone();
        inner.update_global_table(input.clone())
        // self.retry(move || inner.update_global_table(input.clone()))
    }

    fn update_global_table_settings(
        &self,
        input: UpdateGlobalTableSettingsInput,
    ) -> RusotoFuture<UpdateGlobalTableSettingsOutput, UpdateGlobalTableSettingsError> {
        let inner = self.inner.clone();
        inner.update_global_table_settings(input.clone())
        // self.retry(move || inner.update_global_table_settings(input.clone()))
    }

    fn update_item(
        &self,
        input: UpdateItemInput,
    ) -> RusotoFuture<UpdateItemOutput, UpdateItemError> {
        let inner = self.inner.clone();
        inner.update_item(input.clone())
        // self.retry(move || inner.update_item(input.clone()))
    }

    fn update_table(
        &self,
        input: UpdateTableInput,
    ) -> RusotoFuture<UpdateTableOutput, UpdateTableError> {
        let inner = self.inner.clone();
        inner.update_table(input.clone())
        // self.retry(move || inner.update_table(input.clone()))
    }

    fn update_time_to_live(
        &self,
        input: UpdateTimeToLiveInput,
    ) -> RusotoFuture<UpdateTimeToLiveOutput, UpdateTimeToLiveError> {
        let inner = self.inner.clone();
        inner.update_time_to_live(input.clone())
        // self.retry(move || inner.update_time_to_live(input.clone()))
    }
}
