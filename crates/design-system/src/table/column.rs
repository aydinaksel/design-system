use std::sync::Arc;

use leptos::prelude::*;

use super::cell_value::{Align, CellValue};
use crate::badge::badge;
use crate::currency::Currency;
use crate::field_values::missing_value;
use crate::tone::Tone;

const TEXT_WIDTH_CHARACTERS: u16 = 16;
const INTEGER_WIDTH_CHARACTERS: u16 = 8;
const QUANTITY_WIDTH_CHARACTERS: u16 = 10;
const QUANTITY_IN_UNIT_WIDTH_CHARACTERS: u16 = 13;
const MONEY_WIDTH_CHARACTERS: u16 = 12;
const CUSTOM_WIDTH_CHARACTERS: u16 = 12;
const ACTIONS_WIDTH_CHARACTERS: u16 = 4;
const BADGE_WIDTH_CHARACTERS: u16 = 12;
const ACTIONS_KEY: &str = "actions";
const HEADER_ICONS_WIDTH_CHARACTERS: u16 = 5;
const UPPERCASE_LETTER_WIDTH_FIFTHS: u16 = 6;
const FIFTHS_PER_CHARACTER: u16 = 5;

/// A single table column. `value` is the typed value used for sorting and
/// searching (written once, generically, inside `DataTable`); `render` is the
/// actual cell view and stays fully custom. Both closures are `Send + Sync` so
/// the column can live in default reactive storage under multi-threaded SSR.
pub struct Column<Row: Send + Sync + 'static> {
    pub key: &'static str,
    pub header: &'static str,
    pub align: Align,
    /// The column's share of the table, in `ch` of the cell text. It is the
    /// narrowest the column gets; spare room is shared out in proportion.
    pub width_characters: u16,
    pub sortable: bool,
    pub searchable: bool,
    pub filterable: bool,
    pub value: Arc<dyn Fn(&Row) -> CellValue + Send + Sync>,
    pub render: Arc<dyn Fn(&Row) -> AnyView + Send + Sync>,
}

impl<Row: Send + Sync + 'static> Clone for Column<Row> {
    fn clone(&self) -> Self {
        Self {
            key: self.key,
            header: self.header,
            align: self.align,
            width_characters: self.width_characters,
            sortable: self.sortable,
            searchable: self.searchable,
            filterable: self.filterable,
            value: Arc::clone(&self.value),
            render: Arc::clone(&self.render),
        }
    }
}

impl<Row: Send + Sync + 'static> Column<Row> {
    /// Plain text cell, shown in full on hover when the column cuts it short.
    /// Missing or blank text shows a dash and sorts last, so the accessor can
    /// return a `String` or an `Option<String>`.
    pub fn text<Text>(
        key: &'static str,
        header: &'static str,
        accessor: impl Fn(&Row) -> Text + Send + Sync + Clone + 'static,
    ) -> Self
    where
        Text: Into<Option<String>>,
    {
        let present = move |row: &Row| {
            let text: Option<String> = accessor(row).into();
            text.filter(|text| !text.trim().is_empty())
        };
        let value_accessor = present.clone();
        Self {
            key,
            header,
            align: Align::Left,
            width_characters: TEXT_WIDTH_CHARACTERS,
            sortable: false,
            searchable: false,
            filterable: false,
            value: Arc::new(move |row| CellValue::optional_text(value_accessor(row).as_ref())),
            render: Arc::new(move |row| match present(row) {
                Some(text) => {
                    let full_text = text.clone();
                    view! { <span title=full_text>{text}</span> }.into_any()
                }
                None => missing_value(),
            }),
        }
    }

    /// A badge naming the row's state in its tone, such as a status. The
    /// label is what the column sorts, searches and filters by.
    pub fn badge<Label>(
        key: &'static str,
        header: &'static str,
        accessor: impl Fn(&Row) -> (Label, Tone) + Send + Sync + Clone + 'static,
    ) -> Self
    where
        Label: Into<String>,
    {
        let value_accessor = accessor.clone();
        Self {
            key,
            header,
            align: Align::Left,
            width_characters: BADGE_WIDTH_CHARACTERS,
            sortable: false,
            searchable: false,
            filterable: false,
            value: Arc::new(move |row| CellValue::Text(value_accessor(row).0.into())),
            render: Arc::new(move |row| {
                let (label, tone) = accessor(row);
                badge(label, tone)
            }),
        }
    }

    /// Integer cell: right-aligned with thousands separators. `None` is an
    /// empty cell and sorts last.
    pub fn integer<Number>(
        key: &'static str,
        header: &'static str,
        accessor: impl Fn(&Row) -> Number + Send + Sync + Clone + 'static,
    ) -> Self
    where
        Number: Into<Option<i64>>,
    {
        let number = move |row: &Row| -> Option<i64> { accessor(row).into() };
        let value_accessor = number.clone();
        Self {
            key,
            header,
            align: Align::Right,
            width_characters: INTEGER_WIDTH_CHARACTERS,
            sortable: false,
            searchable: false,
            filterable: false,
            value: Arc::new(move |row| {
                value_accessor(row).map_or(CellValue::Empty, CellValue::Int)
            }),
            render: Arc::new(move |row| crate::formatting::format_integer(number(row)).into_any()),
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
            width_characters: QUANTITY_WIDTH_CHARACTERS,
            sortable: false,
            searchable: false,
            filterable: false,
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
            width_characters: QUANTITY_IN_UNIT_WIDTH_CHARACTERS,
            ..Self::quantity(key, header, accessor)
        }
    }

    pub fn money(
        key: &'static str,
        header: &'static str,
        accessor: impl Fn(&Row) -> Option<f64> + Send + Sync + Clone + 'static,
        currency: impl Fn(&Row) -> Currency + Send + Sync + 'static,
    ) -> Self {
        let value_accessor = accessor.clone();
        Self {
            key,
            header,
            align: Align::Right,
            width_characters: MONEY_WIDTH_CHARACTERS,
            sortable: false,
            searchable: false,
            filterable: false,
            value: Arc::new(move |row| match value_accessor(row) {
                Some(amount) => CellValue::Number(amount),
                None => CellValue::Empty,
            }),
            render: Arc::new(move |row| {
                crate::formatting::format_money(accessor(row), currency(row)).into_any()
            }),
        }
    }

    pub fn money_decimal(
        key: &'static str,
        header: &'static str,
        accessor: impl Fn(&Row) -> Option<rust_decimal::Decimal> + Send + Sync + Clone + 'static,
        currency: impl Fn(&Row) -> Currency + Send + Sync + 'static,
    ) -> Self {
        let value_accessor = accessor.clone();
        Self {
            key,
            header,
            align: Align::Right,
            width_characters: MONEY_WIDTH_CHARACTERS,
            sortable: false,
            searchable: false,
            filterable: false,
            value: Arc::new(move |row| {
                use rust_decimal::prelude::ToPrimitive;
                match value_accessor(row).and_then(|amount| amount.to_f64()) {
                    Some(amount) => CellValue::Number(amount),
                    None => CellValue::Empty,
                }
            }),
            render: Arc::new(move |row| {
                crate::formatting::format_money_decimal(accessor(row), currency(row)).into_any()
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
            width_characters: TEXT_WIDTH_CHARACTERS,
            sortable: false,
            searchable: false,
            filterable: false,
            value: Arc::new(move |row| CellValue::Text(value_accessor(row))),
            render: Arc::new(move |row| {
                let target = href(row);
                let label = text(row);
                let full_label = label.clone();
                view! {
                    <a
                        href=target
                        title=full_label
                        class="text-blue-600 hover:text-blue-800 hover:underline"
                    >
                        {label}
                    </a>
                }
                .into_any()
            }),
        }
    }

    /// The row's own actions, such as `RowActions`: no header, right-aligned
    /// and only as wide as the menu button.
    pub fn actions(render: impl Fn(&Row) -> AnyView + Send + Sync + 'static) -> Self {
        Self {
            key: ACTIONS_KEY,
            header: "",
            align: Align::Right,
            width_characters: ACTIONS_WIDTH_CHARACTERS,
            sortable: false,
            searchable: false,
            filterable: false,
            value: Arc::new(|_| CellValue::Empty),
            render: Arc::new(render),
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
            width_characters: CUSTOM_WIDTH_CHARACTERS,
            sortable: false,
            searchable: false,
            filterable: false,
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
    pub fn width_characters(mut self, width_characters: u16) -> Self {
        self.width_characters = width_characters;
        self
    }

    /// The declared width, widened so the header label and its icons never
    /// get cut short.
    #[must_use]
    pub fn rendered_width_characters(&self) -> u16 {
        let header_letters = u16::try_from(self.header.chars().count()).unwrap_or(u16::MAX);
        let header_width = header_letters
            .saturating_mul(UPPERCASE_LETTER_WIDTH_FIFTHS)
            .div_ceil(FIFTHS_PER_CHARACTER)
            .saturating_add(self.header_icons_width_characters());
        self.width_characters.max(header_width)
    }

    fn header_icons_width_characters(&self) -> u16 {
        if self.sortable || self.filterable {
            HEADER_ICONS_WIDTH_CHARACTERS
        } else {
            0
        }
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
}
