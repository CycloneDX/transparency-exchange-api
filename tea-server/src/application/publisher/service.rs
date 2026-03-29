use uuid::Uuid;

use crate::domain::artifact::entity::Artifact;
use crate::domain::artifact::repository::ArtifactRepository;
use crate::domain::artifact::service::ArtifactService;
use crate::domain::collection::entity::Collection;
use crate::domain::collection::repository::CollectionRepository;
use crate::domain::collection::service::CollectionService;
use crate::domain::common::deprecation::Deprecation;
use crate::domain::common::error::{DomainError, RepositoryError};
use crate::domain::common::pagination::PaginationParams;
use crate::domain::component::entity::{Component, ComponentRelease};
use crate::domain::component::repository::ComponentRepository;
use crate::domain::component::service::ComponentService;
use crate::domain::product::entity::{Product, ProductRelease};
use crate::domain::product::repository::ProductRepository;
use crate::domain::product::service::ProductService;

/// Trait defining the Publisher Application Service interface.
/// Used for gRPC service generic bounds.
#[tonic::async_trait]
pub trait PublisherApplicationService: Send + Sync {
    // Product operations
    async fn get_product(&self, uuid: &Uuid) -> Result<Option<Product>, DomainError>;
    async fn create_product(&self, product: Product) -> Result<Product, DomainError>;
    async fn update_product(&self, product: Product) -> Result<Product, DomainError>;
    async fn delete_product(&self, uuid: &Uuid) -> Result<(), DomainError>;
    async fn get_product_release(&self, uuid: &Uuid)
        -> Result<Option<ProductRelease>, DomainError>;
    async fn list_product_releases(
        &self,
        product_uuid: &Uuid,
    ) -> Result<Vec<ProductRelease>, DomainError>;
    async fn create_product_release(
        &self,
        release: ProductRelease,
    ) -> Result<ProductRelease, DomainError>;
    async fn update_product_release(
        &self,
        release: ProductRelease,
    ) -> Result<ProductRelease, DomainError>;
    async fn delete_product_release(&self, uuid: &Uuid) -> Result<(), DomainError>;
    async fn deprecate_product(
        &self,
        uuid: Uuid,
        deprecation: Deprecation,
    ) -> Result<Product, DomainError>;

    // Component operations
    async fn get_component(&self, uuid: &Uuid) -> Result<Option<Component>, DomainError>;
    async fn create_component(&self, component: Component) -> Result<Component, DomainError>;
    async fn update_component(&self, component: Component) -> Result<Component, DomainError>;
    async fn delete_component(&self, uuid: &Uuid) -> Result<(), DomainError>;
    async fn get_component_release(
        &self,
        uuid: &Uuid,
    ) -> Result<Option<ComponentRelease>, DomainError>;
    async fn list_component_releases(
        &self,
        component_uuid: &Uuid,
    ) -> Result<Vec<ComponentRelease>, DomainError>;
    async fn create_component_release(
        &self,
        release: ComponentRelease,
    ) -> Result<ComponentRelease, DomainError>;
    async fn update_component_release(
        &self,
        release: ComponentRelease,
    ) -> Result<ComponentRelease, DomainError>;
    async fn delete_component_release(&self, uuid: &Uuid) -> Result<(), DomainError>;
    async fn deprecate_component(
        &self,
        uuid: Uuid,
        deprecation: Deprecation,
    ) -> Result<Component, DomainError>;

    // Artifact operations
    async fn get_artifact(&self, uuid: &Uuid) -> Result<Option<Artifact>, DomainError>;
    async fn create_artifact(&self, artifact: Artifact) -> Result<Artifact, DomainError>;
    async fn update_artifact(&self, artifact: Artifact) -> Result<Artifact, DomainError>;
    async fn delete_artifact(&self, uuid: &Uuid) -> Result<(), DomainError>;
    async fn deprecate_artifact(
        &self,
        uuid: Uuid,
        deprecation: Deprecation,
    ) -> Result<Artifact, DomainError>;

    // Collection operations
    async fn get_collection(&self, uuid: &Uuid) -> Result<Option<Collection>, DomainError>;
    async fn list_collections(&self) -> Result<Vec<Collection>, DomainError>;
    async fn create_collection(&self, collection: Collection) -> Result<Collection, DomainError>;
    async fn update_collection(&self, collection: Collection) -> Result<Collection, DomainError>;
    async fn delete_collection(&self, uuid: &Uuid) -> Result<(), DomainError>;
    async fn deprecate_collection(
        &self,
        uuid: Uuid,
        deprecation: Deprecation,
    ) -> Result<Collection, DomainError>;
}

pub struct PublisherApplicationServiceImpl<
    P: ProductRepository + Send + Sync,
    C: ComponentRepository + Send + Sync,
    Col: CollectionRepository + Send + Sync,
    A: ArtifactRepository + Send + Sync,
> {
    product_service: ProductService<P>,
    component_service: ComponentService<C>,
    collection_service: CollectionService<Col>,
    artifact_service: ArtifactService<A>,
}

impl<P, C, Col, A> PublisherApplicationServiceImpl<P, C, Col, A>
where
    P: ProductRepository + Send + Sync,
    C: ComponentRepository + Send + Sync,
    Col: CollectionRepository + Send + Sync,
    A: ArtifactRepository + Send + Sync,
    Self: Send + Sync,
{
    pub fn new(
        product_service: ProductService<P>,
        component_service: ComponentService<C>,
        collection_service: CollectionService<Col>,
        artifact_service: ArtifactService<A>,
    ) -> Self {
        Self {
            product_service,
            component_service,
            collection_service,
            artifact_service,
        }
    }

    // ─── Product Operations ──────────────────────────────────────────────────────

    pub async fn get_product(&self, uuid: &Uuid) -> Result<Option<Product>, DomainError> {
        self.product_service.get_product(uuid).await
    }

    pub async fn create_product(&self, product: Product) -> Result<Product, DomainError> {
        self.product_service.create_product(product).await
    }

    pub async fn update_product(&self, product: Product) -> Result<Product, DomainError> {
        self.product_service.update_product(product).await
    }

    pub async fn delete_product(&self, uuid: &Uuid) -> Result<(), DomainError> {
        self.product_service.delete_product(uuid).await
    }

    pub async fn get_product_release(
        &self,
        uuid: &Uuid,
    ) -> Result<Option<ProductRelease>, DomainError> {
        self.product_service.get_release(uuid).await
    }

    pub async fn list_product_releases(
        &self,
        product_uuid: &Uuid,
    ) -> Result<Vec<ProductRelease>, DomainError> {
        self.product_service.list_releases(product_uuid).await
    }

    async fn validate_product_release_graph(
        &self,
        release: &ProductRelease,
    ) -> Result<(), DomainError> {
        if self
            .product_service
            .get_product(&release.product_uuid)
            .await?
            .is_none()
        {
            return Err(DomainError::NotFound(format!(
                "Product {} not found",
                release.product_uuid
            )));
        }

        for component in &release.components {
            if self
                .component_service
                .get_component(&component.component_uuid)
                .await?
                .is_none()
            {
                return Err(DomainError::NotFound(format!(
                    "Component {} not found",
                    component.component_uuid
                )));
            }

            let release_entity = self
                .component_service
                .get_release(&component.release_uuid)
                .await?
                .ok_or_else(|| {
                    DomainError::NotFound(format!(
                        "Component release {} not found",
                        component.release_uuid
                    ))
                })?;

            if release_entity.component_uuid != component.component_uuid {
                return Err(DomainError::Validation(format!(
                    "component release {} does not belong to component {}",
                    component.release_uuid, component.component_uuid
                )));
            }
        }

        Ok(())
    }

    pub async fn create_product_release(
        &self,
        release: ProductRelease,
    ) -> Result<ProductRelease, DomainError> {
        self.validate_product_release_graph(&release).await?;
        self.product_service.create_release(release).await
    }

    pub async fn update_product_release(
        &self,
        release: ProductRelease,
    ) -> Result<ProductRelease, DomainError> {
        self.validate_product_release_graph(&release).await?;
        self.product_service.update_release(release).await
    }

    pub async fn delete_product_release(&self, uuid: &Uuid) -> Result<(), DomainError> {
        self.product_service.delete_release(uuid).await?;

        match self.collection_service.delete_collection(uuid).await {
            Ok(()) | Err(DomainError::Repository(RepositoryError::NotFound)) => Ok(()),
            Err(error) => Err(error),
        }
    }

    pub async fn deprecate_product(
        &self,
        uuid: Uuid,
        deprecation: Deprecation,
    ) -> Result<Product, DomainError> {
        let product = self
            .product_service
            .deprecate_product(&uuid, deprecation)
            .await?;

        tracing::info!(
            product_uuid = %uuid,
            deprecation_state = ?product.deprecation.as_ref().map(|d| &d.state),
            "Product deprecated"
        );

        Ok(product)
    }

    // ─── Component Operations ────────────────────────────────────────────────────

    pub async fn get_component(&self, uuid: &Uuid) -> Result<Option<Component>, DomainError> {
        self.component_service.get_component(uuid).await
    }

    pub async fn create_component(&self, component: Component) -> Result<Component, DomainError> {
        self.component_service.create_component(component).await
    }

    pub async fn update_component(&self, component: Component) -> Result<Component, DomainError> {
        self.component_service.update_component(component).await
    }

    pub async fn delete_component(&self, uuid: &Uuid) -> Result<(), DomainError> {
        self.component_service.delete_component(uuid).await
    }

    pub async fn get_component_release(
        &self,
        uuid: &Uuid,
    ) -> Result<Option<ComponentRelease>, DomainError> {
        self.component_service.get_release(uuid).await
    }

    pub async fn list_component_releases(
        &self,
        component_uuid: &Uuid,
    ) -> Result<Vec<ComponentRelease>, DomainError> {
        self.component_service.list_releases(component_uuid).await
    }

    pub async fn deprecate_component(
        &self,
        uuid: Uuid,
        deprecation: Deprecation,
    ) -> Result<Component, DomainError> {
        let component = self
            .component_service
            .deprecate_component(&uuid, deprecation)
            .await?;

        tracing::info!(
            component_uuid = %uuid,
            deprecation_state = ?component.deprecation.as_ref().map(|d| &d.state),
            "Component deprecated"
        );

        Ok(component)
    }

    pub async fn create_component_release(
        &self,
        release: ComponentRelease,
    ) -> Result<ComponentRelease, DomainError> {
        self.component_service.create_release(release).await
    }

    pub async fn update_component_release(
        &self,
        release: ComponentRelease,
    ) -> Result<ComponentRelease, DomainError> {
        self.component_service.update_release(release).await
    }

    pub async fn delete_component_release(&self, uuid: &Uuid) -> Result<(), DomainError> {
        self.component_service.delete_release(uuid).await?;

        match self.collection_service.delete_collection(uuid).await {
            Ok(()) | Err(DomainError::Repository(RepositoryError::NotFound)) => Ok(()),
            Err(error) => Err(error),
        }
    }

    // ─── Artifact Operations ──────────────────────────────────────────────────────

    pub async fn get_artifact(&self, uuid: &Uuid) -> Result<Option<Artifact>, DomainError> {
        self.artifact_service.get_artifact(uuid).await
    }

    pub async fn create_artifact(&self, artifact: Artifact) -> Result<Artifact, DomainError> {
        self.artifact_service.create_artifact(artifact).await
    }

    pub async fn update_artifact(&self, artifact: Artifact) -> Result<Artifact, DomainError> {
        self.artifact_service.update_artifact(artifact).await
    }

    pub async fn delete_artifact(&self, uuid: &Uuid) -> Result<(), DomainError> {
        self.artifact_service.delete_artifact(uuid).await
    }

    pub async fn deprecate_artifact(
        &self,
        uuid: Uuid,
        deprecation: Deprecation,
    ) -> Result<Artifact, DomainError> {
        let artifact = self
            .artifact_service
            .deprecate_artifact(&uuid, deprecation)
            .await?;

        tracing::info!(
            artifact_uuid = %uuid,
            deprecation_state = ?artifact.deprecation.as_ref().map(|d| &d.state),
            "Artifact deprecated"
        );

        Ok(artifact)
    }

    // ─── Collection Operations ────────────────────────────────────────────────────

    pub async fn get_collection(&self, uuid: &Uuid) -> Result<Option<Collection>, DomainError> {
        self.collection_service.get_collection(uuid).await
    }

    pub async fn list_collections(&self) -> Result<Vec<Collection>, DomainError> {
        Ok(self
            .collection_service
            .list_collections(PaginationParams {
                limit: usize::MAX,
                offset: 0,
            })
            .await?
            .items)
    }

    pub async fn create_collection(
        &self,
        collection: Collection,
    ) -> Result<Collection, DomainError> {
        self.collection_service.create_collection(collection).await
    }

    pub async fn update_collection(
        &self,
        collection: Collection,
    ) -> Result<Collection, DomainError> {
        self.collection_service.update_collection(collection).await
    }

    pub async fn delete_collection(&self, uuid: &Uuid) -> Result<(), DomainError> {
        self.collection_service.delete_collection(uuid).await
    }

    pub async fn deprecate_collection(
        &self,
        uuid: Uuid,
        deprecation: Deprecation,
    ) -> Result<Collection, DomainError> {
        let collection = self
            .collection_service
            .deprecate_collection(&uuid, deprecation)
            .await?;

        tracing::info!(
            collection_uuid = %uuid,
            deprecation_state = ?collection.deprecation.as_ref().map(|d| &d.state),
            "Collection deprecated"
        );

        Ok(collection)
    }
}

// Implement the trait for the concrete implementation
#[tonic::async_trait]
impl<P, C, Col, A> PublisherApplicationService for PublisherApplicationServiceImpl<P, C, Col, A>
where
    P: ProductRepository + Send + Sync,
    C: ComponentRepository + Send + Sync,
    Col: CollectionRepository + Send + Sync,
    A: ArtifactRepository + Send + Sync,
    Self: Send + Sync,
{
    async fn get_product(&self, uuid: &Uuid) -> Result<Option<Product>, DomainError> {
        self.product_service.get_product(uuid).await
    }

    async fn create_product(&self, product: Product) -> Result<Product, DomainError> {
        self.product_service.create_product(product).await
    }

    async fn update_product(&self, product: Product) -> Result<Product, DomainError> {
        self.product_service.update_product(product).await
    }

    async fn delete_product(&self, uuid: &Uuid) -> Result<(), DomainError> {
        self.product_service.delete_product(uuid).await
    }

    async fn get_product_release(
        &self,
        uuid: &Uuid,
    ) -> Result<Option<ProductRelease>, DomainError> {
        self.product_service.get_release(uuid).await
    }

    async fn list_product_releases(
        &self,
        product_uuid: &Uuid,
    ) -> Result<Vec<ProductRelease>, DomainError> {
        self.product_service.list_releases(product_uuid).await
    }

    async fn create_product_release(
        &self,
        release: ProductRelease,
    ) -> Result<ProductRelease, DomainError> {
        self.validate_product_release_graph(&release).await?;
        self.product_service.create_release(release).await
    }

    async fn update_product_release(
        &self,
        release: ProductRelease,
    ) -> Result<ProductRelease, DomainError> {
        self.validate_product_release_graph(&release).await?;
        self.product_service.update_release(release).await
    }

    async fn delete_product_release(&self, uuid: &Uuid) -> Result<(), DomainError> {
        self.product_service.delete_release(uuid).await?;
        match self.collection_service.delete_collection(uuid).await {
            Ok(()) | Err(DomainError::Repository(RepositoryError::NotFound)) => Ok(()),
            Err(error) => Err(error),
        }
    }

    async fn deprecate_product(
        &self,
        uuid: Uuid,
        deprecation: Deprecation,
    ) -> Result<Product, DomainError> {
        self.product_service
            .deprecate_product(&uuid, deprecation)
            .await
    }

    async fn get_component(&self, uuid: &Uuid) -> Result<Option<Component>, DomainError> {
        self.component_service.get_component(uuid).await
    }

    async fn create_component(&self, component: Component) -> Result<Component, DomainError> {
        self.component_service.create_component(component).await
    }

    async fn update_component(&self, component: Component) -> Result<Component, DomainError> {
        self.component_service.update_component(component).await
    }

    async fn delete_component(&self, uuid: &Uuid) -> Result<(), DomainError> {
        self.component_service.delete_component(uuid).await
    }

    async fn get_component_release(
        &self,
        uuid: &Uuid,
    ) -> Result<Option<ComponentRelease>, DomainError> {
        self.component_service.get_release(uuid).await
    }

    async fn list_component_releases(
        &self,
        component_uuid: &Uuid,
    ) -> Result<Vec<ComponentRelease>, DomainError> {
        self.component_service.list_releases(component_uuid).await
    }

    async fn create_component_release(
        &self,
        release: ComponentRelease,
    ) -> Result<ComponentRelease, DomainError> {
        self.component_service.create_release(release).await
    }

    async fn update_component_release(
        &self,
        release: ComponentRelease,
    ) -> Result<ComponentRelease, DomainError> {
        self.component_service.update_release(release).await
    }

    async fn delete_component_release(&self, uuid: &Uuid) -> Result<(), DomainError> {
        self.component_service.delete_release(uuid).await?;
        match self.collection_service.delete_collection(uuid).await {
            Ok(()) | Err(DomainError::Repository(RepositoryError::NotFound)) => Ok(()),
            Err(error) => Err(error),
        }
    }

    async fn deprecate_component(
        &self,
        uuid: Uuid,
        deprecation: Deprecation,
    ) -> Result<Component, DomainError> {
        self.component_service
            .deprecate_component(&uuid, deprecation)
            .await
    }

    async fn get_artifact(&self, uuid: &Uuid) -> Result<Option<Artifact>, DomainError> {
        self.artifact_service.get_artifact(uuid).await
    }

    async fn create_artifact(&self, artifact: Artifact) -> Result<Artifact, DomainError> {
        self.artifact_service.create_artifact(artifact).await
    }

    async fn update_artifact(&self, artifact: Artifact) -> Result<Artifact, DomainError> {
        self.artifact_service.update_artifact(artifact).await
    }

    async fn delete_artifact(&self, uuid: &Uuid) -> Result<(), DomainError> {
        self.artifact_service.delete_artifact(uuid).await
    }

    async fn deprecate_artifact(
        &self,
        uuid: Uuid,
        deprecation: Deprecation,
    ) -> Result<Artifact, DomainError> {
        self.artifact_service
            .deprecate_artifact(&uuid, deprecation)
            .await
    }

    async fn get_collection(&self, uuid: &Uuid) -> Result<Option<Collection>, DomainError> {
        self.collection_service.get_collection(uuid).await
    }

    async fn list_collections(&self) -> Result<Vec<Collection>, DomainError> {
        Ok(self
            .collection_service
            .list_collections(PaginationParams {
                limit: usize::MAX,
                offset: 0,
            })
            .await?
            .items)
    }

    async fn create_collection(&self, collection: Collection) -> Result<Collection, DomainError> {
        self.collection_service.create_collection(collection).await
    }

    async fn update_collection(&self, collection: Collection) -> Result<Collection, DomainError> {
        self.collection_service.update_collection(collection).await
    }

    async fn delete_collection(&self, uuid: &Uuid) -> Result<(), DomainError> {
        self.collection_service.delete_collection(uuid).await
    }

    async fn deprecate_collection(
        &self,
        uuid: Uuid,
        deprecation: Deprecation,
    ) -> Result<Collection, DomainError> {
        self.collection_service
            .deprecate_collection(&uuid, deprecation)
            .await
    }
}
