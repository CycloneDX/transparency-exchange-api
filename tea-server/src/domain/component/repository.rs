use async_trait::async_trait;
use std::sync::Arc;
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

    async fn find_release_by_uuid(
        &self,
        uuid: &Uuid,
    ) -> Result<Option<ComponentRelease>, RepositoryError>;
    async fn find_releases_by_component(
        &self,
        component_uuid: &Uuid,
    ) -> Result<Vec<ComponentRelease>, RepositoryError>;
    async fn save_release(&self, release: &ComponentRelease) -> Result<(), RepositoryError>;
    async fn update_release(&self, release: &ComponentRelease) -> Result<(), RepositoryError>;
    async fn delete_release(&self, uuid: &Uuid) -> Result<(), RepositoryError>;
}

#[async_trait]
impl<T> ComponentRepository for Box<T>
where
    T: ComponentRepository + ?Sized + Send + Sync,
{
    async fn find_by_uuid(&self, uuid: &Uuid) -> Result<Option<Component>, RepositoryError> {
        (**self).find_by_uuid(uuid).await
    }

    async fn find_by_name(&self, name: &str) -> Result<Vec<Component>, RepositoryError> {
        (**self).find_by_name(name).await
    }

    async fn save(&self, component: &Component) -> Result<(), RepositoryError> {
        (**self).save(component).await
    }

    async fn update(&self, component: &Component) -> Result<(), RepositoryError> {
        (**self).update(component).await
    }

    async fn delete(&self, uuid: &Uuid) -> Result<(), RepositoryError> {
        (**self).delete(uuid).await
    }

    async fn find_release_by_uuid(
        &self,
        uuid: &Uuid,
    ) -> Result<Option<ComponentRelease>, RepositoryError> {
        (**self).find_release_by_uuid(uuid).await
    }

    async fn find_releases_by_component(
        &self,
        component_uuid: &Uuid,
    ) -> Result<Vec<ComponentRelease>, RepositoryError> {
        (**self).find_releases_by_component(component_uuid).await
    }

    async fn save_release(&self, release: &ComponentRelease) -> Result<(), RepositoryError> {
        (**self).save_release(release).await
    }

    async fn update_release(&self, release: &ComponentRelease) -> Result<(), RepositoryError> {
        (**self).update_release(release).await
    }

    async fn delete_release(&self, uuid: &Uuid) -> Result<(), RepositoryError> {
        (**self).delete_release(uuid).await
    }
}

#[async_trait]
impl<T> ComponentRepository for Arc<T>
where
    T: ComponentRepository + ?Sized + Send + Sync,
{
    async fn find_by_uuid(&self, uuid: &Uuid) -> Result<Option<Component>, RepositoryError> {
        (**self).find_by_uuid(uuid).await
    }

    async fn find_by_name(&self, name: &str) -> Result<Vec<Component>, RepositoryError> {
        (**self).find_by_name(name).await
    }

    async fn save(&self, component: &Component) -> Result<(), RepositoryError> {
        (**self).save(component).await
    }

    async fn update(&self, component: &Component) -> Result<(), RepositoryError> {
        (**self).update(component).await
    }

    async fn delete(&self, uuid: &Uuid) -> Result<(), RepositoryError> {
        (**self).delete(uuid).await
    }

    async fn find_release_by_uuid(
        &self,
        uuid: &Uuid,
    ) -> Result<Option<ComponentRelease>, RepositoryError> {
        (**self).find_release_by_uuid(uuid).await
    }

    async fn find_releases_by_component(
        &self,
        component_uuid: &Uuid,
    ) -> Result<Vec<ComponentRelease>, RepositoryError> {
        (**self).find_releases_by_component(component_uuid).await
    }

    async fn save_release(&self, release: &ComponentRelease) -> Result<(), RepositoryError> {
        (**self).save_release(release).await
    }

    async fn update_release(&self, release: &ComponentRelease) -> Result<(), RepositoryError> {
        (**self).update_release(release).await
    }

    async fn delete_release(&self, uuid: &Uuid) -> Result<(), RepositoryError> {
        (**self).delete_release(uuid).await
    }
}
