use std::sync::Arc;

use super::data_table::RowView;

/// How a table draws its rows. Every layout keeps the toolbar, tabs and filters.
#[derive(Default)]
pub enum Layout<Row> {
    #[default]
    Table,
    Cards(RowView<Row>),
    /// Places each row on a month grid by `date_column_key`, a column whose
    /// value is `YYYY-MM-DD` text. Needs a server query, so use it through
    /// `ServerTable`.
    Calendar { date_column_key: &'static str },
}

impl<Row> Layout<Row> {
    pub(crate) fn is_calendar(&self) -> bool {
        matches!(self, Self::Calendar { .. })
    }
}

impl<Row> Clone for Layout<Row> {
    fn clone(&self) -> Self {
        match self {
            Self::Table => Self::Table,
            Self::Cards(card_for_row) => Self::Cards(Arc::clone(card_for_row)),
            Self::Calendar { date_column_key } => Self::Calendar { date_column_key },
        }
    }
}
