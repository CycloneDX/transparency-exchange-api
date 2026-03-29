use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

use super::entity::{Product, ProductRelease};
use crate::domain::common::error::RepositoryError;

#[async_trait]
pub trait ProductRepository {
    async fn find_by_uuid(&self, uuid: &Uuid) -> Result<Option<Product>, RepositoryError>;
    async fn find_by_name(&self, name: &str) -> Result<Vec<Product>, RepositoryError>;
    async fn save(&self, product: &Product) -> Result<(), RepositoryError>;
    async fn update(&self, product: &Product) -> Result<(), RepositoryError>;
    async fn delete(&self, uuid: &Uuid) -> Result<(), RepositoryError>;

    async fn find_release_by_uuid(
        &self,
        uuid: &Uuid,
    ) -> Result<Option<ProductRelease>, RepositoryError>;
    async fn find_releases_by_product(
        &self,
        product_uuid: &Uuid,
    ) -> Result<Vec<ProductRelease>, RepositoryError>;
    async fn save_release(&self, release: &ProductRelease) -> Result<(), RepositoryError>;
    async fn update_release(&self, release: &ProductRelease) -> Result<(), RepositoryError>;
    async fn delete_release(&self, uuid: &Uuid) -> Result<(), RepositoryError>;
}

#[async_trait]
impl<T> ProductRepository for Box<T>
where
    T: ProductRepository + ?Sized + Send + Sync,
{
    async fn find_by_uuid(&self, uuid: &Uuid) -> Result<Option<Product>, RepositoryError> {
        (**self).find_by_uuid(uuid).await
    }

    async fn find_by_name(&self, name: &str) -> Result<Vec<Product>, RepositoryError> {
        (**self).find_by_name(name).await
    }

    async fn save(&self, product: &Product) -> Result<(), RepositoryError> {
        (**self).save(product).await
    }

    async fn update(&self, product: &Product) -> Result<(), RepositoryError> {
        (**self).update(product).await
    }

    async fn delete(&self, uuid: &Uuid) -> Result<(), RepositoryError> {
        (**self).delete(uuid).await
    }

    async fn find_release_by_uuid(
        &self,
        uuid: &Uuid,
    ) -> Result<Option<ProductRelease>, RepositoryError> {
        (**self).find_release_by_uuid(uuid).await
    }

    async fn find_releases_by_product(
        &self,
        product_uuid: &Uuid,
    ) -> Result<Vec<ProductRelease>, RepositoryError> {
        (**self).find_releases_by_product(product_uuid).await
    }

    async fn save_release(&self, release: &ProductRelease) -> Result<(), RepositoryError> {
        (**self).save_release(release).await
    }

    async fn update_release(&self, release: &ProductRelease) -> Result<(), RepositoryError> {
        (**self).update_release(release).await
    }

    async fn delete_release(&self, uuid: &Uuid) -> Result<(), RepositoryError> {
        (**self).delete_release(uuid).await
    }
}

#[async_trait]
impl<T> ProductRepository for Arc<T>
where
    T: ProductRepository + ?Sized + Send + Sync,
{
    async fn find_by_uuid(&self, uuid: &Uuid) -> Result<Option<Product>, RepositoryError> {
        (**self).find_by_uuid(uuid).await
    }

    async fn find_by_name(&self, name: &str) -> Result<Vec<Product>, RepositoryError> {
        (**self).find_by_name(name).await
    }

    async fn save(&self, product: &Product) -> Result<(), RepositoryError> {
        (**self).save(product).await
    }

    async fn update(&self, product: &Product) -> Result<(), RepositoryError> {
        (**self).update(product).await
    }

    async fn delete(&self, uuid: &Uuid) -> Result<(), RepositoryError> {
        (**self).delete(uuid).await
    }

    async fn find_release_by_uuid(
        &self,
        uuid: &Uuid,
    ) -> Result<Option<ProductRelease>, RepositoryError> {
        (**self).find_release_by_uuid(uuid).await
    }

    async fn find_releases_by_product(
        &self,
        product_uuid: &Uuid,
    ) -> Result<Vec<ProductRelease>, RepositoryError> {
        (**self).find_releases_by_product(product_uuid).await
    }

    async fn save_release(&self, release: &ProductRelease) -> Result<(), RepositoryError> {
        (**self).save_release(release).await
    }

    async fn update_release(&self, release: &ProductRelease) -> Result<(), RepositoryError> {
        (**self).update_release(release).await
    }

    async fn delete_release(&self, uuid: &Uuid) -> Result<(), RepositoryError> {
        (**self).delete_release(uuid).await
    }
}
