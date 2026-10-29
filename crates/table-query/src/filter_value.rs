use serde::{Deserialize, Serialize};

/// One selectable value in a server-driven column filter, counted under the
/// rest of the query's filters.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilterValue {
    pub value: String,
    pub row_count: u64,
    /// What the rail shows in place of a coded `value`.
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

/// Every filterable column's values in one payload, for a facet rail.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FacetValues {
    pub column_key: String,
    pub values: Vec<FilterValue>,
}

pub type LoadedFacetValues = Option<Result<Vec<FacetValues>, String>>;
