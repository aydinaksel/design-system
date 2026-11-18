use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilterValue {
    pub value: String,
    pub row_count: u64,
    #[serde(default)]
    pub label: Option<String>,
}

const BLANK_LABEL: &str = "(Blanks)";

impl FilterValue {
    #[must_use]
    pub fn label(&self) -> &str {
        match self.label.as_deref() {
            Some(label) => label,
            None if self.value.is_empty() => BLANK_LABEL,
            None => self.value.as_str(),
        }
    }
}

pub type LoadedFilterValues = Option<Result<Vec<FilterValue>, String>>;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FacetValues {
    pub column_key: String,
    pub values: Vec<FilterValue>,
}

pub type LoadedFacetValues = Option<Result<Vec<FacetValues>, String>>;
