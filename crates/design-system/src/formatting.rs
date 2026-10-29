#[must_use]
pub fn format_with_thousands(integer_part: &str) -> String {
    let (sign, digits) = match integer_part.strip_prefix('-') {
        Some(without_sign) => ("-", without_sign),
        None => ("", integer_part),
    };
    let mut output = String::with_capacity(
        sign.len()
            .saturating_add(digits.len())
            .saturating_add(digits.len() / 3),
    );
    output.push_str(sign);
    let length = digits.len();
    for (index, character) in digits.chars().enumerate() {
        if index > 0 && length.saturating_sub(index) % 3 == 0 {
            output.push(',');
        }
        output.push(character);
    }
    output
}

#[must_use]
pub fn format_money(value: Option<f64>) -> String {
    match value {
        Some(amount) => {
            let formatted = format!("{amount:.2}");
            let (integer_part, decimal_part) = formatted
                .split_once('.')
                .unwrap_or((formatted.as_str(), "00"));
            let with_commas = format_with_thousands(integer_part);
            let (sign, body) = match with_commas.strip_prefix('-') {
                Some(rest) => ("-", rest.to_owned()),
                None => ("", with_commas),
            };
            format!("{sign}${body}.{decimal_part}")
        }
        None => String::new(),
    }
}

#[must_use]
pub fn format_integer(value: Option<i32>) -> String {
    match value {
        Some(number) => format_with_thousands(&number.to_string()),
        None => String::new(),
    }
}

#[must_use]
pub fn format_money_decimal(value: Option<rust_decimal::Decimal>) -> String {
    match value {
        Some(amount) => {
            let formatted = amount.round_dp(2).to_string();
            let (integer_part, decimal_part) = formatted
                .split_once('.')
                .unwrap_or((formatted.as_str(), "00"));
            let padded_decimal = match decimal_part.len() {
                0 => "00".to_owned(),
                1 => format!("{decimal_part}0"),
                _ => decimal_part.to_owned(),
            };
            let with_commas = format_with_thousands(integer_part);
            let (sign, body) = match with_commas.strip_prefix('-') {
                Some(rest) => ("-", rest.to_owned()),
                None => ("", with_commas),
            };
            format!("{sign}${body}.{padded_decimal}")
        }
        None => String::new(),
    }
}

#[must_use]
pub fn format_quantity(quantity: f64) -> String {
    let rounded = format!("{quantity:.2}");
    let trimmed = rounded.trim_end_matches('0').trim_end_matches('.');
    let (integer_part, fraction) = trimmed.split_once('.').unwrap_or((trimmed, ""));
    let grouped = format_with_thousands(integer_part);
    if fraction.is_empty() {
        grouped
    } else {
        format!("{grouped}.{fraction}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn money_is_grouped_with_commas() {
        assert_eq!(format_money(Some(1_234_567.5)), "$1,234,567.50");
        assert_eq!(format_money(Some(-980.25)), "-$980.25");
        assert_eq!(format_money(Some(12.0)), "$12.00");
        assert_eq!(format_money(None), "");
    }

    #[test]
    fn integers_are_grouped_with_commas() {
        assert_eq!(format_integer(Some(1_234_567)), "1,234,567");
        assert_eq!(format_integer(Some(-42)), "-42");
        assert_eq!(format_integer(None), "");
    }
}
