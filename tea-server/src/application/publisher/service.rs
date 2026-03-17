use uuid::Uuid;

use crate::domain::artifact::entity::Artifact;
use crate::domain::artifact::repository::ArtifactRepository;
use crate::domain::artifact::service::ArtifactService;
use crate::domain::collection::entity::Collection;
use crate::domain::collection::repository::CollectionRepository;
use crate::domain::collection::service::CollectionService;
use crate::domain::common::deprecation::Deprecation;
use crate::domain::common::error::DomainError;
use crate::domain::component::entity::{Component, ComponentRelease};
use crate::domain::component::repository::ComponentRepository;
use crate::domain::component::service::ComponentService;
use crate::domain::product::entity::Product;
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
