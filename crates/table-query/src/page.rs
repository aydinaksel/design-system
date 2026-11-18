use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Page<Row> {
    pub rows: Vec<Row>,
    pub total_rows: u64,
}
