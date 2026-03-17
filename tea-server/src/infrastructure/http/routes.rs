//! HTTP routes for TEA REST API.
//!
//! Provides REST endpoints that mirror the gRPC Publisher service.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::application::publisher::service::PublisherApplicationService;

/// Create the HTTP router for the TEA API.
pub fn create_router<A: PublisherApplicationService + Clone + Send + Sync + 'static>(
    app_service: A,
) -> Router {
    Router::new()
        .route("/api/v1/products", axum::routing::post(create_product::<A>))
        .route(
            "/api/v1/products/:uuid",
            axum::routing::get(get_product::<A>),
        )
        .route(
            "/api/v1/products/:uuid",
            axum::routing::put(update_product::<A>),
        )
        .route(
            "/api/v1/products/:uuid",
            axum::routing::delete(delete_product::<A>),
        )
        .route(
            "/api/v1/products/:uuid/deprecate",
            axum::routing::post(deprecate_product::<A>),
        )
        .route(
            "/api/v1/artifacts",
            axum::routing::post(create_artifact::<A>),
        )
        .route(
            "/api/v1/artifacts/:uuid",
            axum::routing::get(get_artifact::<A>),
        )
        .route(
            "/api/v1/artifacts/:uuid",
            axum::routing::put(update_artifact::<A>),
        )
        .route(
            "/api/v1/artifacts/:uuid",
            axum::routing::delete(delete_artifact::<A>),
        )
        .route(
            "/api/v1/artifacts/:uuid/deprecate",
            axum::routing::post(deprecate_artifact::<A>),
        )
        .route(
            "/api/v1/collections",
            axum::routing::post(create_collection::<A>),
        )
        .route(
            "/api/v1/collections/:uuid",
            axum::routing::get(get_collection::<A>),
        )
        .route(
            "/api/v1/collections/:uuid",
            axum::routing::put(update_collection::<A>),
        )
        .route(
            "/api/v1/collections/:uuid",
            axum::routing::delete(delete_collection::<A>),
        )
        .route(
            "/api/v1/collections/:uuid/deprecate",
            axum::routing::post(deprecate_collection::<A>),
        )
        .with_state(app_service)
}

// ─── Request/Response DTOs ─────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateProductRequest {
    pub name: String,
    pub description: Option<String>,
    pub vendor: VendorDto,
    pub homepage_url: Option<String>,
    pub documentation_url: Option<String>,
    pub vcs_url: Option<String>,
    pub identifiers: Option<Vec<IdentifierDto>>,
}

#[derive(Debug, Serialize)]
pub struct ProductResponse {
    pub uuid: String,
    pub name: String,
    pub description: Option<String>,
    pub vendor: VendorDto,
    pub homepage_url: Option<String>,
    pub documentation_url: Option<String>,
    pub vcs_url: Option<String>,
    pub identifiers: Vec<IdentifierDto>,
    pub deprecation: Option<DeprecationDto>,
    pub created_date: String,
    pub modified_date: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProductRequest {
    pub name: String,
    pub description: Option<String>,
    pub vendor: VendorDto,
    pub homepage_url: Option<String>,
    pub documentation_url: Option<String>,
    pub vcs_url: Option<String>,
    pub identifiers: Option<Vec<IdentifierDto>>,
}

#[derive(Debug, Deserialize)]
pub struct DeprecateRequest {
    pub state: String,
    pub reason: Option<String>,
    pub effective_date: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DeprecationDto {
    pub state: String,
    pub reason: Option<String>,
    pub effective_date: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct VendorDto {
    pub name: String,
    pub url: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct IdentifierDto {
    pub id_type: i32,
    pub id_value: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateArtifactRequest {
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub type_: Option<String>,
    pub formats: Vec<ArtifactFormatDto>,
}

#[derive(Debug, Serialize)]
pub struct ArtifactResponse {
    pub uuid: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub type_: String,
    pub formats: Vec<ArtifactFormatDto>,
    pub deprecation: Option<DeprecationDto>,
    pub created_date: String,
    pub modified_date: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ArtifactFormatDto {
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    pub description: Option<String>,
    pub url: String,
    #[serde(rename = "signatureUrl")]
    pub signature_url: Option<String>,
    pub checksums: Vec<ChecksumDto>,
    #[serde(rename = "sizeBytes")]
    pub size_bytes: Option<i64>,
    pub encoding: Option<String>,
    #[serde(rename = "specVersion")]
    pub spec_version: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ChecksumDto {
    #[serde(rename = "algType")]
    pub alg_type: String,
    #[serde(rename = "algValue")]
    pub alg_value: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateCollectionRequest {
    pub name: String,
    #[allow(dead_code)]
    pub description: Option<String>,
    pub version: i32,
    pub belongs_to: Option<String>,
    pub update_reason: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CollectionResponse {
    pub uuid: String,
    pub name: String,
    pub version: i32,
    pub belongs_to: String,
    pub update_reason: String,
    pub deprecation: Option<DeprecationDto>,
    pub created_date: String,
    pub modified_date: String,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

// ─── Product Handlers ──────────────────────────────────────────────────────────

async fn create_product<A: PublisherApplicationService>(
    State(app_service): State<A>,
    Json(req): Json<CreateProductRequest>,
) -> Result<Json<ProductResponse>, (StatusCode, Json<ErrorResponse>)> {
    let product = crate::domain::product::entity::Product {
        uuid: Uuid::new_v4(),
        name: req.name,
        description: req.description,
        created_date: chrono::Utc::now(),
        modified_date: chrono::Utc::now(),
        deprecation: None,
        identifiers: req
            .identifiers
            .unwrap_or_default()
            .into_iter()
            .map(|id| crate::domain::common::identifier::Identifier {
                id_type: proto_to_identifier_type(id.id_type),
                id_value: id.id_value,
            })
            .collect(),
        vendor: crate::domain::product::entity::Vendor {
            name: req.vendor.name,
            uuid: None,
            url: req.vendor.url,
            contacts: vec![],
        },
        homepage_url: req.homepage_url,
        documentation_url: req.documentation_url,
        vcs_url: req.vcs_url,
        dependencies: vec![],
    };

    let created = app_service
        .create_product(product)
        .await
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    Ok(Json(product_to_response(created)))
}

async fn get_product<A: PublisherApplicationService>(
    State(app_service): State<A>,
    Path(uuid_str): Path<String>,
) -> Result<Json<ProductResponse>, (StatusCode, Json<ErrorResponse>)> {
    let uuid = uuid_str
        .parse::<Uuid>()
        .map_err(|e| error_response(StatusCode::BAD_REQUEST, &format!("Invalid UUID: {}", e)))?;

    let product = app_service
        .get_product(&uuid)
        .await
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?
        .ok_or_else(|| error_response(StatusCode::NOT_FOUND, "Product not found"))?;

    Ok(Json(product_to_response(product)))
}

async fn update_product<A: PublisherApplicationService>(
    State(app_service): State<A>,
    Path(uuid_str): Path<String>,
    Json(req): Json<UpdateProductRequest>,
) -> Result<Json<ProductResponse>, (StatusCode, Json<ErrorResponse>)> {
    let uuid = uuid_str
        .parse::<Uuid>()
        .map_err(|e| error_response(StatusCode::BAD_REQUEST, &format!("Invalid UUID: {}", e)))?;

    let product = crate::domain::product::entity::Product {
        uuid,
        name: req.name,
        description: req.description,
        created_date: chrono::Utc::now(),
        modified_date: chrono::Utc::now(),
        deprecation: None,
        identifiers: req
            .identifiers
            .unwrap_or_default()
            .into_iter()
            .map(|id| crate::domain::common::identifier::Identifier {
                id_type: proto_to_identifier_type(id.id_type),
                id_value: id.id_value,
            })
            .collect(),
        vendor: crate::domain::product::entity::Vendor {
            name: req.vendor.name,
            uuid: None,
            url: req.vendor.url,
            contacts: vec![],
        },
        homepage_url: req.homepage_url,
        documentation_url: req.documentation_url,
        vcs_url: req.vcs_url,
        dependencies: vec![],
    };

    let updated = app_service
        .update_product(product)
        .await
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    Ok(Json(product_to_response(updated)))
}

async fn delete_product<A: PublisherApplicationService>(
    State(app_service): State<A>,
    Path(uuid_str): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let uuid = uuid_str
        .parse::<Uuid>()
        .map_err(|e| error_response(StatusCode::BAD_REQUEST, &format!("Invalid UUID: {}", e)))?;

    app_service
        .delete_product(&uuid)
        .await
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

async fn deprecate_product<A: PublisherApplicationService>(
    State(app_service): State<A>,
    Path(uuid_str): Path<String>,
    Json(req): Json<DeprecateRequest>,
) -> Result<Json<ProductResponse>, (StatusCode, Json<ErrorResponse>)> {
    let uuid = uuid_str
        .parse::<Uuid>()
        .map_err(|e| error_response(StatusCode::BAD_REQUEST, &format!("Invalid UUID: {}", e)))?;

    let deprecation = crate::domain::common::deprecation::Deprecation {
        state: parse_deprecation_state(&req.state),
        reason: req.reason,
        announced_date: None,
        effective_date: req.effective_date.and_then(|s| s.parse().ok()),
        replacement_identifiers: vec![],
    };

    let product = app_service
        .deprecate_product(uuid, deprecation)
        .await
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    Ok(Json(product_to_response(product)))
}

// ─── Artifact Handlers ──────────────────────────────────────────────────────────

async fn create_artifact<A: PublisherApplicationService>(
    State(app_service): State<A>,
    Json(req): Json<CreateArtifactRequest>,
) -> Result<Json<ArtifactResponse>, (StatusCode, Json<ErrorResponse>)> {
    if req.formats.is_empty() {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            "Artifact must have at least one format",
        ));
    }

    let artifact = crate::domain::artifact::entity::Artifact {
        uuid: Uuid::new_v4(),
        name: req.name,
        description: req.description,
        type_: parse_artifact_type(req.type_.as_deref()),
        created_date: chrono::Utc::now(),
        modified_date: chrono::Utc::now(),
        deprecation: None,
        component_distributions: vec![],
        subject: None,
        formats: req
            .formats
            .into_iter()
            .map(|f| crate::domain::artifact::entity::ArtifactFormat {
                mime_type: f.mime_type,
                description: f.description,
                url: f.url,
                signature_url: f.signature_url,
                checksums: f
                    .checksums
                    .into_iter()
                    .map(|c| crate::domain::common::checksum::Checksum {
                        alg_type: parse_checksum_algorithm(&c.alg_type),
                        alg_value: c.alg_value,
                    })
                    .collect(),
                size_bytes: f.size_bytes,
                encoding: f.encoding,
                spec_version: f.spec_version,
            })
            .collect(),
        dependencies: vec![],
    };

    let created = app_service
        .create_artifact(artifact)
        .await
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    Ok(Json(artifact_to_response(created)))
}

async fn get_artifact<A: PublisherApplicationService>(
    State(app_service): State<A>,
    Path(uuid_str): Path<String>,
) -> Result<Json<ArtifactResponse>, (StatusCode, Json<ErrorResponse>)> {
    let uuid = uuid_str
        .parse::<Uuid>()
        .map_err(|e| error_response(StatusCode::BAD_REQUEST, &format!("Invalid UUID: {}", e)))?;

    let artifact = app_service
        .get_artifact(&uuid)
        .await
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?
        .ok_or_else(|| error_response(StatusCode::NOT_FOUND, "Artifact not found"))?;

    Ok(Json(artifact_to_response(artifact)))
}

async fn update_artifact<A: PublisherApplicationService>(
    State(_app_service): State<A>,
    Path(_uuid_str): Path<String>,
    Json(_req): Json<CreateArtifactRequest>,
) -> Result<Json<ArtifactResponse>, (StatusCode, Json<ErrorResponse>)> {
    Err(error_response(
        StatusCode::NOT_IMPLEMENTED,
        "Artifact update not yet implemented",
    ))
}

async fn delete_artifact<A: PublisherApplicationService>(
    State(app_service): State<A>,
    Path(uuid_str): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let uuid = uuid_str
        .parse::<Uuid>()
        .map_err(|e| error_response(StatusCode::BAD_REQUEST, &format!("Invalid UUID: {}", e)))?;

    app_service
        .delete_artifact(&uuid)
        .await
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

async fn deprecate_artifact<A: PublisherApplicationService>(
    State(app_service): State<A>,
    Path(uuid_str): Path<String>,
    Json(req): Json<DeprecateRequest>,
) -> Result<Json<ArtifactResponse>, (StatusCode, Json<ErrorResponse>)> {
    let uuid = uuid_str
        .parse::<Uuid>()
        .map_err(|e| error_response(StatusCode::BAD_REQUEST, &format!("Invalid UUID: {}", e)))?;

    let deprecation = crate::domain::common::deprecation::Deprecation {
        state: parse_deprecation_state(&req.state),
        reason: req.reason,
        announced_date: None,
        effective_date: req.effective_date.and_then(|s| s.parse().ok()),
        replacement_identifiers: vec![],
    };

    let artifact = app_service
        .deprecate_artifact(uuid, deprecation)
        .await
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    Ok(Json(artifact_to_response(artifact)))
}

// ─── Collection Handlers ────────────────────────────────────────────────────────

async fn create_collection<A: PublisherApplicationService>(
    State(app_service): State<A>,
    Json(req): Json<CreateCollectionRequest>,
) -> Result<Json<CollectionResponse>, (StatusCode, Json<ErrorResponse>)> {
    let collection = crate::domain::collection::entity::Collection {
        uuid: Uuid::new_v4(),
        name: req.name,
        created_date: chrono::Utc::now(),
        modified_date: chrono::Utc::now(),
        date: chrono::Utc::now(),
        deprecation: None,
        version: req.version,
        belongs_to: parse_collection_scope(req.belongs_to.as_deref()),
        update_reason: parse_update_reason(req.update_reason.as_deref()),
        artifacts: vec![],
        dependencies: vec![],
    };

    let created = app_service
        .create_collection(collection)
        .await
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    Ok(Json(collection_to_response(created)))
}

async fn get_collection<A: PublisherApplicationService>(
    State(app_service): State<A>,
    Path(uuid_str): Path<String>,
) -> Result<Json<CollectionResponse>, (StatusCode, Json<ErrorResponse>)> {
    let uuid = uuid_str
        .parse::<Uuid>()
        .map_err(|e| error_response(StatusCode::BAD_REQUEST, &format!("Invalid UUID: {}", e)))?;

    let collection = app_service
        .get_collection(&uuid)
        .await
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?
        .ok_or_else(|| error_response(StatusCode::NOT_FOUND, "Collection not found"))?;

    Ok(Json(collection_to_response(collection)))
}

async fn update_collection<A: PublisherApplicationService>(
    State(_app_service): State<A>,
    Path(_uuid_str): Path<String>,
    Json(_req): Json<CreateCollectionRequest>,
) -> Result<Json<CollectionResponse>, (StatusCode, Json<ErrorResponse>)> {
    Err(error_response(
        StatusCode::NOT_IMPLEMENTED,
        "Collection update not yet implemented",
    ))
}

async fn delete_collection<A: PublisherApplicationService>(
    State(app_service): State<A>,
    Path(uuid_str): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let uuid = uuid_str
        .parse::<Uuid>()
        .map_err(|e| error_response(StatusCode::BAD_REQUEST, &format!("Invalid UUID: {}", e)))?;

    app_service
        .delete_collection(&uuid)
        .await
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

async fn deprecate_collection<A: PublisherApplicationService>(
    State(app_service): State<A>,
    Path(uuid_str): Path<String>,
    Json(req): Json<DeprecateRequest>,
) -> Result<Json<CollectionResponse>, (StatusCode, Json<ErrorResponse>)> {
    let uuid = uuid_str
        .parse::<Uuid>()
        .map_err(|e| error_response(StatusCode::BAD_REQUEST, &format!("Invalid UUID: {}", e)))?;

    let deprecation = crate::domain::common::deprecation::Deprecation {
        state: parse_deprecation_state(&req.state),
        reason: req.reason,
        announced_date: None,
        effective_date: req.effective_date.and_then(|s| s.parse().ok()),
        replacement_identifiers: vec![],
    };

    let collection = app_service
        .deprecate_collection(uuid, deprecation)
        .await
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    Ok(Json(collection_to_response(collection)))
}

// ─── Helper Functions ────────────────────────────────────────────────────────────

fn error_response(status: StatusCode, message: &str) -> (StatusCode, Json<ErrorResponse>) {
    (
        status,
        Json(ErrorResponse {
            error: format!("{:?}", status),
            message: message.to_string(),
        }),
    )
}

fn product_to_response(product: crate::domain::product::entity::Product) -> ProductResponse {
    ProductResponse {
        uuid: product.uuid.to_string(),
        name: product.name,
        description: product.description,
        vendor: VendorDto {
            name: product.vendor.name,
            url: product.vendor.url,
        },
        homepage_url: product.homepage_url,
        documentation_url: product.documentation_url,
        vcs_url: product.vcs_url,
        identifiers: product
            .identifiers
            .into_iter()
            .map(|id| IdentifierDto {
                id_type: identifier_type_to_proto(&id.id_type),
                id_value: id.id_value,
            })
            .collect(),
        deprecation: product.deprecation.map(|d| DeprecationDto {
            state: deprecation_state_to_string(&d.state),
            reason: d.reason,
            effective_date: d.effective_date.map(|dt| dt.to_rfc3339()),
        }),
        created_date: product.created_date.to_rfc3339(),
        modified_date: product.modified_date.to_rfc3339(),
    }
}

fn artifact_to_response(artifact: crate::domain::artifact::entity::Artifact) -> ArtifactResponse {
    ArtifactResponse {
        uuid: artifact.uuid.to_string(),
        name: artifact.name,
        description: artifact.description,
        type_: artifact_type_to_string(&artifact.type_),
        formats: artifact
            .formats
            .into_iter()
            .map(|f| ArtifactFormatDto {
                mime_type: f.mime_type,
                description: f.description,
                url: f.url,
                signature_url: f.signature_url,
                checksums: f
                    .checksums
                    .into_iter()
                    .map(|c| ChecksumDto {
                        alg_type: checksum_algorithm_to_string(&c.alg_type),
                        alg_value: c.alg_value,
                    })
                    .collect(),
                size_bytes: f.size_bytes,
                encoding: f.encoding,
                spec_version: f.spec_version,
            })
            .collect(),
        deprecation: artifact.deprecation.map(|d| DeprecationDto {
            state: deprecation_state_to_string(&d.state),
            reason: d.reason,
            effective_date: d.effective_date.map(|dt| dt.to_rfc3339()),
        }),
        created_date: artifact.created_date.to_rfc3339(),
        modified_date: artifact.modified_date.to_rfc3339(),
    }
}

fn collection_to_response(
    collection: crate::domain::collection::entity::Collection,
) -> CollectionResponse {
    CollectionResponse {
        uuid: collection.uuid.to_string(),
        name: collection.name,
        version: collection.version,
        belongs_to: collection_scope_to_string(&collection.belongs_to),
        update_reason: update_reason_to_string(&collection.update_reason),
        deprecation: collection.deprecation.map(|d| DeprecationDto {
            state: deprecation_state_to_string(&d.state),
            reason: d.reason,
            effective_date: d.effective_date.map(|dt| dt.to_rfc3339()),
        }),
        created_date: collection.created_date.to_rfc3339(),
        modified_date: collection.modified_date.to_rfc3339(),
    }
}

fn proto_to_identifier_type(id_type: i32) -> crate::domain::common::identifier::IdentifierType {
    match id_type {
        1 => crate::domain::common::identifier::IdentifierType::Tei,
        2 => crate::domain::common::identifier::IdentifierType::Purl,
        3 => crate::domain::common::identifier::IdentifierType::Cpe,
        4 => crate::domain::common::identifier::IdentifierType::Swid,
        5 => crate::domain::common::identifier::IdentifierType::Gav,
        6 => crate::domain::common::identifier::IdentifierType::Gtin,
        7 => crate::domain::common::identifier::IdentifierType::Gmn,
        8 => crate::domain::common::identifier::IdentifierType::Udi,
        9 => crate::domain::common::identifier::IdentifierType::Asin,
        10 => crate::domain::common::identifier::IdentifierType::Hash,
        11 => crate::domain::common::identifier::IdentifierType::Conformance,
        _ => crate::domain::common::identifier::IdentifierType::Unspecified,
    }
}

fn identifier_type_to_proto(id_type: &crate::domain::common::identifier::IdentifierType) -> i32 {
    match id_type {
        crate::domain::common::identifier::IdentifierType::Unspecified => 0,
        crate::domain::common::identifier::IdentifierType::Tei => 1,
        crate::domain::common::identifier::IdentifierType::Purl => 2,
        crate::domain::common::identifier::IdentifierType::Cpe => 3,
        crate::domain::common::identifier::IdentifierType::Swid => 4,
        crate::domain::common::identifier::IdentifierType::Gav => 5,
        crate::domain::common::identifier::IdentifierType::Gtin => 6,
        crate::domain::common::identifier::IdentifierType::Gmn => 7,
        crate::domain::common::identifier::IdentifierType::Udi => 8,
        crate::domain::common::identifier::IdentifierType::Asin => 9,
        crate::domain::common::identifier::IdentifierType::Hash => 10,
        crate::domain::common::identifier::IdentifierType::Conformance => 11,
    }
}

fn parse_deprecation_state(s: &str) -> crate::domain::common::deprecation::DeprecationState {
    match s.to_uppercase().as_str() {
        "ACTIVE" => crate::domain::common::deprecation::DeprecationState::Active,
        "DEPRECATED" => crate::domain::common::deprecation::DeprecationState::Deprecated,
        "RETIRED" => crate::domain::common::deprecation::DeprecationState::Retired,
        _ => crate::domain::common::deprecation::DeprecationState::Unspecified,
    }
}

fn deprecation_state_to_string(
    state: &crate::domain::common::deprecation::DeprecationState,
) -> String {
    match state {
        crate::domain::common::deprecation::DeprecationState::Unspecified => {
            "UNSPECIFIED".to_string()
        }
        crate::domain::common::deprecation::DeprecationState::Active => "ACTIVE".to_string(),
        crate::domain::common::deprecation::DeprecationState::Deprecated => {
            "DEPRECATED".to_string()
        }
        crate::domain::common::deprecation::DeprecationState::Retired => "RETIRED".to_string(),
    }
}

fn parse_artifact_type(s: Option<&str>) -> crate::domain::artifact::entity::ArtifactType {
    match s.map(|s| s.to_uppercase()).as_deref() {
        Some("ATTESTATION") => crate::domain::artifact::entity::ArtifactType::Attestation,
        Some("BOM") => crate::domain::artifact::entity::ArtifactType::Bom,
        Some("BUILD_META") => crate::domain::artifact::entity::ArtifactType::BuildMeta,
        Some("CERTIFICATION") => crate::domain::artifact::entity::ArtifactType::Certification,
        Some("FORMULATION") => crate::domain::artifact::entity::ArtifactType::Formulation,
        Some("LICENSE") => crate::domain::artifact::entity::ArtifactType::License,
        Some("RELEASE_NOTES") => crate::domain::artifact::entity::ArtifactType::ReleaseNotes,
        Some("SECURITY_TXT") => crate::domain::artifact::entity::ArtifactType::SecurityTxt,
        Some("THREAT_MODEL") => crate::domain::artifact::entity::ArtifactType::ThreatModel,
        Some("VULNERABILITIES") => crate::domain::artifact::entity::ArtifactType::Vulnerabilities,
        Some("CLE") => crate::domain::artifact::entity::ArtifactType::Cle,
        Some("CDXA") => crate::domain::artifact::entity::ArtifactType::Cdxa,
        Some("CBOM") => crate::domain::artifact::entity::ArtifactType::Cbom,
        Some("MODEL_CARD") => crate::domain::artifact::entity::ArtifactType::ModelCard,
        Some("STATIC_ANALYSIS") => crate::domain::artifact::entity::ArtifactType::StaticAnalysis,
        Some("DYNAMIC_ANALYSIS") => crate::domain::artifact::entity::ArtifactType::DynamicAnalysis,
        Some("PENTEST_REPORT") => crate::domain::artifact::entity::ArtifactType::PentestReport,
        Some("RISK_ASSESSMENT") => crate::domain::artifact::entity::ArtifactType::RiskAssessment,
        Some("POAM") => crate::domain::artifact::entity::ArtifactType::Poam,
        Some("QUALITY_METRICS") => crate::domain::artifact::entity::ArtifactType::QualityMetrics,
        Some("HARNESS") => crate::domain::artifact::entity::ArtifactType::Harness,
        Some("CONFORMANCE") => crate::domain::artifact::entity::ArtifactType::Conformance,
        Some("OTHER") => crate::domain::artifact::entity::ArtifactType::Other,
        _ => crate::domain::artifact::entity::ArtifactType::Unspecified,
    }
}

fn artifact_type_to_string(type_: &crate::domain::artifact::entity::ArtifactType) -> String {
    match type_ {
        crate::domain::artifact::entity::ArtifactType::Unspecified => "UNSPECIFIED".to_string(),
        crate::domain::artifact::entity::ArtifactType::Attestation => "ATTESTATION".to_string(),
        crate::domain::artifact::entity::ArtifactType::Bom => "BOM".to_string(),
        crate::domain::artifact::entity::ArtifactType::BuildMeta => "BUILD_META".to_string(),
        crate::domain::artifact::entity::ArtifactType::Certification => "CERTIFICATION".to_string(),
        crate::domain::artifact::entity::ArtifactType::Formulation => "FORMULATION".to_string(),
        crate::domain::artifact::entity::ArtifactType::License => "LICENSE".to_string(),
        crate::domain::artifact::entity::ArtifactType::ReleaseNotes => "RELEASE_NOTES".to_string(),
        crate::domain::artifact::entity::ArtifactType::SecurityTxt => "SECURITY_TXT".to_string(),
        crate::domain::artifact::entity::ArtifactType::ThreatModel => "THREAT_MODEL".to_string(),
        crate::domain::artifact::entity::ArtifactType::Vulnerabilities => {
            "VULNERABILITIES".to_string()
        }
        crate::domain::artifact::entity::ArtifactType::Cle => "CLE".to_string(),
        crate::domain::artifact::entity::ArtifactType::Cdxa => "CDXA".to_string(),
        crate::domain::artifact::entity::ArtifactType::Cbom => "CBOM".to_string(),
        crate::domain::artifact::entity::ArtifactType::ModelCard => "MODEL_CARD".to_string(),
        crate::domain::artifact::entity::ArtifactType::StaticAnalysis => {
            "STATIC_ANALYSIS".to_string()
        }
        crate::domain::artifact::entity::ArtifactType::DynamicAnalysis => {
            "DYNAMIC_ANALYSIS".to_string()
        }
        crate::domain::artifact::entity::ArtifactType::PentestReport => {
            "PENTEST_REPORT".to_string()
        }
        crate::domain::artifact::entity::ArtifactType::RiskAssessment => {
            "RISK_ASSESSMENT".to_string()
        }
        crate::domain::artifact::entity::ArtifactType::Poam => "POAM".to_string(),
        crate::domain::artifact::entity::ArtifactType::QualityMetrics => {
            "QUALITY_METRICS".to_string()
        }
        crate::domain::artifact::entity::ArtifactType::Harness => "HARNESS".to_string(),
        crate::domain::artifact::entity::ArtifactType::Conformance => "CONFORMANCE".to_string(),
        crate::domain::artifact::entity::ArtifactType::Other => "OTHER".to_string(),
    }
}

fn parse_checksum_algorithm(s: &str) -> crate::domain::common::checksum::ChecksumAlgorithm {
    match s.to_uppercase().as_str() {
        "MD5" => crate::domain::common::checksum::ChecksumAlgorithm::Md5,
        "SHA1" => crate::domain::common::checksum::ChecksumAlgorithm::Sha1,
        "SHA256" => crate::domain::common::checksum::ChecksumAlgorithm::Sha256,
        "SHA384" => crate::domain::common::checksum::ChecksumAlgorithm::Sha384,
        "SHA512" => crate::domain::common::checksum::ChecksumAlgorithm::Sha512,
        "SHA3_256" => crate::domain::common::checksum::ChecksumAlgorithm::Sha3_256,
        "SHA3_384" => crate::domain::common::checksum::ChecksumAlgorithm::Sha3_384,
        "SHA3_512" => crate::domain::common::checksum::ChecksumAlgorithm::Sha3_512,
        "BLAKE2B256" => crate::domain::common::checksum::ChecksumAlgorithm::Blake2b256,
        "BLAKE2B384" => crate::domain::common::checksum::ChecksumAlgorithm::Blake2b384,
        "BLAKE2B512" => crate::domain::common::checksum::ChecksumAlgorithm::Blake2b512,
        "BLAKE3" => crate::domain::common::checksum::ChecksumAlgorithm::Blake3,
        _ => crate::domain::common::checksum::ChecksumAlgorithm::Unspecified,
    }
}

fn checksum_algorithm_to_string(
    alg: &crate::domain::common::checksum::ChecksumAlgorithm,
) -> String {
    match alg {
        crate::domain::common::checksum::ChecksumAlgorithm::Unspecified => {
            "UNSPECIFIED".to_string()
        }
        crate::domain::common::checksum::ChecksumAlgorithm::Md5 => "MD5".to_string(),
        crate::domain::common::checksum::ChecksumAlgorithm::Sha1 => "SHA1".to_string(),
        crate::domain::common::checksum::ChecksumAlgorithm::Sha256 => "SHA256".to_string(),
        crate::domain::common::checksum::ChecksumAlgorithm::Sha384 => "SHA384".to_string(),
        crate::domain::common::checksum::ChecksumAlgorithm::Sha512 => "SHA512".to_string(),
        crate::domain::common::checksum::ChecksumAlgorithm::Sha3_256 => "SHA3_256".to_string(),
        crate::domain::common::checksum::ChecksumAlgorithm::Sha3_384 => "SHA3_384".to_string(),
        crate::domain::common::checksum::ChecksumAlgorithm::Sha3_512 => "SHA3_512".to_string(),
        crate::domain::common::checksum::ChecksumAlgorithm::Blake2b256 => "BLAKE2B256".to_string(),
        crate::domain::common::checksum::ChecksumAlgorithm::Blake2b384 => "BLAKE2B384".to_string(),
        crate::domain::common::checksum::ChecksumAlgorithm::Blake2b512 => "BLAKE2B512".to_string(),
        crate::domain::common::checksum::ChecksumAlgorithm::Blake3 => "BLAKE3".to_string(),
    }
}

fn parse_collection_scope(s: Option<&str>) -> crate::domain::collection::entity::CollectionScope {
    match s.map(|s| s.to_uppercase()).as_deref() {
        Some("RELEASE") => crate::domain::collection::entity::CollectionScope::Release,
        Some("PRODUCT_RELEASE") => {
            crate::domain::collection::entity::CollectionScope::ProductRelease
        }
        _ => crate::domain::collection::entity::CollectionScope::Unspecified,
    }
}

fn collection_scope_to_string(
    scope: &crate::domain::collection::entity::CollectionScope,
) -> String {
    match scope {
        crate::domain::collection::entity::CollectionScope::Unspecified => {
            "UNSPECIFIED".to_string()
        }
        crate::domain::collection::entity::CollectionScope::Release => "RELEASE".to_string(),
        crate::domain::collection::entity::CollectionScope::ProductRelease => {
            "PRODUCT_RELEASE".to_string()
        }
    }
}

fn parse_update_reason(s: Option<&str>) -> crate::domain::collection::entity::UpdateReason {
    match s.map(|s| s.to_uppercase()).as_deref() {
        Some("INITIAL_RELEASE") => crate::domain::collection::entity::UpdateReason::InitialRelease,
        Some("VEX_UPDATED") => crate::domain::collection::entity::UpdateReason::VexUpdated,
        Some("ARTIFACT_UPDATED") => {
            crate::domain::collection::entity::UpdateReason::ArtifactUpdated
        }
        Some("ARTIFACT_REMOVED") => {
            crate::domain::collection::entity::UpdateReason::ArtifactRemoved
        }
        Some("ARTIFACT_ADDED") => crate::domain::collection::entity::UpdateReason::ArtifactAdded,
        _ => crate::domain::collection::entity::UpdateReason::Unspecified,
    }
}

fn update_reason_to_string(reason: &crate::domain::collection::entity::UpdateReason) -> String {
    match reason {
        crate::domain::collection::entity::UpdateReason::Unspecified => "UNSPECIFIED".to_string(),
        crate::domain::collection::entity::UpdateReason::InitialRelease => {
            "INITIAL_RELEASE".to_string()
        }
        crate::domain::collection::entity::UpdateReason::VexUpdated => "VEX_UPDATED".to_string(),
        crate::domain::collection::entity::UpdateReason::ArtifactUpdated => {
            "ARTIFACT_UPDATED".to_string()
        }
        crate::domain::collection::entity::UpdateReason::ArtifactRemoved => {
            "ARTIFACT_REMOVED".to_string()
        }
        crate::domain::collection::entity::UpdateReason::ArtifactAdded => {
            "ARTIFACT_ADDED".to_string()
        }
    }
}
