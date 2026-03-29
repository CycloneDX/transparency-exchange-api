use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::common::deprecation::Deprecation;
use crate::domain::common::identifier::Identifier;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Collection {
    #[serde(skip_deserializing, default)]
    pub uuid: Uuid,
    pub name: String,
    pub version: i32,
    #[serde(skip_deserializing, default = "crate::domain::common::now")]
    pub date: DateTime<Utc>,
    #[serde(
        rename = "createdDate",
        skip_deserializing,
        default = "crate::domain::common::now"
    )]
    pub created_date: DateTime<Utc>,
    #[serde(
        rename = "modifiedDate",
        skip_deserializing,
        default = "crate::domain::common::now"
    )]
    pub modified_date: DateTime<Utc>,
    pub belongs_to: CollectionScope,
    pub update_reason: UpdateReason,
    #[serde(default)]
    pub artifacts: Vec<Uuid>,
    pub deprecation: Option<Deprecation>,
    #[serde(default)]
    pub dependencies: Vec<Identifier>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CollectionScope {
    Unspecified,
    Release,
    ProductRelease,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UpdateReason {
    Unspecified,
    InitialRelease,
    VexUpdated,
    ArtifactUpdated,
    ArtifactRemoved,
    ArtifactAdded,
}
