use axum::{
    error_handling::HandleErrorLayer,
    extract::{Path, Query, Request, State},
    http::{HeaderValue, Method, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Json, Response},
    routing::{delete, get, post, put},
    Router,
};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use tea_server::application::publisher::service::{
    PublisherApplicationService, PublisherApplicationServiceImpl,
};
use tea_server::config::settings::{self, PersistenceBackend};
use tea_server::domain::artifact::entity::Artifact;
use tea_server::domain::artifact::repository::ArtifactRepository;
use tea_server::domain::artifact::service::ArtifactService;
use tea_server::domain::collection::entity::Collection;
use tea_server::domain::collection::repository::CollectionRepository;
use tea_server::domain::collection::service::CollectionService;
use tea_server::domain::common::deprecation::Deprecation;
use tea_server::domain::common::error::DomainError;
use tea_server::domain::common::identifier::Identifier;
use tea_server::domain::common::pagination::PaginationParams;
use tea_server::domain::component::entity::{Component, ComponentType, LicenseInfo};
use tea_server::domain::component::repository::ComponentRepository;
use tea_server::domain::component::service::ComponentService;
use tea_server::domain::product::entity::{Product, Vendor};
use tea_server::domain::product::repository::ProductRepository;
use tea_server::domain::product::service::ProductService;
use tea_server::gen::tea::v1::consumer_service_server::ConsumerServiceServer;
use tea_server::gen::tea::v1::discovery_service_server::DiscoveryServiceServer;
use tea_server::gen::tea::v1::publisher_service_server::PublisherServiceServer;
use tea_server::infrastructure::auth::jwt::require_auth;
use tea_server::infrastructure::auth::mtls::load_tls_server_config;
use tea_server::infrastructure::grpc::{
    publisher_auth_interceptor, ConsumerGrpcService, DiscoveryGrpcService, PublisherGrpcService,
};
use tea_server::infrastructure::middleware::rate_limit::{
    rate_limit, rate_limit_writes, RateLimitConfig, RateLimiter,
};
use tea_server::infrastructure::observability::telemetry::init_telemetry;
use tea_server::infrastructure::persistence::memory::artifact_repository::InMemoryArtifactRepository;
use tea_server::infrastructure::persistence::memory::collection_repository::InMemoryCollectionRepository;
use tea_server::infrastructure::persistence::memory::component_repository::InMemoryComponentRepository;
use tea_server::infrastructure::persistence::memory::product_repository::InMemoryProductRepository;
use tea_server::infrastructure::persistence::postgres::artifact_repository::PostgresArtifactRepository;
use tea_server::infrastructure::persistence::postgres::collection_repository::PostgresCollectionRepository;
use tea_server::infrastructure::persistence::postgres::component_repository::PostgresComponentRepository;
use tea_server::infrastructure::persistence::postgres::product_repository::PostgresProductRepository;
use tea_server::interface::discovery::well_known_tea;
use tea_server::interface::error::{AppError, AppJson};
use tokio::sync::watch;
use tonic::transport::Server;
use tower::{BoxError, ServiceBuilder};
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::trace::TraceLayer;
use uuid::Uuid;

type DynProductRepository = Arc<dyn ProductRepository + Send + Sync>;
type DynComponentRepository = Arc<dyn ComponentRepository + Send + Sync>;
type DynArtifactRepository = Arc<dyn ArtifactRepository + Send + Sync>;
type DynCollectionRepository = Arc<dyn CollectionRepository + Send + Sync>;
type DynPublisherApplicationService = Arc<dyn PublisherApplicationService>;
type AppResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// SA-08/16: All per-domain services, loaded configuration, and discovery
/// base_url are held on AppState, shared via Arc across requests.
#[derive(Clone)]
struct AppState {
    product_service: Arc<ProductService<DynProductRepository>>,
    component_service: Arc<ComponentService<DynComponentRepository>>,
    artifact_service: Arc<ArtifactService<DynArtifactRepository>>,
    collection_service: Arc<CollectionService<DynCollectionRepository>>,
    publisher_service: DynPublisherApplicationService,
    readiness: ReadinessProbe,
}

#[derive(Clone)]
enum ReadinessProbe {
    Memory,
    Postgres(PgPool),
}

impl ReadinessProbe {
    fn backend_name(&self) -> &'static str {
        match self {
            Self::Memory => "memory",
            Self::Postgres(_) => "postgres",
        }
    }

    async fn check(&self) -> Result<(), String> {
        match self {
            Self::Memory => Ok(()),
            Self::Postgres(pool) => sqlx::query_scalar::<_, i32>("SELECT 1")
                .fetch_one(pool)
                .await
                .map(|_| ())
                .map_err(|e| format!("postgres readiness check failed: {e}")),
        }
    }
}

#[tokio::main]
async fn main() -> AppResult<()> {
    init_telemetry();

    // SA-08/16: Load + validate all config at startup. Panics on misconfiguration.
    let cfg = settings::load();
    let shutdown = install_shutdown_notifier();
    let grpc_enabled = cfg.grpc.enabled;
    let grpc_port = cfg.grpc.port;
    let grpc_publisher_enabled = cfg.grpc.publisher_enabled;
    let server_url = cfg.server_url.clone();

    let (
        product_repository,
        component_repository,
        artifact_repository,
        collection_repository,
        readiness_probe,
    ): (
        DynProductRepository,
        DynComponentRepository,
        DynArtifactRepository,
        DynCollectionRepository,
        ReadinessProbe,
    ) = match cfg.persistence_backend {
        PersistenceBackend::Memory => (
            Arc::new(InMemoryProductRepository::new()),
            Arc::new(InMemoryComponentRepository::new()),
            Arc::new(InMemoryArtifactRepository::new()),
            Arc::new(InMemoryCollectionRepository::new()),
            ReadinessProbe::Memory,
        ),
        PersistenceBackend::Postgres => {
            let pool = PgPoolOptions::new()
                .max_connections(10)
                .connect(&cfg.database_url)
                .await?;

            (
                Arc::new(PostgresProductRepository::new(pool.clone())),
                Arc::new(PostgresComponentRepository::new(pool.clone())),
                Arc::new(PostgresArtifactRepository::new(pool.clone())),
                Arc::new(PostgresCollectionRepository::new(pool.clone())),
                ReadinessProbe::Postgres(pool),
            )
        }
    };

    let grpc_consumer_service = ConsumerGrpcService::new(
        product_repository.clone(),
        component_repository.clone(),
        collection_repository.clone(),
        artifact_repository.clone(),
    );
    let grpc_discovery_service = DiscoveryGrpcService::new(&server_url);
    let grpc_publisher_service: DynPublisherApplicationService =
        Arc::new(PublisherApplicationServiceImpl::new(
            ProductService::new(product_repository.clone()),
            ComponentService::new(component_repository.clone()),
            CollectionService::new(collection_repository.clone()),
            ArtifactService::new(artifact_repository.clone()),
        ));

    let state = Arc::new(AppState {
        product_service: Arc::new(ProductService::new(product_repository.clone())),
        component_service: Arc::new(ComponentService::new(component_repository.clone())),
        artifact_service: Arc::new(ArtifactService::new(artifact_repository.clone())),
        collection_service: Arc::new(CollectionService::new(collection_repository.clone())),
        publisher_service: grpc_publisher_service.clone(),
        readiness: readiness_probe,
    });
    let global_rate_limit_config = RateLimitConfig::from_env();
    let global_rate_limiter = Arc::new(RateLimiter::new(global_rate_limit_config.clone()));
    let write_rate_limiter = Arc::new(RateLimiter::new(RateLimitConfig {
        requests_per_minute: 10,
        burst: 2,
        enabled: global_rate_limit_config.enabled,
    }));

    // SA-15: base_url held in Arc<String>, shared to discovery handler via separate State
    let base_url = Arc::new(server_url.clone());
    let request_timeout = Duration::from_secs(cfg.request_timeout_secs);

    {
        let limiter = global_rate_limiter.clone();
        let cleanup_every = Duration::from_secs(cfg.rate_limit_cleanup_secs);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(cleanup_every);
            loop {
                interval.tick().await;
                limiter.cleanup().await;
            }
        });
    }

    {
        let limiter = write_rate_limiter.clone();
        let cleanup_every = Duration::from_secs(cfg.rate_limit_cleanup_secs);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(cleanup_every);
            loop {
                interval.tick().await;
                limiter.cleanup().await;
            }
        });
    }

    // ── Security headers (SA-06) ──────────────────────────────────────────
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
            .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
            .allow_headers([
                axum::http::header::CONTENT_TYPE,
                axum::http::header::AUTHORIZATION,
            ])
    };

    // ── Write routes — require JWT auth (SA-01) ───────────────────────────
    let v1_write_routes = Router::new()
        .route("/products", post(create_product))
        .route(
            "/products/:uuid",
            put(update_product).delete(delete_product),
        )
        .route("/products/:uuid/deprecate", post(deprecate_product))
        .route("/components", post(create_component))
        .route(
            "/components/:uuid",
            put(update_component).delete(delete_component),
        )
        .route("/components/:uuid/deprecate", post(deprecate_component))
        .route("/artifacts", post(create_artifact))
        .route("/artifacts/:uuid", delete(delete_artifact))
        .route("/artifacts/:uuid/deprecate", post(deprecate_artifact))
        .route("/collections", post(create_collection))
        .route("/collections/:uuid", delete(delete_collection))
        .route("/collections/:uuid/deprecate", post(deprecate_collection))
        .route_layer(middleware::from_fn_with_state(
            write_rate_limiter.clone(),
            rate_limit_writes,
        ))
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
        .route("/collections/:uuid", get(get_collection))
        .route("/collections/:uuid/versions", get(list_collection_versions))
        .route(
            "/collections/:uuid/versions/:version",
            get(get_collection_version),
        )
        .route(
            "/collections/:uuid/compare",
            get(compare_collection_versions),
        );

    let v1_write_routes = v1_write_routes.route(
        "/collections/:uuid/versions",
        post(create_collection_version),
    );

    let v1_routes = Router::new().merge(v1_write_routes).merge(v1_read_routes);

    let app = Router::new()
        // SA-15: Pass base_url Arc to discovery handler as separate State
        .route("/.well-known/tea", get(well_known_tea).with_state(base_url))
        .route("/health", get(|| async { (StatusCode::OK, "OK") }))
        .route("/ready", get(readiness))
        .nest("/v1", v1_routes)
        .with_state(state)
        .layer(middleware::from_fn_with_state(
            global_rate_limiter.clone(),
            rate_limit,
        ))
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(|_: BoxError| async {
                    (
                        StatusCode::REQUEST_TIMEOUT,
                        Json(serde_json::json!({
                            "error": "Request Timeout",
                            "message": "The request took too long to complete.",
                            "status": 408,
                        })),
                    )
                }))
                .layer(tower::timeout::TimeoutLayer::new(request_timeout)),
        )
        .layer(TraceLayer::new_for_http())
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        // SA-05: 64 KiB body size limit — prevents memory exhaustion via oversized bodies
        .layer(RequestBodyLimitLayer::new(64 * 1024))
        // SA-07: CORS policy
        .layer(cors)
        // SA-06: Security response headers
        .layer(nosniff)
        .layer(no_frame)
        .layer(referrer)
        // Content-Type enforcement on write methods (existing fix, kept)
        .layer(middleware::from_fn(require_json_content_type));

    let app = if cfg.server_url.starts_with("https://") {
        app.layer(SetResponseHeaderLayer::overriding(
            axum::http::header::HeaderName::from_static("strict-transport-security"),
            HeaderValue::from_static("max-age=31536000; includeSubDomains"),
        ))
    } else {
        app
    };

    let addr = format!("0.0.0.0:{}", cfg.port);
    let addr: std::net::SocketAddr = addr.parse()?;

    // Load TLS configuration if enabled
    let tls_server_config = load_tls_server_config(&cfg.tls).map_err(|e| {
        Box::new(std::io::Error::other(e)) as Box<dyn std::error::Error + Send + Sync>
    })?;

    tracing::info!(
        addr = %addr,
        base_url = %cfg.server_url,
        persistence_backend = ?cfg.persistence_backend,
        tls_enabled = cfg.tls.enabled,
        mtls_enabled = cfg.tls.mtls_enabled,
        "TEA Server listening"
    );

    if grpc_enabled {
        let grpc_addr = std::net::SocketAddr::from(([0, 0, 0, 0], grpc_port));
        tracing::info!(
            grpc_addr = %grpc_addr,
            publisher_enabled = grpc_publisher_enabled,
            "TEA gRPC server enabled"
        );

        tokio::try_join!(
            run_http_server(addr, tls_server_config, app, shutdown.clone()),
            run_grpc_server(
                grpc_addr,
                grpc_discovery_service,
                grpc_consumer_service,
                grpc_publisher_service,
                grpc_publisher_enabled,
                shutdown.clone(),
            ),
        )?;
    } else {
        run_http_server(addr, tls_server_config, app, shutdown.clone()).await?;
    }
    Ok(())
}

async fn run_http_server(
    addr: std::net::SocketAddr,
    tls_server_config: Option<Arc<rustls::ServerConfig>>,
    app: Router,
    mut shutdown: watch::Receiver<bool>,
) -> AppResult<()> {
    let handle = axum_server::Handle::new();
    let shutdown_handle = handle.clone();
    tokio::spawn(async move {
        let _ = shutdown.changed().await;
        tracing::info!("shutdown signal received, draining HTTP listener");
        shutdown_handle.graceful_shutdown(Some(Duration::from_secs(30)));
    });

    match tls_server_config {
        Some(tls_config) => {
            use axum_server::tls_rustls::RustlsConfig;

            tracing::info!("TLS enabled - secure HTTPS server");
            let rustls_config = RustlsConfig::from_config(tls_config);
            axum_server::bind_rustls(addr, rustls_config)
                .handle(handle)
                .serve(app.into_make_service_with_connect_info::<std::net::SocketAddr>())
                .await?;
        }
        None => {
            axum_server::bind(addr)
                .handle(handle)
                .serve(app.into_make_service_with_connect_info::<std::net::SocketAddr>())
                .await?;
        }
    }

    Ok(())
}

async fn run_grpc_server(
    grpc_addr: std::net::SocketAddr,
    discovery_service: DiscoveryGrpcService,
    consumer_service: ConsumerGrpcService<
        dyn ProductRepository + Send + Sync,
        dyn ComponentRepository + Send + Sync,
        dyn CollectionRepository + Send + Sync,
        dyn ArtifactRepository + Send + Sync,
    >,
    publisher_service: DynPublisherApplicationService,
    publisher_enabled: bool,
    mut shutdown: watch::Receiver<bool>,
) -> AppResult<()> {
    let mut builder = Server::builder()
        .accept_http1(true)
        .http2_keepalive_interval(Some(Duration::from_secs(30)))
        .tcp_keepalive(Some(Duration::from_secs(30)));

    let discovery = tonic_web::enable(DiscoveryServiceServer::new(discovery_service));
    let consumer = tonic_web::enable(ConsumerServiceServer::new(consumer_service));

    if publisher_enabled {
        let publisher = tonic_web::enable(PublisherServiceServer::with_interceptor(
            PublisherGrpcService::new(publisher_service),
            publisher_auth_interceptor,
        ));
        builder
            .add_service(discovery)
            .add_service(consumer)
            .add_service(publisher)
            .serve_with_shutdown(grpc_addr, async move {
                let _ = shutdown.changed().await;
                tracing::info!("shutdown signal received, draining gRPC listener");
            })
            .await?;
    } else {
        builder
            .add_service(discovery)
            .add_service(consumer)
            .serve_with_shutdown(grpc_addr, async move {
                let _ = shutdown.changed().await;
                tracing::info!("shutdown signal received, draining gRPC listener");
            })
            .await?;
    }

    Ok(())
}

fn install_shutdown_notifier() -> watch::Receiver<bool> {
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    tokio::spawn(async move {
        shutdown_signal().await;
        let _ = shutdown_tx.send(true);
    });
    shutdown_rx
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C shutdown handler");
    };

    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{signal, SignalKind};

        signal(SignalKind::terminate())
            .expect("failed to install SIGTERM shutdown handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
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

async fn readiness(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match state.readiness.check().await {
        Ok(()) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "status": "ready",
                "backend": state.readiness.backend_name(),
            })),
        ),
        Err(error) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "status": "not_ready",
                "backend": state.readiness.backend_name(),
                "error": error,
            })),
        ),
    }
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

#[derive(Debug, serde::Deserialize)]
struct UpdateProductRequest {
    name: String,
    description: Option<String>,
    #[serde(default)]
    identifiers: Vec<Identifier>,
    vendor: Vendor,
    homepage_url: Option<String>,
    documentation_url: Option<String>,
    vcs_url: Option<String>,
}

#[derive(Debug, Default, serde::Deserialize)]
struct DeleteProductParams {
    #[serde(default)]
    cascade: bool,
}

async fn update_product(
    Path(uuid): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    AppJson(req): AppJson<UpdateProductRequest>,
) -> Result<Json<Product>, AppError> {
    let existing = state
        .product_service
        .get_product(&uuid)
        .await?
        .ok_or_else(|| DomainError::NotFound(format!("Product {uuid} not found")))?;

    let updated = Product {
        uuid,
        name: req.name,
        description: req.description,
        identifiers: req.identifiers,
        vendor: req.vendor,
        created_date: existing.created_date,
        modified_date: existing.modified_date,
        homepage_url: req.homepage_url,
        documentation_url: req.documentation_url,
        vcs_url: req.vcs_url,
        deprecation: existing.deprecation,
        dependencies: existing.dependencies,
    };

    let product = state.product_service.update_product(updated).await?;
    Ok(Json(product))
}

async fn delete_product(
    Path(uuid): Path<Uuid>,
    Query(params): Query<DeleteProductParams>,
    State(state): State<Arc<AppState>>,
) -> Result<StatusCode, AppError> {
    let releases = state.publisher_service.list_product_releases(&uuid).await?;

    if !params.cascade && !releases.is_empty() {
        return Err(DomainError::Conflict(
            "product still has releases; set cascade=true to delete them".to_string(),
        )
        .into());
    }

    if params.cascade {
        for release in &releases {
            state
                .publisher_service
                .delete_product_release(&release.uuid)
                .await?;
        }
    }

    state.product_service.delete_product(&uuid).await?;
    Ok(StatusCode::NO_CONTENT)
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

#[derive(Debug, serde::Deserialize)]
struct UpdateComponentRequest {
    name: String,
    description: Option<String>,
    #[serde(default)]
    identifiers: Vec<Identifier>,
    component_type: ComponentType,
    #[serde(default)]
    licenses: Vec<LicenseInfo>,
    publisher: Option<String>,
    homepage_url: Option<String>,
    vcs_url: Option<String>,
}

#[derive(Debug, Default, serde::Deserialize)]
struct DeleteComponentParams {
    #[serde(default)]
    cascade: bool,
}

async fn update_component(
    Path(uuid): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    AppJson(req): AppJson<UpdateComponentRequest>,
) -> Result<Json<Component>, AppError> {
    let existing = state
        .component_service
        .get_component(&uuid)
        .await?
        .ok_or_else(|| DomainError::NotFound(format!("Component {uuid} not found")))?;

    let updated = Component {
        uuid,
        name: req.name,
        description: req.description,
        identifiers: req.identifiers,
        component_type: req.component_type,
        licenses: req.licenses,
        publisher: req.publisher,
        homepage_url: req.homepage_url,
        vcs_url: req.vcs_url,
        created_date: existing.created_date,
        modified_date: existing.modified_date,
        deprecation: existing.deprecation,
        dependencies: existing.dependencies,
    };

    let component = state.component_service.update_component(updated).await?;
    Ok(Json(component))
}

async fn delete_component(
    Path(uuid): Path<Uuid>,
    Query(params): Query<DeleteComponentParams>,
    State(state): State<Arc<AppState>>,
) -> Result<StatusCode, AppError> {
    let releases = state
        .publisher_service
        .list_component_releases(&uuid)
        .await?;

    if !params.cascade && !releases.is_empty() {
        return Err(DomainError::Conflict(
            "component still has releases; set cascade=true to delete them".to_string(),
        )
        .into());
    }

    if params.cascade {
        for release in &releases {
            state
                .publisher_service
                .delete_component_release(&release.uuid)
                .await?;
        }
    }

    state.component_service.delete_component(&uuid).await?;
    Ok(StatusCode::NO_CONTENT)
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

#[derive(Debug, Default, serde::Deserialize)]
struct DeleteArtifactParams {
    #[serde(default)]
    force: bool,
}

async fn delete_artifact(
    Path(uuid): Path<Uuid>,
    Query(params): Query<DeleteArtifactParams>,
    State(state): State<Arc<AppState>>,
) -> Result<StatusCode, AppError> {
    let referenced_by = state
        .publisher_service
        .list_collections()
        .await?
        .into_iter()
        .filter(|collection| collection.artifacts.contains(&uuid))
        .map(|collection| collection.uuid.to_string())
        .collect::<Vec<_>>();

    if !referenced_by.is_empty() {
        let message = if params.force {
            format!(
                "force delete is not supported while artifact {uuid} is still referenced by {} collection(s)",
                referenced_by.len()
            )
        } else {
            format!(
                "artifact {uuid} is still referenced by {} collection(s)",
                referenced_by.len()
            )
        };
        return Err(DomainError::Conflict(message).into());
    }

    state.artifact_service.delete_artifact(&uuid).await?;
    Ok(StatusCode::NO_CONTENT)
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
    ensure_artifacts_exist(&state, &collection.artifacts).await?;
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

#[derive(Debug, serde::Deserialize)]
struct CreateCollectionVersionRequest {
    artifacts: Vec<Uuid>,
    update_reason: tea_server::domain::collection::entity::UpdateReason,
}

#[derive(Debug, serde::Serialize)]
struct CompareCollectionVersionsResponse {
    uuid: Uuid,
    base_version: i32,
    target_version: i32,
    added_artifact_uuids: Vec<Uuid>,
    removed_artifact_uuids: Vec<Uuid>,
    modified_artifact_uuids: Vec<Uuid>,
}

#[derive(Debug, serde::Deserialize)]
struct CompareCollectionVersionsParams {
    base_version: i32,
    target_version: i32,
}

async fn delete_collection(
    Path(uuid): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> Result<StatusCode, AppError> {
    state.collection_service.delete_collection(&uuid).await?;
    Ok(StatusCode::NO_CONTENT)
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

async fn list_collection_versions(
    Path(uuid): Path<Uuid>,
    Query(params): Query<PaginationParams>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<tea_server::domain::common::pagination::Page<Collection>>, AppError> {
    let versions = state
        .collection_service
        .list_collection_versions(&uuid)
        .await?;
    Ok(Json(tea_server::domain::common::pagination::Page::new(
        versions, &params,
    )))
}

async fn get_collection_version(
    Path((uuid, version)): Path<(Uuid, i32)>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Collection>, AppError> {
    let collection = state
        .collection_service
        .list_collection_versions(&uuid)
        .await?
        .into_iter()
        .find(|collection| collection.version == version)
        .ok_or_else(|| {
            DomainError::NotFound(format!("Collection {uuid} version {version} not found"))
        })?;
    Ok(Json(collection))
}

async fn create_collection_version(
    Path(uuid): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    AppJson(req): AppJson<CreateCollectionVersionRequest>,
) -> Result<impl IntoResponse, AppError> {
    ensure_artifacts_exist(&state, &req.artifacts).await?;
    let created = state
        .collection_service
        .create_next_version(&uuid, req.artifacts, req.update_reason)
        .await?;
    let location = format!(
        "/v1/collections/{}/versions/{}",
        created.uuid, created.version
    );
    Ok((
        StatusCode::CREATED,
        [(axum::http::header::LOCATION, location)],
        Json(created),
    ))
}

async fn compare_collection_versions(
    Path(uuid): Path<Uuid>,
    Query(params): Query<CompareCollectionVersionsParams>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<CompareCollectionVersionsResponse>, AppError> {
    let versions = state
        .collection_service
        .list_collection_versions(&uuid)
        .await?;
    let base = versions
        .iter()
        .find(|collection| collection.version == params.base_version)
        .ok_or_else(|| {
            DomainError::NotFound(format!(
                "Collection {uuid} version {} not found",
                params.base_version
            ))
        })?;
    let target = versions
        .iter()
        .find(|collection| collection.version == params.target_version)
        .ok_or_else(|| {
            DomainError::NotFound(format!(
                "Collection {uuid} version {} not found",
                params.target_version
            ))
        })?;

    let base_ids: HashSet<Uuid> = base.artifacts.iter().copied().collect();
    let target_ids: HashSet<Uuid> = target.artifacts.iter().copied().collect();

    let mut added_artifact_uuids: Vec<Uuid> = target_ids.difference(&base_ids).copied().collect();
    let mut removed_artifact_uuids: Vec<Uuid> = base_ids.difference(&target_ids).copied().collect();
    added_artifact_uuids.sort();
    removed_artifact_uuids.sort();

    Ok(Json(CompareCollectionVersionsResponse {
        uuid,
        base_version: params.base_version,
        target_version: params.target_version,
        added_artifact_uuids,
        removed_artifact_uuids,
        modified_artifact_uuids: vec![],
    }))
}

async fn ensure_artifacts_exist(state: &AppState, artifact_uuids: &[Uuid]) -> Result<(), AppError> {
    for artifact_uuid in artifact_uuids {
        let exists = state.artifact_service.get_artifact(artifact_uuid).await?;
        if exists.is_none() {
            return Err(
                DomainError::NotFound(format!("Artifact {artifact_uuid} not found")).into(),
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        compare_collection_versions, create_collection, create_collection_version, delete_artifact,
        delete_component, delete_product, get_collection_version, list_collection_versions,
        update_product, AppJson, AppState, CompareCollectionVersionsParams,
        CreateCollectionVersionRequest, DeleteArtifactParams, DeleteComponentParams,
        DeleteProductParams, ReadinessProbe, UpdateProductRequest,
    };
    use axum::extract::{Path, Query, State};
    use axum::http::StatusCode;
    use axum::response::IntoResponse;
    use axum::Json;
    use chrono::Utc;
    use sqlx::postgres::PgPoolOptions;
    use std::sync::Arc;
    use std::time::Duration;
    use tea_server::application::publisher::service::{
        PublisherApplicationService, PublisherApplicationServiceImpl,
    };
    use tea_server::domain::artifact::entity::{Artifact, ArtifactFormat, ArtifactType};
    use tea_server::domain::artifact::service::ArtifactService;
    use tea_server::domain::collection::entity::{Collection, CollectionScope, UpdateReason};
    use tea_server::domain::collection::service::CollectionService;
    use tea_server::domain::common::checksum::{Checksum, ChecksumAlgorithm};
    use tea_server::domain::common::deprecation::{Deprecation, DeprecationState};
    use tea_server::domain::common::error::DomainError;
    use tea_server::domain::common::pagination::{Page, PaginationParams};
    use tea_server::domain::component::entity::{Component, ComponentRelease, ComponentType};
    use tea_server::domain::component::service::ComponentService;
    use tea_server::domain::product::entity::{Product, ProductRelease, Vendor};
    use tea_server::domain::product::service::ProductService;
    use tea_server::infrastructure::persistence::memory::artifact_repository::InMemoryArtifactRepository;
    use tea_server::infrastructure::persistence::memory::collection_repository::InMemoryCollectionRepository;
    use tea_server::infrastructure::persistence::memory::component_repository::InMemoryComponentRepository;
    use tea_server::infrastructure::persistence::memory::product_repository::InMemoryProductRepository;
    use uuid::Uuid;

    fn test_state() -> Arc<AppState> {
        let product_repository = Arc::new(InMemoryProductRepository::new());
        let component_repository = Arc::new(InMemoryComponentRepository::new());
        let artifact_repository = Arc::new(InMemoryArtifactRepository::new());
        let collection_repository = Arc::new(InMemoryCollectionRepository::new());
        let publisher_service: Arc<dyn PublisherApplicationService> =
            Arc::new(PublisherApplicationServiceImpl::new(
                ProductService::new(product_repository.clone()),
                ComponentService::new(component_repository.clone()),
                CollectionService::new(collection_repository.clone()),
                ArtifactService::new(artifact_repository.clone()),
            ));

        Arc::new(AppState {
            product_service: Arc::new(ProductService::new(product_repository)),
            component_service: Arc::new(ComponentService::new(component_repository)),
            artifact_service: Arc::new(ArtifactService::new(artifact_repository)),
            collection_service: Arc::new(CollectionService::new(collection_repository)),
            publisher_service,
            readiness: ReadinessProbe::Memory,
        })
    }

    #[tokio::test]
    async fn memory_readiness_is_ready() {
        let probe = ReadinessProbe::Memory;
        assert!(probe.check().await.is_ok());
        assert_eq!(probe.backend_name(), "memory");
    }

    #[tokio::test]
    async fn unreachable_postgres_is_not_ready() {
        let pool = PgPoolOptions::new()
            .acquire_timeout(Duration::from_millis(50))
            .connect_lazy("postgres://postgres:postgres@127.0.0.1:1/postgres")
            .expect("lazy pool creation should succeed");
        let probe = ReadinessProbe::Postgres(pool);

        assert!(probe.check().await.is_err());
        assert_eq!(probe.backend_name(), "postgres");
    }

    #[tokio::test]
    async fn update_product_preserves_created_date_and_deprecation() {
        let state = test_state();
        let created = state
            .product_service
            .create_product(Product {
                uuid: Uuid::nil(),
                name: "before".to_string(),
                description: Some("before".to_string()),
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
        let deprecated = state
            .product_service
            .deprecate_product(
                &created.uuid,
                Deprecation {
                    state: DeprecationState::Deprecated,
                    reason: Some("moved".to_string()),
                    announced_date: None,
                    effective_date: None,
                    replacement_identifiers: vec![],
                },
            )
            .await
            .unwrap();
        let created_date = deprecated.created_date;
        let original_deprecation = deprecated.deprecation.clone();

        let Json(updated) = update_product(
            Path(deprecated.uuid),
            State(state),
            AppJson(UpdateProductRequest {
                name: "after".to_string(),
                description: Some("after".to_string()),
                identifiers: vec![],
                vendor: Vendor {
                    name: "ACME".to_string(),
                    uuid: None,
                    url: Some("https://example.com".to_string()),
                    contacts: vec![],
                },
                homepage_url: Some("https://example.com/product".to_string()),
                documentation_url: None,
                vcs_url: None,
            }),
        )
        .await
        .unwrap();

        assert_eq!(updated.name, "after");
        assert_eq!(updated.created_date, created_date);
        assert_eq!(updated.deprecation, original_deprecation);
    }

    #[tokio::test]
    async fn delete_component_requires_cascade_when_releases_exist() {
        let state = test_state();
        let component = state
            .component_service
            .create_component(Component {
                uuid: Uuid::nil(),
                name: "component".to_string(),
                description: None,
                identifiers: vec![],
                component_type: ComponentType::Library,
                licenses: vec![],
                publisher: Some("ACME".to_string()),
                homepage_url: None,
                vcs_url: None,
                created_date: Utc::now(),
                modified_date: Utc::now(),
                deprecation: None,
                dependencies: vec![],
            })
            .await
            .unwrap();
        let release = state
            .publisher_service
            .create_component_release(ComponentRelease {
                uuid: Uuid::nil(),
                component_uuid: component.uuid,
                version: "1.0.0".to_string(),
                release_date: None,
                pre_release: false,
                identifiers: vec![],
                distributions: vec![],
            })
            .await
            .unwrap();

        let error = delete_component(
            Path(component.uuid),
            Query(DeleteComponentParams { cascade: false }),
            State(state.clone()),
        )
        .await
        .err()
        .expect("delete_artifact should fail");
        assert!(matches!(error.0, DomainError::Conflict(_)));

        let status = delete_component(
            Path(component.uuid),
            Query(DeleteComponentParams { cascade: true }),
            State(state.clone()),
        )
        .await
        .unwrap();
        assert_eq!(status, StatusCode::NO_CONTENT);
        assert!(state
            .component_service
            .get_component(&component.uuid)
            .await
            .unwrap()
            .is_none());
        assert!(state
            .publisher_service
            .get_component_release(&release.uuid)
            .await
            .unwrap()
            .is_none());
    }

    #[tokio::test]
    async fn delete_product_requires_cascade_when_releases_exist() {
        let state = test_state();
        let product = state
            .product_service
            .create_product(Product {
                uuid: Uuid::nil(),
                name: "product".to_string(),
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
        let release = state
            .publisher_service
            .create_product_release(ProductRelease {
                uuid: Uuid::nil(),
                product_uuid: product.uuid,
                version: "2026.03".to_string(),
                created_date: Utc::now(),
                modified_date: Utc::now(),
                release_date: None,
                pre_release: false,
                identifiers: vec![],
                components: vec![],
            })
            .await
            .unwrap();

        let error = delete_product(
            Path(product.uuid),
            Query(DeleteProductParams { cascade: false }),
            State(state.clone()),
        )
        .await
        .err()
        .expect("create_collection should fail");
        assert!(matches!(error.0, DomainError::Conflict(_)));

        let status = delete_product(
            Path(product.uuid),
            Query(DeleteProductParams { cascade: true }),
            State(state.clone()),
        )
        .await
        .unwrap();
        assert_eq!(status, StatusCode::NO_CONTENT);
        assert!(state
            .product_service
            .get_product(&product.uuid)
            .await
            .unwrap()
            .is_none());
        assert!(state
            .publisher_service
            .get_product_release(&release.uuid)
            .await
            .unwrap()
            .is_none());
    }

    #[tokio::test]
    async fn delete_artifact_blocks_referenced_artifacts_even_with_force() {
        let state = test_state();
        let artifact = state
            .artifact_service
            .create_artifact(Artifact {
                uuid: Uuid::nil(),
                name: "sbom".to_string(),
                type_: ArtifactType::Bom,
                component_distributions: vec![],
                formats: vec![ArtifactFormat {
                    mime_type: "application/vnd.cyclonedx+json".to_string(),
                    description: None,
                    url: "https://example.com/sbom.json".to_string(),
                    signature_url: None,
                    checksums: vec![Checksum {
                        alg_type: ChecksumAlgorithm::Sha256,
                        alg_value: "deadbeef".to_string(),
                    }],
                    size_bytes: None,
                    encoding: None,
                    spec_version: Some("1.6".to_string()),
                }],
                created_date: Utc::now(),
                modified_date: Utc::now(),
                description: None,
                subject: None,
                deprecation: None,
                dependencies: vec![],
            })
            .await
            .unwrap();
        state
            .collection_service
            .create_collection(Collection {
                uuid: Uuid::nil(),
                name: "release collection".to_string(),
                version: 1,
                date: Utc::now(),
                created_date: Utc::now(),
                modified_date: Utc::now(),
                belongs_to: CollectionScope::Release,
                update_reason: UpdateReason::InitialRelease,
                artifacts: vec![artifact.uuid],
                deprecation: None,
                dependencies: vec![],
            })
            .await
            .unwrap();

        let error = delete_artifact(
            Path(artifact.uuid),
            Query(DeleteArtifactParams { force: false }),
            State(state.clone()),
        )
        .await
        .err()
        .expect("create_collection should fail");
        assert!(matches!(error.0, DomainError::Conflict(_)));

        let error = delete_artifact(
            Path(artifact.uuid),
            Query(DeleteArtifactParams { force: true }),
            State(state.clone()),
        )
        .await
        .err()
        .expect("create_collection should fail");
        match error.0 {
            DomainError::Conflict(message) => {
                assert!(message.contains("force delete is not supported"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[tokio::test]
    async fn create_collection_rejects_unknown_artifacts() {
        let state = test_state();

        let error = create_collection(
            State(state),
            AppJson(Collection {
                uuid: Uuid::nil(),
                name: "release collection".to_string(),
                version: 1,
                date: Utc::now(),
                created_date: Utc::now(),
                modified_date: Utc::now(),
                belongs_to: CollectionScope::Release,
                update_reason: UpdateReason::InitialRelease,
                artifacts: vec![Uuid::new_v4()],
                deprecation: None,
                dependencies: vec![],
            }),
        )
        .await
        .err()
        .expect("create_collection should fail");

        assert!(matches!(error.0, DomainError::NotFound(_)));
    }

    #[tokio::test]
    async fn collection_version_routes_create_and_compare_versions() {
        let state = test_state();
        let artifact_v1 = state
            .artifact_service
            .create_artifact(Artifact {
                uuid: Uuid::nil(),
                name: "sbom-v1".to_string(),
                type_: ArtifactType::Bom,
                component_distributions: vec![],
                formats: vec![ArtifactFormat {
                    mime_type: "application/vnd.cyclonedx+json".to_string(),
                    description: None,
                    url: "https://example.com/sbom-v1.json".to_string(),
                    signature_url: None,
                    checksums: vec![Checksum {
                        alg_type: ChecksumAlgorithm::Sha256,
                        alg_value: "deadbeef".to_string(),
                    }],
                    size_bytes: None,
                    encoding: None,
                    spec_version: Some("1.6".to_string()),
                }],
                created_date: Utc::now(),
                modified_date: Utc::now(),
                description: None,
                subject: None,
                deprecation: None,
                dependencies: vec![],
            })
            .await
            .unwrap();
        let artifact_v2 = state
            .artifact_service
            .create_artifact(Artifact {
                uuid: Uuid::nil(),
                name: "sbom-v2".to_string(),
                type_: ArtifactType::Bom,
                component_distributions: vec![],
                formats: vec![ArtifactFormat {
                    mime_type: "application/vnd.cyclonedx+json".to_string(),
                    description: None,
                    url: "https://example.com/sbom-v2.json".to_string(),
                    signature_url: None,
                    checksums: vec![Checksum {
                        alg_type: ChecksumAlgorithm::Sha256,
                        alg_value: "feedface".to_string(),
                    }],
                    size_bytes: None,
                    encoding: None,
                    spec_version: Some("1.6".to_string()),
                }],
                created_date: Utc::now(),
                modified_date: Utc::now(),
                description: None,
                subject: None,
                deprecation: None,
                dependencies: vec![],
            })
            .await
            .unwrap();

        let created = state
            .collection_service
            .create_collection(Collection {
                uuid: Uuid::nil(),
                name: "release collection".to_string(),
                version: 1,
                date: Utc::now(),
                created_date: Utc::now(),
                modified_date: Utc::now(),
                belongs_to: CollectionScope::Release,
                update_reason: UpdateReason::InitialRelease,
                artifacts: vec![artifact_v1.uuid],
                deprecation: None,
                dependencies: vec![],
            })
            .await
            .unwrap();

        let response = create_collection_version(
            Path(created.uuid),
            State(state.clone()),
            AppJson(CreateCollectionVersionRequest {
                artifacts: vec![artifact_v1.uuid, artifact_v2.uuid],
                update_reason: UpdateReason::ArtifactAdded,
            }),
        )
        .await
        .unwrap()
        .into_response();
        let status = response.status();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let next: Collection = serde_json::from_slice(&body).unwrap();

        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(next.version, 2);
        assert_eq!(next.artifacts, vec![artifact_v1.uuid, artifact_v2.uuid]);

        let Json(version_page): Json<Page<Collection>> = list_collection_versions(
            Path(created.uuid),
            Query(PaginationParams {
                limit: 10,
                offset: 0,
            }),
            State(state.clone()),
        )
        .await
        .unwrap();
        assert_eq!(version_page.items.len(), 2);

        let Json(version_one) =
            get_collection_version(Path((created.uuid, 1)), State(state.clone()))
                .await
                .unwrap();
        assert_eq!(version_one.artifacts, vec![artifact_v1.uuid]);

        let Json(diff) = compare_collection_versions(
            Path(created.uuid),
            Query(CompareCollectionVersionsParams {
                base_version: 1,
                target_version: 2,
            }),
            State(state),
        )
        .await
        .unwrap();
        assert_eq!(diff.added_artifact_uuids, vec![artifact_v2.uuid]);
        assert!(diff.removed_artifact_uuids.is_empty());
    }
}
