use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

use super::entity::Artifact;
use crate::domain::common::error::RepositoryError;

#[async_trait]
pub trait ArtifactRepository {
    async fn find_by_uuid(&self, uuid: &Uuid) -> Result<Option<Artifact>, RepositoryError>;
    async fn find_by_name(&self, name: &str) -> Result<Vec<Artifact>, RepositoryError>;
    async fn save(&self, artifact: &Artifact) -> Result<(), RepositoryError>;
    async fn update(&self, artifact: &Artifact) -> Result<(), RepositoryError>;
    async fn delete(&self, uuid: &Uuid) -> Result<(), RepositoryError>;
}

#[async_trait]
impl<T> ArtifactRepository for Box<T>
where
    T: ArtifactRepository + ?Sized + Send + Sync,
{
    async fn find_by_uuid(&self, uuid: &Uuid) -> Result<Option<Artifact>, RepositoryError> {
        (**self).find_by_uuid(uuid).await
    }

    async fn find_by_name(&self, name: &str) -> Result<Vec<Artifact>, RepositoryError> {
        (**self).find_by_name(name).await
    }

    async fn save(&self, artifact: &Artifact) -> Result<(), RepositoryError> {
        (**self).save(artifact).await
    }

    async fn update(&self, artifact: &Artifact) -> Result<(), RepositoryError> {
        (**self).update(artifact).await
    }

    async fn delete(&self, uuid: &Uuid) -> Result<(), RepositoryError> {
        (**self).delete(uuid).await
    }
}

#[async_trait]
impl<T> ArtifactRepository for Arc<T>
where
    T: ArtifactRepository + ?Sized + Send + Sync,
{
    async fn find_by_uuid(&self, uuid: &Uuid) -> Result<Option<Artifact>, RepositoryError> {
        (**self).find_by_uuid(uuid).await
    }

    async fn find_by_name(&self, name: &str) -> Result<Vec<Artifact>, RepositoryError> {
        (**self).find_by_name(name).await
    }

    async fn save(&self, artifact: &Artifact) -> Result<(), RepositoryError> {
        (**self).save(artifact).await
    }

    async fn update(&self, artifact: &Artifact) -> Result<(), RepositoryError> {
        (**self).update(artifact).await
    }

    async fn delete(&self, uuid: &Uuid) -> Result<(), RepositoryError> {
        (**self).delete(uuid).await
    }
}
