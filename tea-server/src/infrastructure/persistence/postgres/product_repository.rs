use async_trait::async_trait;
use sqlx::{PgPool, Postgres, Row, Transaction};
use uuid::Uuid;

use crate::domain::common::deprecation::{Deprecation, DeprecationState};
use crate::domain::common::error::RepositoryError;
use crate::domain::common::identifier::Identifier;
use crate::domain::product::entity::{ComponentRef, Contact, Product, ProductRelease, Vendor};
use crate::domain::product::repository::ProductRepository;

pub struct PostgresProductRepository {
    pool: PgPool,
}

impl PostgresProductRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn json_err(e: serde_json::Error) -> RepositoryError {
    RepositoryError::Database(sqlx::Error::Decode(e.into()))
}

async fn load_component_refs(
    pool: &PgPool,
    product_release_uuid: &Uuid,
) -> Result<Vec<ComponentRef>, RepositoryError> {
    let rows = sqlx::query(
        r#"
        SELECT component_uuid, release_uuid
        FROM component_references
        WHERE product_release_uuid = $1
        ORDER BY component_uuid, release_uuid
        "#,
    )
    .bind(product_release_uuid)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|row| {
            Ok(ComponentRef {
                component_uuid: row.try_get("component_uuid")?,
                release_uuid: row.try_get("release_uuid")?,
            })
        })
        .collect()
}

async fn replace_component_refs(
    tx: &mut Transaction<'_, Postgres>,
    release: &ProductRelease,
) -> Result<(), RepositoryError> {
    sqlx::query("DELETE FROM component_references WHERE product_release_uuid = $1")
        .bind(release.uuid)
        .execute(&mut **tx)
        .await?;

    for component in &release.components {
        sqlx::query(
            r#"
            INSERT INTO component_references (product_release_uuid, component_uuid, release_uuid)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(release.uuid)
        .bind(component.component_uuid)
        .bind(component.release_uuid)
        .execute(&mut **tx)
        .await?;
    }

    Ok(())
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

        row.map(|row| map_product_row(&row)).transpose()
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

        rows.iter().map(map_product_row).collect()
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
        .bind(db_state(product.deprecation.as_ref()))
        .bind(
            product
                .deprecation
                .as_ref()
                .and_then(|d| d.reason.as_deref()),
        )
        .bind(product.deprecation.as_ref().and_then(|d| d.effective_date))
        .bind(
            product
                .deprecation
                .as_ref()
                .and_then(|d| d.replacement_identifiers.first())
                .map(|id| id.id_value.as_str()),
        )
        .execute(&self.pool)
        .await?;

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
        .bind(db_state(product.deprecation.as_ref()))
        .bind(
            product
                .deprecation
                .as_ref()
                .and_then(|d| d.reason.as_deref()),
        )
        .bind(product.deprecation.as_ref().and_then(|d| d.effective_date))
        .bind(
            product
                .deprecation
                .as_ref()
                .and_then(|d| d.replacement_identifiers.first())
                .map(|id| id.id_value.as_str()),
        )
        .execute(&self.pool)
        .await?;

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

    async fn find_release_by_uuid(
        &self,
        uuid: &Uuid,
    ) -> Result<Option<ProductRelease>, RepositoryError> {
        let row = sqlx::query(
            r#"
            SELECT uuid, product_uuid, version, created_date, modified_date,
                   release_date, pre_release, identifiers
            FROM tea_product_releases
            WHERE uuid = $1
            "#,
        )
        .bind(uuid)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => {
                let components = load_component_refs(&self.pool, uuid).await?;
                Ok(Some(map_release_row(&row, components)?))
            }
            None => Ok(None),
        }
    }

    async fn find_releases_by_product(
        &self,
        product_uuid: &Uuid,
    ) -> Result<Vec<ProductRelease>, RepositoryError> {
        let rows = sqlx::query(
            r#"
            SELECT uuid, product_uuid, version, created_date, modified_date,
                   release_date, pre_release, identifiers
            FROM tea_product_releases
            WHERE product_uuid = $1
            ORDER BY release_date DESC NULLS LAST, created_date DESC
            "#,
        )
        .bind(product_uuid)
        .fetch_all(&self.pool)
        .await?;

        let mut releases = Vec::with_capacity(rows.len());
        for row in rows {
            let release_uuid: Uuid = row.try_get("uuid")?;
            let components = load_component_refs(&self.pool, &release_uuid).await?;
            releases.push(map_release_row(&row, components)?);
        }
        Ok(releases)
    }

    async fn save_release(&self, release: &ProductRelease) -> Result<(), RepositoryError> {
        let mut tx = self.pool.begin().await?;
        let result = sqlx::query(
            r#"
            INSERT INTO tea_product_releases (
                uuid, product_uuid, version, created_date, modified_date,
                release_date, pre_release, identifiers
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (uuid) DO NOTHING
            "#,
        )
        .bind(release.uuid)
        .bind(release.product_uuid)
        .bind(&release.version)
        .bind(release.created_date)
        .bind(release.modified_date)
        .bind(release.release_date)
        .bind(release.pre_release)
        .bind(serde_json::to_value(&release.identifiers).map_err(json_err)?)
        .execute(&mut *tx)
        .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::Conflict);
        }

        replace_component_refs(&mut tx, release).await?;
        tx.commit().await?;
        Ok(())
    }

    async fn update_release(&self, release: &ProductRelease) -> Result<(), RepositoryError> {
        let mut tx = self.pool.begin().await?;
        let result = sqlx::query(
            r#"
            UPDATE tea_product_releases
            SET product_uuid = $2,
                version = $3,
                modified_date = $4,
                release_date = $5,
                pre_release = $6,
                identifiers = $7
            WHERE uuid = $1
            "#,
        )
        .bind(release.uuid)
        .bind(release.product_uuid)
        .bind(&release.version)
        .bind(release.modified_date)
        .bind(release.release_date)
        .bind(release.pre_release)
        .bind(serde_json::to_value(&release.identifiers).map_err(json_err)?)
        .execute(&mut *tx)
        .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }

        replace_component_refs(&mut tx, release).await?;
        tx.commit().await?;
        Ok(())
    }

    async fn delete_release(&self, uuid: &Uuid) -> Result<(), RepositoryError> {
        let result = sqlx::query("DELETE FROM tea_product_releases WHERE uuid = $1")
            .bind(uuid)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }
        Ok(())
    }
}

fn map_product_row(row: &sqlx::postgres::PgRow) -> Result<Product, RepositoryError> {
    let identifiers: Vec<Identifier> =
        serde_json::from_value(row.try_get("identifiers")?).unwrap_or_default();
    let contacts: Vec<Contact> =
        serde_json::from_value(row.try_get("vendor_contacts")?).unwrap_or_default();

    let deprecation_state: Option<String> = row.try_get("deprecation_state")?;
    let deprecation = deprecation_state.map(|state| {
        let parsed_state = parse_state(&state);
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

fn map_release_row(
    row: &sqlx::postgres::PgRow,
    components: Vec<ComponentRef>,
) -> Result<ProductRelease, RepositoryError> {
    let identifiers: Vec<Identifier> =
        serde_json::from_value(row.try_get("identifiers")?).unwrap_or_default();

    Ok(ProductRelease {
        uuid: row.try_get("uuid")?,
        product_uuid: row.try_get("product_uuid")?,
        version: row.try_get("version")?,
        created_date: row.try_get("created_date")?,
        modified_date: row.try_get("modified_date")?,
        release_date: row.try_get("release_date")?,
        pre_release: row.try_get("pre_release")?,
        identifiers,
        components,
    })
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
