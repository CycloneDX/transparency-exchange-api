use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::common::checksum::Checksum;
use crate::domain::common::deprecation::Deprecation;
use crate::domain::common::identifier::Identifier;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Component {
    #[serde(skip_deserializing, default)]
    pub uuid: Uuid,
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub identifiers: Vec<Identifier>,
    pub component_type: ComponentType,
    #[serde(default)]
    pub licenses: Vec<LicenseInfo>,
    pub publisher: Option<String>,
    pub homepage_url: Option<String>,
    pub vcs_url: Option<String>,
    #[serde(skip_deserializing, default = "crate::domain::common::now")]
    pub created_date: DateTime<Utc>,
    #[serde(skip_deserializing, default = "crate::domain::common::now")]
    pub modified_date: DateTime<Utc>,
    pub deprecation: Option<Deprecation>,
    #[serde(default)]
    pub dependencies: Vec<Identifier>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ComponentType {
    Unspecified,
    Application,
    Framework,
    Library,
    Container,
    OperatingSystem,
    Device,
    File,
    Firmware,
    Other,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComponentRelease {
    pub uuid: Uuid,
    pub component_uuid: Uuid,
    pub version: String,
    pub release_date: Option<DateTime<Utc>>,
    #[serde(default)]
    pub pre_release: bool,
    #[serde(default)]
    pub identifiers: Vec<Identifier>,
    #[serde(default)]
    pub distributions: Vec<Distribution>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Distribution {
    pub distribution_type: String,
    pub description: Option<String>,
    #[serde(default)]
    pub identifiers: Vec<Identifier>,
    pub url: Option<String>,
    pub signature_url: Option<String>,
    #[serde(default)]
    pub checksums: Vec<Checksum>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LicenseInfo {
    pub license_type: LicenseType,
    pub license_id: String,
    pub url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LicenseType {
    Unspecified,
    Spdx,
    Other,
}
