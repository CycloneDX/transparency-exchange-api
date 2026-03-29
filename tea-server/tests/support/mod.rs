#![allow(dead_code)]

use axum::{routing::get, Router};
use std::sync::{Arc, OnceLock};

use chrono::Utc;
use jsonwebtoken::{encode, EncodingKey, Header};
use tokio::net::TcpListener;
use tokio::sync::{oneshot, Mutex};
use tokio_stream::wrappers::TcpListenerStream;
use tonic::{transport::Server, Request};
use uuid::Uuid;

use tea_server::application::publisher::service::PublisherApplicationServiceImpl;
use tea_server::domain::artifact::entity::{Artifact, ArtifactFormat, ArtifactType};
use tea_server::domain::artifact::repository::ArtifactRepository;
use tea_server::domain::artifact::service::ArtifactService;
use tea_server::domain::collection::service::CollectionService;
use tea_server::domain::component::service::ComponentService;
use tea_server::domain::product::entity::{Product, Vendor};
use tea_server::domain::product::repository::ProductRepository;
use tea_server::domain::product::service::ProductService;
use tea_server::gen::tea::v1::{
    consumer_service_server::ConsumerServiceServer,
    discovery_service_server::DiscoveryServiceServer,
    publisher_service_server::PublisherServiceServer,
};
use tea_server::infrastructure::auth::jwt::{AudienceClaim, Claims};
use tea_server::infrastructure::grpc::{
    publisher_auth_interceptor, ConsumerGrpcService, DiscoveryGrpcService, PublisherGrpcService,
};
use tea_server::infrastructure::persistence::memory::artifact_repository::InMemoryArtifactRepository;
use tea_server::infrastructure::persistence::memory::collection_repository::InMemoryCollectionRepository;
use tea_server::infrastructure::persistence::memory::component_repository::InMemoryComponentRepository;
use tea_server::infrastructure::persistence::memory::product_repository::InMemoryProductRepository;

static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

pub fn env_lock() -> &'static Mutex<()> {
    ENV_LOCK.get_or_init(|| Mutex::new(()))
}

pub struct ServerHandle {
    pub endpoint: String,
    shutdown: oneshot::Sender<()>,
    task: tokio::task::JoinHandle<()>,
    pub product_uuid: Uuid,
    pub artifact_uuid: Uuid,
    pub artifact_uuid_2: Uuid,
}

pub async fn spawn_grpc_server(publisher_enabled: bool) -> ServerHandle {
    let product_repository = Arc::new(InMemoryProductRepository::new());
    let component_repository = Arc::new(InMemoryComponentRepository::new());
    let collection_repository = Arc::new(InMemoryCollectionRepository::new());
    let artifact_repository = Arc::new(InMemoryArtifactRepository::new());
    let product_uuid = Uuid::new_v4();
    let artifact_uuid = Uuid::new_v4();
    let artifact_uuid_2 = Uuid::new_v4();

    product_repository
        .save(&Product {
            uuid: product_uuid,
            name: "Transport Widget".to_string(),
            description: Some("gRPC smoke-test product".to_string()),
            identifiers: vec![],
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
        })
        .await
        .unwrap();

    for (uuid, name) in [
        (artifact_uuid, "transport-widget.sbom.json"),
        (artifact_uuid_2, "transport-widget.vex.json"),
    ] {
        artifact_repository
            .save(&Artifact {
                uuid,
                name: name.to_string(),
                type_: ArtifactType::Bom,
                component_distributions: vec![],
                formats: vec![ArtifactFormat {
                    mime_type: "application/json".to_string(),
                    description: None,
                    url: format!("https://example.com/{name}"),
                    signature_url: None,
                    checksums: vec![],
                    size_bytes: None,
                    encoding: None,
                    spec_version: Some("1.6".to_string()),
                }],
                created_date: Utc::now(),
                modified_date: Utc::now(),
                description: Some("seed artifact".to_string()),
                subject: None,
                deprecation: None,
                dependencies: vec![],
            })
            .await
            .unwrap();
    }

    let consumer = ConsumerGrpcService::new(
        product_repository.clone(),
        component_repository.clone(),
        collection_repository.clone(),
        artifact_repository.clone(),
    );
    let discovery = DiscoveryGrpcService::new("https://tea.example.com");
    let publisher = PublisherGrpcService::new(Arc::new(PublisherApplicationServiceImpl::new(
        ProductService::new(product_repository),
        ComponentService::new(component_repository),
        CollectionService::new(collection_repository),
        ArtifactService::new(artifact_repository),
    )));

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let endpoint = format!("http://{}", listener.local_addr().unwrap());
    let incoming = TcpListenerStream::new(listener);
    let (shutdown, shutdown_rx) = oneshot::channel();

    let task = tokio::spawn(async move {
        let mut builder = Server::builder().accept_http1(true);
        let discovery = tonic_web::enable(DiscoveryServiceServer::new(discovery));
        let consumer = tonic_web::enable(ConsumerServiceServer::new(consumer));

        let result = if publisher_enabled {
            let publisher = tonic_web::enable(PublisherServiceServer::with_interceptor(
                publisher,
                publisher_auth_interceptor,
            ));
            builder
                .add_service(discovery)
                .add_service(consumer)
                .add_service(publisher)
                .serve_with_incoming_shutdown(incoming, async {
                    let _ = shutdown_rx.await;
                })
                .await
        } else {
            builder
                .add_service(discovery)
                .add_service(consumer)
                .serve_with_incoming_shutdown(incoming, async {
                    let _ = shutdown_rx.await;
                })
                .await
        };

        result.unwrap();
    });

    ServerHandle {
        endpoint,
        shutdown,
        task,
        product_uuid,
        artifact_uuid,
        artifact_uuid_2,
    }
}

pub async fn stop_server(handle: ServerHandle) {
    let _ = handle.shutdown.send(());
    handle.task.await.unwrap();
}

pub struct SourceServerHandle {
    pub base_url: String,
    shutdown: oneshot::Sender<()>,
    task: tokio::task::JoinHandle<()>,
}

pub async fn spawn_source_server(body: Vec<u8>, content_type: &'static str) -> SourceServerHandle {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base_url = format!("http://{}", listener.local_addr().unwrap());
    let (shutdown, shutdown_rx) = oneshot::channel();

    let app = Router::new().route(
        "/artifact",
        get(move || {
            let body = body.clone();
            async move { ([(axum::http::header::CONTENT_TYPE, content_type)], body) }
        }),
    );

    let task = tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(async {
                let _ = shutdown_rx.await;
            })
            .await
            .unwrap();
    });

    SourceServerHandle {
        base_url,
        shutdown,
        task,
    }
}

pub async fn stop_source_server(handle: SourceServerHandle) {
    let _ = handle.shutdown.send(());
    handle.task.await.unwrap();
}

pub fn publisher_token() -> String {
    encode(
        &Header::default(),
        &Claims {
            sub: "publisher".to_string(),
            exp: usize::MAX,
            iss: Some("issuer".to_string()),
            aud: Some(AudienceClaim::Single("tea-api".to_string())),
            scope: Some("tea:write".to_string()),
            permissions: vec![],
            role: None,
        },
        &EncodingKey::from_secret(b"dev-only-insecure-secret-32-bytes--"),
    )
    .unwrap()
}

pub const PUBLISHER_ENV_KEYS: [&str; 6] = [
    "TEA_JWT_SECRET",
    "TEA_JWT_AUDIENCE",
    "TEA_JWT_WRITE_SCOPE",
    "TEA_JWT_WRITE_ROLE",
    "TEA_JWT_ISSUER",
    "TEA_ALLOW_PRIVATE_SOURCE_URLS",
];

pub fn save_publisher_env() -> [(&'static str, Option<String>); 6] {
    PUBLISHER_ENV_KEYS.map(|key| (key, std::env::var(key).ok()))
}

pub fn configure_publisher_env(allow_private_source_urls: Option<&str>) {
    std::env::set_var("TEA_JWT_SECRET", "dev-only-insecure-secret-32-bytes--");
    std::env::set_var("TEA_JWT_AUDIENCE", "tea-api");
    std::env::set_var("TEA_JWT_WRITE_SCOPE", "tea:write");
    std::env::set_var("TEA_JWT_WRITE_ROLE", "tea-writer");
    std::env::set_var("TEA_JWT_ISSUER", "issuer");
    match allow_private_source_urls {
        Some(value) => std::env::set_var("TEA_ALLOW_PRIVATE_SOURCE_URLS", value),
        None => std::env::remove_var("TEA_ALLOW_PRIVATE_SOURCE_URLS"),
    }
}

pub fn restore_publisher_env(saved: [(&'static str, Option<String>); 6]) {
    for (key, value) in saved {
        match value {
            Some(value) => std::env::set_var(key, value),
            None => std::env::remove_var(key),
        }
    }
}

pub fn authorize<T>(request: &mut Request<T>) {
    let metadata = format!("Bearer {}", publisher_token()).parse().unwrap();
    request.metadata_mut().insert("authorization", metadata);
}
