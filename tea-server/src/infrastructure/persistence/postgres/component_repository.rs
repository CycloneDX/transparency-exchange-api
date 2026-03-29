use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::domain::common::deprecation::{Deprecation, DeprecationState};
use crate::domain::common::error::RepositoryError;
use crate::domain::common::identifier::Identifier;
use crate::domain::component::entity::{
    Component, ComponentRelease, ComponentType, Distribution, LicenseInfo,
};
use crate::domain::component::repository::ComponentRepository;

pub struct PostgresComponentRepository {
    pool: PgPool,
}

impl PostgresComponentRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

// ─── error helper ──────────────────────────────────────────────────────────
fn json_err(e: serde_json::Error) -> RepositoryError {
    RepositoryError::Database(sqlx::Error::Decode(e.into()))
}

// ─── type string encoders ───────────────────────────────────────────────────
fn component_type_str(t: &ComponentType) -> &'static str {
    match t {
        ComponentType::Application => "APPLICATION",
        ComponentType::Framework => "FRAMEWORK",
        ComponentType::Library => "LIBRARY",
        ComponentType::Container => "CONTAINER",
        ComponentType::OperatingSystem => "OPERATING_SYSTEM",
        ComponentType::Device => "DEVICE",
        ComponentType::File => "FILE",
        ComponentType::Firmware => "FIRMWARE",
        ComponentType::Other => "OTHER",
        ComponentType::Unspecified => "UNSPECIFIED",
    }
}

fn parse_component_type(s: Option<&str>) -> ComponentType {
    match s {
        Some("APPLICATION") => ComponentType::Application,
        Some("FRAMEWORK") => ComponentType::Framework,
        Some("LIBRARY") => ComponentType::Library,
        Some("CONTAINER") => ComponentType::Container,
        Some("OPERATING_SYSTEM") => ComponentType::OperatingSystem,
        Some("DEVICE") => ComponentType::Device,
        Some("FILE") => ComponentType::File,
        Some("FIRMWARE") => ComponentType::Firmware,
        Some("OTHER") => ComponentType::Other,
        _ => ComponentType::Unspecified,
    }
}

fn db_state(deprecation: Option<&Deprecation>) -> String {
    match deprecation {
        Some(dep) => format!("{:?}", dep.state),
        None => format!("{:?}", DeprecationState::Active),
    }
}

fn parse_state(state: &str) -> DeprecationState {
    match state {
        "Active" | "ACTIVE" => DeprecationState::Active,
        "Deprecated" | "DEPRECATED" => DeprecationState::Deprecated,
        "Retired" | "RETIRED" => DeprecationState::Retired,
        _ => DeprecationState::Unspecified,
    }
}

// ─── row mappers ───────────────────────────────────────────────────────────
fn map_component_row(row: &sqlx::postgres::PgRow) -> Result<Component, RepositoryError> {
    let identifiers: Vec<Identifier> =
        serde_json::from_value(row.try_get("identifiers")?).unwrap_or_default();
    let licenses: Vec<LicenseInfo> =
        serde_json::from_value(row.try_get("licenses")?).unwrap_or_default();
    let component_type: Option<String> = row.try_get("component_type")?;
    let deprecation_state: Option<String> = row.try_get("deprecation_state")?;
    let deprecation = deprecation_state.map(|state| Deprecation {
        state: parse_state(&state),
        reason: row.try_get("deprecation_reason").ok().flatten(),
        announced_date: None,
        effective_date: row.try_get("deprecated_date").ok().flatten(),
        replacement_identifiers: vec![],
    });

    Ok(Component {
        uuid: row.try_get("uuid")?,
        name: row.try_get("name")?,
        description: row.try_get("description")?,
        identifiers,
        component_type: parse_component_type(component_type.as_deref()),
        licenses,
        publisher: row.try_get("publisher")?,
        homepage_url: row.try_get("homepage_url")?,
        vcs_url: row.try_get("vcs_url")?,
        created_date: row.try_get("created_date")?,
        modified_date: row.try_get("modified_date")?,
        deprecation,
        dependencies: vec![],
    })
}

fn map_release_row(row: &sqlx::postgres::PgRow) -> Result<ComponentRelease, RepositoryError> {
    let identifiers: Vec<Identifier> =
        serde_json::from_value(row.try_get("identifiers")?).unwrap_or_default();
    let distributions: Vec<Distribution> =
        serde_json::from_value(row.try_get("distributions")?).unwrap_or_default();
    Ok(ComponentRelease {
        uuid: row.try_get("uuid")?,
        component_uuid: row.try_get("component_uuid")?,
        version: row.try_get("version")?,
        release_date: row.try_get("release_date")?,
        pre_release: row.try_get("pre_release")?,
        identifiers,
        distributions,
    })
}

#[async_trait]
impl ComponentRepository for PostgresComponentRepository {
    async fn find_by_uuid(&self, uuid: &Uuid) -> Result<Option<Component>, RepositoryError> {
        let row = sqlx::query(
            r#"
            SELECT uuid, name, description, identifiers, component_type, licenses, publisher,
                   homepage_url, vcs_url, created_date, modified_date,
                   deprecation_state, deprecation_reason, deprecated_date
            FROM tea_components
            WHERE uuid = $1
            "#,
        )
        .bind(uuid)
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| map_component_row(&r)).transpose()
    }

    async fn find_by_name(&self, name: &str) -> Result<Vec<Component>, RepositoryError> {
        let rows = if name.is_empty() {
            sqlx::query(
                r#"
                SELECT uuid, name, description, identifiers, component_type, licenses, publisher,
                       homepage_url, vcs_url, created_date, modified_date,
                       deprecation_state, deprecation_reason, deprecated_date
                FROM tea_components
                ORDER BY created_date DESC
                "#,
            )
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query(
                r#"
                SELECT uuid, name, description, identifiers, component_type, licenses, publisher,
                       homepage_url, vcs_url, created_date, modified_date,
                       deprecation_state, deprecation_reason, deprecated_date
                FROM tea_components
                WHERE name ILIKE $1
                ORDER BY created_date DESC
                "#,
            )
            .bind(format!("%{name}%"))
            .fetch_all(&self.pool)
            .await?
        };
        rows.iter().map(map_component_row).collect()
    }

    async fn save(&self, component: &Component) -> Result<(), RepositoryError> {
        let result = sqlx::query(
            r#"
            INSERT INTO tea_components (
                uuid, name, description, identifiers, component_type, licenses, publisher,
                homepage_url, vcs_url, created_date, modified_date,
                deprecation_state, deprecation_reason, deprecated_date
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            ON CONFLICT (uuid) DO NOTHING
            "#,
        )
        .bind(component.uuid)
        .bind(&component.name)
        .bind(&component.description)
        .bind(serde_json::to_value(&component.identifiers).map_err(json_err)?)
        .bind(component_type_str(&component.component_type))
        .bind(serde_json::to_value(&component.licenses).map_err(json_err)?)
        .bind(&component.publisher)
        .bind(&component.homepage_url)
        .bind(&component.vcs_url)
        .bind(component.created_date)
        .bind(component.modified_date)
        .bind(db_state(component.deprecation.as_ref()))
        .bind(
            component
                .deprecation
                .as_ref()
                .and_then(|d| d.reason.as_deref()),
        )
        .bind(
            component
                .deprecation
                .as_ref()
                .and_then(|d| d.effective_date),
        )
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::Conflict);
        }
        Ok(())
    }

    async fn update(&self, component: &Component) -> Result<(), RepositoryError> {
        let result = sqlx::query(
            r#"
            UPDATE tea_components
            SET name = $2, description = $3, identifiers = $4, component_type = $5,
                licenses = $6, publisher = $7, homepage_url = $8, vcs_url = $9,
                modified_date = $10,
                deprecation_state = $11, deprecation_reason = $12, deprecated_date = $13
            WHERE uuid = $1
            "#,
        )
        .bind(component.uuid)
        .bind(&component.name)
        .bind(&component.description)
        .bind(serde_json::to_value(&component.identifiers).map_err(json_err)?)
        .bind(component_type_str(&component.component_type))
        .bind(serde_json::to_value(&component.licenses).map_err(json_err)?)
        .bind(&component.publisher)
        .bind(&component.homepage_url)
        .bind(&component.vcs_url)
        .bind(component.modified_date)
        .bind(db_state(component.deprecation.as_ref()))
        .bind(
            component
                .deprecation
                .as_ref()
                .and_then(|d| d.reason.as_deref()),
        )
        .bind(
            component
                .deprecation
                .as_ref()
                .and_then(|d| d.effective_date),
        )
        .execute(&self.pool)
        .await?;

        // C2 fix: detect missing entity
        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }
        Ok(())
    }

    async fn delete(&self, uuid: &Uuid) -> Result<(), RepositoryError> {
        sqlx::query("DELETE FROM tea_components WHERE uuid = $1")
            .bind(uuid)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn find_release_by_uuid(
        &self,
        uuid: &Uuid,
    ) -> Result<Option<ComponentRelease>, RepositoryError> {
        let row = sqlx::query(
            r#"
            SELECT uuid, component_uuid, version, release_date, pre_release, identifiers, distributions
            FROM tea_component_releases
            WHERE uuid = $1
            "#,
        )
        .bind(uuid)
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| map_release_row(&r)).transpose()
    }

    async fn find_releases_by_component(
        &self,
        component_uuid: &Uuid,
    ) -> Result<Vec<ComponentRelease>, RepositoryError> {
        let rows = sqlx::query(
            r#"
            SELECT uuid, component_uuid, version, release_date, pre_release, identifiers, distributions
            FROM tea_component_releases
            WHERE component_uuid = $1
            ORDER BY release_date DESC
            "#,
        )
        .bind(component_uuid)
        .fetch_all(&self.pool)
        .await?;

        rows.iter().map(map_release_row).collect()
    }

    async fn save_release(&self, release: &ComponentRelease) -> Result<(), RepositoryError> {
        sqlx::query(
            r#"
            INSERT INTO tea_component_releases
                (uuid, component_uuid, version, release_date, pre_release, identifiers, distributions)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(release.uuid)
        .bind(release.component_uuid)
        .bind(&release.version)
        .bind(release.release_date)
        .bind(release.pre_release)
        .bind(serde_json::to_value(&release.identifiers).map_err(json_err)?)
        .bind(serde_json::to_value(&release.distributions).map_err(json_err)?)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn update_release(&self, release: &ComponentRelease) -> Result<(), RepositoryError> {
        let result = sqlx::query(
            r#"
            UPDATE tea_component_releases
            SET component_uuid = $2, version = $3, release_date = $4,
                pre_release = $5, identifiers = $6, distributions = $7
            WHERE uuid = $1
            "#,
        )
        .bind(release.uuid)
        .bind(release.component_uuid)
        .bind(&release.version)
        .bind(release.release_date)
        .bind(release.pre_release)
        .bind(serde_json::to_value(&release.identifiers).map_err(json_err)?)
        .bind(serde_json::to_value(&release.distributions).map_err(json_err)?)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }
        Ok(())
    }

    async fn delete_release(&self, uuid: &Uuid) -> Result<(), RepositoryError> {
        sqlx::query("DELETE FROM tea_component_releases WHERE uuid = $1")
            .bind(uuid)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
