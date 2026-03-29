mod support;

use prost_types::Timestamp;
use tonic::Request;
use uuid::Uuid;

use support::{
    authorize, configure_publisher_env, env_lock, restore_publisher_env, save_publisher_env,
    spawn_grpc_server, stop_server,
};
use tea_server::gen::tea::v1::{
    self as proto, consumer_service_client::ConsumerServiceClient,
    publisher_service_client::PublisherServiceClient,
};

#[tokio::test]
async fn publisher_grpc_can_update_components() {
    let _guard = env_lock().lock().await;
    let saved = save_publisher_env();
    configure_publisher_env(None);

    let handle = spawn_grpc_server(true).await;
    let mut publisher = PublisherServiceClient::connect(handle.endpoint.clone())
        .await
        .unwrap();

    let component_uuid = Uuid::new_v4().to_string();
    let mut create_component = Request::new(proto::CreateComponentRequest {
        name: "component before update".to_string(),
        description: "old description".to_string(),
        identifiers: vec![],
        component_type: proto::ComponentType::Library as i32,
        licenses: vec![],
        publisher: "ACME".to_string(),
        homepage_url: "https://example.com/old".to_string(),
        vcs_url: String::new(),
        uuid: Some(component_uuid.clone()),
    });
    authorize(&mut create_component);
    publisher.create_component(create_component).await.unwrap();

    let mut update_component = Request::new(proto::UpdateComponentRequest {
        uuid: component_uuid,
        update_mask: Some(prost_types::FieldMask {
            paths: vec!["description".to_string(), "homepage_url".to_string()],
        }),
        name: String::new(),
        description: "new description".to_string(),
        identifiers: vec![],
        component_type: proto::ComponentType::Unspecified as i32,
        licenses: vec![],
        publisher: String::new(),
        homepage_url: "https://example.com/new".to_string(),
        vcs_url: String::new(),
    });
    authorize(&mut update_component);
    let updated = publisher
        .update_component(update_component)
        .await
        .unwrap()
        .into_inner();

    assert_eq!(updated.description, "new description");
    assert_eq!(updated.homepage_url, "https://example.com/new");
    assert_eq!(updated.name, "component before update");

    stop_server(handle).await;
    restore_publisher_env(saved);
}

#[tokio::test]
async fn publisher_grpc_can_delete_product_releases_directly() {
    let _guard = env_lock().lock().await;
    let saved = save_publisher_env();
    configure_publisher_env(None);

    let handle = spawn_grpc_server(true).await;
    let mut publisher = PublisherServiceClient::connect(handle.endpoint.clone())
        .await
        .unwrap();
    let mut consumer = ConsumerServiceClient::connect(handle.endpoint.clone())
        .await
        .unwrap();

    let release_uuid = Uuid::new_v4().to_string();
    let mut create_release = Request::new(proto::CreateProductReleaseRequest {
        product_uuid: Some(handle.product_uuid.to_string()),
        version: "2026.08".to_string(),
        release_date: Some(Timestamp {
            seconds: 1_700_800_000,
            nanos: 0,
        }),
        pre_release: false,
        identifiers: vec![],
        components: vec![],
        uuid: Some(release_uuid.clone()),
    });
    authorize(&mut create_release);
    publisher
        .create_product_release(create_release)
        .await
        .unwrap();

    let mut delete_release = Request::new(proto::DeleteProductReleaseRequest {
        uuid: release_uuid.clone(),
    });
    authorize(&mut delete_release);
    let deleted = publisher
        .delete_product_release(delete_release)
        .await
        .unwrap()
        .into_inner();
    assert_eq!(deleted.uuid, release_uuid);

    let missing = consumer
        .get_product_release(Request::new(proto::GetProductReleaseRequest {
            uuid: release_uuid,
        }))
        .await
        .unwrap_err();
    assert_eq!(missing.code(), tonic::Code::NotFound);

    stop_server(handle).await;
    restore_publisher_env(saved);
}

#[tokio::test]
async fn publisher_grpc_can_delete_component_releases_directly() {
    let _guard = env_lock().lock().await;
    let saved = save_publisher_env();
    configure_publisher_env(None);

    let handle = spawn_grpc_server(true).await;
    let mut publisher = PublisherServiceClient::connect(handle.endpoint.clone())
        .await
        .unwrap();
    let mut consumer = ConsumerServiceClient::connect(handle.endpoint.clone())
        .await
        .unwrap();

    let component_uuid = Uuid::new_v4().to_string();
    let mut create_component = Request::new(proto::CreateComponentRequest {
        name: "delete-release component".to_string(),
        description: String::new(),
        identifiers: vec![],
        component_type: proto::ComponentType::Library as i32,
        licenses: vec![],
        publisher: "ACME".to_string(),
        homepage_url: "https://example.com/component".to_string(),
        vcs_url: String::new(),
        uuid: Some(component_uuid.clone()),
    });
    authorize(&mut create_component);
    publisher.create_component(create_component).await.unwrap();

    let release_uuid = Uuid::new_v4().to_string();
    let mut create_release = Request::new(proto::CreateComponentReleaseRequest {
        component_uuid,
        version: "1.2.3".to_string(),
        release_date: Some(Timestamp {
            seconds: 1_700_900_000,
            nanos: 0,
        }),
        pre_release: false,
        identifiers: vec![],
        distributions: vec![],
        uuid: Some(release_uuid.clone()),
    });
    authorize(&mut create_release);
    publisher
        .create_component_release(create_release)
        .await
        .unwrap();

    let mut delete_release = Request::new(proto::DeleteComponentReleaseRequest {
        uuid: release_uuid.clone(),
    });
    authorize(&mut delete_release);
    let deleted = publisher
        .delete_component_release(delete_release)
        .await
        .unwrap()
        .into_inner();
    assert_eq!(deleted.uuid, release_uuid);

    let missing = consumer
        .get_component_release(Request::new(proto::GetComponentReleaseRequest {
            uuid: release_uuid,
        }))
        .await
        .unwrap_err();
    assert_eq!(missing.code(), tonic::Code::NotFound);

    stop_server(handle).await;
    restore_publisher_env(saved);
}

#[tokio::test]
async fn publisher_grpc_upload_artifact_fails_closed() {
    let _guard = env_lock().lock().await;
    let saved = save_publisher_env();
    configure_publisher_env(None);

    let handle = spawn_grpc_server(true).await;
    let mut publisher = PublisherServiceClient::connect(handle.endpoint.clone())
        .await
        .unwrap();

    let stream = tokio_stream::iter(vec![proto::UploadArtifactRequest {
        data: Some(proto::upload_artifact_request::Data::Metadata(
            proto::ArtifactMetadata {
                name: "artifact-upload.cdx.json".to_string(),
                r#type: proto::ArtifactType::Bom as i32,
                mime_type: "application/vnd.cyclonedx+json".to_string(),
                description: String::new(),
                component_distributions: vec![],
                subject: None,
                spec_version: "1.6".to_string(),
                uuid: Some(Uuid::new_v4().to_string()),
                expected_checksums: vec![],
            },
        )),
    }]);
    let mut request = Request::new(stream);
    authorize(&mut request);
    let error = publisher.upload_artifact(request).await.unwrap_err();
    assert_eq!(error.code(), tonic::Code::Unimplemented);

    stop_server(handle).await;
    restore_publisher_env(saved);
}

#[tokio::test]
async fn publisher_grpc_batch_upload_artifacts_fails_closed() {
    let _guard = env_lock().lock().await;
    let saved = save_publisher_env();
    configure_publisher_env(None);

    let handle = spawn_grpc_server(true).await;
    let mut publisher = PublisherServiceClient::connect(handle.endpoint.clone())
        .await
        .unwrap();

    let stream = tokio_stream::iter(vec![proto::BatchUploadArtifactsRequest {
        data: Some(proto::batch_upload_artifacts_request::Data::BatchMetadata(
            proto::BatchArtifactMetadata {
                total_count: 1,
                collection_uuid: None,
            },
        )),
    }]);
    let mut request = Request::new(stream);
    authorize(&mut request);
    let error = publisher.batch_upload_artifacts(request).await.unwrap_err();
    assert_eq!(error.code(), tonic::Code::Unimplemented);

    stop_server(handle).await;
    restore_publisher_env(saved);
}

#[tokio::test]
async fn publisher_grpc_import_collection_fails_closed() {
    let _guard = env_lock().lock().await;
    let saved = save_publisher_env();
    configure_publisher_env(None);

    let handle = spawn_grpc_server(true).await;
    let mut publisher = PublisherServiceClient::connect(handle.endpoint.clone())
        .await
        .unwrap();

    let stream = tokio_stream::iter(vec![proto::ImportCollectionRequest {
        data: Some(proto::import_collection_request::Data::ImportMetadata(
            proto::ImportMetadata {
                source_system: "test-suite".to_string(),
                total_collections: 1,
                total_artifacts: 0,
                overwrite: false,
            },
        )),
    }]);
    let mut request = Request::new(stream);
    authorize(&mut request);
    let error = publisher.import_collection(request).await.unwrap_err();
    assert_eq!(error.code(), tonic::Code::Unimplemented);

    stop_server(handle).await;
    restore_publisher_env(saved);
}
