use async_trait::async_trait;
use sqlx::{PgPool, Postgres, Row, Transaction};
use uuid::Uuid;

use crate::domain::collection::entity::{Collection, CollectionScope, UpdateReason};
use crate::domain::collection::repository::CollectionRepository;
use crate::domain::common::deprecation::{Deprecation, DeprecationState};
use crate::domain::common::error::RepositoryError;
use crate::domain::common::identifier::Identifier;

pub struct PostgresCollectionRepository {
    pool: PgPool,
}

impl PostgresCollectionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

// ─── error helper ──────────────────────────────────────────────────────────
fn json_err(e: serde_json::Error) -> RepositoryError {
    RepositoryError::Database(sqlx::Error::Decode(e.into()))
}

async fn load_artifacts(
    pool: &PgPool,
    collection_uuid: Uuid,
    collection_version: i32,
) -> Result<Vec<Uuid>, RepositoryError> {
    let rows = sqlx::query(
        r#"
        SELECT artifact_uuid
        FROM collection_artifacts
        WHERE collection_uuid = $1 AND collection_version = $2
        ORDER BY position ASC, added_date ASC, artifact_uuid ASC
        "#,
    )
    .bind(collection_uuid)
    .bind(collection_version)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|row| {
            row.try_get("artifact_uuid")
                .map_err(RepositoryError::Database)
        })
        .collect()
}

async fn replace_artifacts(
    tx: &mut Transaction<'_, Postgres>,
    collection: &Collection,
) -> Result<(), RepositoryError> {
    sqlx::query(
        r#"
        DELETE FROM collection_artifacts
        WHERE collection_uuid = $1 AND collection_version = $2
        "#,
    )
    .bind(collection.uuid)
    .bind(collection.version)
    .execute(&mut **tx)
    .await?;

    for (position, artifact_uuid) in collection.artifacts.iter().enumerate() {
        sqlx::query(
            r#"
            INSERT INTO collection_artifacts (
                collection_uuid, collection_version, artifact_uuid, position
            )
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(collection.uuid)
        .bind(collection.version)
        .bind(artifact_uuid)
        .bind(position as i32)
        .execute(&mut **tx)
        .await?;
    }

    Ok(())
}

fn parse_scope(s: Option<&str>) -> CollectionScope {
    match s {
        Some("RELEASE") => CollectionScope::Release,
        Some("PRODUCT_RELEASE") => CollectionScope::ProductRelease,
        _ => CollectionScope::Unspecified,
    }
}

fn parse_reason(s: Option<&str>) -> UpdateReason {
    match s {
        Some("INITIAL_RELEASE") => UpdateReason::InitialRelease,
        Some("VEX_UPDATED") => UpdateReason::VexUpdated,
        Some("ARTIFACT_UPDATED") => UpdateReason::ArtifactUpdated,
        Some("ARTIFACT_REMOVED") => UpdateReason::ArtifactRemoved,
        Some("ARTIFACT_ADDED") => UpdateReason::ArtifactAdded,
        _ => UpdateReason::Unspecified,
    }
}

fn scope_str(s: &CollectionScope) -> &'static str {
    match s {
        CollectionScope::Release => "RELEASE",
        CollectionScope::ProductRelease => "PRODUCT_RELEASE",
        CollectionScope::Unspecified => "UNSPECIFIED",
    }
}

fn reason_str(r: &UpdateReason) -> &'static str {
    match r {
        UpdateReason::InitialRelease => "INITIAL_RELEASE",
        UpdateReason::VexUpdated => "VEX_UPDATED",
        UpdateReason::ArtifactUpdated => "ARTIFACT_UPDATED",
        UpdateReason::ArtifactRemoved => "ARTIFACT_REMOVED",
        UpdateReason::ArtifactAdded => "ARTIFACT_ADDED",
        UpdateReason::Unspecified => "UNSPECIFIED",
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

fn map_collection_row(row: &sqlx::postgres::PgRow) -> Result<Collection, RepositoryError> {
    let artifacts: Vec<Uuid> =
        serde_json::from_value(row.try_get("artifacts")?).unwrap_or_default();
    let belongs_to: Option<String> = row.try_get("belongs_to")?;
    let update_reason: Option<String> = row.try_get("update_reason")?;
    let deprecation_state: Option<String> = row.try_get("deprecation_state")?;
    let dependencies: Vec<Identifier> =
        serde_json::from_value(row.try_get("dependencies")?).unwrap_or_default();

    let deprecation = deprecation_state.map(|state| Deprecation {
        state: parse_state(&state),
        reason: row.try_get("deprecation_reason").ok().flatten(),
        announced_date: None,
        effective_date: row.try_get("deprecated_date").ok().flatten(),
        replacement_identifiers: vec![],
    });

    Ok(Collection {
        uuid: row.try_get("uuid")?,
        name: row.try_get("name")?,
        version: row.try_get("version")?,
        date: row.try_get("date")?,
        created_date: row.try_get("created_date")?,
        modified_date: row.try_get("modified_date")?,
        belongs_to: parse_scope(belongs_to.as_deref()),
        update_reason: parse_reason(update_reason.as_deref()),
        artifacts,
        deprecation,
        dependencies,
    })
}

async fn load_collection(
    pool: &PgPool,
    row: &sqlx::postgres::PgRow,
) -> Result<Collection, RepositoryError> {
    let mut collection = map_collection_row(row)?;
    let linked_artifacts = load_artifacts(pool, collection.uuid, collection.version).await?;
    if !linked_artifacts.is_empty() || collection.artifacts.is_empty() {
        collection.artifacts = linked_artifacts;
    }
    Ok(collection)
}

const SELECT_COLS: &str = r#"
    uuid, name, version, date, created_date, modified_date,
    belongs_to, update_reason, artifacts, dependencies,
    deprecation_state, deprecation_reason, deprecated_date
"#;

#[async_trait]
impl CollectionRepository for PostgresCollectionRepository {
    async fn find_by_uuid(&self, uuid: &Uuid) -> Result<Option<Collection>, RepositoryError> {
        let query = format!(
            "SELECT {SELECT_COLS} FROM tea_collections WHERE uuid = $1 ORDER BY version DESC LIMIT 1"
        );
        let row = sqlx::query(&query)
            .bind(uuid)
            .fetch_optional(&self.pool)
            .await?;

        match row {
            Some(row) => load_collection(&self.pool, &row).await.map(Some),
            None => Ok(None),
        }
    }

    async fn find_versions_by_uuid(&self, uuid: &Uuid) -> Result<Vec<Collection>, RepositoryError> {
        let query = format!(
            "SELECT {SELECT_COLS} FROM tea_collections WHERE uuid = $1 ORDER BY version DESC"
        );
        let rows = sqlx::query(&query).bind(uuid).fetch_all(&self.pool).await?;
        let mut collections = Vec::with_capacity(rows.len());
        for row in rows {
            collections.push(load_collection(&self.pool, &row).await?);
        }
        Ok(collections)
    }

    async fn find_by_name(&self, name: &str) -> Result<Vec<Collection>, RepositoryError> {
        let rows = if name.is_empty() {
            let query =
                format!("SELECT {SELECT_COLS} FROM tea_collections ORDER BY created_date DESC, version DESC");
            sqlx::query(&query).fetch_all(&self.pool).await?
        } else {
            let query = format!(
                "SELECT {SELECT_COLS} FROM tea_collections WHERE name ILIKE $1 ORDER BY created_date DESC, version DESC"
            );
            sqlx::query(&query)
                .bind(format!("%{name}%"))
                .fetch_all(&self.pool)
                .await?
        };
        let mut collections = Vec::with_capacity(rows.len());
        for row in rows {
            collections.push(load_collection(&self.pool, &row).await?);
        }
        Ok(collections)
    }

    async fn save(&self, collection: &Collection) -> Result<(), RepositoryError> {
        let mut tx = self.pool.begin().await?;
        let result = sqlx::query(
            r#"
            INSERT INTO tea_collections (
                uuid, name, version, date, created_date, modified_date,
                belongs_to, update_reason, artifacts, dependencies,
                deprecation_state, deprecation_reason, deprecated_date
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            ON CONFLICT (uuid, version) DO NOTHING
            "#,
        )
        .bind(collection.uuid)
        .bind(&collection.name)
        .bind(collection.version)
        .bind(collection.date)
        .bind(collection.created_date)
        .bind(collection.modified_date)
        .bind(scope_str(&collection.belongs_to))
        .bind(reason_str(&collection.update_reason))
        .bind(serde_json::to_value(&collection.artifacts).map_err(json_err)?)
        .bind(serde_json::to_value(&collection.dependencies).map_err(json_err)?)
        .bind(db_state(collection.deprecation.as_ref()))
        .bind(
            collection
                .deprecation
                .as_ref()
                .and_then(|d| d.reason.as_deref()),
        )
        .bind(
            collection
                .deprecation
                .as_ref()
                .and_then(|d| d.effective_date),
        )
        .execute(&mut *tx)
        .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::Conflict);
        }
        replace_artifacts(&mut tx, collection).await?;
        tx.commit().await?;
        Ok(())
    }

    async fn update(&self, collection: &Collection) -> Result<(), RepositoryError> {
        let mut tx = self.pool.begin().await?;
        let result = sqlx::query(
            r#"
            UPDATE tea_collections
            SET name = $2, version = $3, modified_date = $4,
                belongs_to = $5, update_reason = $6, artifacts = $7,
                dependencies = $8,
                deprecation_state = $9, deprecation_reason = $10, deprecated_date = $11
            WHERE uuid = $1 AND version = $3
            "#,
        )
        .bind(collection.uuid)
        .bind(&collection.name)
        .bind(collection.version)
        .bind(collection.modified_date)
        .bind(scope_str(&collection.belongs_to))
        .bind(reason_str(&collection.update_reason))
        .bind(serde_json::to_value(&collection.artifacts).map_err(json_err)?)
        .bind(serde_json::to_value(&collection.dependencies).map_err(json_err)?)
        .bind(db_state(collection.deprecation.as_ref()))
        .bind(
            collection
                .deprecation
                .as_ref()
                .and_then(|d| d.reason.as_deref()),
        )
        .bind(
            collection
                .deprecation
                .as_ref()
                .and_then(|d| d.effective_date),
        )
        .execute(&mut *tx)
        .await?;

        // C2 fix: detect missing entity
        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }
        replace_artifacts(&mut tx, collection).await?;
        tx.commit().await?;
        Ok(())
    }

    async fn delete(&self, uuid: &Uuid) -> Result<(), RepositoryError> {
        let result = sqlx::query("DELETE FROM tea_collections WHERE uuid = $1")
            .bind(uuid)
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }
        Ok(())
    }
}
