use axum::{
    extract::{Path, Query, Request, State},
    http::{HeaderValue, Method, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Json, Response},
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tea_server::config::settings;
use tea_server::domain::artifact::entity::Artifact;
use tea_server::domain::artifact::service::ArtifactService;
use tea_server::domain::collection::entity::Collection;
use tea_server::domain::collection::service::CollectionService;
use tea_server::domain::common::deprecation::Deprecation;
use tea_server::domain::common::error::DomainError;
use tea_server::domain::common::pagination::PaginationParams;
use tea_server::domain::component::entity::Component;
use tea_server::domain::component::service::ComponentService;
use tea_server::domain::product::entity::Product;
use tea_server::domain::product::service::ProductService;
use tea_server::infrastructure::auth::jwt::require_auth;
use tea_server::infrastructure::auth::mtls::load_tls_server_config;
use tea_server::infrastructure::observability::telemetry::init_telemetry;
use tea_server::infrastructure::persistence::memory::artifact_repository::InMemoryArtifactRepository;
use tea_server::infrastructure::persistence::memory::collection_repository::InMemoryCollectionRepository;
use tea_server::infrastructure::persistence::memory::component_repository::InMemoryComponentRepository;
use tea_server::infrastructure::persistence::memory::product_repository::InMemoryProductRepository;
use tea_server::interface::discovery::well_known_tea;
use tea_server::interface::error::{AppError, AppJson};
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::set_header::SetResponseHeaderLayer;
use uuid::Uuid;

/// SA-08/16: All per-domain services, loaded configuration, and discovery
/// base_url are held on AppState, shared via Arc across requests.
#[derive(Clone)]
struct AppState {
    product_service: Arc<ProductService<InMemoryProductRepository>>,
    component_service: Arc<ComponentService<InMemoryComponentRepository>>,
    artifact_service: Arc<ArtifactService<InMemoryArtifactRepository>>,
    collection_service: Arc<CollectionService<InMemoryCollectionRepository>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_telemetry();

    // SA-08/16: Load + validate all config at startup. Panics on misconfiguration.
    let cfg = settings::load();

    let state = Arc::new(AppState {
        product_service: Arc::new(ProductService::new(InMemoryProductRepository::new())),
        component_service: Arc::new(ComponentService::new(InMemoryComponentRepository::new())),
        artifact_service: Arc::new(ArtifactService::new(InMemoryArtifactRepository::new())),
        collection_service: Arc::new(CollectionService::new(InMemoryCollectionRepository::new())),
    });

    // SA-15: base_url held in Arc<String>, shared to discovery handler via separate State
    let base_url = Arc::new(cfg.server_url.clone());

    // ── Security headers (SA-06) ──────────────────────────────────────────
    let hsts = SetResponseHeaderLayer::overriding(
        axum::http::header::HeaderName::from_static("strict-transport-security"),
        HeaderValue::from_static("max-age=31536000; includeSubDomains"),
    );
    let nosniff = SetResponseHeaderLayer::overriding(
        axum::http::header::HeaderName::from_static("x-content-type-options"),
        HeaderValue::from_static("nosniff"),
    );
    let no_frame = SetResponseHeaderLayer::overriding(
        axum::http::header::HeaderName::from_static("x-frame-options"),
        HeaderValue::from_static("DENY"),
    );
    let referrer = SetResponseHeaderLayer::overriding(
        axum::http::header::HeaderName::from_static("referrer-policy"),
        HeaderValue::from_static("no-referrer"),
    );

    // ── CORS (SA-07) ─────────────────────────────────────────────────────
    let cors = if cfg.allowed_origins.is_empty() {
        // No CORS header set — same-origin only (browser default, safest)
        CorsLayer::new()
    } else {
        let origins: Vec<HeaderValue> = cfg
            .allowed_origins
            .iter()
            .filter_map(|o| o.parse().ok())
            .collect();
        CorsLayer::new()
            .allow_origin(AllowOrigin::list(origins))
            .allow_methods([Method::GET, Method::POST])
            .allow_headers([
                axum::http::header::CONTENT_TYPE,
                axum::http::header::AUTHORIZATION,
            ])
    };

    // ── Write routes — require JWT auth (SA-01) ───────────────────────────
    let v1_write_routes = Router::new()
        .route("/products", post(create_product))
        .route("/products/:uuid/deprecate", post(deprecate_product))
        .route("/components", post(create_component))
        .route("/components/:uuid/deprecate", post(deprecate_component))
        .route("/artifacts", post(create_artifact))
        .route("/artifacts/:uuid/deprecate", post(deprecate_artifact))
        .route("/collections", post(create_collection))
        .route("/collections/:uuid/deprecate", post(deprecate_collection))
        .route_layer(middleware::from_fn(require_auth)); // ← all writes need a valid JWT

    // ── Read routes — public (TEA spec: read access is unauthenticated) ──
    let v1_read_routes = Router::new()
        .route("/products", get(list_products))
        .route("/products/:uuid", get(get_product))
        .route("/components", get(list_components))
        .route("/components/:uuid", get(get_component))
        .route("/artifacts", get(list_artifacts))
        .route("/artifacts/:uuid", get(get_artifact))
        .route("/collections", get(list_collections))
        .route("/collections/:uuid", get(get_collection));

    let v1_routes = Router::new().merge(v1_write_routes).merge(v1_read_routes);

    let app = Router::new()
        // SA-15: Pass base_url Arc to discovery handler as separate State
        .route("/.well-known/tea", get(well_known_tea).with_state(base_url))
        .route("/health", get(|| async { (StatusCode::OK, "OK") }))
        .nest("/v1", v1_routes)
        .with_state(state)
        // SA-05: 64 KiB body size limit — prevents memory exhaustion via oversized bodies
        .layer(RequestBodyLimitLayer::new(64 * 1024))
        // SA-07: CORS policy
        .layer(cors)
        // SA-06: Security response headers
        .layer(hsts)
        .layer(nosniff)
        .layer(no_frame)
        .layer(referrer)
        // Content-Type enforcement on write methods (existing fix, kept)
        .layer(middleware::from_fn(require_json_content_type));

    let addr = format!("0.0.0.0:{}", cfg.port);
    let addr: std::net::SocketAddr = addr.parse()?;

    // Load TLS configuration if enabled
    let tls_server_config = load_tls_server_config(&cfg.tls).map_err(|e| {
        Box::new(std::io::Error::new(std::io::ErrorKind::Other, e)) as Box<dyn std::error::Error>
    })?;

    tracing::info!(
        addr = %addr,
        base_url = %cfg.server_url,
        tls_enabled = cfg.tls.enabled,
        mtls_enabled = cfg.tls.mtls_enabled,
        "TEA Server listening"
    );

    match tls_server_config {
        Some(tls_config) => {
            // TLS/mTLS mode - use axum-server with rustls
            use axum_server::tls_rustls::RustlsConfig;
            tracing::info!("TLS enabled - secure HTTPS server");
            let rustls_config = RustlsConfig::from_config(tls_config);
            axum_server::bind_rustls(addr, rustls_config)
                .serve(app.into_make_service())
                .await?;
        }
        None => {
            // Plaintext mode (for reverse proxy termination)
            axum_server::bind(addr)
                .serve(app.into_make_service())
                .await?;
        }
    }
    Ok(())
}

// ─── Content-Type enforcement ──────────────────────────────────────────────
async fn require_json_content_type(req: Request, next: Next) -> Response {
    let method = req.method().clone();
    let needs_body = matches!(method, Method::POST | Method::PUT | Method::PATCH);
    if needs_body {
        let ct = req
            .headers()
            .get(axum::http::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        if !ct.starts_with("application/json") {
            return (
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                Json(serde_json::json!({
                    "error": "Unsupported Media Type",
                    "message": "Content-Type must be application/json",
                    "status": 415,
                })),
            )
                .into_response();
        }
    }
    next.run(req).await
}

// ─────────────────────────────── Products ─────────────────────────────────

async fn list_products(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, AppError> {
    let page = state.product_service.list_products(params).await?;
    Ok(Json(page))
}

async fn get_product(
    Path(uuid): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Product>, AppError> {
    let product = state
        .product_service
        .get_product(&uuid)
        .await?
        .ok_or_else(|| DomainError::NotFound(format!("Product {uuid} not found")))?;
    Ok(Json(product))
}

async fn create_product(
    State(state): State<Arc<AppState>>,
    AppJson(product): AppJson<Product>,
) -> Result<impl IntoResponse, AppError> {
    let created = state.product_service.create_product(product).await?;
    let location = format!("/v1/products/{}", created.uuid);
    Ok((
        StatusCode::CREATED,
        [(axum::http::header::LOCATION, location)],
        Json(created),
    ))
}

async fn deprecate_product(
    Path(uuid): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    AppJson(deprecation): AppJson<Deprecation>,
) -> Result<Json<Product>, AppError> {
    let product = state
        .product_service
        .deprecate_product(&uuid, deprecation)
        .await?;
    Ok(Json(product))
}

// ─────────────────────────────── Components ───────────────────────────────

async fn list_components(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, AppError> {
    let page = state.component_service.list_components(params).await?;
    Ok(Json(page))
}

async fn get_component(
    Path(uuid): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Component>, AppError> {
    let component = state
        .component_service
        .get_component(&uuid)
        .await?
        .ok_or_else(|| DomainError::NotFound(format!("Component {uuid} not found")))?;
    Ok(Json(component))
}

async fn create_component(
    State(state): State<Arc<AppState>>,
    AppJson(component): AppJson<Component>,
) -> Result<impl IntoResponse, AppError> {
    let created = state.component_service.create_component(component).await?;
    let location = format!("/v1/components/{}", created.uuid);
    Ok((
        StatusCode::CREATED,
        [(axum::http::header::LOCATION, location)],
        Json(created),
    ))
}

async fn deprecate_component(
    Path(uuid): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    AppJson(deprecation): AppJson<Deprecation>,
) -> Result<Json<Component>, AppError> {
    let component = state
        .component_service
        .deprecate_component(&uuid, deprecation)
        .await?;
    Ok(Json(component))
}

// ─────────────────────────────── Artifacts ────────────────────────────────

async fn list_artifacts(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, AppError> {
    let page = state.artifact_service.list_artifacts(params).await?;
    Ok(Json(page))
}

async fn get_artifact(
    Path(uuid): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Artifact>, AppError> {
    let artifact = state
        .artifact_service
        .get_artifact(&uuid)
        .await?
        .ok_or_else(|| DomainError::NotFound(format!("Artifact {uuid} not found")))?;
    Ok(Json(artifact))
}

async fn create_artifact(
    State(state): State<Arc<AppState>>,
    AppJson(artifact): AppJson<Artifact>,
) -> Result<impl IntoResponse, AppError> {
    let created = state.artifact_service.create_artifact(artifact).await?;
    let location = format!("/v1/artifacts/{}", created.uuid);
    Ok((
        StatusCode::CREATED,
        [(axum::http::header::LOCATION, location)],
        Json(created),
    ))
}

async fn deprecate_artifact(
    Path(uuid): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    AppJson(deprecation): AppJson<Deprecation>,
) -> Result<Json<Artifact>, AppError> {
    let artifact = state
        .artifact_service
        .deprecate_artifact(&uuid, deprecation)
        .await?;
    Ok(Json(artifact))
}

// ─────────────────────────────── Collections ──────────────────────────────

async fn list_collections(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, AppError> {
    let page = state.collection_service.list_collections(params).await?;
    Ok(Json(page))
}

async fn get_collection(
    Path(uuid): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Collection>, AppError> {
    let collection = state
        .collection_service
        .get_collection(&uuid)
        .await?
        .ok_or_else(|| DomainError::NotFound(format!("Collection {uuid} not found")))?;
    Ok(Json(collection))
}

async fn create_collection(
    State(state): State<Arc<AppState>>,
    AppJson(collection): AppJson<Collection>,
) -> Result<impl IntoResponse, AppError> {
    let created = state
        .collection_service
        .create_collection(collection)
        .await?;
    let location = format!("/v1/collections/{}", created.uuid);
    Ok((
        StatusCode::CREATED,
        [(axum::http::header::LOCATION, location)],
        Json(created),
    ))
}

async fn deprecate_collection(
    Path(uuid): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    AppJson(deprecation): AppJson<Deprecation>,
) -> Result<Json<Collection>, AppError> {
    let collection = state
        .collection_service
        .deprecate_collection(&uuid, deprecation)
        .await?;
    Ok(Json(collection))
}
