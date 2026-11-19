use std::sync::Arc;

use leptos::prelude::{RwSignal, Signal};
use table_query::LoadedFilterValues;

use super::data_table::RowView;
use super::url_query::UrlTableQuery;

#[derive(Default)]
pub enum Layout<Row> {
    #[default]
    Table,
    Cards(RowView<Row>),
}

impl<Row> Clone for Layout<Row> {
    fn clone(&self) -> Self {
        match self {
            Self::Table => Self::Table,
            Self::Cards(card_for_row) => Self::Cards(Arc::clone(card_for_row)),
        }
    }
}

#[derive(Default)]
pub enum ServerLayout<Row> {
    #[default]
    Table,
    Cards(RowView<Row>),
    /// Places each row on a month grid by `date_column_key`, a column whose
    /// value is `YYYY-MM-DD` text.
    Calendar {
        date_column_key: &'static str,
    },
}

impl<Row> ServerLayout<Row> {
    pub(crate) fn calendar_date_column_key(&self) -> Option<&'static str> {
        match self {
            Self::Calendar { date_column_key } => Some(date_column_key),
            Self::Table | Self::Cards(_) => None,
        }
    }

    pub(crate) fn row_layout(self) -> Layout<Row> {
        match self {
            Self::Cards(card_for_row) => Layout::Cards(card_for_row),
            Self::Table | Self::Calendar { .. } => Layout::Table,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct ServerFiltering {
    pub(crate) requested_column: RwSignal<Option<String>>,
    pub(crate) values: Signal<LoadedFilterValues>,
}

#[derive(Clone, Copy)]
pub struct ServerRows {
    pub(crate) query: UrlTableQuery,
    pub(crate) total_rows: Signal<Option<u64>>,
    pub(crate) filtering: Option<ServerFiltering>,
    pub(crate) calendar_date_column_key: Option<&'static str>,
}
