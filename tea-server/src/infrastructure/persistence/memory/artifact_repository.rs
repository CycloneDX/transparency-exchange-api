use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::domain::artifact::entity::Artifact;
use crate::domain::artifact::repository::ArtifactRepository;
use crate::domain::common::error::RepositoryError;

pub struct InMemoryArtifactRepository {
    storage: Arc<RwLock<HashMap<Uuid, Artifact>>>,
}

impl InMemoryArtifactRepository {
    pub fn new() -> Self {
        Self {
            storage: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryArtifactRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ArtifactRepository for InMemoryArtifactRepository {
    async fn find_by_uuid(&self, uuid: &Uuid) -> Result<Option<Artifact>, RepositoryError> {
        let storage = self.storage.read().await;
        Ok(storage.get(uuid).cloned())
    }

    async fn find_by_name(&self, name: &str) -> Result<Vec<Artifact>, RepositoryError> {
        let storage = self.storage.read().await;
        let artifacts: Vec<Artifact> = if name.is_empty() {
            storage.values().cloned().collect()
        } else {
            storage
                .values()
                .filter(|a| a.name.to_lowercase().contains(&name.to_lowercase()))
                .cloned()
                .collect()
        };
        Ok(artifacts)
    }

    async fn save(&self, artifact: &Artifact) -> Result<(), RepositoryError> {
        let mut storage = self.storage.write().await;
        storage.insert(artifact.uuid, artifact.clone());
        Ok(())
    }

    async fn update(&self, artifact: &Artifact) -> Result<(), RepositoryError> {
        let mut storage = self.storage.write().await;
        if !storage.contains_key(&artifact.uuid) {
            return Err(RepositoryError::NotFound);
        }
        storage.insert(artifact.uuid, artifact.clone());
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
