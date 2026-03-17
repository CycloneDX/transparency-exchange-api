use chrono::Utc;
use uuid::Uuid;

use super::entity::Artifact;
use super::repository::ArtifactRepository;
use crate::domain::common::deprecation::Deprecation;
use crate::domain::common::error::DomainError;
use crate::domain::common::pagination::{Page, PaginationParams};
use crate::domain::common::validation::{validate_max_len, validate_non_empty, validate_optional_url, validate_url};

pub struct ArtifactService<R: ArtifactRepository> {
    repository: R,
}

impl<R> ArtifactService<R>
where
    R: ArtifactRepository + Send + Sync,
{
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn get_artifact(&self, uuid: &Uuid) -> Result<Option<Artifact>, DomainError> {
        self.repository
            .find_by_uuid(uuid)
            .await
            .map_err(DomainError::Repository)
    }

    pub async fn list_artifacts(&self, params: PaginationParams) -> Result<Page<Artifact>, DomainError> {
        let all = self.repository.find_by_name("").await.map_err(DomainError::Repository)?;
        Ok(Page::new(all, &params))
    }

    pub async fn create_artifact(&self, mut artifact: Artifact) -> Result<Artifact, DomainError> {
        validate_non_empty("name", &artifact.name)?;
        validate_max_len("name", &artifact.name, 4096)?;

        if artifact.formats.is_empty() {
            return Err(DomainError::Validation(
                "Artifact must have at least one format".to_string(),
            ));
        }

        // H3 fix: validate all format URLs
        for (i, fmt) in artifact.formats.iter().enumerate() {
            validate_url(&format!("formats[{i}].url"), &fmt.url)?;
            if let Some(ref sig_url) = fmt.signature_url {
                validate_optional_url(&format!("formats[{i}].signature_url"), &Some(sig_url.clone()))?;
            }
        }

        artifact.uuid = Uuid::new_v4();
        artifact.created_date = Utc::now();
        artifact.modified_date = Utc::now();

        self.repository
            .save(&artifact)
            .await
            .map_err(DomainError::Repository)?;
        Ok(artifact)
    }

    pub async fn update_artifact(&self, mut artifact: Artifact) -> Result<Artifact, DomainError> {
        artifact.modified_date = Utc::now();
        self.repository
            .update(&artifact)
            .await
            .map_err(DomainError::Repository)?;
        Ok(artifact)
    }

    pub async fn delete_artifact(&self, uuid: &Uuid) -> Result<(), DomainError> {
        self.repository
            .delete(uuid)
            .await
            .map_err(DomainError::Repository)
    }

    pub async fn deprecate_artifact(
        &self,
        uuid: &Uuid,
        deprecation: Deprecation,
    ) -> Result<Artifact, DomainError> {
        let mut artifact = self
            .repository
            .find_by_uuid(uuid)
            .await
            .map_err(DomainError::Repository)?
            .ok_or_else(|| DomainError::NotFound(format!("Artifact {uuid} not found")))?;

        artifact.deprecation = Some(deprecation);
        artifact.modified_date = Utc::now();

        self.repository
            .update(&artifact)
            .await
            .map_err(DomainError::Repository)?;

        tracing::info!(artifact_uuid = %uuid, "Artifact deprecated");
        Ok(artifact)
    }

    pub async fn get_dependents(&self, _uuid: &Uuid) -> Result<Vec<Uuid>, DomainError> {
        // TODO(cross-domain): implement via a DependencyResolver port
        Ok(vec![])
    }
}
