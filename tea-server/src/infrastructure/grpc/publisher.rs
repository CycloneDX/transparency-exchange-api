#![allow(clippy::result_large_err)]

use std::net::{IpAddr, Ipv6Addr};
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Utc};
use futures_util::StreamExt;
use prost_types::{FieldMask, Timestamp};
use sha2::Digest;
use tonic::{Request, Response, Status, Streaming};
use uuid::Uuid;

use crate::application::publisher::service::PublisherApplicationService;
use crate::domain::artifact::entity::{
    Artifact, ArtifactFormat, ArtifactType as DomainArtifactType, Subject,
    SubjectType as DomainSubjectType,
};
use crate::domain::collection::entity::{
    Collection, CollectionScope as DomainCollectionScope, UpdateReason as DomainUpdateReason,
};
use crate::domain::common::checksum::{Checksum, ChecksumAlgorithm};
use crate::domain::common::error::DomainError;
use crate::domain::common::identifier::{Identifier, IdentifierType};
use crate::domain::component::entity::{
    Component, ComponentRelease, ComponentType as DomainComponentType, Distribution, LicenseInfo,
    LicenseType,
};
use crate::domain::product::entity::{
    ComponentRef as ProductReleaseComponentRef, Contact, ContactType, Product, ProductRelease,
    Vendor,
};
use crate::gen::tea::v1::{self as proto, publisher_service_server::PublisherService};

use super::conversions::{
    artifact_to_proto, collection_to_proto, component_release_to_proto, component_to_proto,
    product_release_to_proto, product_to_proto,
};

#[derive(Clone)]
pub struct PublisherGrpcService<A: PublisherApplicationService + ?Sized> {
    app_service: Arc<A>,
}

impl<A: PublisherApplicationService + ?Sized> PublisherGrpcService<A> {
    pub fn new(app_service: Arc<A>) -> Self {
        Self { app_service }
    }
}

impl<A> PublisherGrpcService<A>
where
    A: PublisherApplicationService + ?Sized + 'static,
{
    async fn fetch_remote_artifact(
        &self,
        source_url: &str,
        expected_checksums: &[Checksum],
    ) -> Result<i64, Status> {
        validate_remote_source_url(source_url).await?;

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|error| {
                Status::internal(format!(
                    "failed to initialize HTTP client for source fetch: {error}"
                ))
            })?;

        let response =
            client.get(source_url).send().await.map_err(|error| {
                Status::unavailable(format!("failed to fetch source_url: {error}"))
            })?;
        let response = response.error_for_status().map_err(|error| {
            Status::failed_precondition(format!("source_url returned an error: {error}"))
        })?;

        if let Some(content_length) = response.content_length() {
            if content_length > max_remote_artifact_bytes() as u64 {
                return Err(Status::resource_exhausted(format!(
                    "source artifact exceeds the {} byte limit",
                    max_remote_artifact_bytes()
                )));
            }
        }

        let mut content = Vec::new();
        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|error| {
                Status::unavailable(format!("failed to read source_url body: {error}"))
            })?;
            if content.len() + chunk.len() > max_remote_artifact_bytes() {
                return Err(Status::resource_exhausted(format!(
                    "source artifact exceeds the {} byte limit",
                    max_remote_artifact_bytes()
                )));
            }
            content.extend_from_slice(&chunk);
        }

        verify_expected_checksums(&content, expected_checksums)?;
        Ok(content.len() as i64)
    }

    async fn resolve_artifacts(
        &self,
        artifact_uuids: &[Uuid],
    ) -> Result<Vec<proto::Artifact>, Status> {
        let mut artifacts = Vec::with_capacity(artifact_uuids.len());
        for artifact_uuid in artifact_uuids {
            let artifact = self
                .app_service
                .get_artifact(artifact_uuid)
                .await
                .map_err(domain_error_to_status)?
                .ok_or_else(|| Status::not_found(format!("artifact {artifact_uuid} not found")))?;
            artifacts.push(artifact_to_proto(artifact));
        }
        Ok(artifacts)
    }

    async fn validate_collection_subject(
        &self,
        uuid: &Uuid,
        belongs_to: &DomainCollectionScope,
    ) -> Result<String, Status> {
        match belongs_to {
            DomainCollectionScope::Release => {
                let release = self
                    .app_service
                    .get_component_release(uuid)
                    .await
                    .map_err(domain_error_to_status)?
                    .ok_or_else(|| {
                        Status::not_found(format!("component release {uuid} not found"))
                    })?;
                Ok(format!("Component release {}", release.version))
            }
            DomainCollectionScope::ProductRelease => {
                let release = self
                    .app_service
                    .get_product_release(uuid)
                    .await
                    .map_err(domain_error_to_status)?
                    .ok_or_else(|| {
                        Status::not_found(format!("product release {uuid} not found"))
                    })?;
                Ok(format!("Product release {}", release.version))
            }
            DomainCollectionScope::Unspecified => Err(Status::invalid_argument(
                "belongs_to must be RELEASE or PRODUCT_RELEASE",
            )),
        }
    }
}

#[tonic::async_trait]
impl<A> PublisherService for PublisherGrpcService<A>
where
    A: PublisherApplicationService + ?Sized + 'static,
{
    async fn create_product(
        &self,
        request: Request<proto::CreateProductRequest>,
    ) -> Result<Response<proto::Product>, Status> {
        let product = self
            .app_service
            .create_product(product_from_create(request.into_inner())?)
            .await
            .map_err(domain_error_to_status)?;
        Ok(Response::new(product_to_proto(product)))
    }

    async fn update_product(
        &self,
        request: Request<proto::UpdateProductRequest>,
    ) -> Result<Response<proto::Product>, Status> {
        let request = request.into_inner();
        let uuid = parse_uuid(&request.uuid, "uuid")?;
        let mut product = self
            .app_service
            .get_product(&uuid)
            .await
            .map_err(domain_error_to_status)?
            .ok_or_else(|| Status::not_found(format!("product {uuid} not found")))?;
        merge_product_update(&mut product, &request)?;
        let product = self
            .app_service
            .update_product(product)
            .await
            .map_err(domain_error_to_status)?;
        Ok(Response::new(product_to_proto(product)))
    }

    async fn delete_product(
        &self,
        request: Request<proto::DeleteProductRequest>,
    ) -> Result<Response<proto::DeleteProductResponse>, Status> {
        let request = request.into_inner();
        let uuid = parse_uuid(&request.uuid, "uuid")?;
        let releases = self
            .app_service
            .list_product_releases(&uuid)
            .await
            .map_err(domain_error_to_status)?;

        if !request.cascade && !releases.is_empty() {
            return Err(Status::failed_precondition(
                "product still has releases; set cascade=true to delete them",
            ));
        }

        let mut releases_deleted = 0;
        if request.cascade {
            for release in &releases {
                self.app_service
                    .delete_product_release(&release.uuid)
                    .await
                    .map_err(domain_error_to_status)?;
                releases_deleted += 1;
            }
        }

        self.app_service
            .delete_product(&uuid)
            .await
            .map_err(domain_error_to_status)?;
        Ok(Response::new(proto::DeleteProductResponse {
            uuid: uuid.to_string(),
            releases_deleted,
        }))
    }

    async fn create_product_release(
        &self,
        request: Request<proto::CreateProductReleaseRequest>,
    ) -> Result<Response<proto::ProductRelease>, Status> {
        let release = self
            .app_service
            .create_product_release(product_release_from_create(request.into_inner())?)
            .await
            .map_err(domain_error_to_status)?;
        Ok(Response::new(product_release_to_proto(release)))
    }

    async fn update_product_release(
        &self,
        request: Request<proto::UpdateProductReleaseRequest>,
    ) -> Result<Response<proto::ProductRelease>, Status> {
        let request = request.into_inner();
        let uuid = parse_uuid(&request.uuid, "uuid")?;
        let mut release = self
            .app_service
            .get_product_release(&uuid)
            .await
            .map_err(domain_error_to_status)?
            .ok_or_else(|| Status::not_found(format!("product release {uuid} not found")))?;
        merge_product_release_update(&mut release, &request)?;
        let release = self
            .app_service
            .update_product_release(release)
            .await
            .map_err(domain_error_to_status)?;
        Ok(Response::new(product_release_to_proto(release)))
    }

    async fn delete_product_release(
        &self,
        request: Request<proto::DeleteProductReleaseRequest>,
    ) -> Result<Response<proto::DeleteProductReleaseResponse>, Status> {
        let uuid = parse_uuid(&request.into_inner().uuid, "uuid")?;
        self.app_service
            .delete_product_release(&uuid)
            .await
            .map_err(domain_error_to_status)?;
        Ok(Response::new(proto::DeleteProductReleaseResponse {
            uuid: uuid.to_string(),
        }))
    }

    async fn create_component(
        &self,
        request: Request<proto::CreateComponentRequest>,
    ) -> Result<Response<proto::Component>, Status> {
        let component = self
            .app_service
            .create_component(component_from_create(request.into_inner())?)
            .await
            .map_err(domain_error_to_status)?;
        Ok(Response::new(component_to_proto(component)))
    }

    async fn update_component(
        &self,
        request: Request<proto::UpdateComponentRequest>,
    ) -> Result<Response<proto::Component>, Status> {
        let request = request.into_inner();
        let uuid = parse_uuid(&request.uuid, "uuid")?;
        let mut component = self
            .app_service
            .get_component(&uuid)
            .await
            .map_err(domain_error_to_status)?
            .ok_or_else(|| Status::not_found(format!("component {uuid} not found")))?;
        merge_component_update(&mut component, &request)?;
        let component = self
            .app_service
            .update_component(component)
            .await
            .map_err(domain_error_to_status)?;
        Ok(Response::new(component_to_proto(component)))
    }

    async fn delete_component(
        &self,
        request: Request<proto::DeleteComponentRequest>,
    ) -> Result<Response<proto::DeleteComponentResponse>, Status> {
        let request = request.into_inner();
        let uuid = parse_uuid(&request.uuid, "uuid")?;
        let releases = self
            .app_service
            .list_component_releases(&uuid)
            .await
            .map_err(domain_error_to_status)?;

        if !request.cascade && !releases.is_empty() {
            return Err(Status::failed_precondition(
                "component still has releases; set cascade=true to delete them",
            ));
        }

        let mut releases_deleted = 0;
        if request.cascade {
            for release in &releases {
                self.app_service
                    .delete_component_release(&release.uuid)
                    .await
                    .map_err(domain_error_to_status)?;
                releases_deleted += 1;
            }
        }

        self.app_service
            .delete_component(&uuid)
            .await
            .map_err(domain_error_to_status)?;
        Ok(Response::new(proto::DeleteComponentResponse {
            uuid: uuid.to_string(),
            releases_deleted,
        }))
    }

    async fn create_component_release(
        &self,
        request: Request<proto::CreateComponentReleaseRequest>,
    ) -> Result<Response<proto::ComponentRelease>, Status> {
        let release = component_release_from_create(request.into_inner())?;
        let parent_component = release.component_uuid;
        if self
            .app_service
            .get_component(&parent_component)
            .await
            .map_err(domain_error_to_status)?
            .is_none()
        {
            return Err(Status::not_found(format!(
                "component {parent_component} not found"
            )));
        }

        let release = self
            .app_service
            .create_component_release(release)
            .await
            .map_err(domain_error_to_status)?;
        Ok(Response::new(component_release_to_proto(release)))
    }

    async fn update_component_release(
        &self,
        request: Request<proto::UpdateComponentReleaseRequest>,
    ) -> Result<Response<proto::ComponentRelease>, Status> {
        let request = request.into_inner();
        let uuid = parse_uuid(&request.uuid, "uuid")?;
        let mut release = self
            .app_service
            .get_component_release(&uuid)
            .await
            .map_err(domain_error_to_status)?
            .ok_or_else(|| Status::not_found(format!("component release {uuid} not found")))?;
        merge_component_release_update(&mut release, &request)?;
        let release = self
            .app_service
            .update_component_release(release)
            .await
            .map_err(domain_error_to_status)?;
        Ok(Response::new(component_release_to_proto(release)))
    }

    async fn delete_component_release(
        &self,
        request: Request<proto::DeleteComponentReleaseRequest>,
    ) -> Result<Response<proto::DeleteComponentReleaseResponse>, Status> {
        let uuid = parse_uuid(&request.into_inner().uuid, "uuid")?;
        self.app_service
            .delete_component_release(&uuid)
            .await
            .map_err(domain_error_to_status)?;
        Ok(Response::new(proto::DeleteComponentReleaseResponse {
            uuid: uuid.to_string(),
        }))
    }

    async fn upload_artifact(
        &self,
        _request: Request<Streaming<proto::UploadArtifactRequest>>,
    ) -> Result<Response<proto::Artifact>, Status> {
        Err(unsupported("artifact upload storage is not wired yet"))
    }

    async fn create_artifact_from_url(
        &self,
        request: Request<proto::CreateArtifactFromUrlRequest>,
    ) -> Result<Response<proto::Artifact>, Status> {
        let request = request.into_inner();
        let metadata = request
            .metadata
            .ok_or_else(|| Status::invalid_argument("metadata is required"))?;
        let source_url = request.source_url;
        let signature_url = option_string(request.signature_url);
        let expected_checksums = merge_expected_checksums(
            metadata.expected_checksums.clone(),
            request.expected_checksums,
        )?;
        let size_bytes = self
            .fetch_remote_artifact(&source_url, &expected_checksums)
            .await?;

        let artifact = artifact_from_metadata(
            metadata,
            source_url,
            signature_url,
            expected_checksums,
            size_bytes,
        )?;

        let artifact = self
            .app_service
            .create_artifact(artifact)
            .await
            .map_err(domain_error_to_status)?;
        Ok(Response::new(artifact_to_proto(artifact)))
    }

    async fn delete_artifact(
        &self,
        request: Request<proto::DeleteArtifactRequest>,
    ) -> Result<Response<proto::DeleteArtifactResponse>, Status> {
        let request = request.into_inner();
        let uuid = parse_uuid(&request.uuid, "uuid")?;
        let referenced_by = collections_referencing_artifact(
            &self
                .app_service
                .list_collections()
                .await
                .map_err(domain_error_to_status)?,
            uuid,
        );

        if !referenced_by.is_empty() {
            if request.force {
                return Err(Status::unimplemented(
                    "force-deleting artifacts that are still referenced by collections is not wired yet",
                ));
            }
            return Err(Status::failed_precondition(format!(
                "artifact {uuid} is still referenced by {} collection(s)",
                referenced_by.len()
            )));
        }

        self.app_service
            .delete_artifact(&uuid)
            .await
            .map_err(domain_error_to_status)?;
        Ok(Response::new(proto::DeleteArtifactResponse {
            uuid: uuid.to_string(),
            affected_collection_uuids: vec![],
        }))
    }

    async fn create_collection(
        &self,
        request: Request<proto::CreateCollectionRequest>,
    ) -> Result<Response<proto::Collection>, Status> {
        let request = request.into_inner();
        let uuid = parse_uuid(&request.uuid, "uuid")?;
        let belongs_to = parse_collection_scope(request.belongs_to)?;
        let update_reason = parse_update_reason(
            request.update_reason,
            "update_reason",
            true,
            Some(DomainUpdateReason::InitialRelease),
        )?;
        let name = self.validate_collection_subject(&uuid, &belongs_to).await?;
        let artifact_uuids = parse_artifact_uuids(request.artifact_uuids)?;
        let artifacts = self.resolve_artifacts(&artifact_uuids).await?;

        let collection = self
            .app_service
            .create_collection(Collection {
                uuid,
                name,
                version: 1,
                date: Utc::now(),
                created_date: Utc::now(),
                modified_date: Utc::now(),
                belongs_to,
                update_reason,
                artifacts: artifact_uuids,
                deprecation: None,
                dependencies: vec![],
            })
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(collection_to_proto(collection, artifacts)))
    }

    async fn update_collection(
        &self,
        request: Request<proto::UpdateCollectionRequest>,
    ) -> Result<Response<proto::Collection>, Status> {
        let request = request.into_inner();
        let uuid = parse_uuid(&request.uuid, "uuid")?;
        let latest = self
            .app_service
            .get_collection(&uuid)
            .await
            .map_err(domain_error_to_status)?
            .ok_or_else(|| Status::not_found(format!("collection {uuid} not found")))?;
        let artifact_uuids = parse_artifact_uuids(request.artifact_uuids)?;
        let update_reason =
            parse_update_reason(request.update_reason, "update_reason", true, None)?;
        let artifacts = self.resolve_artifacts(&artifact_uuids).await?;

        let collection = self
            .app_service
            .create_collection(Collection {
                uuid: latest.uuid,
                name: latest.name,
                version: latest.version + 1,
                date: Utc::now(),
                created_date: latest.created_date,
                modified_date: Utc::now(),
                belongs_to: latest.belongs_to,
                update_reason,
                artifacts: artifact_uuids,
                deprecation: latest.deprecation,
                dependencies: latest.dependencies,
            })
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(collection_to_proto(collection, artifacts)))
    }

    async fn sign_collection(
        &self,
        _request: Request<proto::SignCollectionRequest>,
    ) -> Result<Response<proto::Collection>, Status> {
        Err(unsupported(
            "collection signing requires a real signing backend and artifact storage target",
        ))
    }

    async fn batch_upload_artifacts(
        &self,
        _request: Request<Streaming<proto::BatchUploadArtifactsRequest>>,
    ) -> Result<Response<proto::BatchUploadArtifactsResponse>, Status> {
        Err(unsupported(
            "batch artifact uploads require object storage; use CreateArtifactFromUrl for pre-uploaded immutable artifacts",
        ))
    }

    async fn import_collection(
        &self,
        _request: Request<Streaming<proto::ImportCollectionRequest>>,
    ) -> Result<Response<proto::ImportCollectionResponse>, Status> {
        Err(unsupported(
            "collection import requires object storage-backed artifact uploads and remains disabled",
        ))
    }
}

fn unsupported(message: &str) -> Status {
    // The protobuf contract is canonical, but the reference server only exposes
    // the subset that has real storage/signing semantics behind it today.
    Status::unimplemented(message)
}

fn domain_error_to_status(error: DomainError) -> Status {
    super::conversions::domain_error_to_status(error)
}

fn parse_uuid(value: &str, field: &str) -> Result<Uuid, Status> {
    super::conversions::parse_uuid(value, field)
}

fn product_from_create(request: proto::CreateProductRequest) -> Result<Product, Status> {
    Ok(Product {
        uuid: parse_optional_uuid(request.uuid, "uuid")?.unwrap_or_default(),
        name: request.name,
        description: option_string(request.description),
        identifiers: parse_identifiers(request.identifiers)?,
        vendor: parse_vendor(request.vendor, true)?,
        created_date: Utc::now(),
        modified_date: Utc::now(),
        homepage_url: option_string(request.homepage_url),
        documentation_url: option_string(request.documentation_url),
        vcs_url: option_string(request.vcs_url),
        deprecation: None,
        dependencies: vec![],
    })
}

fn merge_product_update(
    product: &mut Product,
    request: &proto::UpdateProductRequest,
) -> Result<(), Status> {
    for path in required_update_mask(request.update_mask.as_ref(), "update product")? {
        match path.as_str() {
            "name" => product.name = request.name.clone(),
            "description" => product.description = option_string(request.description.clone()),
            "identifiers" => product.identifiers = parse_identifiers(request.identifiers.clone())?,
            "vendor" => product.vendor = parse_vendor(request.vendor.clone(), true)?,
            "homepage_url" | "homepageUrl" => {
                product.homepage_url = option_string(request.homepage_url.clone())
            }
            "documentation_url" | "documentationUrl" => {
                product.documentation_url = option_string(request.documentation_url.clone())
            }
            "vcs_url" | "vcsUrl" => product.vcs_url = option_string(request.vcs_url.clone()),
            other => {
                return Err(Status::invalid_argument(format!(
                    "unsupported update field for product: {other}"
                )))
            }
        }
    }
    Ok(())
}

fn product_release_from_create(
    request: proto::CreateProductReleaseRequest,
) -> Result<ProductRelease, Status> {
    Ok(ProductRelease {
        uuid: parse_optional_uuid(request.uuid, "uuid")?.unwrap_or_default(),
        product_uuid: parse_required_optional_uuid(
            request.product_uuid,
            "product_uuid",
            "product_uuid is required by the current reference server",
        )?,
        version: request.version,
        created_date: Utc::now(),
        modified_date: Utc::now(),
        release_date: timestamp_from_proto(request.release_date, "release_date")?,
        pre_release: request.pre_release,
        identifiers: parse_identifiers(request.identifiers)?,
        components: request
            .components
            .into_iter()
            .enumerate()
            .map(|(index, component)| parse_product_release_component_ref(component, index))
            .collect::<Result<Vec<_>, _>>()?,
    })
}

fn merge_product_release_update(
    release: &mut ProductRelease,
    request: &proto::UpdateProductReleaseRequest,
) -> Result<(), Status> {
    for path in required_update_mask(request.update_mask.as_ref(), "update product release")? {
        match path.as_str() {
            "version" => release.version = request.version.clone(),
            "release_date" | "releaseDate" => {
                release.release_date =
                    timestamp_from_proto(request.release_date, "release_date")?
            }
            "pre_release" | "preRelease" => {
                if !release.pre_release && request.pre_release {
                    return Err(Status::invalid_argument(
                        "pre_release can only change from true to false",
                    ));
                }
                release.pre_release = request.pre_release;
            }
            "identifiers" => release.identifiers = parse_identifiers(request.identifiers.clone())?,
            "components" => {
                release.components = request
                    .components
                    .clone()
                    .into_iter()
                    .enumerate()
                    .map(|(index, component)| parse_product_release_component_ref(component, index))
                    .collect::<Result<Vec<_>, _>>()?
            }
            other => {
                return Err(Status::invalid_argument(format!(
                    "unsupported update field for product release: {other}"
                )))
            }
        }
    }
    Ok(())
}

fn component_from_create(request: proto::CreateComponentRequest) -> Result<Component, Status> {
    Ok(Component {
        uuid: parse_optional_uuid(request.uuid, "uuid")?.unwrap_or_default(),
        name: request.name,
        description: option_string(request.description),
        identifiers: parse_identifiers(request.identifiers)?,
        component_type: parse_component_type(request.component_type),
        licenses: request
            .licenses
            .into_iter()
            .map(parse_license)
            .collect::<Result<Vec<_>, _>>()?,
        publisher: option_string(request.publisher),
        homepage_url: option_string(request.homepage_url),
        vcs_url: option_string(request.vcs_url),
        created_date: Utc::now(),
        modified_date: Utc::now(),
        deprecation: None,
        dependencies: vec![],
    })
}

fn merge_component_update(
    component: &mut Component,
    request: &proto::UpdateComponentRequest,
) -> Result<(), Status> {
    for path in required_update_mask(request.update_mask.as_ref(), "update component")? {
        match path.as_str() {
            "name" => component.name = request.name.clone(),
            "description" => component.description = option_string(request.description.clone()),
            "identifiers" => {
                component.identifiers = parse_identifiers(request.identifiers.clone())?
            }
            "component_type" | "componentType" => {
                component.component_type = parse_component_type(request.component_type)
            }
            "licenses" => {
                component.licenses = request
                    .licenses
                    .clone()
                    .into_iter()
                    .map(parse_license)
                    .collect::<Result<Vec<_>, _>>()?
            }
            "publisher" => component.publisher = option_string(request.publisher.clone()),
            "homepage_url" | "homepageUrl" => {
                component.homepage_url = option_string(request.homepage_url.clone())
            }
            "vcs_url" | "vcsUrl" => component.vcs_url = option_string(request.vcs_url.clone()),
            other => {
                return Err(Status::invalid_argument(format!(
                    "unsupported update field for component: {other}"
                )))
            }
        }
    }
    Ok(())
}

fn component_release_from_create(
    request: proto::CreateComponentReleaseRequest,
) -> Result<ComponentRelease, Status> {
    Ok(ComponentRelease {
        uuid: parse_optional_uuid(request.uuid, "uuid")?.unwrap_or_default(),
        component_uuid: parse_uuid(&request.component_uuid, "component_uuid")?,
        version: request.version,
        release_date: timestamp_from_proto(request.release_date, "release_date")?,
        pre_release: request.pre_release,
        identifiers: parse_identifiers(request.identifiers)?,
        distributions: request
            .distributions
            .into_iter()
            .map(parse_distribution)
            .collect::<Result<Vec<_>, _>>()?,
    })
}

fn merge_component_release_update(
    release: &mut ComponentRelease,
    request: &proto::UpdateComponentReleaseRequest,
) -> Result<(), Status> {
    for path in required_update_mask(request.update_mask.as_ref(), "update component release")? {
        match path.as_str() {
            "version" => release.version = request.version.clone(),
            "release_date" | "releaseDate" => {
                release.release_date =
                    timestamp_from_proto(request.release_date, "release_date")?
            }
            "pre_release" | "preRelease" => {
                if !release.pre_release && request.pre_release {
                    return Err(Status::invalid_argument(
                        "pre_release can only change from true to false",
                    ));
                }
                release.pre_release = request.pre_release;
            }
            "identifiers" => release.identifiers = parse_identifiers(request.identifiers.clone())?,
            "distributions" => {
                release.distributions = request
                    .distributions
                    .clone()
                    .into_iter()
                    .map(parse_distribution)
                    .collect::<Result<Vec<_>, _>>()?
            }
            other => {
                return Err(Status::invalid_argument(format!(
                    "unsupported update field for component release: {other}"
                )))
            }
        }
    }
    Ok(())
}

fn parse_vendor(vendor: Option<proto::Vendor>, required: bool) -> Result<Vendor, Status> {
    match vendor {
        Some(vendor) => {
            let uuid = vendor
                .uuid
                .as_deref()
                .map(|value| parse_uuid(value, "vendor.uuid"))
                .transpose()?;
            let mut contacts = Vec::new();
            for contact in vendor.contacts {
                if !contact.email.is_empty() {
                    contacts.push(Contact {
                        type_: ContactType::Email,
                        value: contact.email,
                    });
                }
                if !contact.phone.is_empty() {
                    contacts.push(Contact {
                        type_: ContactType::Phone,
                        value: contact.phone,
                    });
                }
                if !contact.name.is_empty() {
                    contacts.push(Contact {
                        type_: ContactType::Other,
                        value: contact.name,
                    });
                }
            }

            Ok(Vendor {
                name: vendor.name,
                uuid,
                url: option_string(vendor.url),
                contacts,
            })
        }
        None if required => Err(Status::invalid_argument("vendor is required")),
        None => Ok(Vendor {
            name: String::new(),
            uuid: None,
            url: None,
            contacts: vec![],
        }),
    }
}

fn parse_license(license: proto::LicenseInfo) -> Result<LicenseInfo, Status> {
    if !license.spdx_id.is_empty() {
        return Ok(LicenseInfo {
            license_type: LicenseType::Spdx,
            license_id: license.spdx_id,
            url: option_string(license.url),
        });
    }

    if !license.name.is_empty() {
        return Ok(LicenseInfo {
            license_type: LicenseType::Other,
            license_id: license.name,
            url: option_string(license.url),
        });
    }

    Err(Status::invalid_argument(
        "license must include either spdx_id or name",
    ))
}

fn parse_distribution(distribution: proto::Distribution) -> Result<Distribution, Status> {
    Ok(Distribution {
        distribution_type: distribution.distribution_type,
        description: option_string(distribution.description),
        identifiers: parse_identifiers(distribution.identifiers)?,
        url: option_string(distribution.url),
        signature_url: option_string(distribution.signature_url),
        checksums: distribution
            .checksums
            .into_iter()
            .map(parse_checksum)
            .collect::<Result<Vec<_>, _>>()?,
    })
}

fn parse_identifiers(values: Vec<proto::Identifier>) -> Result<Vec<Identifier>, Status> {
    values.into_iter().map(parse_identifier).collect()
}

fn parse_identifier(value: proto::Identifier) -> Result<Identifier, Status> {
    Ok(Identifier {
        id_type: parse_identifier_type(value.id_type),
        id_value: value.id_value,
    })
}

fn parse_identifier_type(value: i32) -> IdentifierType {
    match proto::IdentifierType::try_from(value).unwrap_or(proto::IdentifierType::Unspecified) {
        proto::IdentifierType::Tei => IdentifierType::Tei,
        proto::IdentifierType::Purl => IdentifierType::Purl,
        proto::IdentifierType::Cpe => IdentifierType::Cpe,
        proto::IdentifierType::Swid => IdentifierType::Swid,
        proto::IdentifierType::Gav => IdentifierType::Gav,
        proto::IdentifierType::Gtin => IdentifierType::Gtin,
        proto::IdentifierType::Gmn => IdentifierType::Gmn,
        proto::IdentifierType::Udi => IdentifierType::Udi,
        proto::IdentifierType::Asin => IdentifierType::Asin,
        proto::IdentifierType::Hash => IdentifierType::Hash,
        proto::IdentifierType::Unspecified => IdentifierType::Unspecified,
    }
}

fn parse_checksum(value: proto::Checksum) -> Result<Checksum, Status> {
    Ok(Checksum {
        alg_type: parse_checksum_algorithm(value.alg_type)?,
        alg_value: value.alg_value,
    })
}

fn parse_checksum_algorithm(value: i32) -> Result<ChecksumAlgorithm, Status> {
    match proto::ChecksumAlgorithm::try_from(value).unwrap_or(proto::ChecksumAlgorithm::Unspecified)
    {
        proto::ChecksumAlgorithm::Unspecified => Ok(ChecksumAlgorithm::Unspecified),
        proto::ChecksumAlgorithm::Md5 => Ok(ChecksumAlgorithm::Md5),
        proto::ChecksumAlgorithm::Sha1 => Ok(ChecksumAlgorithm::Sha1),
        proto::ChecksumAlgorithm::Sha256 => Ok(ChecksumAlgorithm::Sha256),
        proto::ChecksumAlgorithm::Sha384 => Ok(ChecksumAlgorithm::Sha384),
        proto::ChecksumAlgorithm::Sha512 => Ok(ChecksumAlgorithm::Sha512),
        proto::ChecksumAlgorithm::Sha3256 => Ok(ChecksumAlgorithm::Sha3_256),
        proto::ChecksumAlgorithm::Sha3384 => Ok(ChecksumAlgorithm::Sha3_384),
        proto::ChecksumAlgorithm::Sha3512 => Ok(ChecksumAlgorithm::Sha3_512),
        proto::ChecksumAlgorithm::Blake2b256 => Ok(ChecksumAlgorithm::Blake2b256),
        proto::ChecksumAlgorithm::Blake2b384 => Ok(ChecksumAlgorithm::Blake2b384),
        proto::ChecksumAlgorithm::Blake2b512 => Ok(ChecksumAlgorithm::Blake2b512),
        proto::ChecksumAlgorithm::Blake3 => Ok(ChecksumAlgorithm::Blake3),
    }
}

fn parse_component_type(value: i32) -> DomainComponentType {
    match proto::ComponentType::try_from(value).unwrap_or(proto::ComponentType::Unspecified) {
        proto::ComponentType::Application => DomainComponentType::Application,
        proto::ComponentType::Framework => DomainComponentType::Framework,
        proto::ComponentType::Library => DomainComponentType::Library,
        proto::ComponentType::OperatingSystem => DomainComponentType::OperatingSystem,
        proto::ComponentType::Device => DomainComponentType::Device,
        proto::ComponentType::Firmware => DomainComponentType::Firmware,
        proto::ComponentType::File => DomainComponentType::File,
        proto::ComponentType::Container => DomainComponentType::Container,
        proto::ComponentType::Platform
        | proto::ComponentType::MachineLearningModel
        | proto::ComponentType::Data
        | proto::ComponentType::CryptographicAsset => DomainComponentType::Other,
        proto::ComponentType::Unspecified => DomainComponentType::Unspecified,
    }
}

fn parse_artifact_type(value: i32) -> Result<DomainArtifactType, Status> {
    match proto::ArtifactType::try_from(value).unwrap_or(proto::ArtifactType::Unspecified) {
        proto::ArtifactType::Attestation => Ok(DomainArtifactType::Attestation),
        proto::ArtifactType::Bom => Ok(DomainArtifactType::Bom),
        proto::ArtifactType::BuildMeta => Ok(DomainArtifactType::BuildMeta),
        proto::ArtifactType::Certification => Ok(DomainArtifactType::Certification),
        proto::ArtifactType::Formulation => Ok(DomainArtifactType::Formulation),
        proto::ArtifactType::License => Ok(DomainArtifactType::License),
        proto::ArtifactType::ReleaseNotes => Ok(DomainArtifactType::ReleaseNotes),
        proto::ArtifactType::SecurityTxt => Ok(DomainArtifactType::SecurityTxt),
        proto::ArtifactType::ThreatModel => Ok(DomainArtifactType::ThreatModel),
        proto::ArtifactType::Vulnerabilities => Ok(DomainArtifactType::Vulnerabilities),
        proto::ArtifactType::Cle => Ok(DomainArtifactType::Cle),
        proto::ArtifactType::Cdxa => Ok(DomainArtifactType::Cdxa),
        proto::ArtifactType::Cbom => Ok(DomainArtifactType::Cbom),
        proto::ArtifactType::ModelCard => Ok(DomainArtifactType::ModelCard),
        proto::ArtifactType::StaticAnalysis => Ok(DomainArtifactType::StaticAnalysis),
        proto::ArtifactType::DynamicAnalysis => Ok(DomainArtifactType::DynamicAnalysis),
        proto::ArtifactType::PentestReport => Ok(DomainArtifactType::PentestReport),
        proto::ArtifactType::RiskAssessment => Ok(DomainArtifactType::RiskAssessment),
        proto::ArtifactType::Poam => Ok(DomainArtifactType::Poam),
        proto::ArtifactType::QualityMetrics => Ok(DomainArtifactType::QualityMetrics),
        proto::ArtifactType::Harness => Ok(DomainArtifactType::Harness),
        proto::ArtifactType::Conformance => Ok(DomainArtifactType::Conformance),
        proto::ArtifactType::Other => Ok(DomainArtifactType::Other),
        proto::ArtifactType::Unspecified => {
            Err(Status::invalid_argument("artifact type must be specified"))
        }
    }
}

fn parse_subject(subject: Option<proto::ArtifactSubject>) -> Result<Option<Subject>, Status> {
    subject
        .map(|subject| {
            Ok(Subject {
                type_: parse_subject_type(subject.r#type)?,
                identifiers: parse_identifiers(subject.identifiers)?,
                name: option_string(subject.name),
                version: option_string(subject.version),
            })
        })
        .transpose()
}

fn parse_subject_type(value: i32) -> Result<DomainSubjectType, Status> {
    match proto::SubjectType::try_from(value).unwrap_or(proto::SubjectType::Unspecified) {
        proto::SubjectType::Component => Ok(DomainSubjectType::Component),
        proto::SubjectType::Product => Ok(DomainSubjectType::Product),
        proto::SubjectType::Service => Ok(DomainSubjectType::Service),
        proto::SubjectType::Organization => Ok(DomainSubjectType::Organization),
        proto::SubjectType::Build => Ok(DomainSubjectType::Build),
        proto::SubjectType::Unspecified => Err(Status::invalid_argument(
            "artifact subject.type must be specified when subject is provided",
        )),
    }
}

fn artifact_from_metadata(
    metadata: proto::ArtifactMetadata,
    source_url: String,
    signature_url: Option<String>,
    checksums: Vec<Checksum>,
    size_bytes: i64,
) -> Result<Artifact, Status> {
    Ok(Artifact {
        uuid: parse_optional_uuid(metadata.uuid, "metadata.uuid")?.unwrap_or_default(),
        name: metadata.name,
        type_: parse_artifact_type(metadata.r#type)?,
        component_distributions: metadata.component_distributions,
        formats: vec![ArtifactFormat {
            mime_type: metadata.mime_type,
            description: None,
            url: source_url,
            signature_url,
            checksums,
            size_bytes: Some(size_bytes),
            encoding: None,
            spec_version: option_string(metadata.spec_version),
        }],
        created_date: Utc::now(),
        modified_date: Utc::now(),
        description: option_string(metadata.description),
        subject: parse_subject(metadata.subject)?,
        deprecation: None,
        dependencies: vec![],
    })
}

fn merge_expected_checksums(
    metadata_checksums: Vec<proto::Checksum>,
    request_checksums: Vec<proto::Checksum>,
) -> Result<Vec<Checksum>, Status> {
    let mut merged = Vec::new();
    for checksum in metadata_checksums.into_iter().chain(request_checksums) {
        let checksum = parse_checksum(checksum)?;
        if checksum.alg_type == ChecksumAlgorithm::Unspecified {
            return Err(Status::invalid_argument(
                "expected_checksums must use a concrete algorithm",
            ));
        }

        if let Some(existing) = merged
            .iter()
            .find(|candidate: &&Checksum| candidate.alg_type == checksum.alg_type)
        {
            if !existing.alg_value.eq_ignore_ascii_case(&checksum.alg_value) {
                return Err(Status::invalid_argument(format!(
                    "expected_checksums contain conflicting values for {:?}",
                    checksum.alg_type
                )));
            }
            continue;
        }

        merged.push(checksum);
    }

    if merged.is_empty() {
        return Err(Status::invalid_argument(
            "expected_checksums is required for remote artifact ingestion",
        ));
    }

    Ok(merged)
}

fn verify_expected_checksums(
    content: &[u8],
    expected_checksums: &[Checksum],
) -> Result<(), Status> {
    for checksum in expected_checksums {
        let actual = compute_checksum(content, &checksum.alg_type)?;
        if !actual.eq_ignore_ascii_case(&checksum.alg_value) {
            return Err(Status::failed_precondition(format!(
                "checksum verification failed for {:?}",
                checksum.alg_type
            )));
        }
    }
    Ok(())
}

fn compute_checksum(content: &[u8], algorithm: &ChecksumAlgorithm) -> Result<String, Status> {
    match algorithm {
        ChecksumAlgorithm::Sha256 => Ok(hex::encode(sha2::Sha256::digest(content))),
        ChecksumAlgorithm::Sha384 => Ok(hex::encode(sha2::Sha384::digest(content))),
        ChecksumAlgorithm::Sha512 => Ok(hex::encode(sha2::Sha512::digest(content))),
        ChecksumAlgorithm::Blake3 => Ok(blake3::hash(content).to_hex().to_string()),
        ChecksumAlgorithm::Unspecified => Err(Status::invalid_argument(
            "checksum algorithm must be specified",
        )),
        unsupported => Err(Status::invalid_argument(format!(
            "remote artifact verification does not yet support {:?}",
            unsupported
        ))),
    }
}

async fn validate_remote_source_url(source_url: &str) -> Result<(), Status> {
    let url = url::Url::parse(source_url).map_err(|error| {
        Status::invalid_argument(format!("source_url must be a valid URL: {error}"))
    })?;
    let host = url
        .host_str()
        .ok_or_else(|| Status::invalid_argument("source_url must include a host"))?;
    let scheme = url.scheme();
    if scheme != "https" && scheme != "http" {
        return Err(Status::invalid_argument(
            "source_url must use http or https",
        ));
    }

    if scheme == "http" && !allow_private_source_urls() && !is_loopback_host(host) {
        return Err(Status::invalid_argument(
            "source_url must use https outside local development",
        ));
    }

    let port = url.port_or_known_default().ok_or_else(|| {
        Status::invalid_argument("source_url must use a scheme with a known default port")
    })?;
    let mut resolved_any = false;
    for address in tokio::net::lookup_host((host, port))
        .await
        .map_err(|error| {
            Status::failed_precondition(format!("failed to resolve source_url host: {error}"))
        })?
    {
        resolved_any = true;
        if !allow_private_source_urls() && is_private_ip(address.ip()) {
            return Err(Status::permission_denied(
                "source_url resolves to a non-public address and is blocked",
            ));
        }
    }

    if !resolved_any {
        return Err(Status::failed_precondition(
            "source_url host did not resolve to any addresses",
        ));
    }

    Ok(())
}

fn allow_private_source_urls() -> bool {
    std::env::var("TEA_ALLOW_PRIVATE_SOURCE_URLS")
        .map(|value| {
            matches!(
                value.to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(cfg!(debug_assertions))
}

fn is_loopback_host(host: &str) -> bool {
    matches!(host, "localhost" | "127.0.0.1" | "::1")
}

fn is_private_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => {
            ip.is_private()
                || ip.is_loopback()
                || ip.is_link_local()
                || ip.is_broadcast()
                || ip.is_documentation()
                || ip.is_unspecified()
                || matches!(ip.octets(), [169, 254, ..])
                || matches!(ip.octets(), [100, b, ..] if (64..=127).contains(&b))
                || matches!(ip.octets(), [198, 18 | 19, ..])
                || matches!(ip.octets(), [224..=255, ..])
        }
        IpAddr::V6(ip) => {
            ip.is_loopback()
                || ip.is_unspecified()
                || ip.is_multicast()
                || is_ipv6_unique_local(ip)
                || is_ipv6_link_local(ip)
                || is_ipv6_documentation(ip)
        }
    }
}

fn is_ipv6_unique_local(ip: Ipv6Addr) -> bool {
    (ip.segments()[0] & 0xfe00) == 0xfc00
}

fn is_ipv6_link_local(ip: Ipv6Addr) -> bool {
    (ip.segments()[0] & 0xffc0) == 0xfe80
}

fn is_ipv6_documentation(ip: Ipv6Addr) -> bool {
    let segments = ip.segments();
    segments[0] == 0x2001 && segments[1] == 0x0db8
}

fn max_remote_artifact_bytes() -> usize {
    50 * 1024 * 1024
}

fn parse_collection_scope(value: i32) -> Result<DomainCollectionScope, Status> {
    match proto::CollectionScope::try_from(value).unwrap_or(proto::CollectionScope::Unspecified) {
        proto::CollectionScope::Release => Ok(DomainCollectionScope::Release),
        proto::CollectionScope::ProductRelease => Ok(DomainCollectionScope::ProductRelease),
        proto::CollectionScope::Unspecified => Err(Status::invalid_argument(
            "belongs_to must be RELEASE or PRODUCT_RELEASE",
        )),
    }
}

fn parse_update_reason(
    value: Option<proto::UpdateReason>,
    field: &str,
    required: bool,
    expected: Option<DomainUpdateReason>,
) -> Result<DomainUpdateReason, Status> {
    let Some(value) = value else {
        if required {
            return Err(Status::invalid_argument(format!("{field} is required")));
        }
        return Ok(DomainUpdateReason::Unspecified);
    };

    let reason = match proto::UpdateReasonType::try_from(value.r#type)
        .unwrap_or(proto::UpdateReasonType::Unspecified)
    {
        proto::UpdateReasonType::InitialRelease => DomainUpdateReason::InitialRelease,
        proto::UpdateReasonType::VexUpdated => DomainUpdateReason::VexUpdated,
        proto::UpdateReasonType::ArtifactUpdated => DomainUpdateReason::ArtifactUpdated,
        proto::UpdateReasonType::ArtifactRemoved => DomainUpdateReason::ArtifactRemoved,
        proto::UpdateReasonType::ArtifactAdded => DomainUpdateReason::ArtifactAdded,
        proto::UpdateReasonType::MetadataCorrection
        | proto::UpdateReasonType::SecurityUpdate
        | proto::UpdateReasonType::Unspecified => {
            return Err(Status::invalid_argument(format!(
                "{field}.type is not supported by the current reference server"
            )))
        }
    };

    if let Some(expected) = expected {
        if reason != expected {
            return Err(Status::invalid_argument(format!(
                "{field}.type must be {:?} when creating a collection",
                expected
            )));
        }
    }

    Ok(reason)
}

fn parse_artifact_uuids(values: Vec<String>) -> Result<Vec<Uuid>, Status> {
    values
        .into_iter()
        .enumerate()
        .map(|(index, value)| parse_uuid(&value, &format!("artifact_uuids[{index}]")))
        .collect()
}

fn collections_referencing_artifact(collections: &[Collection], uuid: Uuid) -> Vec<String> {
    collections
        .iter()
        .filter(|collection| collection.artifacts.contains(&uuid))
        .map(|collection| collection.uuid.to_string())
        .collect()
}

fn parse_product_release_component_ref(
    value: proto::ComponentRef,
    index: usize,
) -> Result<ProductReleaseComponentRef, Status> {
    Ok(ProductReleaseComponentRef {
        component_uuid: parse_uuid(&value.uuid, &format!("components[{index}].uuid"))?,
        release_uuid: parse_required_optional_uuid(
            value.release,
            &format!("components[{index}].release"),
            "component references must include release UUIDs in the current reference server",
        )?,
    })
}

fn parse_optional_uuid(value: Option<String>, field: &str) -> Result<Option<Uuid>, Status> {
    value
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .map(|value| parse_uuid(value, field))
        .transpose()
}

fn parse_required_optional_uuid(
    value: Option<String>,
    field: &str,
    missing_message: &str,
) -> Result<Uuid, Status> {
    parse_optional_uuid(value, field)?.ok_or_else(|| Status::invalid_argument(missing_message))
}

fn option_string(value: String) -> Option<String> {
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

fn required_update_mask(mask: Option<&FieldMask>, operation: &str) -> Result<Vec<String>, Status> {
    let paths = mask
        .map(|mask| mask.paths.clone())
        .unwrap_or_default()
        .into_iter()
        .filter(|path| !path.trim().is_empty())
        .collect::<Vec<_>>();
    if paths.is_empty() {
        return Err(Status::invalid_argument(format!(
            "{operation} requires a non-empty update_mask"
        )));
    }
    Ok(paths)
}

fn timestamp_from_proto(
    value: Option<Timestamp>,
    field: &str,
) -> Result<Option<DateTime<Utc>>, Status> {
    value
        .map(|value| {
            DateTime::<Utc>::from_timestamp(value.seconds, value.nanos as u32).ok_or_else(|| {
                Status::invalid_argument(format!("{field} must be a valid timestamp"))
            })
        })
        .transpose()
}

#[cfg(test)]
mod tests {
    use super::{
        merge_component_release_update, merge_product_release_update, merge_product_update,
        product_from_create, product_release_from_create,
    };
    use crate::domain::component::entity::ComponentRelease;
    use crate::domain::product::entity::{Product, ProductRelease, Vendor};
    use crate::gen::tea::v1 as proto;
    use prost_types::{FieldMask, Timestamp};
    use uuid::Uuid;

    #[test]
    fn create_product_preserves_client_uuid_when_present() {
        let uuid = Uuid::new_v4();
        let product = product_from_create(proto::CreateProductRequest {
            uuid: Some(uuid.to_string()),
            name: "Widget".to_string(),
            description: String::new(),
            identifiers: vec![],
            vendor: Some(proto::Vendor {
                name: "ACME".to_string(),
                uuid: None,
                url: String::new(),
                contacts: vec![],
            }),
            homepage_url: String::new(),
            documentation_url: String::new(),
            vcs_url: String::new(),
        })
        .unwrap();

        assert_eq!(product.uuid, uuid);
    }

    #[test]
    fn merge_product_update_applies_masked_fields_only() {
        let mut product = Product {
            uuid: Uuid::new_v4(),
            name: "before".to_string(),
            description: Some("keep".to_string()),
            identifiers: vec![],
            vendor: Vendor {
                name: "vendor".to_string(),
                uuid: None,
                url: None,
                contacts: vec![],
            },
            created_date: chrono::Utc::now(),
            modified_date: chrono::Utc::now(),
            homepage_url: Some("https://example.com".to_string()),
            documentation_url: None,
            vcs_url: None,
            deprecation: None,
            dependencies: vec![],
        };
        let product_uuid = product.uuid.to_string();

        merge_product_update(
            &mut product,
            &proto::UpdateProductRequest {
                uuid: product_uuid,
                update_mask: Some(FieldMask {
                    paths: vec!["name".to_string()],
                }),
                name: "after".to_string(),
                description: String::new(),
                identifiers: vec![],
                vendor: None,
                homepage_url: String::new(),
                documentation_url: String::new(),
                vcs_url: String::new(),
            },
        )
        .unwrap();

        assert_eq!(product.name, "after");
        assert_eq!(product.description.as_deref(), Some("keep"));
    }

    #[test]
    fn merge_component_release_update_blocks_pre_release_promotion() {
        let mut release = ComponentRelease {
            uuid: Uuid::new_v4(),
            component_uuid: Uuid::new_v4(),
            version: "1.0.0".to_string(),
            release_date: None,
            pre_release: false,
            identifiers: vec![],
            distributions: vec![],
        };
        let release_uuid = release.uuid.to_string();

        let error = merge_component_release_update(
            &mut release,
            &proto::UpdateComponentReleaseRequest {
                uuid: release_uuid,
                update_mask: Some(FieldMask {
                    paths: vec!["pre_release".to_string()],
                }),
                version: String::new(),
                release_date: Some(Timestamp {
                    seconds: 0,
                    nanos: 0,
                }),
                pre_release: true,
                identifiers: vec![],
                distributions: vec![],
            },
        )
        .unwrap_err();

        assert_eq!(error.code(), tonic::Code::InvalidArgument);
    }

    #[test]
    fn create_product_release_requires_product_uuid() {
        let error = product_release_from_create(proto::CreateProductReleaseRequest {
            product_uuid: None,
            version: "2026.03".to_string(),
            release_date: None,
            pre_release: false,
            identifiers: vec![],
            components: vec![],
            uuid: None,
        })
        .unwrap_err();

        assert_eq!(error.code(), tonic::Code::InvalidArgument);
    }

    #[test]
    fn merge_product_release_update_blocks_pre_release_promotion() {
        let mut release = ProductRelease {
            uuid: Uuid::new_v4(),
            product_uuid: Uuid::new_v4(),
            version: "2026.03".to_string(),
            created_date: chrono::Utc::now(),
            modified_date: chrono::Utc::now(),
            release_date: None,
            pre_release: false,
            identifiers: vec![],
            components: vec![],
        };
        let release_uuid = release.uuid.to_string();

        let error = merge_product_release_update(
            &mut release,
            &proto::UpdateProductReleaseRequest {
                uuid: release_uuid,
                update_mask: Some(FieldMask {
                    paths: vec!["pre_release".to_string()],
                }),
                version: String::new(),
                release_date: None,
                pre_release: true,
                identifiers: vec![],
                components: vec![],
            },
        )
        .unwrap_err();

        assert_eq!(error.code(), tonic::Code::InvalidArgument);
    }
}
