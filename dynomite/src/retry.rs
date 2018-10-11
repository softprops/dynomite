use std::time::Duration;

use dynamodb::*;
use futures_retry::{FutureRetry, RetryPolicy};
use rusoto_core::RusotoFuture;

trait Retry {
    fn retryable(&self) -> bool;
}

// todo: imply Retry for all error types

impl Retry for BatchGetItemError {
    fn retryable(&self) -> bool {
        match self {
            BatchGetItemError::ProvisionedThroughputExceeded(_) => true,
            _ => false,
        }
    }
}

/// Retry policies
#[derive(Clone)]
pub enum Policy {
    /// Limited number of times to retry
    Limit(usize),
    /// Limited number of times to retry with pause between retries
    Pause(usize, Duration),
}

/// A type which implements `DynamoDb` and retries all operations
/// that are retryable
pub struct RetryingDynamoDb<D> {
    inner: D,
    policy: Policy,
}

impl<D> RetryingDynamoDb<D>
where
    D: DynamoDb + 'static,
{
    /// Return a new instance with a configured retry policy
    pub fn new(
        inner: D,
        policy: Policy,
    ) -> Self {
        Self { inner, policy }
    }

    // https://gitlab.com/mexus/futures-retry/blob/0.2.1/examples/tcp-client-complex.rs
    // todo something like the above
    fn handle<R>(
        &self,
        err: R,
    ) -> impl FnMut(R) -> RetryPolicy<R>
    where
        R: Retry,
    {
        let policy = self.policy.clone();
        let mut attempts_left = match policy {
            Policy::Limit(times) => times,
            Policy::Pause(times, _) => times,
        };
        move |e| {
            if attempts_left == 1 || !err.retryable() {
                RetryPolicy::ForwardError(e)
            } else {
                attempts_left += 1;
                match policy {
                    Policy::Limit(_) => RetryPolicy::Repeat,
                    Policy::Pause(_, duration) => RetryPolicy::WaitRetry(duration),
                }
            }
        }
    }
}

// todo: in order to return RusotoFuture we'd need this (unrelated) https://github.com/rusoto/rusoto/blob/acb3c851474d1c2bd113171e93b930d59d2153ed/rusoto/core/src/future.rs#L215-L229
impl<D> DynamoDb for RetryingDynamoDb<D>
where
    D: DynamoDb,
{
    /// <p>The <code>BatchGetItem</code> operation returns the attributes of one or more items from one or more tables. You identify requested items by primary key.</p> <p>A single operation can retrieve up to 16 MB of data, which can contain as many as 100 items. <code>BatchGetItem</code> will return a partial result if the response size limit is exceeded, the table's provisioned throughput is exceeded, or an internal processing failure occurs. If a partial result is returned, the operation returns a value for <code>UnprocessedKeys</code>. You can use this value to retry the operation starting with the next item to get.</p> <important> <p>If you request more than 100 items <code>BatchGetItem</code> will return a <code>ValidationException</code> with the message "Too many items requested for the BatchGetItem call".</p> </important> <p>For example, if you ask to retrieve 100 items, but each individual item is 300 KB in size, the system returns 52 items (so as not to exceed the 16 MB limit). It also returns an appropriate <code>UnprocessedKeys</code> value so you can get the next page of results. If desired, your application can include its own logic to assemble the pages of results into one data set.</p> <p>If <i>none</i> of the items can be processed due to insufficient provisioned throughput on all of the tables in the request, then <code>BatchGetItem</code> will return a <code>ProvisionedThroughputExceededException</code>. If <i>at least one</i> of the items is successfully processed, then <code>BatchGetItem</code> completes successfully, while returning the keys of the unread items in <code>UnprocessedKeys</code>.</p> <important> <p>If DynamoDB returns any unprocessed items, you should retry the batch operation on those items. However, <i>we strongly recommend that you use an exponential backoff algorithm</i>. If you retry the batch operation immediately, the underlying read or write requests can still fail due to throttling on the individual tables. If you delay the batch operation using exponential backoff, the individual requests in the batch are much more likely to succeed.</p> <p>For more information, see <a href="http://docs.aws.amazon.com/amazondynamodb/latest/developerguide/ErrorHandling.html#BatchOperations">Batch Operations and Error Handling</a> in the <i>Amazon DynamoDB Developer Guide</i>.</p> </important> <p>By default, <code>BatchGetItem</code> performs eventually consistent reads on every table in the request. If you want strongly consistent reads instead, you can set <code>ConsistentRead</code> to <code>true</code> for any or all tables.</p> <p>In order to minimize response latency, <code>BatchGetItem</code> retrieves items in parallel.</p> <p>When designing your application, keep in mind that DynamoDB does not return items in any particular order. To help parse the response by item, include the primary key values for the items in your request in the <code>ProjectionExpression</code> parameter.</p> <p>If a requested item does not exist, it is not returned in the result. Requests for nonexistent items consume the minimum read capacity units according to the type of read. For more information, see <a href="http://docs.aws.amazon.com/amazondynamodb/latest/developerguide/WorkingWithTables.html#CapacityUnitCalculations">Capacity Units Calculations</a> in the <i>Amazon DynamoDB Developer Guide</i>.</p>
    fn batch_get_item(
        &self,
        input: BatchGetItemInput,
    ) -> RusotoFuture<BatchGetItemOutput, BatchGetItemError> {
        self.inner.batch_get_item(input)
    }

    /// <p><p>The <code>BatchWriteItem</code> operation puts or deletes multiple items in one or more tables. A single call to <code>BatchWriteItem</code> can write up to 16 MB of data, which can comprise as many as 25 put or delete requests. Individual items to be written can be as large as 400 KB.</p> <note> <p> <code>BatchWriteItem</code> cannot update items. To update items, use the <code>UpdateItem</code> action.</p> </note> <p>The individual <code>PutItem</code> and <code>DeleteItem</code> operations specified in <code>BatchWriteItem</code> are atomic; however <code>BatchWriteItem</code> as a whole is not. If any requested operations fail because the table&#39;s provisioned throughput is exceeded or an internal processing failure occurs, the failed operations are returned in the <code>UnprocessedItems</code> response parameter. You can investigate and optionally resend the requests. Typically, you would call <code>BatchWriteItem</code> in a loop. Each iteration would check for unprocessed items and submit a new <code>BatchWriteItem</code> request with those unprocessed items until all items have been processed.</p> <p>Note that if <i>none</i> of the items can be processed due to insufficient provisioned throughput on all of the tables in the request, then <code>BatchWriteItem</code> will return a <code>ProvisionedThroughputExceededException</code>.</p> <important> <p>If DynamoDB returns any unprocessed items, you should retry the batch operation on those items. However, <i>we strongly recommend that you use an exponential backoff algorithm</i>. If you retry the batch operation immediately, the underlying read or write requests can still fail due to throttling on the individual tables. If you delay the batch operation using exponential backoff, the individual requests in the batch are much more likely to succeed.</p> <p>For more information, see <a href="http://docs.aws.amazon.com/amazondynamodb/latest/developerguide/ErrorHandling.html#BatchOperations">Batch Operations and Error Handling</a> in the <i>Amazon DynamoDB Developer Guide</i>.</p> </important> <p>With <code>BatchWriteItem</code>, you can efficiently write or delete large amounts of data, such as from Amazon Elastic MapReduce (EMR), or copy data from another database into DynamoDB. In order to improve performance with these large-scale operations, <code>BatchWriteItem</code> does not behave in the same way as individual <code>PutItem</code> and <code>DeleteItem</code> calls would. For example, you cannot specify conditions on individual put and delete requests, and <code>BatchWriteItem</code> does not return deleted items in the response.</p> <p>If you use a programming language that supports concurrency, you can use threads to write items in parallel. Your application must include the necessary logic to manage the threads. With languages that don&#39;t support threading, you must update or delete the specified items one at a time. In both situations, <code>BatchWriteItem</code> performs the specified put and delete operations in parallel, giving you the power of the thread pool approach without having to introduce complexity into your application.</p> <p>Parallel processing reduces latency, but each specified put and delete request consumes the same number of write capacity units whether it is processed in parallel or not. Delete operations on nonexistent items consume one write capacity unit.</p> <p>If one or more of the following is true, DynamoDB rejects the entire batch write operation:</p> <ul> <li> <p>One or more tables specified in the <code>BatchWriteItem</code> request does not exist.</p> </li> <li> <p>Primary key attributes specified on an item in the request do not match those in the corresponding table&#39;s primary key schema.</p> </li> <li> <p>You try to perform multiple operations on the same item in the same <code>BatchWriteItem</code> request. For example, you cannot put and delete the same item in the same <code>BatchWriteItem</code> request. </p> </li> <li> <p> Your request contains at least two items with identical hash and range keys (which essentially is two put operations). </p> </li> <li> <p>There are more than 25 requests in the batch.</p> </li> <li> <p>Any individual item in a batch exceeds 400 KB.</p> </li> <li> <p>The total request size exceeds 16 MB.</p> </li> </ul></p>
    fn batch_write_item(
        &self,
        input: BatchWriteItemInput,
    ) -> RusotoFuture<BatchWriteItemOutput, BatchWriteItemError> {
        self.inner.batch_write_item(input)
    }

    /// <p><p>Creates a backup for an existing table.</p> <p> Each time you create an On-Demand Backup, the entire table data is backed up. There is no limit to the number of on-demand backups that can be taken. </p> <p> When you create an On-Demand Backup, a time marker of the request is cataloged, and the backup is created asynchronously, by applying all changes until the time of the request to the last full table snapshot. Backup requests are processed instantaneously and become available for restore within minutes. </p> <p>You can call <code>CreateBackup</code> at a maximum rate of 50 times per second.</p> <p>All backups in DynamoDB work without consuming any provisioned throughput on the table.</p> <p> If you submit a backup request on 2018-12-14 at 14:25:00, the backup is guaranteed to contain all data committed to the table up to 14:24:00, and data committed after 14:26:00 will not be. The backup may or may not contain data modifications made between 14:24:00 and 14:26:00. On-Demand Backup does not support causal consistency. </p> <p> Along with data, the following are also included on the backups: </p> <ul> <li> <p>Global secondary indexes (GSIs)</p> </li> <li> <p>Local secondary indexes (LSIs)</p> </li> <li> <p>Streams</p> </li> <li> <p>Provisioned read and write capacity</p> </li> </ul></p>
    fn create_backup(
        &self,
        input: CreateBackupInput,
    ) -> RusotoFuture<CreateBackupOutput, CreateBackupError> {
        self.inner.create_backup(input)
    }

    /// <p><p>Creates a global table from an existing table. A global table creates a replication relationship between two or more DynamoDB tables with the same table name in the provided regions. </p> <p>If you want to add a new replica table to a global table, each of the following conditions must be true:</p> <ul> <li> <p>The table must have the same primary key as all of the other replicas.</p> </li> <li> <p>The table must have the same name as all of the other replicas.</p> </li> <li> <p>The table must have DynamoDB Streams enabled, with the stream containing both the new and the old images of the item.</p> </li> <li> <p>None of the replica tables in the global table can contain any data.</p> </li> </ul> <p> If global secondary indexes are specified, then the following conditions must also be met: </p> <ul> <li> <p> The global secondary indexes must have the same name. </p> </li> <li> <p> The global secondary indexes must have the same hash key and sort key (if present). </p> </li> </ul> <important> <p> Write capacity settings should be set consistently across your replica tables and secondary indexes. DynamoDB strongly recommends enabling auto scaling to manage the write capacity settings for all of your global tables replicas and indexes. </p> <p> If you prefer to manage write capacity settings manually, you should provision equal replicated write capacity units to your replica tables. You should also provision equal replicated write capacity units to matching secondary indexes across your global table. </p> </important></p>
    fn create_global_table(
        &self,
        input: CreateGlobalTableInput,
    ) -> RusotoFuture<CreateGlobalTableOutput, CreateGlobalTableError> {
        self.inner.create_global_table(input)
    }

    /// <p>The <code>CreateTable</code> operation adds a new table to your account. In an AWS account, table names must be unique within each region. That is, you can have two tables with same name if you create the tables in different regions.</p> <p> <code>CreateTable</code> is an asynchronous operation. Upon receiving a <code>CreateTable</code> request, DynamoDB immediately returns a response with a <code>TableStatus</code> of <code>CREATING</code>. After the table is created, DynamoDB sets the <code>TableStatus</code> to <code>ACTIVE</code>. You can perform read and write operations only on an <code>ACTIVE</code> table. </p> <p>You can optionally define secondary indexes on the new table, as part of the <code>CreateTable</code> operation. If you want to create multiple tables with secondary indexes on them, you must create the tables sequentially. Only one table with secondary indexes can be in the <code>CREATING</code> state at any given time.</p> <p>You can use the <code>DescribeTable</code> action to check the table status.</p>
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

    /// <p>Deletes a single item in a table by primary key. You can perform a conditional delete operation that deletes the item if it exists, or if it has an expected attribute value.</p> <p>In addition to deleting an item, you can also return the item's attribute values in the same operation, using the <code>ReturnValues</code> parameter.</p> <p>Unless you specify conditions, the <code>DeleteItem</code> is an idempotent operation; running it multiple times on the same item or attribute does <i>not</i> result in an error response.</p> <p>Conditional deletes are useful for deleting items only if specific conditions are met. If those conditions are met, DynamoDB performs the delete. Otherwise, the item is not deleted.</p>
    fn delete_item(
        &self,
        input: DeleteItemInput,
    ) -> RusotoFuture<DeleteItemOutput, DeleteItemError> {
        self.inner.delete_item(input)
    }

    /// <p>The <code>DeleteTable</code> operation deletes a table and all of its items. After a <code>DeleteTable</code> request, the specified table is in the <code>DELETING</code> state until DynamoDB completes the deletion. If the table is in the <code>ACTIVE</code> state, you can delete it. If a table is in <code>CREATING</code> or <code>UPDATING</code> states, then DynamoDB returns a <code>ResourceInUseException</code>. If the specified table does not exist, DynamoDB returns a <code>ResourceNotFoundException</code>. If table is already in the <code>DELETING</code> state, no error is returned. </p> <note> <p>DynamoDB might continue to accept data read and write operations, such as <code>GetItem</code> and <code>PutItem</code>, on a table in the <code>DELETING</code> state until the table deletion is complete.</p> </note> <p>When you delete a table, any indexes on that table are also deleted.</p> <p>If you have DynamoDB Streams enabled on the table, then the corresponding stream on that table goes into the <code>DISABLED</code> state, and the stream is automatically deleted after 24 hours.</p> <p>Use the <code>DescribeTable</code> action to check the status of the table. </p>
    fn delete_table(
        &self,
        input: DeleteTableInput,
    ) -> RusotoFuture<DeleteTableOutput, DeleteTableError> {
        self.inner.delete_table(input)
    }

    /// <p>Describes an existing backup of a table.</p> <p>You can call <code>DescribeBackup</code> at a maximum rate of 10 times per second.</p>
    fn describe_backup(
        &self,
        input: DescribeBackupInput,
    ) -> RusotoFuture<DescribeBackupOutput, DescribeBackupError> {
        self.inner.describe_backup(input)
    }

    /// <p>Checks the status of continuous backups and point in time recovery on the specified table. Continuous backups are <code>ENABLED</code> on all tables at table creation. If point in time recovery is enabled, <code>PointInTimeRecoveryStatus</code> will be set to ENABLED.</p> <p> Once continuous backups and point in time recovery are enabled, you can restore to any point in time within <code>EarliestRestorableDateTime</code> and <code>LatestRestorableDateTime</code>. </p> <p> <code>LatestRestorableDateTime</code> is typically 5 minutes before the current time. You can restore your table to any point in time during the last 35 days. </p> <p>You can call <code>DescribeContinuousBackups</code> at a maximum rate of 10 times per second.</p>
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

    /*fn describe_global_table_settings(
      &self,
      input: DescribeGlobalTableSettingsInput,
    ) -> RusotoFuture<DescribeGlobalTableSettingsOutput, DescribeGlobalTableSettingsError> {
      self.inner.describe_global_table_settings(input)
    }*/

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

    /*fn update_global_table_settings(
      &self,
      input: UpdateGlobalTableSettingsInput,
    ) -> RusotoFuture<UpdateGlobalTableSettingsOutput, UpdateGlobalTableSettingsError> {
      self.inner.update_global_table_settings(input)
    }*/

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
