use async_trait::async_trait;
use uuid::Uuid;

use super::entity::{Component, ComponentRelease};
use crate::domain::common::error::RepositoryError;

#[async_trait]
pub trait ComponentRepository {
    async fn find_by_uuid(&self, uuid: &Uuid) -> Result<Option<Component>, RepositoryError>;
    async fn find_by_name(&self, name: &str) -> Result<Vec<Component>, RepositoryError>;
    async fn save(&self, component: &Component) -> Result<(), RepositoryError>;
    async fn update(&self, component: &Component) -> Result<(), RepositoryError>;
    async fn delete(&self, uuid: &Uuid) -> Result<(), RepositoryError>;

    async fn find_release_by_uuid(&self, uuid: &Uuid) -> Result<Option<ComponentRelease>, RepositoryError>;
    async fn find_releases_by_component(&self, component_uuid: &Uuid) -> Result<Vec<ComponentRelease>, RepositoryError>;
    async fn save_release(&self, release: &ComponentRelease) -> Result<(), RepositoryError>;
    async fn update_release(&self, release: &ComponentRelease) -> Result<(), RepositoryError>;
    async fn delete_release(&self, uuid: &Uuid) -> Result<(), RepositoryError>;
}
