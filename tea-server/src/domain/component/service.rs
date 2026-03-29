use chrono::Utc;
use uuid::Uuid;

use super::entity::{Component, ComponentRelease};
use super::repository::ComponentRepository;
use crate::domain::common::deprecation::Deprecation;
use crate::domain::common::error::DomainError;
use crate::domain::common::pagination::{Page, PaginationParams};
use crate::domain::common::validation::{
    validate_max_len, validate_non_empty, validate_optional_url,
};

pub struct ComponentService<R: ComponentRepository> {
    repository: R,
}

impl<R> ComponentService<R>
where
    R: ComponentRepository + Send + Sync,
{
    fn validate_component(component: &Component) -> Result<(), DomainError> {
        validate_non_empty("name", &component.name)?;
        validate_max_len("name", &component.name, 4096)?;
        validate_optional_url("homepage_url", &component.homepage_url)?;
        validate_optional_url("vcs_url", &component.vcs_url)?;
        Ok(())
    }

    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn get_component(&self, uuid: &Uuid) -> Result<Option<Component>, DomainError> {
        self.repository
            .find_by_uuid(uuid)
            .await
            .map_err(DomainError::Repository)
    }

    pub async fn list_components(
        &self,
        params: PaginationParams,
    ) -> Result<Page<Component>, DomainError> {
        let all = self
            .repository
            .find_by_name("")
            .await
            .map_err(DomainError::Repository)?;
        Ok(Page::new(all, &params))
    }

    pub async fn create_component(
        &self,
        mut component: Component,
    ) -> Result<Component, DomainError> {
        Self::validate_component(&component)?;

        if component.uuid.is_nil() {
            component.uuid = Uuid::new_v4();
        }
        component.created_date = Utc::now();
        component.modified_date = Utc::now();

        self.repository
            .save(&component)
            .await
            .map_err(DomainError::Repository)?;
        Ok(component)
    }

    pub async fn update_component(
        &self,
        mut component: Component,
    ) -> Result<Component, DomainError> {
        Self::validate_component(&component)?;
        component.modified_date = Utc::now();
        self.repository
            .update(&component)
            .await
            .map_err(DomainError::Repository)?;
        Ok(component)
    }

    pub async fn delete_component(&self, uuid: &Uuid) -> Result<(), DomainError> {
        self.repository
            .delete(uuid)
            .await
            .map_err(DomainError::Repository)
    }

    pub async fn deprecate_component(
        &self,
        uuid: &Uuid,
        deprecation: Deprecation,
    ) -> Result<Component, DomainError> {
        let mut component = self
            .repository
            .find_by_uuid(uuid)
            .await
            .map_err(DomainError::Repository)?
            .ok_or_else(|| DomainError::NotFound(format!("Component {uuid} not found")))?;

        component.deprecation = Some(deprecation);
        component.modified_date = Utc::now();

        self.repository
            .update(&component)
            .await
            .map_err(DomainError::Repository)?;

        tracing::info!(component_uuid = %uuid, "Component deprecated");
        Ok(component)
    }

    pub async fn get_dependents(&self, _uuid: &Uuid) -> Result<Vec<Uuid>, DomainError> {
        // TODO(cross-domain): implement via a DependencyResolver port
        Ok(vec![])
    }

    pub async fn get_release(&self, uuid: &Uuid) -> Result<Option<ComponentRelease>, DomainError> {
        self.repository
            .find_release_by_uuid(uuid)
            .await
            .map_err(DomainError::Repository)
    }

    pub async fn list_releases(
        &self,
        component_uuid: &Uuid,
    ) -> Result<Vec<ComponentRelease>, DomainError> {
        self.repository
            .find_releases_by_component(component_uuid)
            .await
            .map_err(DomainError::Repository)
    }

    pub async fn create_release(
        &self,
        mut release: ComponentRelease,
    ) -> Result<ComponentRelease, DomainError> {
        validate_non_empty("version", &release.version)?;
        validate_max_len("version", &release.version, 256)?;

        if release.uuid.is_nil() {
            release.uuid = Uuid::new_v4();
        }

        self.repository
            .save_release(&release)
            .await
            .map_err(DomainError::Repository)?;
        Ok(release)
    }

    pub async fn update_release(
        &self,
        release: ComponentRelease,
    ) -> Result<ComponentRelease, DomainError> {
        self.repository
            .update_release(&release)
            .await
            .map_err(DomainError::Repository)?;
        Ok(release)
    }

    pub async fn delete_release(&self, uuid: &Uuid) -> Result<(), DomainError> {
        self.repository
            .delete_release(uuid)
            .await
            .map_err(DomainError::Repository)
    }
}
