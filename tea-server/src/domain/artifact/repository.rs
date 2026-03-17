use async_trait::async_trait;
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
