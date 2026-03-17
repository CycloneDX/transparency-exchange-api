use async_trait::async_trait;
use uuid::Uuid;

use super::entity::Product;
use crate::domain::common::error::RepositoryError;

#[async_trait]
pub trait ProductRepository {
    async fn find_by_uuid(&self, uuid: &Uuid) -> Result<Option<Product>, RepositoryError>;
    async fn find_by_name(&self, name: &str) -> Result<Vec<Product>, RepositoryError>;
    async fn save(&self, product: &Product) -> Result<(), RepositoryError>;
    async fn update(&self, product: &Product) -> Result<(), RepositoryError>;
    async fn delete(&self, uuid: &Uuid) -> Result<(), RepositoryError>;
}
