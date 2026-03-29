use std::collections::HashSet;
use std::pin::Pin;
use std::sync::Arc;

use tokio_stream::Stream;
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::domain::artifact::repository::ArtifactRepository;
use crate::domain::collection::entity::Collection;
use crate::domain::collection::repository::CollectionRepository;
use crate::domain::common::error::{DomainError, RepositoryError};
use crate::domain::common::pagination::Page;
use crate::domain::component::entity::ComponentRelease;
use crate::domain::component::repository::ComponentRepository;
use crate::domain::product::repository::ProductRepository;
use crate::gen::tea::v1::{self as proto, consumer_service_server::ConsumerService};

use super::conversions::{
    artifact_stub, artifact_to_proto, checksum_to_proto, collection_to_proto,
    collection_version_info, component_release_to_proto, component_to_proto, identifier_matches,
    page_response, pagination_from_proto, parse_uuid, product_release_to_proto, product_to_proto,
    within_range,
};

pub struct ConsumerGrpcService<P: ?Sized, C: ?Sized, Col: ?Sized, A: ?Sized> {
    product_repository: Arc<P>,
    component_repository: Arc<C>,
    collection_repository: Arc<Col>,
    artifact_repository: Arc<A>,
}

impl<P: ?Sized, C: ?Sized, Col: ?Sized, A: ?Sized> ConsumerGrpcService<P, C, Col, A> {
    pub fn new(
        product_repository: Arc<P>,
        component_repository: Arc<C>,
        collection_repository: Arc<Col>,
        artifact_repository: Arc<A>,
    ) -> Self {
        Self {
            product_repository,
            component_repository,
            collection_repository,
            artifact_repository,
        }
    }
}

impl<P, C, Col, A> ConsumerGrpcService<P, C, Col, A>
where
    P: ProductRepository + ?Sized + Send + Sync + 'static,
    C: ComponentRepository + ?Sized + Send + Sync + 'static,
    Col: CollectionRepository + ?Sized + Send + Sync + 'static,
    A: ArtifactRepository + ?Sized + Send + Sync + 'static,
{
    async fn find_collection_version(
        &self,
        uuid: &Uuid,
        version: Option<i32>,
    ) -> Result<Option<Collection>, Status> {
        let mut collections = self
            .collection_repository
            .find_versions_by_uuid(uuid)
            .await
            .map_err(repository_error_to_status)?;

        if let Some(version) = version {
            Ok(collections
                .into_iter()
                .find(|collection| collection.version == version))
        } else {
            collections.sort_by_key(|collection| std::cmp::Reverse(collection.version));
            Ok(collections.into_iter().next())
        }
    }

    async fn resolve_collection_artifacts(
        &self,
        collection: &Collection,
        include_artifacts: bool,
    ) -> Result<Vec<proto::Artifact>, Status> {
        if !include_artifacts {
            return Ok(collection
                .artifacts
                .iter()
                .copied()
                .map(artifact_stub)
                .collect());
        }

        let mut artifacts = Vec::with_capacity(collection.artifacts.len());
        for artifact_uuid in &collection.artifacts {
            let artifact = self
                .artifact_repository
                .find_by_uuid(artifact_uuid)
                .await
                .map_err(repository_error_to_status)?
                .ok_or_else(|| Status::not_found(format!("artifact {artifact_uuid} not found")))?;
            artifacts.push(artifact_to_proto(artifact));
        }
        Ok(artifacts)
    }

    async fn all_component_releases(&self) -> Result<Vec<ComponentRelease>, Status> {
        let components = self
            .component_repository
            .find_by_name("")
            .await
            .map_err(repository_error_to_status)?;
        let mut releases = Vec::new();
        for component in components {
            let mut component_releases = self
                .component_repository
                .find_releases_by_component(&component.uuid)
                .await
                .map_err(repository_error_to_status)?;
            releases.append(&mut component_releases);
        }
        Ok(releases)
    }

    async fn all_product_releases(
        &self,
        product_uuid: Option<&Uuid>,
    ) -> Result<Vec<crate::domain::product::entity::ProductRelease>, Status> {
        let products = if let Some(uuid) = product_uuid {
            match self
                .product_repository
                .find_by_uuid(uuid)
                .await
                .map_err(repository_error_to_status)?
            {
                Some(product) => vec![product],
                None => return Ok(vec![]),
            }
        } else {
            self.product_repository
                .find_by_name("")
                .await
                .map_err(repository_error_to_status)?
        };

        let mut releases = Vec::new();
        for product in products {
            let mut product_releases = self
                .product_repository
                .find_releases_by_product(&product.uuid)
                .await
                .map_err(repository_error_to_status)?;
            releases.append(&mut product_releases);
        }
        Ok(releases)
    }
}

#[tonic::async_trait]
impl<P, C, Col, A> ConsumerService for ConsumerGrpcService<P, C, Col, A>
where
    P: ProductRepository + ?Sized + Send + Sync + 'static,
    C: ComponentRepository + ?Sized + Send + Sync + 'static,
    Col: CollectionRepository + ?Sized + Send + Sync + 'static,
    A: ArtifactRepository + ?Sized + Send + Sync + 'static,
{
    type StreamArtifactContentStream =
        Pin<Box<dyn Stream<Item = Result<proto::ArtifactContentChunk, Status>> + Send>>;

    async fn list_products(
        &self,
        request: Request<proto::ListProductsRequest>,
    ) -> Result<Response<proto::ListProductsResponse>, Status> {
        let request = request.into_inner();
        let params = pagination_from_proto(request.pagination);
        let query = request.query.trim();
        let mut products = self
            .product_repository
            .find_by_name(query)
            .await
            .map_err(repository_error_to_status)?;

        if let Some(vendor_uuid) = request.vendor_uuid.as_deref() {
            let vendor_uuid = parse_uuid(vendor_uuid, "vendor_uuid")?;
            products.retain(|product| product.vendor.uuid == Some(vendor_uuid));
        }

        if let Some(identifier) = request.identifier.as_ref() {
            products.retain(|product| {
                product
                    .identifiers
                    .iter()
                    .any(|candidate| identifier_matches(candidate, identifier))
            });
        }

        let page = Page::new(products, &params);
        let pagination = page_response(&page);
        Ok(Response::new(proto::ListProductsResponse {
            products: page.items.into_iter().map(product_to_proto).collect(),
            pagination: Some(pagination),
        }))
    }

    async fn get_product(
        &self,
        request: Request<proto::GetProductRequest>,
    ) -> Result<Response<proto::Product>, Status> {
        let uuid = parse_uuid(&request.into_inner().uuid, "uuid")?;
        let product = self
            .product_repository
            .find_by_uuid(&uuid)
            .await
            .map_err(repository_error_to_status)?
            .ok_or_else(|| Status::not_found(format!("product {uuid} not found")))?;
        Ok(Response::new(product_to_proto(product)))
    }

    async fn list_product_releases(
        &self,
        request: Request<proto::ListProductReleasesRequest>,
    ) -> Result<Response<proto::ListProductReleasesResponse>, Status> {
        let request = request.into_inner();
        let product_uuid = parse_uuid(&request.product_uuid, "product_uuid")?;
        let params = pagination_from_proto(request.pagination);
        let mut releases = self.all_product_releases(Some(&product_uuid)).await?;

        if !request.include_pre_releases {
            releases.retain(|release| !release.pre_release);
        }

        releases.retain(|release| {
            within_range(release.release_date, request.release_date_range.as_ref())
        });
        releases.sort_by_key(|release| release.release_date);
        releases.reverse();

        let page = Page::new(releases, &params);
        let pagination = page_response(&page);
        Ok(Response::new(proto::ListProductReleasesResponse {
            releases: page
                .items
                .into_iter()
                .map(product_release_to_proto)
                .collect(),
            pagination: Some(pagination),
        }))
    }

    async fn get_product_release(
        &self,
        request: Request<proto::GetProductReleaseRequest>,
    ) -> Result<Response<proto::ProductRelease>, Status> {
        let uuid = parse_uuid(&request.into_inner().uuid, "uuid")?;
        let release = self
            .product_repository
            .find_release_by_uuid(&uuid)
            .await
            .map_err(repository_error_to_status)?
            .ok_or_else(|| Status::not_found(format!("product release {uuid} not found")))?;
        Ok(Response::new(product_release_to_proto(release)))
    }

    async fn get_product_release_collection(
        &self,
        request: Request<proto::GetProductReleaseCollectionRequest>,
    ) -> Result<Response<proto::Collection>, Status> {
        let request = request.into_inner();
        let uuid = parse_uuid(&request.uuid, "uuid")?;
        let collection = self
            .find_collection_version(&uuid, request.version)
            .await?
            .ok_or_else(|| Status::not_found(format!("collection for release {uuid} not found")))?;
        let artifacts = self.resolve_collection_artifacts(&collection, true).await?;
        Ok(Response::new(collection_to_proto(collection, artifacts)))
    }

    async fn list_components(
        &self,
        request: Request<proto::ListComponentsRequest>,
    ) -> Result<Response<proto::ListComponentsResponse>, Status> {
        let request = request.into_inner();
        let params = pagination_from_proto(request.pagination);
        let query = request.query.trim();
        let mut components = self
            .component_repository
            .find_by_name(query)
            .await
            .map_err(repository_error_to_status)?;

        if request.component_type != proto::ComponentType::Unspecified as i32 {
            components.retain(|component| {
                component_to_proto(component.clone()).component_type == request.component_type
            });
        }

        if let Some(identifier) = request.identifier.as_ref() {
            components.retain(|component| {
                component
                    .identifiers
                    .iter()
                    .any(|candidate| identifier_matches(candidate, identifier))
            });
        }

        let page = Page::new(components, &params);
        let pagination = page_response(&page);
        Ok(Response::new(proto::ListComponentsResponse {
            components: page.items.into_iter().map(component_to_proto).collect(),
            pagination: Some(pagination),
        }))
    }

    async fn get_component(
        &self,
        request: Request<proto::GetComponentRequest>,
    ) -> Result<Response<proto::Component>, Status> {
        let uuid = parse_uuid(&request.into_inner().uuid, "uuid")?;
        let component = self
            .component_repository
            .find_by_uuid(&uuid)
            .await
            .map_err(repository_error_to_status)?
            .ok_or_else(|| Status::not_found(format!("component {uuid} not found")))?;
        Ok(Response::new(component_to_proto(component)))
    }

    async fn list_component_releases(
        &self,
        request: Request<proto::ListComponentReleasesRequest>,
    ) -> Result<Response<proto::ListComponentReleasesResponse>, Status> {
        let request = request.into_inner();
        let component_uuid = parse_uuid(&request.component_uuid, "component_uuid")?;
        let params = pagination_from_proto(request.pagination);
        let mut releases = self
            .component_repository
            .find_releases_by_component(&component_uuid)
            .await
            .map_err(repository_error_to_status)?;

        if !request.include_pre_releases {
            releases.retain(|release| !release.pre_release);
        }

        releases.retain(|release| {
            within_range(release.release_date, request.release_date_range.as_ref())
        });
        releases.sort_by_key(|release| release.release_date);
        releases.reverse();

        let page = Page::new(releases, &params);
        let pagination = page_response(&page);
        Ok(Response::new(proto::ListComponentReleasesResponse {
            releases: page
                .items
                .into_iter()
                .map(component_release_to_proto)
                .collect(),
            pagination: Some(pagination),
        }))
    }

    async fn get_component_release(
        &self,
        request: Request<proto::GetComponentReleaseRequest>,
    ) -> Result<Response<proto::ComponentRelease>, Status> {
        let uuid = parse_uuid(&request.into_inner().uuid, "uuid")?;
        let release = self
            .component_repository
            .find_release_by_uuid(&uuid)
            .await
            .map_err(repository_error_to_status)?
            .ok_or_else(|| Status::not_found(format!("component release {uuid} not found")))?;
        Ok(Response::new(component_release_to_proto(release)))
    }

    async fn get_component_release_collection(
        &self,
        request: Request<proto::GetComponentReleaseCollectionRequest>,
    ) -> Result<Response<proto::Collection>, Status> {
        let request = request.into_inner();
        let uuid = parse_uuid(&request.uuid, "uuid")?;
        let collection = self
            .find_collection_version(&uuid, request.version)
            .await?
            .ok_or_else(|| {
                Status::not_found(format!("collection for component release {uuid} not found"))
            })?;
        let artifacts = self.resolve_collection_artifacts(&collection, true).await?;
        Ok(Response::new(collection_to_proto(collection, artifacts)))
    }

    async fn get_collection(
        &self,
        request: Request<proto::GetCollectionRequest>,
    ) -> Result<Response<proto::Collection>, Status> {
        let request = request.into_inner();
        let uuid = parse_uuid(&request.uuid, "uuid")?;
        let collection = self
            .find_collection_version(&uuid, None)
            .await?
            .ok_or_else(|| Status::not_found(format!("collection {uuid} not found")))?;
        let artifacts = self
            .resolve_collection_artifacts(&collection, request.include_artifacts)
            .await?;
        Ok(Response::new(collection_to_proto(collection, artifacts)))
    }

    async fn list_collection_versions(
        &self,
        request: Request<proto::ListCollectionVersionsRequest>,
    ) -> Result<Response<proto::ListCollectionVersionsResponse>, Status> {
        let request = request.into_inner();
        let uuid = parse_uuid(&request.uuid, "uuid")?;
        let params = pagination_from_proto(request.pagination);
        let mut collections = self
            .collection_repository
            .find_versions_by_uuid(&uuid)
            .await
            .map_err(repository_error_to_status)?;
        collections.sort_by_key(|collection| std::cmp::Reverse(collection.version));

        let page = Page::new(collections, &params);
        let mut full_collections = Vec::new();
        if request.include_artifacts {
            for collection in &page.items {
                let artifacts = self.resolve_collection_artifacts(collection, true).await?;
                full_collections.push(collection_to_proto(collection.clone(), artifacts));
            }
        }

        let versions = page
            .items
            .iter()
            .map(|collection| collection_version_info(collection, collection.artifacts.len()))
            .collect();

        Ok(Response::new(proto::ListCollectionVersionsResponse {
            versions,
            collections: full_collections,
            pagination: Some(page_response(&page)),
        }))
    }

    async fn get_collection_version(
        &self,
        request: Request<proto::GetCollectionVersionRequest>,
    ) -> Result<Response<proto::Collection>, Status> {
        let request = request.into_inner();
        let uuid = parse_uuid(&request.uuid, "uuid")?;
        let collection = self
            .find_collection_version(&uuid, Some(request.version))
            .await?
            .ok_or_else(|| {
                Status::not_found(format!(
                    "collection {uuid} version {} not found",
                    request.version
                ))
            })?;
        let artifacts = self.resolve_collection_artifacts(&collection, true).await?;
        Ok(Response::new(collection_to_proto(collection, artifacts)))
    }

    async fn compare_collection_versions(
        &self,
        request: Request<proto::CompareCollectionVersionsRequest>,
    ) -> Result<Response<proto::CompareCollectionVersionsResponse>, Status> {
        let request = request.into_inner();
        let uuid = parse_uuid(&request.uuid, "uuid")?;

        let base = self
            .find_collection_version(&uuid, Some(request.base_version))
            .await?
            .ok_or_else(|| {
                Status::not_found(format!(
                    "collection {uuid} version {} not found",
                    request.base_version
                ))
            })?;
        let target = self
            .find_collection_version(&uuid, Some(request.target_version))
            .await?
            .ok_or_else(|| {
                Status::not_found(format!(
                    "collection {uuid} version {} not found",
                    request.target_version
                ))
            })?;

        let base_ids: HashSet<String> = base.artifacts.iter().map(Uuid::to_string).collect();
        let target_ids: HashSet<String> = target.artifacts.iter().map(Uuid::to_string).collect();

        let added_artifact_uuids = target_ids.difference(&base_ids).cloned().collect();
        let removed_artifact_uuids = base_ids.difference(&target_ids).cloned().collect();

        Ok(Response::new(proto::CompareCollectionVersionsResponse {
            uuid: uuid.to_string(),
            base_version: request.base_version,
            target_version: request.target_version,
            added_artifact_uuids,
            removed_artifact_uuids,
            modified_artifact_uuids: vec![],
        }))
    }

    async fn get_artifact(
        &self,
        request: Request<proto::GetArtifactRequest>,
    ) -> Result<Response<proto::Artifact>, Status> {
        let uuid = parse_uuid(&request.into_inner().uuid, "uuid")?;
        let artifact = self
            .artifact_repository
            .find_by_uuid(&uuid)
            .await
            .map_err(repository_error_to_status)?
            .ok_or_else(|| Status::not_found(format!("artifact {uuid} not found")))?;
        Ok(Response::new(artifact_to_proto(artifact)))
    }

    async fn get_artifact_content(
        &self,
        _request: Request<proto::GetArtifactContentRequest>,
    ) -> Result<Response<proto::GetArtifactContentResponse>, Status> {
        Err(unimplemented_status(
            "artifact content storage is not wired yet",
        ))
    }

    async fn stream_artifact_content(
        &self,
        _request: Request<proto::GetArtifactContentRequest>,
    ) -> Result<Response<Self::StreamArtifactContentStream>, Status> {
        Err(unimplemented_status(
            "artifact content storage is not wired yet",
        ))
    }

    async fn head_artifact_content(
        &self,
        _request: Request<proto::HeadArtifactContentRequest>,
    ) -> Result<Response<proto::HeadArtifactContentResponse>, Status> {
        Err(unimplemented_status(
            "artifact content storage is not wired yet",
        ))
    }

    async fn search_by_identifier(
        &self,
        request: Request<proto::SearchByIdentifierRequest>,
    ) -> Result<Response<proto::SearchByIdentifierResponse>, Status> {
        let request = request.into_inner();
        let needle = request
            .identifier
            .ok_or_else(|| Status::invalid_argument("identifier is required"))?;

        let wants_all = request.entity_types.is_empty();
        let wants = |entity_type: proto::EntityType| {
            wants_all || request.entity_types.contains(&(entity_type as i32))
        };

        let products = if wants(proto::EntityType::Product) {
            self.product_repository
                .find_by_name("")
                .await
                .map_err(repository_error_to_status)?
                .into_iter()
                .filter(|product| {
                    product
                        .identifiers
                        .iter()
                        .any(|candidate| identifier_matches(candidate, &needle))
                })
                .map(product_to_proto)
                .collect()
        } else {
            vec![]
        };

        let product_releases = if wants(proto::EntityType::ProductRelease) {
            self.all_product_releases(None)
                .await?
                .into_iter()
                .filter(|release| {
                    release
                        .identifiers
                        .iter()
                        .any(|candidate| identifier_matches(candidate, &needle))
                })
                .map(product_release_to_proto)
                .collect()
        } else {
            vec![]
        };

        let components = if wants(proto::EntityType::Component) {
            self.component_repository
                .find_by_name("")
                .await
                .map_err(repository_error_to_status)?
                .into_iter()
                .filter(|component| {
                    component
                        .identifiers
                        .iter()
                        .any(|candidate| identifier_matches(candidate, &needle))
                })
                .map(component_to_proto)
                .collect()
        } else {
            vec![]
        };

        let component_releases = if wants(proto::EntityType::ComponentRelease) {
            self.all_component_releases()
                .await?
                .into_iter()
                .filter(|release| {
                    release
                        .identifiers
                        .iter()
                        .any(|candidate| identifier_matches(candidate, &needle))
                })
                .map(component_release_to_proto)
                .collect()
        } else {
            vec![]
        };

        let total_count =
            products.len() + product_releases.len() + components.len() + component_releases.len();

        Ok(Response::new(proto::SearchByIdentifierResponse {
            products,
            product_releases,
            components,
            component_releases,
            pagination: Some(proto::PageResponse {
                next_page_token: String::new(),
                total_count: Some(total_count as i64),
            }),
        }))
    }

    async fn search_by_checksum(
        &self,
        request: Request<proto::SearchByChecksumRequest>,
    ) -> Result<Response<proto::SearchByChecksumResponse>, Status> {
        let request = request.into_inner();
        let checksum = request
            .checksum
            .ok_or_else(|| Status::invalid_argument("checksum is required"))?;
        let params = pagination_from_proto(request.pagination);
        let releases = self
            .all_component_releases()
            .await?
            .into_iter()
            .filter(|release| {
                release.distributions.iter().any(|distribution| {
                    distribution.checksums.iter().any(|candidate| {
                        let proto_candidate = checksum_to_proto(candidate.clone());
                        proto_candidate.alg_type == checksum.alg_type
                            && proto_candidate.alg_value == checksum.alg_value
                    })
                })
            })
            .collect::<Vec<_>>();

        let page = Page::new(releases, &params);
        let pagination = page_response(&page);
        Ok(Response::new(proto::SearchByChecksumResponse {
            releases: page
                .items
                .into_iter()
                .map(component_release_to_proto)
                .collect(),
            pagination: Some(pagination),
        }))
    }

    async fn search_artifacts(
        &self,
        request: Request<proto::SearchArtifactsRequest>,
    ) -> Result<Response<proto::SearchArtifactsResponse>, Status> {
        let request = request.into_inner();
        let params = pagination_from_proto(request.pagination);
        let query = request.query.trim();
        let mut artifacts = self
            .artifact_repository
            .find_by_name(query)
            .await
            .map_err(repository_error_to_status)?;

        if request.artifact_type != proto::ArtifactType::Unspecified as i32 {
            artifacts.retain(|artifact| {
                artifact_to_proto(artifact.clone()).r#type == request.artifact_type
            });
        }

        if !request.mime_type.trim().is_empty() {
            let mime_type = request.mime_type.trim();
            artifacts.retain(|artifact| {
                artifact
                    .formats
                    .iter()
                    .any(|format| format.mime_type == mime_type)
            });
        }

        if let Some(subject_identifier) = request.subject_identifier.as_ref() {
            artifacts.retain(|artifact| {
                artifact
                    .subject
                    .as_ref()
                    .map(|subject| {
                        subject
                            .identifiers
                            .iter()
                            .any(|candidate| identifier_matches(candidate, subject_identifier))
                    })
                    .unwrap_or(false)
            });
        }

        artifacts.retain(|artifact| {
            within_range(
                Some(artifact.created_date),
                request.created_date_range.as_ref(),
            )
        });

        let page = Page::new(artifacts, &params);
        let pagination = page_response(&page);
        Ok(Response::new(proto::SearchArtifactsResponse {
            artifacts: page.items.into_iter().map(artifact_to_proto).collect(),
            pagination: Some(pagination),
        }))
    }
}

fn repository_error_to_status(error: RepositoryError) -> Status {
    super::conversions::domain_error_to_status(DomainError::Repository(error))
}

fn unimplemented_status(message: &str) -> Status {
    Status::unimplemented(message)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use chrono::Utc;
    use tonic::Request;
    use uuid::Uuid;

    use crate::domain::artifact::entity::{Artifact, ArtifactFormat, ArtifactType};
    use crate::domain::collection::entity::{Collection, CollectionScope, UpdateReason};
    use crate::domain::common::checksum::{Checksum, ChecksumAlgorithm};
    use crate::domain::common::identifier::Identifier;
    use crate::domain::component::entity::{Component, ComponentRelease, ComponentType};
    use crate::domain::product::entity::{ComponentRef, Product, ProductRelease, Vendor};
    use crate::domain::{
        artifact::repository::ArtifactRepository, collection::repository::CollectionRepository,
        component::repository::ComponentRepository, product::repository::ProductRepository,
    };
    use crate::gen::tea::v1 as proto;
    use crate::gen::tea::v1::consumer_service_server::ConsumerService;
    use crate::infrastructure::persistence::memory::artifact_repository::InMemoryArtifactRepository;
    use crate::infrastructure::persistence::memory::collection_repository::InMemoryCollectionRepository;
    use crate::infrastructure::persistence::memory::component_repository::InMemoryComponentRepository;
    use crate::infrastructure::persistence::memory::product_repository::InMemoryProductRepository;

    use super::ConsumerGrpcService;

    fn make_service() -> ConsumerGrpcService<
        InMemoryProductRepository,
        InMemoryComponentRepository,
        InMemoryCollectionRepository,
        InMemoryArtifactRepository,
    > {
        ConsumerGrpcService::new(
            Arc::new(InMemoryProductRepository::new()),
            Arc::new(InMemoryComponentRepository::new()),
            Arc::new(InMemoryCollectionRepository::new()),
            Arc::new(InMemoryArtifactRepository::new()),
        )
    }

    #[tokio::test]
    async fn get_product_returns_proto_message() {
        let service = make_service();
        let product = Product {
            uuid: Uuid::new_v4(),
            name: "Widget".to_string(),
            description: Some("Test product".to_string()),
            identifiers: vec![Identifier::purl("pkg:cargo/acme/widget@1.0.0")],
            vendor: Vendor {
                name: "ACME".to_string(),
                uuid: None,
                url: None,
                contacts: vec![],
            },
            created_date: Utc::now(),
            modified_date: Utc::now(),
            homepage_url: Some("https://example.com/widget".to_string()),
            documentation_url: None,
            vcs_url: None,
            deprecation: None,
            dependencies: vec![],
        };
        service.product_repository.save(&product).await.unwrap();

        let response = service
            .get_product(Request::new(proto::GetProductRequest {
                uuid: product.uuid.to_string(),
            }))
            .await
            .unwrap()
            .into_inner();

        assert_eq!(response.uuid, product.uuid.to_string());
        assert_eq!(response.name, "Widget");
        assert_eq!(response.identifiers.len(), 1);
    }

    #[tokio::test]
    async fn list_and_get_product_releases_return_proto_messages() {
        let service = make_service();
        let product_uuid = Uuid::new_v4();
        let release_uuid = Uuid::new_v4();
        service
            .product_repository
            .save(&Product {
                uuid: product_uuid,
                name: "Widget".to_string(),
                description: None,
                identifiers: vec![],
                vendor: Vendor {
                    name: "ACME".to_string(),
                    uuid: None,
                    url: None,
                    contacts: vec![],
                },
                created_date: Utc::now(),
                modified_date: Utc::now(),
                homepage_url: None,
                documentation_url: None,
                vcs_url: None,
                deprecation: None,
                dependencies: vec![],
            })
            .await
            .unwrap();
        service
            .product_repository
            .save_release(&ProductRelease {
                uuid: release_uuid,
                product_uuid,
                version: "2026.03".to_string(),
                created_date: Utc::now(),
                modified_date: Utc::now(),
                release_date: Some(Utc::now()),
                pre_release: false,
                identifiers: vec![Identifier::tei(format!(
                    "urn:tei:uuid:tea.example.com:{release_uuid}"
                ))],
                components: vec![ComponentRef {
                    component_uuid: Uuid::new_v4(),
                    release_uuid: Uuid::new_v4(),
                }],
            })
            .await
            .unwrap();

        let listed = service
            .list_product_releases(Request::new(proto::ListProductReleasesRequest {
                product_uuid: product_uuid.to_string(),
                pagination: None,
                include_pre_releases: true,
                release_date_range: None,
                sort: None,
            }))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(listed.releases.len(), 1);
        assert_eq!(listed.releases[0].uuid, release_uuid.to_string());

        let fetched = service
            .get_product_release(Request::new(proto::GetProductReleaseRequest {
                uuid: release_uuid.to_string(),
            }))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(fetched.version, "2026.03");
        assert_eq!(fetched.product, Some(product_uuid.to_string()));
        assert_eq!(fetched.components.len(), 1);
    }

    #[tokio::test]
    async fn list_component_releases_filters_pre_releases() {
        let service = make_service();
        let component_uuid = Uuid::new_v4();
        service
            .component_repository
            .save(&Component {
                uuid: component_uuid,
                name: "lib".to_string(),
                description: None,
                identifiers: vec![],
                component_type: ComponentType::Library,
                licenses: vec![],
                publisher: None,
                homepage_url: None,
                vcs_url: None,
                created_date: Utc::now(),
                modified_date: Utc::now(),
                deprecation: None,
                dependencies: vec![],
            })
            .await
            .unwrap();
        service
            .component_repository
            .save_release(&ComponentRelease {
                uuid: Uuid::new_v4(),
                component_uuid,
                version: "1.0.0".to_string(),
                release_date: Some(Utc::now()),
                pre_release: false,
                identifiers: vec![],
                distributions: vec![],
            })
            .await
            .unwrap();
        service
            .component_repository
            .save_release(&ComponentRelease {
                uuid: Uuid::new_v4(),
                component_uuid,
                version: "2.0.0-rc1".to_string(),
                release_date: Some(Utc::now()),
                pre_release: true,
                identifiers: vec![],
                distributions: vec![],
            })
            .await
            .unwrap();

        let response = service
            .list_component_releases(Request::new(proto::ListComponentReleasesRequest {
                component_uuid: component_uuid.to_string(),
                pagination: None,
                include_pre_releases: false,
                release_date_range: None,
                sort: None,
            }))
            .await
            .unwrap()
            .into_inner();

        assert_eq!(response.releases.len(), 1);
        assert_eq!(response.releases[0].version, "1.0.0");
    }

    #[tokio::test]
    async fn get_collection_resolves_full_artifacts() {
        let service = make_service();
        let artifact_uuid = Uuid::new_v4();
        service
            .artifact_repository
            .save(&Artifact {
                uuid: artifact_uuid,
                name: "SBOM".to_string(),
                type_: ArtifactType::Bom,
                component_distributions: vec!["oci".to_string()],
                formats: vec![ArtifactFormat {
                    mime_type: "application/json".to_string(),
                    description: None,
                    url: "https://example.com/sbom.json".to_string(),
                    signature_url: None,
                    checksums: vec![Checksum {
                        alg_type: ChecksumAlgorithm::Sha256,
                        alg_value: "a".repeat(64),
                    }],
                    size_bytes: None,
                    encoding: None,
                    spec_version: Some("1.6".to_string()),
                }],
                created_date: Utc::now(),
                modified_date: Utc::now(),
                description: Some("artifact".to_string()),
                subject: None,
                deprecation: None,
                dependencies: vec![],
            })
            .await
            .unwrap();

        let collection_uuid = Uuid::new_v4();
        service
            .collection_repository
            .save(&Collection {
                uuid: collection_uuid,
                name: "Release Collection".to_string(),
                version: 1,
                date: Utc::now(),
                created_date: Utc::now(),
                modified_date: Utc::now(),
                belongs_to: CollectionScope::ProductRelease,
                update_reason: UpdateReason::InitialRelease,
                artifacts: vec![artifact_uuid],
                deprecation: None,
                dependencies: vec![],
            })
            .await
            .unwrap();

        let response = service
            .get_collection(Request::new(proto::GetCollectionRequest {
                uuid: collection_uuid.to_string(),
                include_artifacts: true,
            }))
            .await
            .unwrap()
            .into_inner();

        assert_eq!(response.uuid, collection_uuid.to_string());
        assert_eq!(response.artifacts.len(), 1);
        assert_eq!(response.artifacts[0].uuid, artifact_uuid.to_string());
        assert_eq!(response.artifacts[0].name, "SBOM");
    }

    #[tokio::test]
    async fn artifact_content_methods_fail_closed_until_storage_is_wired() {
        let service = make_service();
        let status = service
            .get_artifact_content(Request::new(proto::GetArtifactContentRequest {
                uuid: Uuid::new_v4().to_string(),
                preferred_format: String::new(),
                if_none_match: String::new(),
                range: String::new(),
            }))
            .await
            .expect_err("content download should remain unimplemented");

        assert_eq!(status.code(), tonic::Code::Unimplemented);
    }
}
