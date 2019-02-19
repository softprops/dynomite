//! Adds retry functionality to dynamodb operations to fullfil [AWS robustness](https://docs.aws.amazon.com/general/latest/gr/api-retries.html)
//! recommendations

use crate::dynamodb::*;
use futures_backoff::Strategy;
use rusoto_core::RusotoFuture;
use std::{sync::Arc, time::Duration};

/// Preconfigured Retry policies
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

impl Retry for BatchGetItemError {
    fn retryable(&self) -> bool {
        match self {
            BatchGetItemError::ProvisionedThroughputExceeded(_) => true,
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

    fn retry<F, T, R>(
        &self,
        operation: F,
    ) -> RusotoFuture<T, R>
    where
        F: FnMut() -> RusotoFuture<T, R> + Send + 'static,
        R: Retry + From<rusoto_core::CredentialsError> + From<rusoto_core::HttpDispatchError>,
    {
        RusotoFuture::from_future(
            self.strategy
                .retry_if(operation, |err: &R| err.retryable()),
        )
    }
}

// todo: in order to return RusotoFuture we'd need this (unrelated) https://github.com/rusoto/rusoto/blob/acb3c851474d1c2bd113171e93b930d59d2153ed/rusoto/core/src/future.rs#L215-L229
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
        self.inner.batch_write_item(input)
    }

    fn create_backup(
        &self,
        input: CreateBackupInput,
    ) -> RusotoFuture<CreateBackupOutput, CreateBackupError> {
        self.inner.create_backup(input)
    }

    fn create_global_table(
        &self,
        input: CreateGlobalTableInput,
    ) -> RusotoFuture<CreateGlobalTableOutput, CreateGlobalTableError> {
        self.inner.create_global_table(input)
    }

    fn create_table(
        &self,
        input: CreateTableInput,
    ) -> RusotoFuture<CreateTableOutput, CreateTableError> {
        self.inner.create_table(input)
    }

    /// <p>Deletes an existing backup of a table.</p> <p>You can call <code>DeleteBackup</code> at a maximum rate of 10 times per second.</p>
    fn delete_backup(
        &self,
        input: DeleteBackupInput,
    ) -> RusotoFuture<DeleteBackupOutput, DeleteBackupError> {
        self.inner.delete_backup(input)
    }

    fn delete_item(
        &self,
        input: DeleteItemInput,
    ) -> RusotoFuture<DeleteItemOutput, DeleteItemError> {
        self.inner.delete_item(input)
    }

    fn delete_table(
        &self,
        input: DeleteTableInput,
    ) -> RusotoFuture<DeleteTableOutput, DeleteTableError> {
        self.inner.delete_table(input)
    }

    fn describe_backup(
        &self,
        input: DescribeBackupInput,
    ) -> RusotoFuture<DescribeBackupOutput, DescribeBackupError> {
        self.inner.describe_backup(input)
    }

    fn describe_continuous_backups(
        &self,
        input: DescribeContinuousBackupsInput,
    ) -> RusotoFuture<DescribeContinuousBackupsOutput, DescribeContinuousBackupsError> {
        self.inner.describe_continuous_backups(input)
    }

    /// <p>Returns information about the specified global table.</p>
    fn describe_global_table(
        &self,
        input: DescribeGlobalTableInput,
    ) -> RusotoFuture<DescribeGlobalTableOutput, DescribeGlobalTableError> {
        self.inner.describe_global_table(input)
    }

    fn describe_global_table_settings(
        &self,
        input: DescribeGlobalTableSettingsInput,
    ) -> RusotoFuture<DescribeGlobalTableSettingsOutput, DescribeGlobalTableSettingsError> {
        self.inner.describe_global_table_settings(input)
    }

    fn describe_limits(&self) -> RusotoFuture<DescribeLimitsOutput, DescribeLimitsError> {
        self.inner.describe_limits()
    }

    fn describe_table(
        &self,
        input: DescribeTableInput,
    ) -> RusotoFuture<DescribeTableOutput, DescribeTableError> {
        self.inner.describe_table(input)
    }

    fn describe_time_to_live(
        &self,
        input: DescribeTimeToLiveInput,
    ) -> RusotoFuture<DescribeTimeToLiveOutput, DescribeTimeToLiveError> {
        self.inner.describe_time_to_live(input)
    }

    fn get_item(
        &self,
        input: GetItemInput,
    ) -> RusotoFuture<GetItemOutput, GetItemError> {
        self.inner.get_item(input)
    }

    fn list_backups(
        &self,
        input: ListBackupsInput,
    ) -> RusotoFuture<ListBackupsOutput, ListBackupsError> {
        self.inner.list_backups(input)
    }

    fn list_global_tables(
        &self,
        input: ListGlobalTablesInput,
    ) -> RusotoFuture<ListGlobalTablesOutput, ListGlobalTablesError> {
        self.inner.list_global_tables(input)
    }

    fn list_tables(
        &self,
        input: ListTablesInput,
    ) -> RusotoFuture<ListTablesOutput, ListTablesError> {
        self.inner.list_tables(input)
    }

    fn list_tags_of_resource(
        &self,
        input: ListTagsOfResourceInput,
    ) -> RusotoFuture<ListTagsOfResourceOutput, ListTagsOfResourceError> {
        self.inner.list_tags_of_resource(input)
    }

    fn put_item(
        &self,
        input: PutItemInput,
    ) -> RusotoFuture<PutItemOutput, PutItemError> {
        self.inner.put_item(input)
    }

    fn query(
        &self,
        input: QueryInput,
    ) -> RusotoFuture<QueryOutput, QueryError> {
        self.inner.query(input)
    }

    fn restore_table_from_backup(
        &self,
        input: RestoreTableFromBackupInput,
    ) -> RusotoFuture<RestoreTableFromBackupOutput, RestoreTableFromBackupError> {
        self.inner.restore_table_from_backup(input)
    }

    fn restore_table_to_point_in_time(
        &self,
        input: RestoreTableToPointInTimeInput,
    ) -> RusotoFuture<RestoreTableToPointInTimeOutput, RestoreTableToPointInTimeError> {
        self.inner.restore_table_to_point_in_time(input)
    }

    fn scan(
        &self,
        input: ScanInput,
    ) -> RusotoFuture<ScanOutput, ScanError> {
        self.inner.scan(input)
    }

    fn tag_resource(
        &self,
        input: TagResourceInput,
    ) -> RusotoFuture<(), TagResourceError> {
        self.inner.tag_resource(input)
    }

    fn untag_resource(
        &self,
        input: UntagResourceInput,
    ) -> RusotoFuture<(), UntagResourceError> {
        self.inner.untag_resource(input)
    }

    fn update_continuous_backups(
        &self,
        input: UpdateContinuousBackupsInput,
    ) -> RusotoFuture<UpdateContinuousBackupsOutput, UpdateContinuousBackupsError> {
        self.inner.update_continuous_backups(input)
    }

    fn update_global_table(
        &self,
        input: UpdateGlobalTableInput,
    ) -> RusotoFuture<UpdateGlobalTableOutput, UpdateGlobalTableError> {
        self.inner.update_global_table(input)
    }

    fn update_global_table_settings(
        &self,
        input: UpdateGlobalTableSettingsInput,
    ) -> RusotoFuture<UpdateGlobalTableSettingsOutput, UpdateGlobalTableSettingsError> {
        self.inner.update_global_table_settings(input)
    }

    fn update_item(
        &self,
        input: UpdateItemInput,
    ) -> RusotoFuture<UpdateItemOutput, UpdateItemError> {
        self.inner.update_item(input)
    }

    fn update_table(
        &self,
        input: UpdateTableInput,
    ) -> RusotoFuture<UpdateTableOutput, UpdateTableError> {
        self.inner.update_table(input)
    }

    fn update_time_to_live(
        &self,
        input: UpdateTimeToLiveInput,
    ) -> RusotoFuture<UpdateTimeToLiveOutput, UpdateTimeToLiveError> {
        self.inner.update_time_to_live(input)
    }
}
