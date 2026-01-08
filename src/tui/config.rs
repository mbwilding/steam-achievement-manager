use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub sort_column: String,
    pub sort_order: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            sort_column: "Percentage".to_string(),
            sort_order: "Descending".to_string(),
        }
    }
}
