use serde::{Deserialize, Serialize};

/// One window of a server-answered [`crate::TableQuery`].
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Page<Row> {
    pub rows: Vec<Row>,
    pub total_rows: u64,
}
