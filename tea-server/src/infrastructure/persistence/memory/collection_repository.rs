use async_trait::async_trait;
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::domain::collection::entity::Collection;
use crate::domain::collection::repository::CollectionRepository;
use crate::domain::common::error::RepositoryError;

pub struct InMemoryCollectionRepository {
    storage: Arc<RwLock<HashMap<Uuid, BTreeMap<i32, Collection>>>>,
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
        Ok(storage
            .get(uuid)
            .and_then(|versions| versions.iter().next_back().map(|(_, c)| c.clone())))
    }

    async fn find_versions_by_uuid(&self, uuid: &Uuid) -> Result<Vec<Collection>, RepositoryError> {
        let storage = self.storage.read().await;
        Ok(storage
            .get(uuid)
            .map(|versions| versions.values().cloned().collect())
            .unwrap_or_default())
    }

    async fn find_by_name(&self, name: &str) -> Result<Vec<Collection>, RepositoryError> {
        let storage = self.storage.read().await;
        let mut collections = storage
            .values()
            .flat_map(|versions| versions.values().cloned())
            .collect::<Vec<_>>();

        if !name.is_empty() {
            let query = name.to_lowercase();
            collections.retain(|collection| collection.name.to_lowercase().contains(&query));
        }

        Ok(collections)
    }

    async fn save(&self, collection: &Collection) -> Result<(), RepositoryError> {
        let mut storage = self.storage.write().await;
        let versions = storage.entry(collection.uuid).or_insert_with(BTreeMap::new);
        if versions.contains_key(&collection.version) {
            return Err(RepositoryError::Conflict);
        }
        versions.insert(collection.version, collection.clone());
        Ok(())
    }

    async fn update(&self, collection: &Collection) -> Result<(), RepositoryError> {
        let mut storage = self.storage.write().await;
        let Some(versions) = storage.get_mut(&collection.uuid) else {
            return Err(RepositoryError::NotFound);
        };
        if !versions.contains_key(&collection.version) {
            return Err(RepositoryError::NotFound);
        }
        versions.insert(collection.version, collection.clone());
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
