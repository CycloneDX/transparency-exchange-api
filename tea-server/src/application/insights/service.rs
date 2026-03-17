use crate::domain::artifact::entity::Artifact;
use crate::domain::artifact::repository::ArtifactRepository;
use crate::domain::common::error::DomainError;

pub struct InsightsApplicationService<A> {
    artifact_repository: A,
}

impl<A> InsightsApplicationService<A>
where
    A: ArtifactRepository + Send + Sync,
{
    pub fn new(artifact_repository: A) -> Self {
        Self {
            artifact_repository,
        }
    }

    pub async fn search_artifacts(&self, name: &str) -> Result<Vec<Artifact>, DomainError> {
        self.artifact_repository
            .find_by_name(name)
            .await
            .map_err(DomainError::Repository)
    }
}
