use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::common::deprecation::Deprecation;
use crate::domain::common::identifier::Identifier;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Product {
    #[serde(skip_deserializing, default)]
    pub uuid: Uuid,
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub identifiers: Vec<Identifier>,
    pub vendor: Vendor,
    #[serde(skip_deserializing, default = "crate::domain::common::now")]
    pub created_date: DateTime<Utc>,
    #[serde(skip_deserializing, default = "crate::domain::common::now")]
    pub modified_date: DateTime<Utc>,
    pub homepage_url: Option<String>,
    pub documentation_url: Option<String>,
    pub vcs_url: Option<String>,
    pub deprecation: Option<Deprecation>,
    #[serde(default)]
    pub dependencies: Vec<Identifier>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProductRelease {
    #[serde(skip_deserializing, default)]
    pub uuid: Uuid,
    pub product_uuid: Uuid,
    pub version: String,
    #[serde(skip_deserializing, default = "crate::domain::common::now")]
    pub created_date: DateTime<Utc>,
    #[serde(skip_deserializing, default = "crate::domain::common::now")]
    pub modified_date: DateTime<Utc>,
    pub release_date: Option<DateTime<Utc>>,
    #[serde(default)]
    pub pre_release: bool,
    #[serde(default)]
    pub identifiers: Vec<Identifier>,
    #[serde(default)]
    pub components: Vec<ComponentRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentRef {
    pub component_uuid: Uuid,
    pub release_uuid: Uuid,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Vendor {
    /// Vendor display name — required.
    pub name: String,
    pub uuid: Option<Uuid>,
    pub url: Option<String>,
    #[serde(default)]
    pub contacts: Vec<Contact>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Contact {
    #[serde(rename = "type")]
    pub type_: ContactType,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ContactType {
    Unspecified,
    Email,
    Phone,
    Url,
    Other,
}
