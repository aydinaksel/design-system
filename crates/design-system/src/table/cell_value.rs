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
    pub fn display_text(&self) -> Option<String> {
        match self {
            CellValue::Text(text) => Some(text.clone()),
            CellValue::Number(number) => Some(crate::formatting::format_quantity(*number)),
            CellValue::Int(integer) => Some(integer.to_string()),
            CellValue::Empty => None,
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

    fn is_numeric(&self) -> bool {
        matches!(self, CellValue::Number(_) | CellValue::Int(_))
    }

    /// Orders two non-empty values consistently, so a sort can never be handed
    /// a contradiction. Numbers come before text, integers and floats compare
    /// exactly, -0 equals 0, and NaN sorts after every number. Empty handling
    /// (always sorted last, independent of direction) lives in `DataTable`'s
    /// sort, not here, so it cannot be flipped by `reverse()`.
    #[must_use]
    pub fn compare(&self, other: &CellValue) -> Ordering {
        match (self, other) {
            (CellValue::Int(left), CellValue::Int(right)) => left.cmp(right),
            (CellValue::Number(left), CellValue::Number(right)) => compare_floats(*left, *right),
            (CellValue::Int(left), CellValue::Number(right)) => {
                compare_integer_with_float(*left, *right)
            }
            (CellValue::Number(left), CellValue::Int(right)) => {
                compare_integer_with_float(*right, *left).reverse()
            }
            _ => match (self.is_numeric(), other.is_numeric()) {
                (true, false) => Ordering::Less,
                (false, true) => Ordering::Greater,
                _ => self.search_text().cmp(&other.search_text()),
            },
        }
    }
}

const SMALLEST_FLOAT_BEYOND_EVERY_INTEGER: f64 = f64::from_bits(0x43E0_0000_0000_0000);

fn compare_floats(left: f64, right: f64) -> Ordering {
    match (left.is_nan(), right.is_nan()) {
        (true, true) => Ordering::Equal,
        (true, false) => Ordering::Greater,
        (false, true) => Ordering::Less,
        (false, false) => left.partial_cmp(&right).unwrap_or(Ordering::Equal),
    }
}

fn compare_integer_with_float(integer: i64, float: f64) -> Ordering {
    if float.is_nan() || float >= SMALLEST_FLOAT_BEYOND_EVERY_INTEGER {
        return Ordering::Less;
    }
    if float < -SMALLEST_FLOAT_BEYOND_EVERY_INTEGER {
        return Ordering::Greater;
    }
    let whole = float.trunc();
    #[expect(
        clippy::as_conversions,
        clippy::cast_possible_truncation,
        reason = "whole is an integral float inside i64's range, so the cast is exact"
    )]
    let whole_integer = whole as i64;
    integer
        .cmp(&whole_integer)
        .then_with(|| compare_floats(0.0, float - whole))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_float_beyond_every_integer_is_two_to_the_sixty_third() {
        assert_eq!(
            SMALLEST_FLOAT_BEYOND_EVERY_INTEGER.to_bits(),
            2f64.powi(63).to_bits()
        );
    }

    #[test]
    fn a_large_integer_is_not_equal_to_a_float_it_rounds_to() {
        let float = 2f64.powi(60);
        assert_eq!(compare_integer_with_float(1 << 60, float), Ordering::Equal);
        assert_eq!(
            compare_integer_with_float((1 << 60) + 1, float),
            Ordering::Greater
        );
    }

    #[test]
    fn negative_zero_equals_zero_and_nan_sorts_after_numbers() {
        assert_eq!(
            CellValue::Number(-0.0).compare(&CellValue::Int(0)),
            Ordering::Equal
        );
        assert_eq!(
            CellValue::Number(-0.0).compare(&CellValue::Number(0.0)),
            Ordering::Equal
        );
        assert_eq!(
            CellValue::Number(f64::NAN).compare(&CellValue::Number(f64::INFINITY)),
            Ordering::Greater
        );
        assert_eq!(
            CellValue::Int(i64::MAX).compare(&CellValue::Number(f64::NAN)),
            Ordering::Less
        );
    }

    #[test]
    fn numbers_sort_before_text() {
        assert_eq!(
            CellValue::Int(10).compare(&CellValue::Text("9".to_owned())),
            Ordering::Less
        );
    }
}
