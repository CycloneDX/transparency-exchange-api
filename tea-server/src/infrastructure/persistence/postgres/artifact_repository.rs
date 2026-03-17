use async_trait::async_trait;
use chrono::Utc;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::domain::artifact::entity::{Artifact, ArtifactFormat, ArtifactType, Subject};
use crate::domain::artifact::repository::ArtifactRepository;
use crate::domain::common::deprecation::{Deprecation, DeprecationState};
use crate::domain::common::error::RepositoryError;
use crate::domain::common::identifier::Identifier;

pub struct PostgresArtifactRepository {
    pool: PgPool,
}

impl PostgresArtifactRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

// ─── error helper ──────────────────────────────────────────────────────────
fn json_err(e: serde_json::Error) -> RepositoryError {
    RepositoryError::Database(sqlx::Error::Decode(e.into()))
}

fn parse_artifact_type(s: Option<&str>) -> ArtifactType {
    match s {
        Some("ATTESTATION") => ArtifactType::Attestation,
        Some("BOM") => ArtifactType::Bom,
        Some("BUILD_META") => ArtifactType::BuildMeta,
        Some("CERTIFICATION") => ArtifactType::Certification,
        Some("FORMULATION") => ArtifactType::Formulation,
        Some("LICENSE") => ArtifactType::License,
        Some("RELEASE_NOTES") => ArtifactType::ReleaseNotes,
        Some("SECURITY_TXT") => ArtifactType::SecurityTxt,
        Some("THREAT_MODEL") => ArtifactType::ThreatModel,
        Some("VULNERABILITIES") => ArtifactType::Vulnerabilities,
        Some("CLE") => ArtifactType::Cle,
        Some("CDXA") => ArtifactType::Cdxa,
        Some("CBOM") => ArtifactType::Cbom,
        Some("MODEL_CARD") => ArtifactType::ModelCard,
        Some("STATIC_ANALYSIS") => ArtifactType::StaticAnalysis,
        Some("DYNAMIC_ANALYSIS") => ArtifactType::DynamicAnalysis,
        Some("PENTEST_REPORT") => ArtifactType::PentestReport,
        Some("RISK_ASSESSMENT") => ArtifactType::RiskAssessment,
        Some("POAM") => ArtifactType::Poam,
        Some("QUALITY_METRICS") => ArtifactType::QualityMetrics,
        Some("HARNESS") => ArtifactType::Harness,
        Some("CONFORMANCE") => ArtifactType::Conformance,
        Some("OTHER") => ArtifactType::Other,
        _ => ArtifactType::Unspecified,
    }
}

fn artifact_type_str(t: &ArtifactType) -> &'static str {
    match t {
        ArtifactType::Attestation => "ATTESTATION",
        ArtifactType::Bom => "BOM",
        ArtifactType::BuildMeta => "BUILD_META",
        ArtifactType::Certification => "CERTIFICATION",
        ArtifactType::Formulation => "FORMULATION",
        ArtifactType::License => "LICENSE",
        ArtifactType::ReleaseNotes => "RELEASE_NOTES",
        ArtifactType::SecurityTxt => "SECURITY_TXT",
        ArtifactType::ThreatModel => "THREAT_MODEL",
        ArtifactType::Vulnerabilities => "VULNERABILITIES",
        ArtifactType::Cle => "CLE",
        ArtifactType::Cdxa => "CDXA",
        ArtifactType::Cbom => "CBOM",
        ArtifactType::ModelCard => "MODEL_CARD",
        ArtifactType::StaticAnalysis => "STATIC_ANALYSIS",
        ArtifactType::DynamicAnalysis => "DYNAMIC_ANALYSIS",
        ArtifactType::PentestReport => "PENTEST_REPORT",
        ArtifactType::RiskAssessment => "RISK_ASSESSMENT",
        ArtifactType::Poam => "POAM",
        ArtifactType::QualityMetrics => "QUALITY_METRICS",
        ArtifactType::Harness => "HARNESS",
        ArtifactType::Conformance => "CONFORMANCE",
        ArtifactType::Other => "OTHER",
        ArtifactType::Unspecified => "UNSPECIFIED",
    }
}

fn map_artifact_row(row: &sqlx::postgres::PgRow) -> Result<Artifact, RepositoryError> {
    let type_str: Option<String> = row.try_get("type")?;
    let component_distributions: Vec<String> =
        serde_json::from_value(row.try_get("component_distributions")?).unwrap_or_default();
    let formats: Vec<ArtifactFormat> =
        serde_json::from_value(row.try_get("formats")?).unwrap_or_default();
    let subject_json: Option<serde_json::Value> = row.try_get("subject")?;
    let subject: Option<Subject> = subject_json.and_then(|v| serde_json::from_value(v).ok());
    let identifiers: Vec<Identifier> =
        serde_json::from_value(row.try_get("identifiers")?).unwrap_or_default();
    let deprecation_state: Option<String> = row.try_get("deprecation_state")?;
    let deprecation = deprecation_state.map(|state| Deprecation {
        state: match state.as_str() {
            "ACTIVE" => DeprecationState::Active,
            "DEPRECATED" => DeprecationState::Deprecated,
            "RETIRED" => DeprecationState::Retired,
            _ => DeprecationState::Unspecified,
        },
        reason: row.try_get("deprecation_reason").ok().flatten(),
        announced_date: None,
        effective_date: row.try_get("deprecated_date").ok().flatten(),
        replacement_identifiers: vec![],
    });

    Ok(Artifact {
        uuid: row.try_get("uuid")?,
        name: row.try_get("name")?,
        type_: parse_artifact_type(type_str.as_deref()),
        component_distributions,
        formats,
        created_date: row.try_get("created_date")?,
        modified_date: row.try_get("modified_date")?,
        description: row.try_get("description")?,
        subject,
        deprecation,
        dependencies: identifiers,
    })
}

#[async_trait]
impl ArtifactRepository for PostgresArtifactRepository {
    async fn find_by_uuid(&self, uuid: &Uuid) -> Result<Option<Artifact>, RepositoryError> {
        let row = sqlx::query(
            r#"
            SELECT uuid, name, type, component_distributions, formats, created_date, modified_date,
                   description, subject, identifiers,
                   deprecation_state, deprecation_reason, deprecated_date
            FROM tea_artifacts
            WHERE uuid = $1
            "#,
        )
        .bind(uuid)
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| map_artifact_row(&r)).transpose()
    }

    async fn find_by_name(&self, name: &str) -> Result<Vec<Artifact>, RepositoryError> {
        let rows = if name.is_empty() {
            sqlx::query(
                r#"
                SELECT uuid, name, type, component_distributions, formats, created_date, modified_date,
                       description, subject, identifiers,
                       deprecation_state, deprecation_reason, deprecated_date
                FROM tea_artifacts
                ORDER BY created_date DESC
                "#,
            )
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query(
                r#"
                SELECT uuid, name, type, component_distributions, formats, created_date, modified_date,
                       description, subject, identifiers,
                       deprecation_state, deprecation_reason, deprecated_date
                FROM tea_artifacts
                WHERE name ILIKE $1
                ORDER BY created_date DESC
                "#,
            )
            .bind(format!("%{name}%"))
            .fetch_all(&self.pool)
            .await?
        };
        rows.iter().map(map_artifact_row).collect()
    }

    async fn save(&self, artifact: &Artifact) -> Result<(), RepositoryError> {
        let subject_json = artifact
            .subject
            .as_ref()
            .map(|s| serde_json::to_value(s))
            .transpose()
            .map_err(json_err)?;

        let result = sqlx::query(
            r#"
            INSERT INTO tea_artifacts (
                uuid, name, type, component_distributions, formats, created_date, modified_date,
                description, subject, identifiers,
                deprecation_state, deprecation_reason, deprecated_date
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            ON CONFLICT (uuid) DO NOTHING
            "#,
        )
        .bind(artifact.uuid)
        .bind(&artifact.name)
        .bind(artifact_type_str(&artifact.type_))
        .bind(serde_json::to_value(&artifact.component_distributions).map_err(json_err)?)
        .bind(serde_json::to_value(&artifact.formats).map_err(json_err)?)
        .bind(artifact.created_date)
        .bind(artifact.modified_date)
        .bind(&artifact.description)
        .bind(subject_json)
        .bind(serde_json::to_value(&artifact.dependencies).map_err(json_err)?)
        .bind(artifact.deprecation.as_ref().map(|d| format!("{:?}", d.state).to_uppercase()))
        .bind(artifact.deprecation.as_ref().and_then(|d| d.reason.as_deref()))
        .bind(artifact.deprecation.as_ref().and_then(|d| d.effective_date))
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::Conflict);
        }
        Ok(())
    }

    async fn update(&self, artifact: &Artifact) -> Result<(), RepositoryError> {
        let subject_json = artifact
            .subject
            .as_ref()
            .map(|s| serde_json::to_value(s))
            .transpose()
            .map_err(json_err)?;

        let result = sqlx::query(
            r#"
            UPDATE tea_artifacts
            SET name = $2, type = $3, component_distributions = $4, formats = $5,
                modified_date = $6, description = $7, subject = $8, identifiers = $9,
                deprecation_state = $10, deprecation_reason = $11, deprecated_date = $12
            WHERE uuid = $1
            "#,
        )
        .bind(artifact.uuid)
        .bind(&artifact.name)
        .bind(artifact_type_str(&artifact.type_))
        .bind(serde_json::to_value(&artifact.component_distributions).map_err(json_err)?)
        .bind(serde_json::to_value(&artifact.formats).map_err(json_err)?)
        .bind(artifact.modified_date)
        .bind(&artifact.description)
        .bind(subject_json)
        .bind(serde_json::to_value(&artifact.dependencies).map_err(json_err)?)
        .bind(artifact.deprecation.as_ref().map(|d| format!("{:?}", d.state).to_uppercase()))
        .bind(artifact.deprecation.as_ref().and_then(|d| d.reason.as_deref()))
        .bind(artifact.deprecation.as_ref().and_then(|d| d.effective_date))
        .execute(&self.pool)
        .await?;

        // C2 fix: detect missing entity
        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }
        Ok(())
    }

    async fn delete(&self, uuid: &Uuid) -> Result<(), RepositoryError> {
        sqlx::query("DELETE FROM tea_artifacts WHERE uuid = $1")
            .bind(uuid)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
