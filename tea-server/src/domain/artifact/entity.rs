use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::common::checksum::Checksum;
use crate::domain::common::deprecation::Deprecation;
use crate::domain::common::identifier::Identifier;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Artifact {
    #[serde(skip_deserializing, default)]
    pub uuid: Uuid,
    pub name: String,
    #[serde(rename = "type")]
    pub type_: ArtifactType,
    #[serde(rename = "componentDistributions", default)]
    pub component_distributions: Vec<String>,
    pub formats: Vec<ArtifactFormat>,
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
    pub description: Option<String>,
    pub subject: Option<Subject>,
    pub deprecation: Option<Deprecation>,
    #[serde(default)]
    pub dependencies: Vec<Identifier>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArtifactFormat {
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    pub description: Option<String>,
    pub url: String,
    #[serde(rename = "signatureUrl")]
    pub signature_url: Option<String>,
    pub checksums: Vec<Checksum>,
    #[serde(rename = "sizeBytes")]
    pub size_bytes: Option<i64>,
    pub encoding: Option<String>,
    #[serde(rename = "specVersion")]
    pub spec_version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Subject {
    #[serde(rename = "type")]
    pub type_: SubjectType,
    #[serde(default)]
    pub identifiers: Vec<Identifier>,
    pub name: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ArtifactType {
    Unspecified,
    Attestation,
    Bom,
    BuildMeta,
    Certification,
    Formulation,
    License,
    ReleaseNotes,
    SecurityTxt,
    ThreatModel,
    Vulnerabilities,
    Cle,
    Cdxa,
    Cbom,
    ModelCard,
    StaticAnalysis,
    DynamicAnalysis,
    PentestReport,
    RiskAssessment,
    Poam,
    QualityMetrics,
    Harness,
    Conformance,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SubjectType {
    Unspecified,
    Component,
    Product,
    Service,
    Organization,
    Build,
}
