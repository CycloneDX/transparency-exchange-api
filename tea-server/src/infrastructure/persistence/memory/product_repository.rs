use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::domain::common::error::RepositoryError;
use crate::domain::product::entity::Product;
use crate::domain::product::repository::ProductRepository;

pub struct InMemoryProductRepository {
    storage: Arc<RwLock<HashMap<Uuid, Product>>>,
}

impl InMemoryProductRepository {
    pub fn new() -> Self {
        Self {
            storage: Arc::new(RwLock::new(HashMap::new())),
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
        let storage = self.storage.read().await;
        Ok(storage.get(uuid).cloned())
    }

    async fn find_by_name(&self, name: &str) -> Result<Vec<Product>, RepositoryError> {
        let storage = self.storage.read().await;
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
        let mut storage = self.storage.write().await;
        storage.insert(product.uuid, product.clone());
        Ok(())
    }

    async fn update(&self, product: &Product) -> Result<(), RepositoryError> {
        let mut storage = self.storage.write().await;
        if !storage.contains_key(&product.uuid) {
            return Err(RepositoryError::NotFound);
        }
        storage.insert(product.uuid, product.clone());
        Ok(())
    }

    async fn delete(&self, uuid: &Uuid) -> Result<(), RepositoryError> {
        let mut storage = self.storage.write().await;
        // M5 fix: distinguish "deleted" from "never existed"
        if storage.remove(uuid).is_none() {
            return Err(RepositoryError::NotFound);
        }
        Ok(())
    }
}
