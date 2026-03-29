use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::domain::common::error::RepositoryError;
use crate::domain::product::entity::{Product, ProductRelease};
use crate::domain::product::repository::ProductRepository;

pub struct InMemoryProductRepository {
    product_storage: Arc<RwLock<HashMap<Uuid, Product>>>,
    release_storage: Arc<RwLock<HashMap<Uuid, ProductRelease>>>,
}

impl InMemoryProductRepository {
    pub fn new() -> Self {
        Self {
            product_storage: Arc::new(RwLock::new(HashMap::new())),
            release_storage: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryProductRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ProductRepository for InMemoryProductRepository {
    async fn find_by_uuid(&self, uuid: &Uuid) -> Result<Option<Product>, RepositoryError> {
        let storage = self.product_storage.read().await;
        Ok(storage.get(uuid).cloned())
    }

    async fn find_by_name(&self, name: &str) -> Result<Vec<Product>, RepositoryError> {
        let storage = self.product_storage.read().await;
        let products: Vec<Product> = if name.is_empty() {
            storage.values().cloned().collect()
        } else {
            storage
                .values()
                .filter(|p| p.name.to_lowercase().contains(&name.to_lowercase()))
                .cloned()
                .collect()
        };
        Ok(products)
    }

    async fn save(&self, product: &Product) -> Result<(), RepositoryError> {
        let mut storage = self.product_storage.write().await;
        storage.insert(product.uuid, product.clone());
        Ok(())
    }

    async fn update(&self, product: &Product) -> Result<(), RepositoryError> {
        let mut storage = self.product_storage.write().await;
        if !storage.contains_key(&product.uuid) {
            return Err(RepositoryError::NotFound);
        }
        storage.insert(product.uuid, product.clone());
        Ok(())
    }

    async fn delete(&self, uuid: &Uuid) -> Result<(), RepositoryError> {
        let mut storage = self.product_storage.write().await;
        // M5 fix: distinguish "deleted" from "never existed"
        if storage.remove(uuid).is_none() {
            return Err(RepositoryError::NotFound);
        }
        Ok(())
    }

    async fn find_release_by_uuid(
        &self,
        uuid: &Uuid,
    ) -> Result<Option<ProductRelease>, RepositoryError> {
        let storage = self.release_storage.read().await;
        Ok(storage.get(uuid).cloned())
    }

    async fn find_releases_by_product(
        &self,
        product_uuid: &Uuid,
    ) -> Result<Vec<ProductRelease>, RepositoryError> {
        let storage = self.release_storage.read().await;
        Ok(storage
            .values()
            .filter(|release| release.product_uuid == *product_uuid)
            .cloned()
            .collect())
    }

    async fn save_release(&self, release: &ProductRelease) -> Result<(), RepositoryError> {
        let mut storage = self.release_storage.write().await;
        storage.insert(release.uuid, release.clone());
        Ok(())
    }

    async fn update_release(&self, release: &ProductRelease) -> Result<(), RepositoryError> {
        let mut storage = self.release_storage.write().await;
        if !storage.contains_key(&release.uuid) {
            return Err(RepositoryError::NotFound);
        }
        storage.insert(release.uuid, release.clone());
        Ok(())
    }

    async fn delete_release(&self, uuid: &Uuid) -> Result<(), RepositoryError> {
        let mut storage = self.release_storage.write().await;
        if storage.remove(uuid).is_none() {
            return Err(RepositoryError::NotFound);
        }
        Ok(())
    }
}
