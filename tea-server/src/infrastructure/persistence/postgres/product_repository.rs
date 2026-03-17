use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::common::deprecation::{Deprecation, DeprecationState};
use crate::domain::common::error::RepositoryError;
use crate::domain::common::identifier::Identifier;
use crate::domain::product::entity::{Contact, Product, Vendor};
use crate::domain::product::repository::ProductRepository;

pub struct PostgresProductRepository {
    pool: PgPool,
}

impl PostgresProductRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

// ─── helper: map serde_json error to RepositoryError ───────────────────────
fn json_err(e: serde_json::Error) -> RepositoryError {
    RepositoryError::Database(sqlx::Error::Decode(e.into()))
}

#[async_trait]
impl ProductRepository for PostgresProductRepository {
    async fn find_by_uuid(&self, uuid: &Uuid) -> Result<Option<Product>, RepositoryError> {
        let row = sqlx::query(
            r#"
            SELECT uuid, name, description, identifiers,
                   vendor_name, vendor_uuid, vendor_url, vendor_contacts,
                   created_date, modified_date,
                   homepage_url, documentation_url, vcs_url,
                   deprecation_state, deprecation_reason, deprecated_date,
                   deprecated_successor_url
            FROM tea_products
            WHERE uuid = $1
            "#,
        )
        .bind(uuid)
        .fetch_optional(&self.pool)
        .await?;

        row.map(|row| map_row(&row)).transpose()
    }

    async fn find_by_name(&self, name: &str) -> Result<Vec<Product>, RepositoryError> {
        let rows = if name.is_empty() {
            sqlx::query(
                r#"
                SELECT uuid, name, description, identifiers,
                       vendor_name, vendor_uuid, vendor_url, vendor_contacts,
                       created_date, modified_date,
                       homepage_url, documentation_url, vcs_url,
                       deprecation_state, deprecation_reason, deprecated_date,
                       deprecated_successor_url
                FROM tea_products
                ORDER BY created_date DESC
                "#,
            )
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query(
                r#"
                SELECT uuid, name, description, identifiers,
                       vendor_name, vendor_uuid, vendor_url, vendor_contacts,
                       created_date, modified_date,
                       homepage_url, documentation_url, vcs_url,
                       deprecation_state, deprecation_reason, deprecated_date,
                       deprecated_successor_url
                FROM tea_products
                WHERE name ILIKE $1
                ORDER BY created_date DESC
                "#,
            )
            .bind(format!("%{name}%"))
            .fetch_all(&self.pool)
            .await?
        };

        rows.iter().map(map_row).collect()
    }

    async fn save(&self, product: &Product) -> Result<(), RepositoryError> {
        let result = sqlx::query(
            r#"
            INSERT INTO tea_products (
                uuid, name, description, identifiers,
                vendor_name, vendor_uuid, vendor_url, vendor_contacts,
                created_date, modified_date,
                homepage_url, documentation_url, vcs_url,
                deprecation_state, deprecation_reason, deprecated_date,
                deprecated_successor_url
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
            ON CONFLICT (uuid) DO NOTHING
            "#,
        )
        .bind(product.uuid)
        .bind(&product.name)
        .bind(&product.description)
        .bind(serde_json::to_value(&product.identifiers).map_err(json_err)?)
        .bind(&product.vendor.name)
        .bind(product.vendor.uuid)
        .bind(&product.vendor.url)
        .bind(serde_json::to_value(&product.vendor.contacts).map_err(json_err)?)
        .bind(product.created_date)
        .bind(product.modified_date)
        .bind(&product.homepage_url)
        .bind(&product.documentation_url)
        .bind(&product.vcs_url)
        .bind(product.deprecation.as_ref().map(|d| format!("{:?}", d.state)))
        .bind(product.deprecation.as_ref().and_then(|d| d.reason.as_deref()))
        .bind(product.deprecation.as_ref().and_then(|d| d.effective_date))
        .bind(product.deprecation.as_ref().and_then(|d| d.replacement_identifiers.first()).map(|id| id.id_value.as_str()))
        .execute(&self.pool)
        .await?;

        // H4 fix: ON CONFLICT DO NOTHING means rows_affected = 0 on duplicate
        if result.rows_affected() == 0 {
            return Err(RepositoryError::Conflict);
        }
        Ok(())
    }

    async fn update(&self, product: &Product) -> Result<(), RepositoryError> {
        let result = sqlx::query(
            r#"
            UPDATE tea_products
            SET name = $2, description = $3, identifiers = $4,
                vendor_name = $5, vendor_uuid = $6, vendor_url = $7, vendor_contacts = $8,
                modified_date = $9,
                homepage_url = $10, documentation_url = $11, vcs_url = $12,
                deprecation_state = $13, deprecation_reason = $14,
                deprecated_date = $15, deprecated_successor_url = $16
            WHERE uuid = $1
            "#,
        )
        .bind(product.uuid)
        .bind(&product.name)
        .bind(&product.description)
        .bind(serde_json::to_value(&product.identifiers).map_err(json_err)?)
        .bind(&product.vendor.name)
        .bind(product.vendor.uuid)
        .bind(&product.vendor.url)
        .bind(serde_json::to_value(&product.vendor.contacts).map_err(json_err)?)
        .bind(product.modified_date)
        .bind(&product.homepage_url)
        .bind(&product.documentation_url)
        .bind(&product.vcs_url)
        .bind(product.deprecation.as_ref().map(|d| format!("{:?}", d.state)))
        .bind(product.deprecation.as_ref().and_then(|d| d.reason.as_deref()))
        .bind(product.deprecation.as_ref().and_then(|d| d.effective_date))
        .bind(product.deprecation.as_ref().and_then(|d| d.replacement_identifiers.first()).map(|id| id.id_value.as_str()))
        .execute(&self.pool)
        .await?;

        // C2 fix: detect missing entity — UPDATE affecting 0 rows is NotFound
        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }
        Ok(())
    }

    async fn delete(&self, uuid: &Uuid) -> Result<(), RepositoryError> {
        sqlx::query("DELETE FROM tea_products WHERE uuid = $1")
            .bind(uuid)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

// ─── row mapper ────────────────────────────────────────────────────────────
fn map_row(row: &sqlx::postgres::PgRow) -> Result<Product, RepositoryError> {
    use sqlx::Row;

    let identifiers: Vec<Identifier> =
        serde_json::from_value(row.try_get("identifiers")?).unwrap_or_default();
    let contacts: Vec<Contact> =
        serde_json::from_value(row.try_get("vendor_contacts")?).unwrap_or_default();

    let deprecation_state: Option<String> = row.try_get("deprecation_state")?;
    let deprecation = deprecation_state.map(|state| {
        let parsed_state = match state.as_str() {
            "Active" => DeprecationState::Active,
            "Deprecated" => DeprecationState::Deprecated,
            "Retired" => DeprecationState::Retired,
            _ => DeprecationState::Unspecified,
        };
        Deprecation {
            state: parsed_state,
            reason: row.try_get("deprecation_reason").ok().flatten(),
            announced_date: None,
            effective_date: row.try_get("deprecated_date").ok().flatten(),
            replacement_identifiers: vec![],
        }
    });

    Ok(Product {
        uuid: row.try_get("uuid")?,
        name: row.try_get("name")?,
        description: row.try_get("description")?,
        identifiers,
        vendor: Vendor {
            name: row.try_get("vendor_name")?,
            uuid: row.try_get("vendor_uuid")?,
            url: row.try_get("vendor_url")?,
            contacts,
        },
        created_date: row.try_get("created_date")?,
        modified_date: row.try_get("modified_date")?,
        homepage_url: row.try_get("homepage_url")?,
        documentation_url: row.try_get("documentation_url")?,
        vcs_url: row.try_get("vcs_url")?,
        deprecation,
        dependencies: vec![],
    })
}
