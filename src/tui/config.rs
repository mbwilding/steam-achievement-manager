use super::models::{SortColumn, SortOrder};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub sort_column: SortColumn,
    pub sort_order: SortOrder,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            sort_column: SortColumn::Percentage,
            sort_order: SortOrder::Descending,
        }
    }
}
