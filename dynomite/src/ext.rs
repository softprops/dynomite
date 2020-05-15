//! Extention interfaces for rusoto `DynamoDb`

use crate::dynamodb::{
    AttributeValue, BackupSummary, DynamoDb, ListBackupsError, ListBackupsInput, ListTablesError,
    ListTablesInput, QueryError, QueryInput, ScanError, ScanInput,
};
use futures::{stream, Stream, TryStreamExt};
#[cfg(feature = "default")]
use rusoto_core_default::RusotoError;
#[cfg(feature = "rustls")]
use rusoto_core_rustls::RusotoError;
use std::{collections::HashMap, pin::Pin};

type DynomiteStream<I, E> = Pin<Box<dyn Stream<Item = Result<I, RusotoError<E>>> + Send>>;

/// Extension methods for DynamoDb client types
///
/// A default impl is provided for `DynamoDb  Clone + Send + Sync + 'static` which adds autopaginating `Stream` interfaces that require
/// taking ownership.
pub trait DynamoDbExt {
    // see https://github.com/boto/botocore/blob/6906e8e7e8701c80f0b270c42be509cff4375e38/botocore/data/dynamodb/2012-08-10/paginators-1.json

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

impl<D> DynamoDbExt for D
where
    D: DynamoDb + Clone + Send + Sync + 'static,
{
    fn list_backups_pages(
        self,
        input: ListBackupsInput,
    ) -> DynomiteStream<BackupSummary, ListBackupsError> {
        enum PageState {
            Next(Option<String>, ListBackupsInput),
            End,
        }
        Box::pin(
            stream::try_unfold(
                PageState::Next(input.exclusive_start_backup_arn.clone(), input),
                move |state| {
                    let clone = self.clone();
                    async move {
                        let (exclusive_start_backup_arn, input) = match state {
                            PageState::Next(start, input) => (start, input),
                            PageState::End => {
                                return Ok(None) as Result<_, RusotoError<ListBackupsError>>
                            }
                        };
                        let resp = clone
                            .list_backups(ListBackupsInput {
                                exclusive_start_backup_arn,
                                ..input.clone()
                            })
                            .await?;
                        let next_state = match resp
                            .last_evaluated_backup_arn
                            .filter(|next| !next.is_empty())
                        {
                            Some(next) => PageState::Next(Some(next), input),
                            _ => PageState::End,
                        };
                        Ok(Some((
                            stream::iter(
                                resp.backup_summaries
                                    .unwrap_or_default()
                                    .into_iter()
                                    .map(Ok),
                            ),
                            next_state,
                        )))
                    }
                },
            )
            .try_flatten(),
        )
    }

    fn list_tables_pages(
        self,
        input: ListTablesInput,
    ) -> DynomiteStream<String, ListTablesError> {
        enum PageState {
            Next(Option<String>, ListTablesInput),
            End,
        }
        Box::pin(
            stream::try_unfold(
                PageState::Next(input.exclusive_start_table_name.clone(), input),
                move |state| {
                    let clone = self.clone();
                    async move {
                        let (exclusive_start_table_name, input) = match state {
                            PageState::Next(start, input) => (start, input),
                            PageState::End => {
                                return Ok(None) as Result<_, RusotoError<ListTablesError>>
                            }
                        };
                        let resp = clone
                            .list_tables(ListTablesInput {
                                exclusive_start_table_name,
                                ..input.clone()
                            })
                            .await?;
                        let next_state = match resp
                            .last_evaluated_table_name
                            .filter(|next| !next.is_empty())
                        {
                            Some(next) => PageState::Next(Some(next), input),
                            _ => PageState::End,
                        };
                        Ok(Some((
                            stream::iter(resp.table_names.unwrap_or_default().into_iter().map(Ok)),
                            next_state,
                        )))
                    }
                },
            )
            .try_flatten(),
        )
    }

    fn query_pages(
        self,
        input: QueryInput,
    ) -> DynomiteStream<HashMap<String, AttributeValue>, QueryError> {
        #[allow(clippy::large_enum_variant)]
        enum PageState {
            Next(Option<HashMap<String, AttributeValue>>, QueryInput),
            End,
        }
        Box::pin(
            stream::try_unfold(
                PageState::Next(input.exclusive_start_key.clone(), input),
                move |state| {
                    let clone = self.clone();
                    async move {
                        let (exclusive_start_key, input) = match state {
                            PageState::Next(start, input) => (start, input),
                            PageState::End => {
                                return Ok(None) as Result<_, RusotoError<QueryError>>
                            }
                        };
                        let resp = clone
                            .query(QueryInput {
                                exclusive_start_key,
                                ..input.clone()
                            })
                            .await?;
                        let next_state =
                            match resp.last_evaluated_key.filter(|next| !next.is_empty()) {
                                Some(next) => PageState::Next(Some(next), input),
                                _ => PageState::End,
                            };
                        Ok(Some((
                            stream::iter(resp.items.unwrap_or_default().into_iter().map(Ok)),
                            next_state,
                        )))
                    }
                },
            )
            .try_flatten(),
        )
    }

    fn scan_pages(
        self,
        input: ScanInput,
    ) -> DynomiteStream<HashMap<String, AttributeValue>, ScanError> {
        #[allow(clippy::large_enum_variant)]
        enum PageState {
            Next(Option<HashMap<String, AttributeValue>>, ScanInput),
            End,
        }
        Box::pin(
            stream::try_unfold(
                PageState::Next(input.exclusive_start_key.clone(), input),
                move |state| {
                    let clone = self.clone();
                    async move {
                        let (exclusive_start_key, input) = match state {
                            PageState::Next(start, input) => (start, input),
                            PageState::End => return Ok(None) as Result<_, RusotoError<ScanError>>,
                        };
                        let resp = clone
                            .scan(ScanInput {
                                exclusive_start_key,
                                ..input.clone()
                            })
                            .await?;
                        let next_state =
                            match resp.last_evaluated_key.filter(|next| !next.is_empty()) {
                                Some(next) => PageState::Next(Some(next), input),
                                _ => PageState::End,
                            };
                        Ok(Some((
                            stream::iter(resp.items.unwrap_or_default().into_iter().map(Ok)),
                            next_state,
                        )))
                    }
                },
            )
            .try_flatten(),
        )
    }
}
