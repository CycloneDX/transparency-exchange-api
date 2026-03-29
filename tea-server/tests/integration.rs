#[cfg(test)]
mod tests {
    use chrono::Utc;
    use sqlx::PgPool;
    use tea_server::domain::artifact::entity::{
        Artifact, ArtifactFormat, ArtifactType, Subject, SubjectType,
    };
    use tea_server::domain::artifact::service::ArtifactService;
    use tea_server::domain::collection::entity::{Collection, CollectionScope, UpdateReason};
    use tea_server::domain::collection::service::CollectionService;
    use tea_server::domain::common::checksum::{Checksum, ChecksumAlgorithm};
    use tea_server::domain::common::deprecation::{Deprecation, DeprecationState};
    use tea_server::domain::common::error::DomainError;
    use tea_server::domain::common::identifier::Identifier;
    use tea_server::domain::component::entity::{
        Component, ComponentRelease, ComponentType, LicenseInfo, LicenseType,
    };
    use tea_server::domain::component::service::ComponentService;
    use tea_server::domain::product::entity::{ComponentRef, Product, ProductRelease, Vendor};
    use tea_server::domain::product::service::ProductService;
    use tea_server::infrastructure::persistence::postgres::artifact_repository::PostgresArtifactRepository;
    use tea_server::infrastructure::persistence::postgres::collection_repository::PostgresCollectionRepository;
    use tea_server::infrastructure::persistence::postgres::component_repository::PostgresComponentRepository;
    use tea_server::infrastructure::persistence::postgres::product_repository::PostgresProductRepository;
    use testcontainers_modules::postgres::Postgres;
    use testcontainers_modules::testcontainers::runners::AsyncRunner;
    use testcontainers_modules::testcontainers::ContainerAsync;
    use uuid::Uuid;

    fn make_product(name: &str) -> Product {
        Product {
            uuid: Uuid::new_v4(),
            name: name.to_string(),
            description: Some("Test".to_string()),
            identifiers: vec![],
            vendor: Vendor {
                name: "ACME Corp".to_string(),
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
        }
    }

    fn make_component(name: &str) -> Component {
        Component {
            uuid: Uuid::new_v4(),
            name: name.to_string(),
            description: Some("Component under test".to_string()),
            identifiers: vec![Identifier::purl("pkg:cargo/acme/component@1.0.0")],
            component_type: ComponentType::Library,
            licenses: vec![LicenseInfo {
                license_type: LicenseType::Spdx,
                license_id: "MIT".to_string(),
                url: None,
            }],
            publisher: Some("ACME Corp".to_string()),
            homepage_url: Some("https://example.com/component".to_string()),
            vcs_url: Some("https://github.com/acme/component".to_string()),
            created_date: Utc::now(),
            modified_date: Utc::now(),
            deprecation: None,
            dependencies: vec![],
        }
    }

    fn make_artifact(name: &str) -> Artifact {
        Artifact {
            uuid: Uuid::new_v4(),
            name: name.to_string(),
            type_: ArtifactType::Bom,
            component_distributions: vec!["oci".to_string()],
            formats: vec![ArtifactFormat {
                mime_type: "application/json".to_string(),
                description: Some("CycloneDX SBOM".to_string()),
                url: "https://example.com/artifacts/sbom.json".to_string(),
                signature_url: None,
                checksums: vec![Checksum {
                    alg_type: ChecksumAlgorithm::Sha256,
                    alg_value: "a".repeat(64),
                }],
                size_bytes: Some(128),
                encoding: None,
                spec_version: Some("1.6".to_string()),
            }],
            created_date: Utc::now(),
            modified_date: Utc::now(),
            description: Some("Artifact under test".to_string()),
            subject: Some(Subject {
                type_: SubjectType::Component,
                identifiers: vec![Identifier::purl("pkg:cargo/acme/component@1.0.0")],
                name: Some("component".to_string()),
                version: Some("1.0.0".to_string()),
            }),
            deprecation: None,
            dependencies: vec![],
        }
    }

    fn make_collection(name: &str) -> Collection {
        Collection {
            uuid: Uuid::new_v4(),
            name: name.to_string(),
            version: 1,
            date: Utc::now(),
            created_date: Utc::now(),
            modified_date: Utc::now(),
            belongs_to: CollectionScope::ProductRelease,
            update_reason: UpdateReason::InitialRelease,
            artifacts: vec![],
            deprecation: None,
            dependencies: vec![],
        }
    }

    // M7 fix: return type correctly uses ContainerAsync<Postgres> — not the IntoContainerPort
    // trait object that caused the type mismatch in the previous testcontainers-modules version.
    async fn make_pool() -> (ContainerAsync<Postgres>, PgPool) {
        let container = Postgres::default()
            .start()
            .await
            .expect("failed to start postgres container");
        let connection_string = format!(
            "postgres://postgres:postgres@127.0.0.1:{}/postgres",
            container
                .get_host_port_ipv4(5432)
                .await
                .expect("failed to get port")
        );
        let pool = PgPool::connect(&connection_string)
            .await
            .expect("failed to connect to postgres");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("failed to run migrations");
        (container, pool)
    }

    // ──────────────────────── Happy path ────────────────────────

    #[tokio::test]
    async fn test_product_creation_and_retrieval() {
        let (_container, pool) = make_pool().await;
        let repo = PostgresProductRepository::new(pool);
        let service = ProductService::new(repo);

        let product = make_product("Test Product");
        let created = service
            .create_product(product)
            .await
            .expect("create_product should succeed");

        assert_eq!(created.name, "Test Product");
        assert_eq!(created.vendor.name, "ACME Corp");

        let retrieved = service
            .get_product(&created.uuid)
            .await
            .expect("get_product should succeed");

        assert_eq!(
            retrieved.expect("product should exist").name,
            "Test Product"
        );
    }

    #[tokio::test]
    async fn test_product_release_postgres_round_trip_persists_component_refs() {
        let (_container, pool) = make_pool().await;
        let product_service = ProductService::new(PostgresProductRepository::new(pool.clone()));
        let component_service = ComponentService::new(PostgresComponentRepository::new(pool));

        let product = product_service
            .create_product(make_product("Release Product"))
            .await
            .expect("create_product should succeed");

        let component = component_service
            .create_component(make_component("Release Component"))
            .await
            .expect("create_component should succeed");

        let component_release = component_service
            .create_release(ComponentRelease {
                uuid: Uuid::new_v4(),
                component_uuid: component.uuid,
                version: "1.2.3".to_string(),
                release_date: Some(Utc::now()),
                pre_release: false,
                identifiers: vec![Identifier::purl("pkg:cargo/acme/component@1.2.3")],
                distributions: vec![],
            })
            .await
            .expect("create_release should succeed");

        let release = product_service
            .create_release(ProductRelease {
                uuid: Uuid::new_v4(),
                product_uuid: product.uuid,
                version: "2026.03".to_string(),
                created_date: Utc::now(),
                modified_date: Utc::now(),
                release_date: Some(Utc::now()),
                pre_release: false,
                identifiers: vec![Identifier::tei(format!(
                    "urn:tei:uuid:tea.example.com:{}",
                    Uuid::new_v4()
                ))],
                components: vec![ComponentRef {
                    component_uuid: component.uuid,
                    release_uuid: component_release.uuid,
                }],
            })
            .await
            .expect("create_release should succeed");

        let fetched = product_service
            .get_release(&release.uuid)
            .await
            .expect("get_release should succeed")
            .expect("release should exist");

        assert_eq!(fetched.product_uuid, product.uuid);
        assert_eq!(fetched.version, "2026.03");
        assert_eq!(fetched.components.len(), 1);
        assert_eq!(fetched.components[0].component_uuid, component.uuid);
        assert_eq!(fetched.components[0].release_uuid, component_release.uuid);
    }

    // ──────────────────────── Postgres round-trip: deprecation ────────────────────────

    #[tokio::test]
    async fn test_postgres_deprecation_round_trips() {
        let (_container, pool) = make_pool().await;
        let repo = PostgresProductRepository::new(pool);
        let service = ProductService::new(repo);

        let product = make_product("Widget v1");
        let created = service.create_product(product).await.unwrap();

        let deprecation = Deprecation {
            state: DeprecationState::Deprecated,
            reason: Some("Superseded by Widget v2".to_string()),
            announced_date: None,
            effective_date: None,
            replacement_identifiers: vec![],
        };

        let _deprecated = service
            .deprecate_product(&created.uuid, deprecation)
            .await
            .unwrap();

        // Fetch again from Postgres — deprecation must survive round-trip (C3 fix)
        let fetched = service.get_product(&created.uuid).await.unwrap().unwrap();
        assert!(
            fetched.deprecation.is_some(),
            "C3 check: deprecation state should persist in Postgres"
        );
        assert_eq!(
            fetched.deprecation.unwrap().state,
            DeprecationState::Deprecated,
        );
    }

    // ──────────────────────── Postgres round-trip: ON CONFLICT (H4) ────────────────────────

    #[tokio::test]
    async fn test_postgres_duplicate_uuid_returns_conflict() {
        use tea_server::domain::common::error::RepositoryError;
        use tea_server::domain::product::repository::ProductRepository;

        let (_container, pool) = make_pool().await;
        let repo = PostgresProductRepository::new(pool);

        let product = make_product("Unique Widget");
        repo.save(&product)
            .await
            .expect("first save should succeed");

        let err = repo
            .save(&product) // same UUID
            .await
            .expect_err("duplicate UUID should return Conflict");

        assert!(
            matches!(err, RepositoryError::Conflict),
            "H4 check: expected Conflict, got: {err:?}"
        );
    }

    #[tokio::test]
    async fn test_component_postgres_round_trip_persists_deprecation() {
        let (_container, pool) = make_pool().await;
        let repo = PostgresComponentRepository::new(pool);
        let service = ComponentService::new(repo);

        let created = service
            .create_component(make_component("Test Component"))
            .await
            .expect("create_component should succeed");

        let deprecation = Deprecation {
            state: DeprecationState::Deprecated,
            reason: Some("Superseded".to_string()),
            announced_date: None,
            effective_date: None,
            replacement_identifiers: vec![],
        };

        service
            .deprecate_component(&created.uuid, deprecation)
            .await
            .expect("deprecate_component should succeed");

        let fetched = service
            .get_component(&created.uuid)
            .await
            .expect("get_component should succeed")
            .expect("component should exist");

        assert_eq!(fetched.name, "Test Component");
        assert_eq!(
            fetched
                .deprecation
                .expect("deprecation should persist")
                .state,
            DeprecationState::Deprecated,
        );
    }

    #[tokio::test]
    async fn test_artifact_postgres_round_trip_persists_deprecation() {
        let (_container, pool) = make_pool().await;
        let repo = PostgresArtifactRepository::new(pool);
        let service = ArtifactService::new(repo);

        let created = service
            .create_artifact(make_artifact("SBOM"))
            .await
            .expect("create_artifact should succeed");

        let deprecation = Deprecation {
            state: DeprecationState::Deprecated,
            reason: Some("Superseded".to_string()),
            announced_date: None,
            effective_date: None,
            replacement_identifiers: vec![],
        };

        service
            .deprecate_artifact(&created.uuid, deprecation)
            .await
            .expect("deprecate_artifact should succeed");

        let fetched = service
            .get_artifact(&created.uuid)
            .await
            .expect("get_artifact should succeed")
            .expect("artifact should exist");

        assert_eq!(fetched.name, "SBOM");
        assert_eq!(
            fetched
                .deprecation
                .expect("deprecation should persist")
                .state,
            DeprecationState::Deprecated,
        );
    }

    #[tokio::test]
    async fn test_collection_postgres_round_trip_persists_deprecation() {
        let (_container, pool) = make_pool().await;
        let repo = PostgresCollectionRepository::new(pool);
        let service = CollectionService::new(repo);

        let created = service
            .create_collection(make_collection("Release Collection"))
            .await
            .expect("create_collection should succeed");

        let deprecation = Deprecation {
            state: DeprecationState::Deprecated,
            reason: Some("Superseded".to_string()),
            announced_date: None,
            effective_date: None,
            replacement_identifiers: vec![],
        };

        service
            .deprecate_collection(&created.uuid, deprecation)
            .await
            .expect("deprecate_collection should succeed");

        let fetched = service
            .get_collection(&created.uuid)
            .await
            .expect("get_collection should succeed")
            .expect("collection should exist");

        assert_eq!(fetched.name, "Release Collection");
        assert_eq!(
            fetched
                .deprecation
                .expect("deprecation should persist")
                .state,
            DeprecationState::Deprecated,
        );
    }

    #[tokio::test]
    async fn test_collection_postgres_supports_multiple_versions() {
        let (_container, pool) = make_pool().await;
        let artifact_service = ArtifactService::new(PostgresArtifactRepository::new(pool.clone()));
        let repo = PostgresCollectionRepository::new(pool);
        let service = CollectionService::new(repo);

        let artifact_v1 = artifact_service
            .create_artifact(make_artifact("Collection Artifact v1"))
            .await
            .expect("create_artifact should succeed");
        let artifact_v2 = artifact_service
            .create_artifact(make_artifact("Collection Artifact v2"))
            .await
            .expect("create_artifact should succeed");

        let mut initial = make_collection("Release Collection");
        initial.artifacts = vec![artifact_v1.uuid];

        let created = service
            .create_collection(initial)
            .await
            .expect("create_collection should succeed");

        let next = service
            .create_next_version(
                &created.uuid,
                vec![artifact_v1.uuid, artifact_v2.uuid],
                UpdateReason::ArtifactAdded,
            )
            .await
            .expect("create_next_version should succeed");

        assert_eq!(next.version, 2);
        assert_eq!(next.artifacts, vec![artifact_v1.uuid, artifact_v2.uuid]);
        let latest = service
            .get_collection(&created.uuid)
            .await
            .expect("get_collection should succeed")
            .expect("latest collection should exist");
        assert_eq!(latest.version, 2);
        assert_eq!(latest.artifacts, vec![artifact_v1.uuid, artifact_v2.uuid]);

        let versions = service
            .list_collection_versions(&created.uuid)
            .await
            .expect("list_collection_versions should succeed");
        assert_eq!(versions.len(), 2);
        assert!(versions.iter().any(|collection| collection.version == 1));
        assert!(versions.iter().any(|collection| collection.version == 2));
        assert!(versions
            .iter()
            .any(|collection| collection.version == 1
                && collection.artifacts == vec![artifact_v1.uuid]));
        assert!(versions.iter().any(|collection| {
            collection.version == 2
                && collection.artifacts == vec![artifact_v1.uuid, artifact_v2.uuid]
        }));
    }

    // ──────────────────────── Validation failures ────────────────────────

    #[tokio::test]
    async fn test_create_product_empty_name_returns_validation_error() {
        use tea_server::infrastructure::persistence::memory::product_repository::InMemoryProductRepository;

        let repo = InMemoryProductRepository::new();
        let service = ProductService::new(repo);

        let mut product = make_product("Valid Name");
        product.name = "".to_string();

        let err = service
            .create_product(product)
            .await
            .expect_err("empty name should fail");

        assert!(
            matches!(err, DomainError::Validation(_)),
            "expected Validation error, got: {err:?}"
        );
    }

    #[tokio::test]
    async fn test_create_product_whitespace_name_returns_validation_error() {
        use tea_server::infrastructure::persistence::memory::product_repository::InMemoryProductRepository;

        let repo = InMemoryProductRepository::new();
        let service = ProductService::new(repo);

        let mut product = make_product("Valid Name");
        product.name = "   ".to_string();

        let err = service
            .create_product(product)
            .await
            .expect_err("whitespace name should fail");

        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[tokio::test]
    async fn test_create_product_invalid_homepage_url_returns_validation_error() {
        use tea_server::infrastructure::persistence::memory::product_repository::InMemoryProductRepository;

        let repo = InMemoryProductRepository::new();
        let service = ProductService::new(repo);

        let mut product = make_product("Valid Name");
        product.homepage_url = Some("not-a-url".to_string());

        let err = service
            .create_product(product)
            .await
            .expect_err("bad URL should fail");

        assert!(matches!(err, DomainError::Validation(_)));
    }

    // ──────────────────────── Not-found semantics ────────────────────────

    #[tokio::test]
    async fn test_get_product_nonexistent_uuid_returns_none() {
        use tea_server::infrastructure::persistence::memory::product_repository::InMemoryProductRepository;

        let repo = InMemoryProductRepository::new();
        let service = ProductService::new(repo);

        let result = service
            .get_product(&Uuid::new_v4())
            .await
            .expect("get_product should not error");

        assert!(result.is_none(), "non-existent product should return None");
    }

    #[tokio::test]
    async fn test_deprecate_nonexistent_product_returns_not_found_error() {
        use tea_server::infrastructure::persistence::memory::product_repository::InMemoryProductRepository;

        let repo = InMemoryProductRepository::new();
        let service = ProductService::new(repo);

        let deprecation = Deprecation {
            state: DeprecationState::Deprecated,
            reason: Some("End of life".to_string()),
            announced_date: None,
            effective_date: None,
            replacement_identifiers: vec![],
        };

        let err = service
            .deprecate_product(&Uuid::new_v4(), deprecation)
            .await
            .expect_err("deprecating non-existent product should fail");

        assert!(
            matches!(err, DomainError::NotFound(_)),
            "expected NotFound error, got: {err:?}"
        );
    }

    // ──────────────────────── Deprecation semantics ────────────────────────

    #[tokio::test]
    async fn test_deprecate_product_sets_deprecation_state() {
        use tea_server::infrastructure::persistence::memory::product_repository::InMemoryProductRepository;

        let repo = InMemoryProductRepository::new();
        let service = ProductService::new(repo);

        let product = make_product("Deprecated Product");
        let created = service
            .create_product(product)
            .await
            .expect("create should succeed");

        let deprecation = Deprecation {
            state: DeprecationState::Deprecated,
            reason: Some("End of life".to_string()),
            announced_date: None,
            effective_date: None,
            replacement_identifiers: vec![],
        };

        let deprecated = service
            .deprecate_product(&created.uuid, deprecation)
            .await
            .expect("deprecate should succeed");

        assert!(deprecated.deprecation.is_some());
        assert_eq!(
            deprecated.deprecation.unwrap().state,
            DeprecationState::Deprecated
        );
    }

    // ──────────────────────── In-memory update semantics ────────────────────────

    #[tokio::test]
    async fn test_in_memory_update_nonexistent_returns_not_found() {
        use tea_server::domain::common::error::RepositoryError;
        use tea_server::domain::product::repository::ProductRepository;
        use tea_server::infrastructure::persistence::memory::product_repository::InMemoryProductRepository;

        let repo = InMemoryProductRepository::new();
        let phantom = make_product("Phantom");

        let err = repo
            .update(&phantom)
            .await
            .expect_err("update on non-existent UUID should fail");

        assert!(
            matches!(err, RepositoryError::NotFound),
            "expected RepositoryError::NotFound, got: {err:?}"
        );
    }

    // ──────────────────────── M5: Delete not-found semantics ────────────────────────

    #[tokio::test]
    async fn test_in_memory_delete_nonexistent_returns_not_found() {
        use tea_server::domain::common::error::RepositoryError;
        use tea_server::domain::product::repository::ProductRepository;
        use tea_server::infrastructure::persistence::memory::product_repository::InMemoryProductRepository;

        let repo = InMemoryProductRepository::new();

        let err = repo
            .delete(&Uuid::new_v4())
            .await
            .expect_err("delete on non-existent UUID should fail");

        assert!(
            matches!(err, RepositoryError::NotFound),
            "M5 check: expected RepositoryError::NotFound, got: {err:?}"
        );
    }
}
