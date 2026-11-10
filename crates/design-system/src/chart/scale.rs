use crate::currency::Currency;
use crate::formatting::{format_money, format_with_thousands};

const TARGET_INTERVALS: f64 = 4.0;
const NICE_STEP_MULTIPLIERS: [f64; 5] = [1.0, 2.0, 2.5, 5.0, 10.0];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValueFormat {
    Money(Currency),
    Count,
}

impl ValueFormat {
    #[must_use]
    pub fn compact(self, value: f64) -> String {
        let prefix = match self {
            Self::Money(currency) => currency.symbol(),
            Self::Count => "",
        };
        let sign = if value < 0.0 { "-" } else { "" };
        let magnitude = value.abs();
        let body = if magnitude >= 1_000_000.0 {
            format!("{}M", trimmed_one_decimal(magnitude / 1_000_000.0))
        } else if magnitude >= 1_000.0 {
            format!("{}K", trimmed_one_decimal(magnitude / 1_000.0))
        } else {
            format!("{magnitude:.0}")
        };
        format!("{sign}{prefix}{body}")
    }

    #[must_use]
    pub fn full(self, value: f64) -> String {
        match self {
            Self::Money(currency) => format_money(Some(value), currency),
            Self::Count => format_with_thousands(&format!("{value:.0}")),
        }
    }
}

fn trimmed_one_decimal(value: f64) -> String {
    let rounded = if value >= 10.0 {
        format!("{value:.0}")
    } else {
        format!("{value:.1}")
    };
    rounded
        .strip_suffix(".0")
        .map_or(rounded.clone(), str::to_owned)
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ValueScale {
    pub top: f64,
    pub ticks: Vec<f64>,
}

impl ValueScale {
    pub(crate) fn covering(maximum: f64) -> Self {
        if maximum.is_nan() || maximum <= 0.0 {
            return Self {
                top: 1.0,
                ticks: vec![0.0],
            };
        }
        let step = nice_step(maximum / TARGET_INTERVALS);
        let top = (maximum / step).ceil() * step;
        let mut ticks = Vec::new();
        let mut tick = 0.0;
        while tick <= top + step / 2.0 {
            ticks.push(tick);
            tick += step;
        }
        Self { top, ticks }
    }

    pub(crate) fn percent_from_top(&self, value: f64) -> f64 {
        100.0 - (value / self.top * 100.0)
    }
}

fn nice_step(raw_step: f64) -> f64 {
    let magnitude = 10_f64.powf(raw_step.log10().floor());
    NICE_STEP_MULTIPLIERS
        .iter()
        .map(|multiplier| multiplier * magnitude)
        .find(|candidate| *candidate >= raw_step)
        .unwrap_or(10.0 * magnitude)
}

#[cfg(test)]
mod tests {
    use super::{ValueFormat, ValueScale};
    use crate::currency::Currency;

    #[test]
    fn the_scale_rounds_up_to_a_clean_top() {
        let scale = ValueScale::covering(87_400.0);

        assert_eq!(
            scale.ticks,
            vec![0.0, 25_000.0, 50_000.0, 75_000.0, 100_000.0]
        );
    }

    #[test]
    fn an_empty_range_still_draws_a_baseline() {
        assert_eq!(ValueScale::covering(0.0).ticks, vec![0.0]);
    }

    #[test]
    fn money_compacts_to_thousands_and_millions() {
        let dollars = ValueFormat::Money(Currency::UsDollar);
        assert_eq!(dollars.compact(25_000.0), "$25K");
        assert_eq!(dollars.compact(2_500.0), "$2.5K");
        assert_eq!(dollars.compact(1_200_000.0), "$1.2M");
        assert_eq!(dollars.compact(0.0), "$0");
    }

    #[test]
    fn money_carries_its_currency_symbol() {
        assert_eq!(
            ValueFormat::Money(Currency::PoundSterling).compact(-4_500.0),
            "-£4.5K"
        );
        assert_eq!(
            ValueFormat::Money(Currency::Euro).full(1_234.5),
            "€1,234.50"
        );
    }
}
