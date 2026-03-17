use chrono::Utc;
use uuid::Uuid;

use super::entity::Collection;
use super::repository::CollectionRepository;
use crate::domain::common::deprecation::Deprecation;
use crate::domain::common::error::DomainError;
use crate::domain::common::validation::{validate_max_len, validate_non_empty, validate_non_negative};
use crate::domain::common::pagination::{Page, PaginationParams};

pub struct CollectionService<R: CollectionRepository> {
    repository: R,
}

impl<R> CollectionService<R>
where
    R: CollectionRepository + Send + Sync,
{
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn get_collection(&self, uuid: &Uuid) -> Result<Option<Collection>, DomainError> {
        self.repository
            .find_by_uuid(uuid)
            .await
            .map_err(DomainError::Repository)
    }

    pub async fn list_collections(&self, params: PaginationParams) -> Result<Page<Collection>, DomainError> {
        let all = self.repository.find_by_name("").await.map_err(DomainError::Repository)?;
        Ok(Page::new(all, &params))
    }

    pub async fn create_collection(
        &self,
        mut collection: Collection,
    ) -> Result<Collection, DomainError> {
        validate_non_empty("name", &collection.name)?;
        validate_max_len("name", &collection.name, 4096)?;
        validate_non_negative("version", collection.version)?;

        collection.uuid = Uuid::new_v4();
        collection.created_date = Utc::now();
        collection.modified_date = Utc::now();
        collection.date = Utc::now();

        self.repository
            .save(&collection)
            .await
            .map_err(DomainError::Repository)?;
        Ok(collection)
    }

    pub async fn update_collection(
        &self,
        mut collection: Collection,
    ) -> Result<Collection, DomainError> {
        collection.modified_date = Utc::now();
        self.repository
            .update(&collection)
            .await
            .map_err(DomainError::Repository)?;
        Ok(collection)
    }

    pub async fn delete_collection(&self, uuid: &Uuid) -> Result<(), DomainError> {
        self.repository
            .delete(uuid)
            .await
            .map_err(DomainError::Repository)
    }

    pub async fn deprecate_collection(
        &self,
        uuid: &Uuid,
        deprecation: Deprecation,
    ) -> Result<Collection, DomainError> {
        let mut collection = self
            .repository
            .find_by_uuid(uuid)
            .await
            .map_err(DomainError::Repository)?
            .ok_or_else(|| DomainError::NotFound(format!("Collection {uuid} not found")))?;

        collection.deprecation = Some(deprecation);
        collection.modified_date = Utc::now();

        self.repository
            .update(&collection)
            .await
            .map_err(DomainError::Repository)?;

        tracing::info!(collection_uuid = %uuid, "Collection deprecated");
        Ok(collection)
    }

    pub async fn get_dependents(&self, _uuid: &Uuid) -> Result<Vec<Uuid>, DomainError> {
        // TODO(cross-domain): implement via a DependencyResolver port
        Ok(vec![])
    }
}
