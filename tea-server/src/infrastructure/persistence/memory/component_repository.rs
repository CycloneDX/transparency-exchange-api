use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::domain::common::error::RepositoryError;
use crate::domain::component::entity::{Component, ComponentRelease};
use crate::domain::component::repository::ComponentRepository;

pub struct InMemoryComponentRepository {
    component_storage: Arc<RwLock<HashMap<Uuid, Component>>>,
    release_storage: Arc<RwLock<HashMap<Uuid, ComponentRelease>>>,
}

impl InMemoryComponentRepository {
    pub fn new() -> Self {
        Self {
            component_storage: Arc::new(RwLock::new(HashMap::new())),
            release_storage: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryComponentRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ComponentRepository for InMemoryComponentRepository {
    async fn find_by_uuid(&self, uuid: &Uuid) -> Result<Option<Component>, RepositoryError> {
        let storage = self.component_storage.read().await;
        Ok(storage.get(uuid).cloned())
    }

    async fn find_by_name(&self, name: &str) -> Result<Vec<Component>, RepositoryError> {
        let storage = self.component_storage.read().await;
        let components: Vec<Component> = if name.is_empty() {
            storage.values().cloned().collect()
        } else {
            storage
                .values()
                .filter(|c| c.name.to_lowercase().contains(&name.to_lowercase()))
                .cloned()
                .collect()
        };
        Ok(components)
    }

    async fn save(&self, component: &Component) -> Result<(), RepositoryError> {
        let mut storage = self.component_storage.write().await;
        storage.insert(component.uuid, component.clone());
        Ok(())
    }

    async fn update(&self, component: &Component) -> Result<(), RepositoryError> {
        let mut storage = self.component_storage.write().await;
        if !storage.contains_key(&component.uuid) {
            return Err(RepositoryError::NotFound);
        }
        storage.insert(component.uuid, component.clone());
        Ok(())
    }

    async fn delete(&self, uuid: &Uuid) -> Result<(), RepositoryError> {
        let mut storage = self.component_storage.write().await;
        storage.remove(uuid);
        Ok(())
    }

    async fn find_release_by_uuid(&self, uuid: &Uuid) -> Result<Option<ComponentRelease>, RepositoryError> {
        let storage = self.release_storage.read().await;
        Ok(storage.get(uuid).cloned())
    }

    async fn find_releases_by_component(&self, component_uuid: &Uuid) -> Result<Vec<ComponentRelease>, RepositoryError> {
        let storage = self.release_storage.read().await;
        let releases: Vec<ComponentRelease> = storage
            .values()
            .filter(|r| r.component_uuid == *component_uuid)
            .cloned()
            .collect();
        Ok(releases)
    }

    async fn save_release(&self, release: &ComponentRelease) -> Result<(), RepositoryError> {
        let mut storage = self.release_storage.write().await;
        storage.insert(release.uuid, release.clone());
        Ok(())
    }

    async fn update_release(&self, release: &ComponentRelease) -> Result<(), RepositoryError> {
        let mut storage = self.release_storage.write().await;
        if !storage.contains_key(&release.uuid) {
            return Err(RepositoryError::NotFound);
        }
        storage.insert(release.uuid, release.clone());
        Ok(())
    }

    async fn delete_release(&self, uuid: &Uuid) -> Result<(), RepositoryError> {
        let mut storage = self.release_storage.write().await;
        // M5 fix: distinguish "deleted" from "never existed"
        if storage.remove(uuid).is_none() {
            return Err(RepositoryError::NotFound);
        }
        Ok(())
    }
}
