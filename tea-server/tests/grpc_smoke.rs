mod support;

use prost_types::{FieldMask, Timestamp};
use sha2::{Digest, Sha256};
use tonic::Request;
use uuid::Uuid;

use support::{
    env_lock, publisher_token, spawn_grpc_server, spawn_source_server, stop_server,
    stop_source_server,
};
use tea_server::gen::tea::v1::{
    self as proto, consumer_service_client::ConsumerServiceClient,
    discovery_service_client::DiscoveryServiceClient,
    publisher_service_client::PublisherServiceClient,
};

#[tokio::test]
async fn discovery_and_consumer_grpc_round_trip() {
    let handle = spawn_grpc_server(false).await;

    let mut discovery = DiscoveryServiceClient::connect(handle.endpoint.clone())
        .await
        .unwrap();
    let well_known = discovery
        .get_well_known(Request::new(proto::GetWellKnownRequest {}))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(well_known.schema_version, 1);
    assert_eq!(well_known.endpoints[0].url, "https://tea.example.com/v1");

    let discovered = discovery
        .discover(Request::new(proto::DiscoverRequest {
            tei: format!("urn:tei:uuid:tea.example.com:{}", handle.product_uuid),
        }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(
        discovered.product_release_uuid,
        handle.product_uuid.to_string()
    );

    let mut consumer = ConsumerServiceClient::connect(handle.endpoint.clone())
        .await
        .unwrap();
    let product = consumer
        .get_product(Request::new(proto::GetProductRequest {
            uuid: handle.product_uuid.to_string(),
        }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(product.name, "Transport Widget");

    stop_server(handle).await;
}

#[tokio::test]
async fn publisher_grpc_can_manage_component_releases_and_cascade_delete() {
    let _guard = env_lock().lock().await;
    let saved = [
        ("TEA_JWT_SECRET", std::env::var("TEA_JWT_SECRET").ok()),
        ("TEA_JWT_AUDIENCE", std::env::var("TEA_JWT_AUDIENCE").ok()),
        (
            "TEA_JWT_WRITE_SCOPE",
            std::env::var("TEA_JWT_WRITE_SCOPE").ok(),
        ),
        (
            "TEA_JWT_WRITE_ROLE",
            std::env::var("TEA_JWT_WRITE_ROLE").ok(),
        ),
        ("TEA_JWT_ISSUER", std::env::var("TEA_JWT_ISSUER").ok()),
    ];

    std::env::set_var("TEA_JWT_SECRET", "dev-only-insecure-secret-32-bytes--");
    std::env::set_var("TEA_JWT_AUDIENCE", "tea-api");
    std::env::set_var("TEA_JWT_WRITE_SCOPE", "tea:write");
    std::env::set_var("TEA_JWT_WRITE_ROLE", "tea-writer");
    std::env::set_var("TEA_JWT_ISSUER", "issuer");

    let handle = spawn_grpc_server(true).await;
    let mut client = PublisherServiceClient::connect(handle.endpoint.clone())
        .await
        .unwrap();

    let component_uuid = Uuid::new_v4().to_string();
    let mut create_component = Request::new(proto::CreateComponentRequest {
        name: "gRPC component".to_string(),
        description: "component description".to_string(),
        identifiers: vec![],
        component_type: proto::ComponentType::Library as i32,
        licenses: vec![proto::LicenseInfo {
            spdx_id: "Apache-2.0".to_string(),
            name: String::new(),
            url: String::new(),
        }],
        publisher: "ACME".to_string(),
        homepage_url: "https://example.com/component".to_string(),
        vcs_url: String::new(),
        uuid: Some(component_uuid.clone()),
    });
    let metadata = format!("Bearer {}", publisher_token()).parse().unwrap();
    create_component
        .metadata_mut()
        .insert("authorization", metadata);
    let component = client
        .create_component(create_component)
        .await
        .unwrap()
        .into_inner();
    assert_eq!(component.uuid, component_uuid);

    let release_uuid = Uuid::new_v4().to_string();
    let mut create_release = Request::new(proto::CreateComponentReleaseRequest {
        component_uuid: component.uuid.clone(),
        version: "1.0.0-rc.1".to_string(),
        release_date: Some(Timestamp {
            seconds: 1_700_000_000,
            nanos: 0,
        }),
        pre_release: true,
        identifiers: vec![],
        distributions: vec![],
        uuid: Some(release_uuid.clone()),
    });
    let metadata = format!("Bearer {}", publisher_token()).parse().unwrap();
    create_release
        .metadata_mut()
        .insert("authorization", metadata);
    let release = client
        .create_component_release(create_release)
        .await
        .unwrap()
        .into_inner();
    assert_eq!(release.uuid, release_uuid);
    assert!(release.pre_release);

    let mut update_release = Request::new(proto::UpdateComponentReleaseRequest {
        uuid: release.uuid.clone(),
        update_mask: Some(FieldMask {
            paths: vec!["pre_release".to_string(), "version".to_string()],
        }),
        version: "1.0.0".to_string(),
        release_date: None,
        pre_release: false,
        identifiers: vec![],
        distributions: vec![],
    });
    let metadata = format!("Bearer {}", publisher_token()).parse().unwrap();
    update_release
        .metadata_mut()
        .insert("authorization", metadata);
    let updated_release = client
        .update_component_release(update_release)
        .await
        .unwrap()
        .into_inner();
    assert_eq!(updated_release.version, "1.0.0");
    assert!(!updated_release.pre_release);

    let mut delete_without_cascade = Request::new(proto::DeleteComponentRequest {
        uuid: component.uuid.clone(),
        cascade: false,
    });
    let metadata = format!("Bearer {}", publisher_token()).parse().unwrap();
    delete_without_cascade
        .metadata_mut()
        .insert("authorization", metadata);
    let blocked = client
        .delete_component(delete_without_cascade)
        .await
        .unwrap_err();
    assert_eq!(blocked.code(), tonic::Code::FailedPrecondition);

    let mut delete_with_cascade = Request::new(proto::DeleteComponentRequest {
        uuid: component.uuid,
        cascade: true,
    });
    let metadata = format!("Bearer {}", publisher_token()).parse().unwrap();
    delete_with_cascade
        .metadata_mut()
        .insert("authorization", metadata);
    let deleted = client
        .delete_component(delete_with_cascade)
        .await
        .unwrap()
        .into_inner();
    assert_eq!(deleted.releases_deleted, 1);

    stop_server(handle).await;

    for (key, value) in saved {
        match value {
            Some(value) => std::env::set_var(key, value),
            None => std::env::remove_var(key),
        }
    }
}

#[tokio::test]
async fn publisher_grpc_can_manage_product_releases_and_product_cascade_delete() {
    let _guard = env_lock().lock().await;
    let saved = [
        ("TEA_JWT_SECRET", std::env::var("TEA_JWT_SECRET").ok()),
        ("TEA_JWT_AUDIENCE", std::env::var("TEA_JWT_AUDIENCE").ok()),
        (
            "TEA_JWT_WRITE_SCOPE",
            std::env::var("TEA_JWT_WRITE_SCOPE").ok(),
        ),
        (
            "TEA_JWT_WRITE_ROLE",
            std::env::var("TEA_JWT_WRITE_ROLE").ok(),
        ),
        ("TEA_JWT_ISSUER", std::env::var("TEA_JWT_ISSUER").ok()),
    ];

    std::env::set_var("TEA_JWT_SECRET", "dev-only-insecure-secret-32-bytes--");
    std::env::set_var("TEA_JWT_AUDIENCE", "tea-api");
    std::env::set_var("TEA_JWT_WRITE_SCOPE", "tea:write");
    std::env::set_var("TEA_JWT_WRITE_ROLE", "tea-writer");
    std::env::set_var("TEA_JWT_ISSUER", "issuer");

    let handle = spawn_grpc_server(true).await;
    let mut publisher = PublisherServiceClient::connect(handle.endpoint.clone())
        .await
        .unwrap();
    let mut consumer = ConsumerServiceClient::connect(handle.endpoint.clone())
        .await
        .unwrap();

    let component_uuid = Uuid::new_v4().to_string();
    let mut create_component = Request::new(proto::CreateComponentRequest {
        name: "product-release component".to_string(),
        description: String::new(),
        identifiers: vec![],
        component_type: proto::ComponentType::Library as i32,
        licenses: vec![proto::LicenseInfo {
            spdx_id: "MIT".to_string(),
            name: String::new(),
            url: String::new(),
        }],
        publisher: "ACME".to_string(),
        homepage_url: "https://example.com/component".to_string(),
        vcs_url: String::new(),
        uuid: Some(component_uuid.clone()),
    });
    let metadata = format!("Bearer {}", publisher_token()).parse().unwrap();
    create_component
        .metadata_mut()
        .insert("authorization", metadata);
    publisher.create_component(create_component).await.unwrap();

    let component_release_uuid = Uuid::new_v4().to_string();
    let mut create_component_release = Request::new(proto::CreateComponentReleaseRequest {
        component_uuid: component_uuid.clone(),
        version: "2.0.0-rc.1".to_string(),
        release_date: Some(Timestamp {
            seconds: 1_700_100_000,
            nanos: 0,
        }),
        pre_release: true,
        identifiers: vec![],
        distributions: vec![],
        uuid: Some(component_release_uuid.clone()),
    });
    let metadata = format!("Bearer {}", publisher_token()).parse().unwrap();
    create_component_release
        .metadata_mut()
        .insert("authorization", metadata);
    publisher
        .create_component_release(create_component_release)
        .await
        .unwrap();

    let release_uuid = Uuid::new_v4().to_string();
    let mut create_release = Request::new(proto::CreateProductReleaseRequest {
        product_uuid: Some(handle.product_uuid.to_string()),
        version: "2026.03-rc.1".to_string(),
        release_date: Some(Timestamp {
            seconds: 1_700_200_000,
            nanos: 0,
        }),
        pre_release: true,
        identifiers: vec![proto::Identifier {
            id_type: proto::IdentifierType::Tei as i32,
            id_value: format!("urn:tei:uuid:tea.example.com:{release_uuid}"),
        }],
        components: vec![proto::ComponentRef {
            uuid: component_uuid.clone(),
            release: Some(component_release_uuid.clone()),
        }],
        uuid: Some(release_uuid.clone()),
    });
    let metadata = format!("Bearer {}", publisher_token()).parse().unwrap();
    create_release
        .metadata_mut()
        .insert("authorization", metadata);
    let created_release = publisher
        .create_product_release(create_release)
        .await
        .unwrap()
        .into_inner();
    assert_eq!(created_release.uuid, release_uuid);
    assert_eq!(
        created_release.product,
        Some(handle.product_uuid.to_string())
    );
    assert!(created_release.pre_release);

    let listed = consumer
        .list_product_releases(Request::new(proto::ListProductReleasesRequest {
            product_uuid: handle.product_uuid.to_string(),
            pagination: None,
            include_pre_releases: true,
            release_date_range: None,
            sort: None,
        }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(listed.releases.len(), 1);

    let mut update_release = Request::new(proto::UpdateProductReleaseRequest {
        uuid: created_release.uuid.clone(),
        update_mask: Some(FieldMask {
            paths: vec!["version".to_string(), "pre_release".to_string()],
        }),
        version: "2026.03".to_string(),
        release_date: None,
        pre_release: false,
        identifiers: vec![],
        components: vec![],
    });
    let metadata = format!("Bearer {}", publisher_token()).parse().unwrap();
    update_release
        .metadata_mut()
        .insert("authorization", metadata);
    let updated_release = publisher
        .update_product_release(update_release)
        .await
        .unwrap()
        .into_inner();
    assert_eq!(updated_release.version, "2026.03");
    assert!(!updated_release.pre_release);

    let fetched = consumer
        .get_product_release(Request::new(proto::GetProductReleaseRequest {
            uuid: created_release.uuid.clone(),
        }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(fetched.version, "2026.03");
    assert_eq!(fetched.components.len(), 1);

    let mut delete_without_cascade = Request::new(proto::DeleteProductRequest {
        uuid: handle.product_uuid.to_string(),
        cascade: false,
    });
    let metadata = format!("Bearer {}", publisher_token()).parse().unwrap();
    delete_without_cascade
        .metadata_mut()
        .insert("authorization", metadata);
    let blocked = publisher
        .delete_product(delete_without_cascade)
        .await
        .unwrap_err();
    assert_eq!(blocked.code(), tonic::Code::FailedPrecondition);

    let mut delete_with_cascade = Request::new(proto::DeleteProductRequest {
        uuid: handle.product_uuid.to_string(),
        cascade: true,
    });
    let metadata = format!("Bearer {}", publisher_token()).parse().unwrap();
    delete_with_cascade
        .metadata_mut()
        .insert("authorization", metadata);
    let deleted = publisher
        .delete_product(delete_with_cascade)
        .await
        .unwrap()
        .into_inner();
    assert_eq!(deleted.releases_deleted, 1);

    stop_server(handle).await;

    for (key, value) in saved {
        match value {
            Some(value) => std::env::set_var(key, value),
            None => std::env::remove_var(key),
        }
    }
}

#[tokio::test]
async fn publisher_grpc_can_create_artifact_from_url() {
    let _guard = env_lock().lock().await;
    let saved = [
        ("TEA_JWT_SECRET", std::env::var("TEA_JWT_SECRET").ok()),
        ("TEA_JWT_AUDIENCE", std::env::var("TEA_JWT_AUDIENCE").ok()),
        (
            "TEA_JWT_WRITE_SCOPE",
            std::env::var("TEA_JWT_WRITE_SCOPE").ok(),
        ),
        (
            "TEA_JWT_WRITE_ROLE",
            std::env::var("TEA_JWT_WRITE_ROLE").ok(),
        ),
        ("TEA_JWT_ISSUER", std::env::var("TEA_JWT_ISSUER").ok()),
        (
            "TEA_ALLOW_PRIVATE_SOURCE_URLS",
            std::env::var("TEA_ALLOW_PRIVATE_SOURCE_URLS").ok(),
        ),
    ];

    std::env::set_var("TEA_JWT_SECRET", "dev-only-insecure-secret-32-bytes--");
    std::env::set_var("TEA_JWT_AUDIENCE", "tea-api");
    std::env::set_var("TEA_JWT_WRITE_SCOPE", "tea:write");
    std::env::set_var("TEA_JWT_WRITE_ROLE", "tea-writer");
    std::env::set_var("TEA_JWT_ISSUER", "issuer");
    std::env::set_var("TEA_ALLOW_PRIVATE_SOURCE_URLS", "true");

    let handle = spawn_grpc_server(true).await;
    let source_body = br#"{"bomFormat":"CycloneDX","specVersion":"1.6"}"#.to_vec();
    let source = spawn_source_server(source_body.clone(), "application/vnd.cyclonedx+json").await;

    let mut publisher = PublisherServiceClient::connect(handle.endpoint.clone())
        .await
        .unwrap();
    let mut consumer = ConsumerServiceClient::connect(handle.endpoint.clone())
        .await
        .unwrap();

    let artifact_uuid = Uuid::new_v4().to_string();
    let checksum = hex::encode(Sha256::digest(&source_body));
    let source_url = format!("{}/artifact", source.base_url);

    let mut request = Request::new(proto::CreateArtifactFromUrlRequest {
        metadata: Some(proto::ArtifactMetadata {
            name: "transport-widget.cdx.json".to_string(),
            r#type: proto::ArtifactType::Bom as i32,
            mime_type: "application/vnd.cyclonedx+json".to_string(),
            description: "Fetched and verified over gRPC".to_string(),
            component_distributions: vec![],
            subject: Some(proto::ArtifactSubject {
                r#type: proto::SubjectType::Product as i32,
                identifiers: vec![],
                name: "Transport Widget".to_string(),
                version: "2026.03".to_string(),
            }),
            spec_version: "1.6".to_string(),
            uuid: Some(artifact_uuid.clone()),
            expected_checksums: vec![proto::Checksum {
                alg_type: proto::ChecksumAlgorithm::Sha256 as i32,
                alg_value: checksum.clone(),
            }],
        }),
        source_url: source_url.clone(),
        expected_checksums: vec![],
        signature_url: String::new(),
    });
    let metadata = format!("Bearer {}", publisher_token()).parse().unwrap();
    request.metadata_mut().insert("authorization", metadata);
    let artifact = publisher
        .create_artifact_from_url(request)
        .await
        .unwrap()
        .into_inner();

    assert_eq!(artifact.uuid, artifact_uuid);
    assert_eq!(artifact.name, "transport-widget.cdx.json");
    assert_eq!(artifact.formats.len(), 1);
    assert_eq!(artifact.formats[0].url, source_url);
    assert_eq!(
        artifact.formats[0].size_bytes,
        Some(source_body.len() as i64)
    );
    assert_eq!(artifact.formats[0].checksums[0].alg_value, checksum);

    let fetched = consumer
        .get_artifact(Request::new(proto::GetArtifactRequest {
            uuid: artifact.uuid.clone(),
        }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(fetched.uuid, artifact.uuid);
    assert_eq!(fetched.formats[0].url, artifact.formats[0].url);

    stop_source_server(source).await;
    stop_server(handle).await;

    for (key, value) in saved {
        match value {
            Some(value) => std::env::set_var(key, value),
            None => std::env::remove_var(key),
        }
    }
}

#[tokio::test]
async fn publisher_grpc_can_create_and_version_collections() {
    let _guard = env_lock().lock().await;
    let saved = [
        ("TEA_JWT_SECRET", std::env::var("TEA_JWT_SECRET").ok()),
        ("TEA_JWT_AUDIENCE", std::env::var("TEA_JWT_AUDIENCE").ok()),
        (
            "TEA_JWT_WRITE_SCOPE",
            std::env::var("TEA_JWT_WRITE_SCOPE").ok(),
        ),
        (
            "TEA_JWT_WRITE_ROLE",
            std::env::var("TEA_JWT_WRITE_ROLE").ok(),
        ),
        ("TEA_JWT_ISSUER", std::env::var("TEA_JWT_ISSUER").ok()),
    ];

    std::env::set_var("TEA_JWT_SECRET", "dev-only-insecure-secret-32-bytes--");
    std::env::set_var("TEA_JWT_AUDIENCE", "tea-api");
    std::env::set_var("TEA_JWT_WRITE_SCOPE", "tea:write");
    std::env::set_var("TEA_JWT_WRITE_ROLE", "tea-writer");
    std::env::set_var("TEA_JWT_ISSUER", "issuer");

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
        version: "2026.04".to_string(),
        release_date: Some(Timestamp {
            seconds: 1_700_300_000,
            nanos: 0,
        }),
        pre_release: false,
        identifiers: vec![],
        components: vec![],
        uuid: Some(release_uuid.clone()),
    });
    let metadata = format!("Bearer {}", publisher_token()).parse().unwrap();
    create_release
        .metadata_mut()
        .insert("authorization", metadata);
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
    let metadata = format!("Bearer {}", publisher_token()).parse().unwrap();
    create_collection
        .metadata_mut()
        .insert("authorization", metadata);
    let created = publisher
        .create_collection(create_collection)
        .await
        .unwrap()
        .into_inner();
    assert_eq!(created.uuid, release_uuid);
    assert_eq!(created.version, 1);
    assert_eq!(created.artifacts.len(), 1);

    let mut update_collection = Request::new(proto::UpdateCollectionRequest {
        uuid: release_uuid.clone(),
        artifact_uuids: vec![
            handle.artifact_uuid.to_string(),
            handle.artifact_uuid_2.to_string(),
        ],
        update_reason: Some(proto::UpdateReason {
            r#type: proto::UpdateReasonType::ArtifactAdded as i32,
            comment: String::new(),
            affected_artifact_uuids: vec![handle.artifact_uuid_2.to_string()],
        }),
    });
    let metadata = format!("Bearer {}", publisher_token()).parse().unwrap();
    update_collection
        .metadata_mut()
        .insert("authorization", metadata);
    let updated = publisher
        .update_collection(update_collection)
        .await
        .unwrap()
        .into_inner();
    assert_eq!(updated.version, 2);
    assert_eq!(updated.artifacts.len(), 2);

    let latest = consumer
        .get_collection(Request::new(proto::GetCollectionRequest {
            uuid: release_uuid.clone(),
            include_artifacts: true,
        }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(latest.version, 2);
    assert_eq!(latest.artifacts.len(), 2);

    let versions = consumer
        .list_collection_versions(Request::new(proto::ListCollectionVersionsRequest {
            uuid: release_uuid.clone(),
            pagination: None,
            include_artifacts: false,
        }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(versions.versions.len(), 2);
    assert_eq!(versions.versions[0].version, 2);
    assert_eq!(versions.versions[1].version, 1);

    let diff = consumer
        .compare_collection_versions(Request::new(proto::CompareCollectionVersionsRequest {
            uuid: release_uuid.clone(),
            base_version: 1,
            target_version: 2,
        }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(
        diff.added_artifact_uuids,
        vec![handle.artifact_uuid_2.to_string()]
    );

    stop_server(handle).await;

    for (key, value) in saved {
        match value {
            Some(value) => std::env::set_var(key, value),
            None => std::env::remove_var(key),
        }
    }
}
