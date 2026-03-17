use chrono::Utc;
use uuid::Uuid;

use super::entity::Product;
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
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn get_product(&self, uuid: &Uuid) -> Result<Option<Product>, DomainError> {
        self.repository
            .find_by_uuid(uuid)
            .await
            .map_err(DomainError::Repository)
    }

    pub async fn list_products(&self, params: PaginationParams) -> Result<Page<Product>, DomainError> {
        let all = self.repository.find_by_name("").await.map_err(DomainError::Repository)?;
        Ok(Page::new(all, &params))
    }

    pub async fn create_product(&self, mut product: Product) -> Result<Product, DomainError> {
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

        product.uuid = Uuid::new_v4();
        product.created_date = Utc::now();
        product.modified_date = Utc::now();

        self.repository
            .save(&product)
            .await
            .map_err(DomainError::Repository)?;
        Ok(product)
    }

    pub async fn update_product(&self, mut product: Product) -> Result<Product, DomainError> {
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
}
