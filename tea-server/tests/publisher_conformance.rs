mod support;

use prost_types::{FieldMask, Timestamp};
use tonic::Request;
use uuid::Uuid;

use support::{
    authorize, configure_publisher_env, env_lock, restore_publisher_env, save_publisher_env,
    spawn_grpc_server, spawn_source_server, stop_server, stop_source_server,
};
use tea_server::gen::tea::v1::{self as proto, publisher_service_client::PublisherServiceClient};

#[tokio::test]
async fn publisher_grpc_requires_auth_before_returning_unimplemented() {
    let _guard = env_lock().lock().await;
    let saved = save_publisher_env();
    configure_publisher_env(None);

    let handle = spawn_grpc_server(true).await;
    let mut client = PublisherServiceClient::connect(handle.endpoint.clone())
        .await
        .unwrap();

    let unauthenticated = client
        .create_product(Request::new(proto::CreateProductRequest {
            name: "publisher product".to_string(),
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
            uuid: None,
        }))
        .await
        .unwrap_err();
    assert_eq!(unauthenticated.code(), tonic::Code::Unauthenticated);

    let mut authenticated_request = Request::new(proto::CreateProductRequest {
        name: "publisher product".to_string(),
        description: "created over gRPC".to_string(),
        identifiers: vec![],
        vendor: Some(proto::Vendor {
            name: "ACME".to_string(),
            uuid: None,
            url: String::new(),
            contacts: vec![],
        }),
        homepage_url: "https://example.com".to_string(),
        documentation_url: String::new(),
        vcs_url: String::new(),
        uuid: None,
    });
    authorize(&mut authenticated_request);
    let created = client
        .create_product(authenticated_request)
        .await
        .unwrap()
        .into_inner();
    assert_eq!(created.name, "publisher product");
    assert_eq!(created.vendor.as_ref().unwrap().name, "ACME");

    let mut update = Request::new(proto::UpdateProductRequest {
        uuid: created.uuid.clone(),
        update_mask: Some(FieldMask {
            paths: vec!["description".to_string(), "homepage_url".to_string()],
        }),
        name: String::new(),
        description: "updated over gRPC".to_string(),
        identifiers: vec![],
        vendor: None,
        homepage_url: "https://example.com/docs".to_string(),
        documentation_url: String::new(),
        vcs_url: String::new(),
    });
    authorize(&mut update);
    let updated = client.update_product(update).await.unwrap().into_inner();
    assert_eq!(updated.description, "updated over gRPC");
    assert_eq!(updated.homepage_url, "https://example.com/docs");

    let mut delete = Request::new(proto::DeleteProductRequest {
        uuid: created.uuid,
        cascade: false,
    });
    authorize(&mut delete);
    let deleted = client.delete_product(delete).await.unwrap().into_inner();
    assert_eq!(deleted.releases_deleted, 0);

    let mut unsupported = Request::new(proto::SignCollectionRequest {
        uuid: Uuid::new_v4().to_string(),
        version: None,
        key_id: String::new(),
        use_sigstore: false,
    });
    authorize(&mut unsupported);
    let unimplemented = client.sign_collection(unsupported).await.unwrap_err();
    assert_eq!(unimplemented.code(), tonic::Code::Unimplemented);

    stop_server(handle).await;
    restore_publisher_env(saved);
}

#[tokio::test]
async fn publisher_grpc_reports_explicit_conformance_failures() {
    let _guard = env_lock().lock().await;
    let saved = save_publisher_env();
    configure_publisher_env(Some("true"));

    let handle = spawn_grpc_server(true).await;
    let source_body = br#"{"bomFormat":"CycloneDX","specVersion":"1.6"}"#.to_vec();
    let source = spawn_source_server(source_body, "application/vnd.cyclonedx+json").await;
    let mut publisher = PublisherServiceClient::connect(handle.endpoint.clone())
        .await
        .unwrap();

    let mut invalid_update = Request::new(proto::UpdateProductRequest {
        uuid: handle.product_uuid.to_string(),
        update_mask: Some(FieldMask {
            paths: vec!["unsupported_field".to_string()],
        }),
        name: String::new(),
        description: String::new(),
        identifiers: vec![],
        vendor: None,
        homepage_url: String::new(),
        documentation_url: String::new(),
        vcs_url: String::new(),
    });
    authorize(&mut invalid_update);
    let invalid_update = publisher.update_product(invalid_update).await.unwrap_err();
    assert_eq!(invalid_update.code(), tonic::Code::InvalidArgument);

    let mut bad_checksum = Request::new(proto::CreateArtifactFromUrlRequest {
        metadata: Some(proto::ArtifactMetadata {
            name: "bad-checksum.cdx.json".to_string(),
            r#type: proto::ArtifactType::Bom as i32,
            mime_type: "application/vnd.cyclonedx+json".to_string(),
            description: String::new(),
            component_distributions: vec![],
            subject: Some(proto::ArtifactSubject {
                r#type: proto::SubjectType::Product as i32,
                identifiers: vec![],
                name: "Transport Widget".to_string(),
                version: "2026.05".to_string(),
            }),
            spec_version: "1.6".to_string(),
            uuid: Some(Uuid::new_v4().to_string()),
            expected_checksums: vec![proto::Checksum {
                alg_type: proto::ChecksumAlgorithm::Sha256 as i32,
                alg_value: "deadbeef".to_string(),
            }],
        }),
        source_url: format!("{}/artifact", source.base_url),
        expected_checksums: vec![],
        signature_url: String::new(),
    });
    authorize(&mut bad_checksum);
    let bad_checksum = publisher
        .create_artifact_from_url(bad_checksum)
        .await
        .unwrap_err();
    assert_eq!(bad_checksum.code(), tonic::Code::FailedPrecondition);

    let release_uuid = Uuid::new_v4().to_string();
    let mut create_release = Request::new(proto::CreateProductReleaseRequest {
        product_uuid: Some(handle.product_uuid.to_string()),
        version: "2026.05".to_string(),
        release_date: Some(Timestamp {
            seconds: 1_700_400_000,
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

    let mut create_collection = Request::new(proto::CreateCollectionRequest {
        uuid: release_uuid.clone(),
        belongs_to: proto::CollectionScope::ProductRelease as i32,
        artifact_uuids: vec![handle.artifact_uuid.to_string()],
        update_reason: Some(proto::UpdateReason {
            r#type: proto::UpdateReasonType::InitialRelease as i32,
            comment: String::new(),
            affected_artifact_uuids: vec![],
        }),
    });
    authorize(&mut create_collection);
    publisher
        .create_collection(create_collection)
        .await
        .unwrap();

    let mut delete_referenced = Request::new(proto::DeleteArtifactRequest {
        uuid: handle.artifact_uuid.to_string(),
        force: false,
    });
    authorize(&mut delete_referenced);
    let delete_referenced = publisher
        .delete_artifact(delete_referenced)
        .await
        .unwrap_err();
    assert_eq!(delete_referenced.code(), tonic::Code::FailedPrecondition);

    let mut force_delete = Request::new(proto::DeleteArtifactRequest {
        uuid: handle.artifact_uuid.to_string(),
        force: true,
    });
    authorize(&mut force_delete);
    let force_delete = publisher.delete_artifact(force_delete).await.unwrap_err();
    assert_eq!(force_delete.code(), tonic::Code::Unimplemented);

    let mut missing_collection = Request::new(proto::UpdateCollectionRequest {
        uuid: Uuid::new_v4().to_string(),
        artifact_uuids: vec![handle.artifact_uuid_2.to_string()],
        update_reason: Some(proto::UpdateReason {
            r#type: proto::UpdateReasonType::ArtifactAdded as i32,
            comment: String::new(),
            affected_artifact_uuids: vec![handle.artifact_uuid_2.to_string()],
        }),
    });
    authorize(&mut missing_collection);
    let missing_collection = publisher
        .update_collection(missing_collection)
        .await
        .unwrap_err();
    assert_eq!(missing_collection.code(), tonic::Code::NotFound);

    stop_source_server(source).await;
    stop_server(handle).await;
    restore_publisher_env(saved);
}

#[tokio::test]
async fn publisher_grpc_requires_parent_product_for_product_releases() {
    let _guard = env_lock().lock().await;
    let saved = save_publisher_env();
    configure_publisher_env(None);

    let handle = spawn_grpc_server(true).await;
    let mut publisher = PublisherServiceClient::connect(handle.endpoint.clone())
        .await
        .unwrap();

    let mut create_release = Request::new(proto::CreateProductReleaseRequest {
        product_uuid: None,
        version: "2026.06".to_string(),
        release_date: Some(Timestamp {
            seconds: 1_700_500_000,
            nanos: 0,
        }),
        pre_release: false,
        identifiers: vec![],
        components: vec![],
        uuid: Some(Uuid::new_v4().to_string()),
    });
    authorize(&mut create_release);
    let error = publisher
        .create_product_release(create_release)
        .await
        .unwrap_err();
    assert_eq!(error.code(), tonic::Code::InvalidArgument);

    stop_server(handle).await;
    restore_publisher_env(saved);
}

#[tokio::test]
async fn publisher_grpc_blocks_private_source_urls_when_not_allowed() {
    let _guard = env_lock().lock().await;
    let saved = save_publisher_env();
    configure_publisher_env(Some("false"));

    let handle = spawn_grpc_server(true).await;
    let source_body = br#"{"bomFormat":"CycloneDX","specVersion":"1.6"}"#.to_vec();
    let source = spawn_source_server(source_body, "application/vnd.cyclonedx+json").await;
    let mut publisher = PublisherServiceClient::connect(handle.endpoint.clone())
        .await
        .unwrap();

    let mut request = Request::new(proto::CreateArtifactFromUrlRequest {
        metadata: Some(proto::ArtifactMetadata {
            name: "private-url.cdx.json".to_string(),
            r#type: proto::ArtifactType::Bom as i32,
            mime_type: "application/vnd.cyclonedx+json".to_string(),
            description: String::new(),
            component_distributions: vec![],
            subject: Some(proto::ArtifactSubject {
                r#type: proto::SubjectType::Product as i32,
                identifiers: vec![],
                name: "Transport Widget".to_string(),
                version: "2026.06".to_string(),
            }),
            spec_version: "1.6".to_string(),
            uuid: Some(Uuid::new_v4().to_string()),
            expected_checksums: vec![],
        }),
        source_url: format!("{}/artifact", source.base_url),
        expected_checksums: vec![],
        signature_url: String::new(),
    });
    authorize(&mut request);
    let error = publisher
        .create_artifact_from_url(request)
        .await
        .unwrap_err();
    assert!(matches!(
        error.code(),
        tonic::Code::InvalidArgument | tonic::Code::PermissionDenied
    ));

    stop_source_server(source).await;
    stop_server(handle).await;
    restore_publisher_env(saved);
}

#[tokio::test]
async fn publisher_grpc_rejects_component_release_pre_release_promotion() {
    let _guard = env_lock().lock().await;
    let saved = save_publisher_env();
    configure_publisher_env(None);

    let handle = spawn_grpc_server(true).await;
    let mut publisher = PublisherServiceClient::connect(handle.endpoint.clone())
        .await
        .unwrap();

    let component_uuid = Uuid::new_v4().to_string();
    let mut create_component = Request::new(proto::CreateComponentRequest {
        name: "conformance component".to_string(),
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
        version: "1.0.0".to_string(),
        release_date: Some(Timestamp {
            seconds: 1_700_600_000,
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

    let mut promote = Request::new(proto::UpdateComponentReleaseRequest {
        uuid: release_uuid,
        update_mask: Some(FieldMask {
            paths: vec!["pre_release".to_string()],
        }),
        version: String::new(),
        release_date: None,
        pre_release: true,
        identifiers: vec![],
        distributions: vec![],
    });
    authorize(&mut promote);
    let error = publisher
        .update_component_release(promote)
        .await
        .unwrap_err();
    assert_eq!(error.code(), tonic::Code::InvalidArgument);

    stop_server(handle).await;
    restore_publisher_env(saved);
}

#[tokio::test]
async fn publisher_grpc_rejects_collections_with_unknown_artifacts() {
    let _guard = env_lock().lock().await;
    let saved = save_publisher_env();
    configure_publisher_env(None);

    let handle = spawn_grpc_server(true).await;
    let mut publisher = PublisherServiceClient::connect(handle.endpoint.clone())
        .await
        .unwrap();

    let release_uuid = Uuid::new_v4().to_string();
    let mut create_release = Request::new(proto::CreateProductReleaseRequest {
        product_uuid: Some(handle.product_uuid.to_string()),
        version: "2026.07".to_string(),
        release_date: Some(Timestamp {
            seconds: 1_700_700_000,
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

    let mut create_collection = Request::new(proto::CreateCollectionRequest {
        uuid: release_uuid,
        belongs_to: proto::CollectionScope::ProductRelease as i32,
        artifact_uuids: vec![Uuid::new_v4().to_string()],
        update_reason: Some(proto::UpdateReason {
            r#type: proto::UpdateReasonType::InitialRelease as i32,
            comment: String::new(),
            affected_artifact_uuids: vec![],
        }),
    });
    authorize(&mut create_collection);
    let error = publisher
        .create_collection(create_collection)
        .await
        .unwrap_err();
    assert_eq!(error.code(), tonic::Code::NotFound);

    stop_server(handle).await;
    restore_publisher_env(saved);
}
