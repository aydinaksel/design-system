use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Currency {
    PoundSterling,
    UsDollar,
    Euro,
}

impl Currency {
    pub const ALL: [Currency; 3] = [Currency::PoundSterling, Currency::UsDollar, Currency::Euro];

    #[must_use]
    pub fn code(self) -> &'static str {
        match self {
            Currency::PoundSterling => "GBP",
            Currency::UsDollar => "USD",
            Currency::Euro => "EUR",
        }
    }

    #[must_use]
    pub fn symbol(self) -> &'static str {
        match self {
            Currency::PoundSterling => "£",
            Currency::UsDollar => "$",
            Currency::Euro => "€",
        }
    }

    #[must_use]
    pub fn from_code(code: &str) -> Option<Currency> {
        Currency::ALL
            .into_iter()
            .find(|currency| currency.code().eq_ignore_ascii_case(code.trim()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_an_iso_code_ignoring_case_and_spacing() {
        assert_eq!(Currency::from_code("gbp"), Some(Currency::PoundSterling));
        assert_eq!(Currency::from_code(" EUR "), Some(Currency::Euro));
        assert_eq!(Currency::from_code("TRY"), None);
    }

    #[test]
    fn every_currency_reads_back_from_its_own_code() {
        for currency in Currency::ALL {
            assert_eq!(Currency::from_code(currency.code()), Some(currency));
        }
    }
}
