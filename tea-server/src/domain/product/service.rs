use chrono::Utc;
use uuid::Uuid;

use super::entity::Product;
use super::entity::ProductRelease;
use super::repository::ProductRepository;
use crate::domain::common::deprecation::Deprecation;
use crate::domain::common::error::DomainError;
use crate::domain::common::pagination::{Page, PaginationParams};
use crate::domain::common::validation::{
    validate_max_len, validate_non_empty, validate_optional_url,
};

pub struct ProductService<R: ProductRepository> {
    repository: R,
}

impl<R> ProductService<R>
where
    R: ProductRepository + Send + Sync,
{
    fn validate_product(product: &Product) -> Result<(), DomainError> {
        validate_non_empty("name", &product.name)?;
        validate_max_len("name", &product.name, 4096)?;
        if let Some(desc) = &product.description {
            validate_max_len("description", desc, 65536)?;
        }
        validate_optional_url("homepage_url", &product.homepage_url)?;
        validate_optional_url("documentation_url", &product.documentation_url)?;
        validate_optional_url("vcs_url", &product.vcs_url)?;
        validate_non_empty("vendor.name", &product.vendor.name)?;
        validate_max_len("vendor.name", &product.vendor.name, 1024)?;
        Ok(())
    }

    fn validate_release(release: &ProductRelease) -> Result<(), DomainError> {
        if release.product_uuid.is_nil() {
            return Err(DomainError::Validation(
                "product_uuid is required".to_string(),
            ));
        }
        validate_non_empty("version", &release.version)?;
        validate_max_len("version", &release.version, 256)?;

        for (index, component) in release.components.iter().enumerate() {
            if component.component_uuid.is_nil() {
                return Err(DomainError::Validation(format!(
                    "components[{index}].component_uuid is required"
                )));
            }
            if component.release_uuid.is_nil() {
                return Err(DomainError::Validation(format!(
                    "components[{index}].release_uuid is required"
                )));
            }
        }
        Ok(())
    }

    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn get_product(&self, uuid: &Uuid) -> Result<Option<Product>, DomainError> {
        self.repository
            .find_by_uuid(uuid)
            .await
            .map_err(DomainError::Repository)
    }

    pub async fn list_products(
        &self,
        params: PaginationParams,
    ) -> Result<Page<Product>, DomainError> {
        let all = self
            .repository
            .find_by_name("")
            .await
            .map_err(DomainError::Repository)?;
        Ok(Page::new(all, &params))
    }

    pub async fn create_product(&self, mut product: Product) -> Result<Product, DomainError> {
        Self::validate_product(&product)?;

        if product.uuid.is_nil() {
            product.uuid = Uuid::new_v4();
        }
        product.created_date = Utc::now();
        product.modified_date = Utc::now();

        self.repository
            .save(&product)
            .await
            .map_err(DomainError::Repository)?;
        Ok(product)
    }

    pub async fn update_product(&self, mut product: Product) -> Result<Product, DomainError> {
        Self::validate_product(&product)?;
        product.modified_date = Utc::now();
        self.repository
            .update(&product)
            .await
            .map_err(DomainError::Repository)?;
        Ok(product)
    }

    pub async fn delete_product(&self, uuid: &Uuid) -> Result<(), DomainError> {
        self.repository
            .delete(uuid)
            .await
            .map_err(DomainError::Repository)
    }

    pub async fn deprecate_product(
        &self,
        uuid: &Uuid,
        deprecation: Deprecation,
    ) -> Result<Product, DomainError> {
        let mut product = self
            .repository
            .find_by_uuid(uuid)
            .await
            .map_err(DomainError::Repository)?
            .ok_or_else(|| DomainError::NotFound(format!("Product {uuid} not found")))?;

        product.deprecation = Some(deprecation.clone());
        product.modified_date = Utc::now();

        self.repository
            .update(&product)
            .await
            .map_err(DomainError::Repository)?;

        tracing::info!(product_uuid = %uuid, "Product deprecated");
        Ok(product)
    }

    pub async fn get_dependents(&self, _uuid: &Uuid) -> Result<Vec<Uuid>, DomainError> {
        // TODO(cross-domain): implement via a DependencyResolver port
        Ok(vec![])
    }

    pub async fn get_release(&self, uuid: &Uuid) -> Result<Option<ProductRelease>, DomainError> {
        self.repository
            .find_release_by_uuid(uuid)
            .await
            .map_err(DomainError::Repository)
    }

    pub async fn list_releases(
        &self,
        product_uuid: &Uuid,
    ) -> Result<Vec<ProductRelease>, DomainError> {
        self.repository
            .find_releases_by_product(product_uuid)
            .await
            .map_err(DomainError::Repository)
    }

    pub async fn create_release(
        &self,
        mut release: ProductRelease,
    ) -> Result<ProductRelease, DomainError> {
        Self::validate_release(&release)?;

        if release.uuid.is_nil() {
            release.uuid = Uuid::new_v4();
        }
        release.created_date = Utc::now();
        release.modified_date = Utc::now();

        self.repository
            .save_release(&release)
            .await
            .map_err(DomainError::Repository)?;
        Ok(release)
    }

    pub async fn update_release(
        &self,
        mut release: ProductRelease,
    ) -> Result<ProductRelease, DomainError> {
        Self::validate_release(&release)?;
        release.modified_date = Utc::now();
        self.repository
            .update_release(&release)
            .await
            .map_err(DomainError::Repository)?;
        Ok(release)
    }

    pub async fn delete_release(&self, uuid: &Uuid) -> Result<(), DomainError> {
        self.repository
            .delete_release(uuid)
            .await
            .map_err(DomainError::Repository)
    }
}
