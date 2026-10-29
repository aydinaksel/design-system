use std::sync::Arc;

use leptos::prelude::*;

use super::cell_value::{Align, CellValue};

/// A single table column. `value` is the typed value used for sorting and
/// searching (written once, generically, inside `DataTable`); `render` is the
/// actual cell view and stays fully custom. Both closures are `Send + Sync` so
/// the column can live in default reactive storage under multi-threaded SSR.
#[expect(
    clippy::struct_excessive_bools,
    reason = "the flags are independent column capabilities, not states of one machine"
)]
pub struct Column<Row: Send + Sync + 'static> {
    pub key: &'static str,
    pub header: &'static str,
    pub align: Align,
    pub sortable: bool,
    pub searchable: bool,
    pub filterable: bool,
    pub editable: bool,
    pub value: Arc<dyn Fn(&Row) -> CellValue + Send + Sync>,
    pub render: Arc<dyn Fn(&Row) -> AnyView + Send + Sync>,
}

impl<Row: Send + Sync + 'static> Clone for Column<Row> {
    fn clone(&self) -> Self {
        Self {
            key: self.key,
            header: self.header,
            align: self.align,
            sortable: self.sortable,
            searchable: self.searchable,
            filterable: self.filterable,
            editable: self.editable,
            value: Arc::clone(&self.value),
            render: Arc::clone(&self.render),
        }
    }
}

impl<Row: Send + Sync + 'static> Column<Row> {
    /// Plain text cell. `value` and `render` both derive from the accessor.
    pub fn text(
        key: &'static str,
        header: &'static str,
        accessor: impl Fn(&Row) -> String + Send + Sync + Clone + 'static,
    ) -> Self {
        let value_accessor = accessor.clone();
        Self {
            key,
            header,
            align: Align::Left,
            sortable: false,
            searchable: false,
            filterable: false,
            editable: false,
            value: Arc::new(move |row| CellValue::Text(value_accessor(row))),
            render: Arc::new(move |row| accessor(row).into_any()),
        }
    }

    /// Money cell: right-aligned, formatted via `format_money`. `None` is an
    /// empty cell and sorts last.
    pub fn integer(
        key: &'static str,
        header: &'static str,
        accessor: impl Fn(&Row) -> Option<i32> + Send + Sync + Clone + 'static,
    ) -> Self {
        let value_accessor = accessor.clone();
        Self {
            key,
            header,
            align: Align::Right,
            sortable: false,
            searchable: false,
            filterable: false,
            editable: false,
            value: Arc::new(move |row| match value_accessor(row) {
                Some(number) => CellValue::Int(i64::from(number)),
                None => CellValue::Empty,
            }),
            render: Arc::new(move |row| {
                crate::formatting::format_integer(accessor(row)).into_any()
            }),
        }
    }

    pub fn quantity(
        key: &'static str,
        header: &'static str,
        accessor: impl Fn(&Row) -> Option<f64> + Send + Sync + Clone + 'static,
    ) -> Self {
        let value_accessor = accessor.clone();
        Self {
            key,
            header,
            align: Align::Right,
            sortable: false,
            searchable: false,
            filterable: false,
            editable: false,
            value: Arc::new(move |row| match value_accessor(row) {
                Some(quantity) => CellValue::Number(quantity),
                None => CellValue::Empty,
            }),
            render: Arc::new(move |row| {
                accessor(row)
                    .map(crate::formatting::format_quantity)
                    .unwrap_or_default()
                    .into_any()
            }),
        }
    }

    /// Quantity cell followed by its unit in muted text, so a number is never
    /// read without the unit it counts.
    pub fn quantity_in_unit(
        key: &'static str,
        header: &'static str,
        accessor: impl Fn(&Row) -> Option<f64> + Send + Sync + Clone + 'static,
        unit: Signal<String>,
    ) -> Self {
        let render_accessor = accessor.clone();
        Self {
            render: Arc::new(move |row| {
                render_accessor(row).map_or_else(
                    || ().into_any(),
                    |quantity| {
                        view! {
                            {crate::formatting::format_quantity(quantity)}
                            <span class="ml-1 text-gray-500">{move || unit.get()}</span>
                        }
                        .into_any()
                    },
                )
            }),
            ..Self::quantity(key, header, accessor)
        }
    }

    pub fn money(
        key: &'static str,
        header: &'static str,
        accessor: impl Fn(&Row) -> Option<f64> + Send + Sync + Clone + 'static,
    ) -> Self {
        let value_accessor = accessor.clone();
        Self {
            key,
            header,
            align: Align::Right,
            sortable: false,
            searchable: false,
            filterable: false,
            editable: false,
            value: Arc::new(move |row| match value_accessor(row) {
                Some(amount) => CellValue::Number(amount),
                None => CellValue::Empty,
            }),
            render: Arc::new(move |row| crate::formatting::format_money(accessor(row)).into_any()),
        }
    }

    pub fn money_decimal(
        key: &'static str,
        header: &'static str,
        accessor: impl Fn(&Row) -> Option<rust_decimal::Decimal> + Send + Sync + Clone + 'static,
    ) -> Self {
        let value_accessor = accessor.clone();
        Self {
            key,
            header,
            align: Align::Right,
            sortable: false,
            searchable: false,
            filterable: false,
            editable: false,
            value: Arc::new(move |row| {
                use rust_decimal::prelude::ToPrimitive;
                match value_accessor(row).and_then(|amount| amount.to_f64()) {
                    Some(amount) => CellValue::Number(amount),
                    None => CellValue::Empty,
                }
            }),
            render: Arc::new(move |row| {
                crate::formatting::format_money_decimal(accessor(row)).into_any()
            }),
        }
    }

    /// A real anchor, so middle-click and open-in-new-tab work even when the
    /// whole row also navigates via `row_href`.
    pub fn link(
        key: &'static str,
        header: &'static str,
        text: impl Fn(&Row) -> String + Send + Sync + Clone + 'static,
        href: impl Fn(&Row) -> String + Send + Sync + Clone + 'static,
    ) -> Self {
        let value_accessor = text.clone();
        Self {
            key,
            header,
            align: Align::Left,
            sortable: false,
            searchable: false,
            filterable: false,
            editable: false,
            value: Arc::new(move |row| CellValue::Text(value_accessor(row))),
            render: Arc::new(move |row| {
                let target = href(row);
                let label = text(row);
                view! {
                    <a href=target class="text-blue-600 hover:text-blue-800 hover:underline">
                        {label}
                    </a>
                }
                .into_any()
            }),
        }
    }

    /// Fully custom cell view. Starts non-sortable/non-searchable with an empty
    /// sort value; attach one with `with_value` to enable sorting/searching.
    pub fn custom(
        key: &'static str,
        header: &'static str,
        render: impl Fn(&Row) -> AnyView + Send + Sync + 'static,
    ) -> Self {
        Self {
            key,
            header,
            align: Align::Left,
            sortable: false,
            searchable: false,
            filterable: false,
            editable: false,
            value: Arc::new(|_| CellValue::Empty),
            render: Arc::new(render),
        }
    }

    #[must_use]
    pub fn with_value(mut self, value: impl Fn(&Row) -> CellValue + Send + Sync + 'static) -> Self {
        self.value = Arc::new(value);
        self
    }

    #[must_use]
    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    #[must_use]
    pub fn sortable(mut self) -> Self {
        self.sortable = true;
        self
    }

    #[must_use]
    pub fn searchable(mut self) -> Self {
        self.searchable = true;
        self
    }

    #[must_use]
    pub fn filterable(mut self) -> Self {
        self.filterable = true;
        self
    }

    #[must_use]
    pub fn editable(mut self) -> Self {
        self.editable = true;
        self
    }
}
