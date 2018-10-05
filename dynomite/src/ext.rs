//!Eextention interfaces for rusoto `DynamoDb`

use std::collections::HashMap;

use futures::{stream, Future, Stream};
use rusoto_dynamodb::{AttributeValue, DynamoDb};
use rusoto_dynamodb::{
  BackupSummary, ListBackupsError, ListBackupsInput, ListTablesError, ListTablesInput, QueryError,
  QueryInput, ScanError, ScanInput,
};

/// Exention methods for DynamoDb implementations
pub trait DynamoDbExt {
  // see https://github.com/boto/botocore/blob/5250e2e7a3209eb995283ac018aea37d3bc1da45/botocore/data/dynamodb/2012-08-10/paginators-1.json

  /// A `Stream` oriented version of `list_backups`
  fn stream_list_backups(
    &'static self,
    input: ListBackupsInput,
  ) -> Box<Stream<Item = BackupSummary, Error = ListBackupsError>>;

  /// A `Stream` oriented version of `list_tables`
  fn stream_list_tables(
    &'static self,
    input: ListTablesInput,
  ) -> Box<Stream<Item = String, Error = ListTablesError>>;

  /// A `Stream` oriented version of `query`
  fn stream_query(
    &'static self,
    input: QueryInput,
  ) -> Box<Stream<Item = HashMap<String, AttributeValue>, Error = QueryError>>;

  /// A `Stream` oriented version of `scan`
  fn stream_scan(
    &'static self,
    input: ScanInput,
  ) -> Box<Stream<Item = HashMap<String, AttributeValue>, Error = ScanError>>;
}

impl<D> DynamoDbExt for D
where
  D: DynamoDb + Send + Sync + 'static,
{
  fn stream_list_backups(
    &'static self,
    input: ListBackupsInput,
  ) -> Box<Stream<Item = BackupSummary, Error = ListBackupsError>> {
    enum PageState {
      Start(Option<String>),
      Next(String),
      End,
    }
    Box::new(
      stream::unfold(PageState::Start(None), move |state| {
        let exclusive_start_backup_arn = match state {
          PageState::Start(start) => start,
          PageState::Next(next) => Some(next),
          PageState::End => return None,
        };
        Some(
          self
            .list_backups(ListBackupsInput {
              exclusive_start_backup_arn,
              ..input.clone()
            }).map(move |resp| {
              let next_state = match resp.last_evaluated_backup_arn {
                Some(next) => if next.is_empty() {
                  PageState::End
                } else {
                  PageState::Next(next)
                },
                _ => PageState::End,
              };
              (
                stream::iter_ok(resp.backup_summaries.unwrap_or_default()),
                next_state,
              )
            }),
        )
      }).flatten(),
    )
  }

  fn stream_list_tables(
    &'static self,
    input: ListTablesInput,
  ) -> Box<Stream<Item = String, Error = ListTablesError>> {
    enum PageState {
      Start(Option<String>),
      Next(String),
      End,
    }
    Box::new(
      stream::unfold(PageState::Start(None), move |state| {
        let exclusive_start_table_name = match state {
          PageState::Start(start) => start,
          PageState::Next(next) => Some(next),
          PageState::End => return None,
        };
        Some(
          self
            .list_tables(ListTablesInput {
              exclusive_start_table_name,
              ..input.clone()
            }).map(move |resp| {
              let next_state = match resp.last_evaluated_table_name {
                Some(next) => if next.is_empty() {
                  PageState::End
                } else {
                  PageState::Next(next)
                },
                _ => PageState::End,
              };
              (
                stream::iter_ok(resp.table_names.unwrap_or_default()),
                next_state,
              )
            }),
        )
      }).flatten(),
    )
  }

  fn stream_query(
    &'static self,
    input: QueryInput,
  ) -> Box<Stream<Item = HashMap<String, AttributeValue>, Error = QueryError>> {
    enum PageState {
      Start(Option<HashMap<String, AttributeValue>>),
      Next(HashMap<String, AttributeValue>),
      End,
    }
    Box::new(
      stream::unfold(PageState::Start(None), move |state| {
        let exclusive_start_key = match state {
          PageState::Start(start) => start,
          PageState::Next(next) => Some(next),
          PageState::End => return None,
        };
        Some(
          self
            .query(QueryInput {
              exclusive_start_key,
              ..input.clone()
            }).map(move |resp| {
              let next_state = match resp.last_evaluated_key {
                Some(next) => if next.is_empty() {
                  PageState::End
                } else {
                  PageState::Next(next)
                },
                _ => PageState::End,
              };
              (stream::iter_ok(resp.items.unwrap_or_default()), next_state)
            }),
        )
      }).flatten(),
    )
  }

  fn stream_scan(
    &'static self,
    input: ScanInput,
  ) -> Box<Stream<Item = HashMap<String, AttributeValue>, Error = ScanError>> {
    enum PageState {
      Start(Option<HashMap<String, AttributeValue>>),
      Next(HashMap<String, AttributeValue>),
      End,
    }
    Box::new(
      stream::unfold(PageState::Start(None), move |state| {
        let exclusive_start_key = match state {
          PageState::Start(start) => start,
          PageState::Next(next) => Some(next),
          PageState::End => return None,
        };
        Some(
          self
            .scan(ScanInput {
              exclusive_start_key,
              ..input.clone()
            }).map(move |resp| {
              let next_state = match resp.last_evaluated_key {
                Some(next) => if next.is_empty() {
                  PageState::End
                } else {
                  PageState::Next(next)
                },
                _ => PageState::End,
              };
              (stream::iter_ok(resp.items.unwrap_or_default()), next_state)
            }),
        )
      }).flatten(),
    )
  }
}
