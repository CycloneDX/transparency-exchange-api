use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

use super::entity::Collection;
use crate::domain::common::error::RepositoryError;

#[async_trait]
pub trait CollectionRepository {
    async fn find_by_uuid(&self, uuid: &Uuid) -> Result<Option<Collection>, RepositoryError>;
    async fn find_versions_by_uuid(&self, uuid: &Uuid) -> Result<Vec<Collection>, RepositoryError>;
    async fn find_by_name(&self, name: &str) -> Result<Vec<Collection>, RepositoryError>;
    async fn save(&self, collection: &Collection) -> Result<(), RepositoryError>;
    async fn update(&self, collection: &Collection) -> Result<(), RepositoryError>;
    async fn delete(&self, uuid: &Uuid) -> Result<(), RepositoryError>;
}

#[async_trait]
impl<T> CollectionRepository for Box<T>
where
    T: CollectionRepository + ?Sized + Send + Sync,
{
    async fn find_by_uuid(&self, uuid: &Uuid) -> Result<Option<Collection>, RepositoryError> {
        (**self).find_by_uuid(uuid).await
    }

    async fn find_by_name(&self, name: &str) -> Result<Vec<Collection>, RepositoryError> {
        (**self).find_by_name(name).await
    }

    async fn find_versions_by_uuid(&self, uuid: &Uuid) -> Result<Vec<Collection>, RepositoryError> {
        (**self).find_versions_by_uuid(uuid).await
    }

    async fn save(&self, collection: &Collection) -> Result<(), RepositoryError> {
        (**self).save(collection).await
    }

    async fn update(&self, collection: &Collection) -> Result<(), RepositoryError> {
        (**self).update(collection).await
    }

    async fn delete(&self, uuid: &Uuid) -> Result<(), RepositoryError> {
        (**self).delete(uuid).await
    }
}

#[async_trait]
impl<T> CollectionRepository for Arc<T>
where
    T: CollectionRepository + ?Sized + Send + Sync,
{
    async fn find_by_uuid(&self, uuid: &Uuid) -> Result<Option<Collection>, RepositoryError> {
        (**self).find_by_uuid(uuid).await
    }

    async fn find_by_name(&self, name: &str) -> Result<Vec<Collection>, RepositoryError> {
        (**self).find_by_name(name).await
    }

    async fn find_versions_by_uuid(&self, uuid: &Uuid) -> Result<Vec<Collection>, RepositoryError> {
        (**self).find_versions_by_uuid(uuid).await
    }

    async fn save(&self, collection: &Collection) -> Result<(), RepositoryError> {
        (**self).save(collection).await
    }

    async fn update(&self, collection: &Collection) -> Result<(), RepositoryError> {
        (**self).update(collection).await
    }

    async fn delete(&self, uuid: &Uuid) -> Result<(), RepositoryError> {
        (**self).delete(uuid).await
    }
}
