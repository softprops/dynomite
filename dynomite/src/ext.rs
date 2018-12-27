//! Extention interfaces for rusoto `DynamoDb`

use crate::dynamodb::{
    AttributeValue, BackupSummary, DynamoDb, ListBackupsError, ListBackupsInput, ListTablesError,
    ListTablesInput, QueryError, QueryInput, ScanError, ScanInput,
};
use futures::{stream, Future, Stream};
use std::{collections::HashMap, sync::Arc};

type DynomiteStream<I, E> = Box<Stream<Item = I, Error = E> + Send>;

/// Extension methods for DynamoDb implementations
///
/// A default impl is provided for `Arc` instances of `DynamoDb` which adds automatinting `Stream` interfaces that require
/// taking ownership. In practice clone your `Arc`'d client when calling the stream methods.
///
pub trait DynamoDbExt {
    // see https://github.com/boto/botocore/blob/5250e2e7a3209eb995283ac018aea37d3bc1da45/botocore/data/dynamodb/2012-08-10/paginators-1.json

    /// An auto-paginating `Stream` oriented version of `list_backups`
    fn list_backups_pages(
        self,
        input: ListBackupsInput,
    ) -> DynomiteStream<BackupSummary, ListBackupsError>;

    /// An auto-paginating `Stream` oriented version of `list_tables`
    fn list_tables_pages(
        self,
        input: ListTablesInput,
    ) -> DynomiteStream<String, ListTablesError>;

    /// An auto-paginating `Stream` oriented version of `query`
    fn query_pages(
        self,
        input: QueryInput,
    ) -> DynomiteStream<HashMap<String, AttributeValue>, QueryError>;

    /// An auto-paginating `Stream` oriented version of `scan`
    fn scan_pages(
        self,
        input: ScanInput,
    ) -> DynomiteStream<HashMap<String, AttributeValue>, ScanError>;
}

impl<D> DynamoDbExt for Arc<D>
where
    D: DynamoDb + Send + Sync + 'static,
{
    fn list_backups_pages(
        self,
        input: ListBackupsInput,
    ) -> DynomiteStream<BackupSummary, ListBackupsError> {
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
                    self.clone()
                        .list_backups(ListBackupsInput {
                            exclusive_start_backup_arn,
                            ..input.clone()
                        })
                        .map(move |resp| {
                            let next_state = match resp.last_evaluated_backup_arn {
                                Some(next) => {
                                    if next.is_empty() {
                                        PageState::End
                                    } else {
                                        PageState::Next(next)
                                    }
                                }
                                _ => PageState::End,
                            };
                            (
                                stream::iter_ok(resp.backup_summaries.unwrap_or_default()),
                                next_state,
                            )
                        }),
                )
            })
            .flatten(),
        )
    }

    fn list_tables_pages(
        self,
        input: ListTablesInput,
    ) -> DynomiteStream<String, ListTablesError> {
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
                    self.list_tables(ListTablesInput {
                        exclusive_start_table_name,
                        ..input.clone()
                    })
                    .map(move |resp| {
                        let next_state = match resp.last_evaluated_table_name {
                            Some(next) => {
                                if next.is_empty() {
                                    PageState::End
                                } else {
                                    PageState::Next(next)
                                }
                            }
                            _ => PageState::End,
                        };
                        (
                            stream::iter_ok(resp.table_names.unwrap_or_default()),
                            next_state,
                        )
                    }),
                )
            })
            .flatten(),
        )
    }

    fn query_pages(
        self,
        input: QueryInput,
    ) -> DynomiteStream<HashMap<String, AttributeValue>, QueryError> {
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
                    self.query(QueryInput {
                        exclusive_start_key,
                        ..input.clone()
                    })
                    .map(move |resp| {
                        let next_state = match resp.last_evaluated_key {
                            Some(next) => {
                                if next.is_empty() {
                                    PageState::End
                                } else {
                                    PageState::Next(next)
                                }
                            }
                            _ => PageState::End,
                        };
                        (stream::iter_ok(resp.items.unwrap_or_default()), next_state)
                    }),
                )
            })
            .flatten(),
        )
    }

    fn scan_pages(
        self,
        input: ScanInput,
    ) -> DynomiteStream<HashMap<String, AttributeValue>, ScanError> {
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
                    self.clone()
                        .scan(ScanInput {
                            exclusive_start_key,
                            ..input.clone()
                        })
                        .map(move |resp| {
                            let next_state = match resp.last_evaluated_key {
                                Some(next) => {
                                    if next.is_empty() {
                                        PageState::End
                                    } else {
                                        PageState::Next(next)
                                    }
                                }
                                _ => PageState::End,
                            };
                            (stream::iter_ok(resp.items.unwrap_or_default()), next_state)
                        }),
                )
            })
            .flatten(),
        )
    }
}
