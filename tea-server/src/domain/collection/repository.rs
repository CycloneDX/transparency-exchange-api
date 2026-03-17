use async_trait::async_trait;
use uuid::Uuid;

use super::entity::Collection;
use crate::domain::common::error::RepositoryError;

#[async_trait]
pub trait CollectionRepository {
    async fn find_by_uuid(&self, uuid: &Uuid) -> Result<Option<Collection>, RepositoryError>;
    async fn find_by_name(&self, name: &str) -> Result<Vec<Collection>, RepositoryError>;
    async fn save(&self, collection: &Collection) -> Result<(), RepositoryError>;
    async fn update(&self, collection: &Collection) -> Result<(), RepositoryError>;
    async fn delete(&self, uuid: &Uuid) -> Result<(), RepositoryError>;
}
