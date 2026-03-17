use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::identifier::Identifier;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Deprecation {
    pub state: DeprecationState,
    pub reason: Option<String>,
    pub announced_date: Option<DateTime<Utc>>,
    pub effective_date: Option<DateTime<Utc>>,
    pub replacement_identifiers: Vec<Identifier>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DeprecationState {
    Unspecified,
    Active,
    Deprecated,
    Retired,
}
