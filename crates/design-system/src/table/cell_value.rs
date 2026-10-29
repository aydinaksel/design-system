use std::cmp::Ordering;

#[derive(Clone, Debug, PartialEq)]
pub enum CellValue {
    Text(String),
    Number(f64),
    Int(i64),
    Empty,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Align {
    #[default]
    Left,
    Right,
    Center,
}

impl Align {
    #[must_use]
    pub fn text_class(self) -> &'static str {
        match self {
            Align::Left => "text-left",
            Align::Right => "text-right",
            Align::Center => "text-center",
        }
    }
}

impl CellValue {
    #[must_use]
    pub fn optional_text(text: Option<&String>) -> CellValue {
        match text {
            Some(value) if !value.is_empty() => CellValue::Text(value.clone()),
            _ => CellValue::Empty,
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        matches!(self, CellValue::Empty)
    }

    #[must_use]
    pub fn search_text(&self) -> String {
        match self {
            CellValue::Text(text) => text.to_lowercase(),
            CellValue::Number(number) => number.to_string(),
            CellValue::Int(integer) => integer.to_string(),
            CellValue::Empty => String::new(),
        }
    }

    #[must_use]
    pub fn filter_key(&self) -> String {
        match self {
            CellValue::Text(text) => text.clone(),
            CellValue::Number(number) => number.to_string(),
            CellValue::Int(integer) => integer.to_string(),
            CellValue::Empty => String::new(),
        }
    }

    #[must_use]
    pub fn filter_label(&self) -> String {
        match self {
            CellValue::Empty => "(Blanks)".to_string(),
            _ => self.filter_key(),
        }
    }

    fn as_number(&self) -> Option<f64> {
        match self {
            CellValue::Number(number) => Some(*number),
            CellValue::Int(integer) =>
            {
                #[expect(
                    clippy::as_conversions,
                    clippy::cast_precision_loss,
                    reason = "a sort key only; the values are document numbers and counts, far inside f64's exact range"
                )]
                Some(*integer as f64)
            }
            _ => None,
        }
    }

    /// Orders two non-empty values. NaN is treated as equal, matching every
    /// hand-rolled page's `partial_cmp(..).unwrap_or(Equal)`. Empty handling
    /// (always sorted last, independent of direction) lives in `DataTable`'s
    /// sort, not here, so it cannot be flipped by `reverse()`.
    #[must_use]
    pub fn compare(&self, other: &CellValue) -> Ordering {
        match (self.as_number(), other.as_number()) {
            (Some(left), Some(right)) => left.partial_cmp(&right).unwrap_or(Ordering::Equal),
            _ => self.search_text().cmp(&other.search_text()),
        }
    }
}
