use crate::domain::artifact::entity::Artifact;
use crate::domain::artifact::repository::ArtifactRepository;
use crate::domain::artifact::service::ArtifactService;
use crate::domain::collection::entity::Collection;
use crate::domain::collection::repository::CollectionRepository;
use crate::domain::collection::service::CollectionService;
use crate::domain::common::error::DomainError;
use crate::domain::component::entity::Component;
use crate::domain::component::repository::ComponentRepository;
use crate::domain::component::service::ComponentService;
use crate::domain::product::entity::Product;
use crate::domain::product::repository::ProductRepository;
use crate::domain::product::service::ProductService;
use uuid::Uuid;

pub struct ConsumerApplicationService<
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

impl<P, C, Col, A> ConsumerApplicationService<P, C, Col, A>
where
    P: ProductRepository + Send + Sync,
    C: ComponentRepository + Send + Sync,
    Col: CollectionRepository + Send + Sync,
    A: ArtifactRepository + Send + Sync,
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

    pub async fn get_product(&self, uuid: &Uuid) -> Result<Option<Product>, DomainError> {
        self.product_service.get_product(uuid).await
    }

    pub async fn get_component(&self, uuid: &Uuid) -> Result<Option<Component>, DomainError> {
        self.component_service.get_component(uuid).await
    }

    pub async fn get_collection(&self, uuid: &Uuid) -> Result<Option<Collection>, DomainError> {
        self.collection_service.get_collection(uuid).await
    }

    pub async fn get_artifact(&self, uuid: &Uuid) -> Result<Option<Artifact>, DomainError> {
        self.artifact_service.get_artifact(uuid).await
    }
}
