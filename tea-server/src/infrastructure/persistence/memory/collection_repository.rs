use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::domain::common::error::RepositoryError;
use crate::domain::collection::entity::Collection;
use crate::domain::collection::repository::CollectionRepository;

pub struct InMemoryCollectionRepository {
    storage: Arc<RwLock<HashMap<Uuid, Collection>>>,
}

impl InMemoryCollectionRepository {
    pub fn new() -> Self {
        Self {
            storage: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryCollectionRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CollectionRepository for InMemoryCollectionRepository {
    async fn find_by_uuid(&self, uuid: &Uuid) -> Result<Option<Collection>, RepositoryError> {
        let storage = self.storage.read().await;
        Ok(storage.get(uuid).cloned())
    }

    async fn find_by_name(&self, name: &str) -> Result<Vec<Collection>, RepositoryError> {
        let storage = self.storage.read().await;
        let collections: Vec<Collection> = if name.is_empty() {
            storage.values().cloned().collect()
        } else {
            storage
                .values()
                .filter(|c| c.name.to_lowercase().contains(&name.to_lowercase()))
                .cloned()
                .collect()
        };
        Ok(collections)
    }

    async fn save(&self, collection: &Collection) -> Result<(), RepositoryError> {
        let mut storage = self.storage.write().await;
        storage.insert(collection.uuid, collection.clone());
        Ok(())
    }

    async fn update(&self, collection: &Collection) -> Result<(), RepositoryError> {
        let mut storage = self.storage.write().await;
        if !storage.contains_key(&collection.uuid) {
            return Err(RepositoryError::NotFound);
        }
        storage.insert(collection.uuid, collection.clone());
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
