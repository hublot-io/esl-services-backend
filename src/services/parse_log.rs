use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseLog {
    pub app: String,
    // string because we forward env-logger's log level
    pub level: String,
    pub message: String,
    pub serial: String,
}
