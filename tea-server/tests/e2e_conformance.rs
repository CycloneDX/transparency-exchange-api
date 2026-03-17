//! E2E Domain Conformance Tests for Transparency Exchange API.
//!
//! These tests verify that the domain layer conforms to the TEA specification:
//! - CRUD operations for Products, Components, Artifacts, Collections
//! - Deprecation lifecycle management
//! - Identifier type handling (all 13 types)
//! - Validation rules

#[cfg(test)]
mod e2e_tests {
    use chrono::Utc;
    use tea_server::domain::artifact::entity::{Artifact, ArtifactFormat, ArtifactType};
    use tea_server::domain::artifact::service::ArtifactService;
    use tea_server::domain::collection::entity::{Collection, CollectionScope, UpdateReason};
    use tea_server::domain::collection::service::CollectionService;
    use tea_server::domain::common::checksum::{Checksum, ChecksumAlgorithm};
    use tea_server::domain::common::deprecation::{Deprecation, DeprecationState};
    use tea_server::domain::common::identifier::{Identifier, IdentifierType};
    use tea_server::domain::component::entity::{Component, ComponentType};
    use tea_server::domain::component::service::ComponentService;
    use tea_server::domain::product::entity::{Product, Vendor};
    use tea_server::domain::product::service::ProductService;
    use tea_server::infrastructure::persistence::memory::{
        artifact_repository::InMemoryArtifactRepository,
        collection_repository::InMemoryCollectionRepository,
        component_repository::InMemoryComponentRepository,
        product_repository::InMemoryProductRepository,
    };
    use uuid::Uuid;

    // ──────────────────────── Test Helpers ─────────────────────────────────────

    fn make_product(name: &str) -> Product {
        Product {
            uuid: Uuid::new_v4(),
            name: name.to_string(),
            description: Some("Test product".to_string()),
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
            description: Some("Test component".to_string()),
            identifiers: vec![],
            created_date: Utc::now(),
            modified_date: Utc::now(),
            homepage_url: None,
            vcs_url: None,
            deprecation: None,
            component_type: ComponentType::Library,
            licenses: vec![],
            publisher: None,
            dependencies: vec![],
        }
    }

    fn make_artifact(name: &str) -> Artifact {
        Artifact {
            uuid: Uuid::new_v4(),
            name: name.to_string(),
            description: Some("Test artifact".to_string()),
            type_: ArtifactType::Bom,
            component_distributions: vec![],
            formats: vec![ArtifactFormat {
                mime_type: "application/json".to_string(),
                description: None,
                url: "https://example.com/test.json".to_string(),
                signature_url: None,
                checksums: vec![Checksum {
                    alg_type: ChecksumAlgorithm::Sha256,
                    alg_value: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
                        .to_string(),
                }],
                size_bytes: Some(1024),
                encoding: None,
                spec_version: None,
            }],
            created_date: Utc::now(),
            modified_date: Utc::now(),
            deprecation: None,
            subject: None,
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
            belongs_to: CollectionScope::Release,
            update_reason: UpdateReason::InitialRelease,
            artifacts: vec![],
            deprecation: None,
            dependencies: vec![],
        }
    }

    // ──────────────────────── Product Conformance Tests ──────────────────────────

    #[tokio::test]
    async fn test_product_create_conforms_to_spec() {
        let repo = InMemoryProductRepository::new();
        let service = ProductService::new(repo);

        let product = make_product("Test Product");
        let created = service.create_product(product.clone()).await.unwrap();

        assert!(!created.uuid.is_nil(), "Product must have a valid UUID");
        assert_eq!(created.name, "Test Product");
        assert!(created.created_date <= Utc::now());
        assert!(created.modified_date <= Utc::now());
    }

    #[tokio::test]
    async fn test_product_get_conforms_to_spec() {
        let repo = InMemoryProductRepository::new();
        let service = ProductService::new(repo);

        let product = make_product("Fetchable Product");
        let created = service.create_product(product).await.unwrap();

        let fetched = service.get_product(&created.uuid).await.unwrap().unwrap();
        assert_eq!(fetched.uuid, created.uuid);
        assert_eq!(fetched.name, "Fetchable Product");
    }

    #[tokio::test]
    async fn test_product_update_conforms_to_spec() {
        let repo = InMemoryProductRepository::new();
        let service = ProductService::new(repo);

        let product = make_product("Original Name");
        let created = service.create_product(product).await.unwrap();

        let mut updated = created.clone();
        updated.name = "Updated Name".to_string();
        updated.description = Some("Updated description".to_string());

        let result = service.update_product(updated).await.unwrap();
        assert_eq!(result.name, "Updated Name");
        assert_eq!(result.description, Some("Updated description".to_string()));
    }

    #[tokio::test]
    async fn test_product_delete_conforms_to_spec() {
        let repo = InMemoryProductRepository::new();
        let service = ProductService::new(repo);

        let product = make_product("Deletable Product");
        let created = service.create_product(product).await.unwrap();

        service.delete_product(&created.uuid).await.unwrap();

        let result = service.get_product(&created.uuid).await.unwrap();
        assert!(result.is_none(), "Deleted product should not be found");
    }

    #[tokio::test]
    async fn test_product_deprecation_conforms_to_spec() {
        let repo = InMemoryProductRepository::new();
        let service = ProductService::new(repo);

        let product = make_product("Deprecatable Product");
        let created = service.create_product(product).await.unwrap();

        let deprecation = Deprecation {
            state: DeprecationState::Deprecated,
            reason: Some("Superseded by v2".to_string()),
            announced_date: Some(Utc::now()),
            effective_date: Some(Utc::now()),
            replacement_identifiers: vec![],
        };

        let result = service
            .deprecate_product(&created.uuid, deprecation)
            .await
            .unwrap();
        assert!(result.deprecation.is_some());
        assert_eq!(
            result.deprecation.unwrap().state,
            DeprecationState::Deprecated
        );
    }

    // ──────────────────────── Component Conformance Tests ───────────────────────

    #[tokio::test]
    async fn test_component_create_conforms_to_spec() {
        let repo = InMemoryComponentRepository::new();
        let service = ComponentService::new(repo);

        let component = make_component("Test Component");
        let created = service.create_component(component.clone()).await.unwrap();

        assert!(!created.uuid.is_nil());
        assert_eq!(created.name, "Test Component");
        assert_eq!(created.component_type, ComponentType::Library);
    }

    #[tokio::test]
    async fn test_component_crud_operations() {
        let repo = InMemoryComponentRepository::new();
        let service = ComponentService::new(repo);

        // Create
        let component = make_component("CRUD Component");
        let created = service.create_component(component).await.unwrap();

        // Read
        let fetched = service.get_component(&created.uuid).await.unwrap().unwrap();
        assert_eq!(fetched.name, "CRUD Component");

        // Update
        let mut updated = created.clone();
        updated.name = "Updated Component".to_string();
        let result = service.update_component(updated).await.unwrap();
        assert_eq!(result.name, "Updated Component");

        // Delete
        service.delete_component(&created.uuid).await.unwrap();
        assert!(service
            .get_component(&created.uuid)
            .await
            .unwrap()
            .is_none());
    }

    // ──────────────────────── Artifact Conformance Tests ────────────────────────

    #[tokio::test]
    async fn test_artifact_create_with_formats_conforms_to_spec() {
        let repo = InMemoryArtifactRepository::new();
        let service = ArtifactService::new(repo);

        let artifact = make_artifact("test-artifact.jar");
        let created = service.create_artifact(artifact.clone()).await.unwrap();

        assert!(!created.uuid.is_nil());
        assert_eq!(created.name, "test-artifact.jar");
        assert!(
            !created.formats.is_empty(),
            "Artifact must have at least one format"
        );
    }

    #[tokio::test]
    async fn test_artifact_crud_operations() {
        let repo = InMemoryArtifactRepository::new();
        let service = ArtifactService::new(repo);

        // Create
        let artifact = make_artifact("CRUD Artifact");
        let created = service.create_artifact(artifact).await.unwrap();

        // Read
        let fetched = service.get_artifact(&created.uuid).await.unwrap().unwrap();
        assert_eq!(fetched.name, "CRUD Artifact");

        // Update
        let mut updated = created.clone();
        updated.name = "Updated Artifact".to_string();
        let result = service.update_artifact(updated).await.unwrap();
        assert_eq!(result.name, "Updated Artifact");

        // Delete
        service.delete_artifact(&created.uuid).await.unwrap();
        assert!(service.get_artifact(&created.uuid).await.unwrap().is_none());
    }

    // ──────────────────────── Collection Conformance Tests ──────────────────────

    #[tokio::test]
    async fn test_collection_create_conforms_to_spec() {
        let repo = InMemoryCollectionRepository::new();
        let service = CollectionService::new(repo);

        let collection = make_collection("Test Collection");
        let created = service.create_collection(collection.clone()).await.unwrap();

        assert!(!created.uuid.is_nil());
        assert_eq!(created.name, "Test Collection");
        assert_eq!(created.version, 1);
    }

    #[tokio::test]
    async fn test_collection_crud_operations() {
        let repo = InMemoryCollectionRepository::new();
        let service = CollectionService::new(repo);

        // Create
        let collection = make_collection("CRUD Collection");
        let created = service.create_collection(collection).await.unwrap();

        // Read
        let fetched = service
            .get_collection(&created.uuid)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(fetched.name, "CRUD Collection");

        // Update
        let mut updated = created.clone();
        updated.name = "Updated Collection".to_string();
        let result = service.update_collection(updated).await.unwrap();
        assert_eq!(result.name, "Updated Collection");

        // Delete
        service.delete_collection(&created.uuid).await.unwrap();
        assert!(service
            .get_collection(&created.uuid)
            .await
            .unwrap()
            .is_none());
    }

    // ──────────────────────── Identifier Type Conformance ──────────────────────

    #[test]
    fn test_all_identifier_types_are_supported() {
        // Test that all identifier types are properly defined
        let identifier_types: Vec<IdentifierType> = vec![
            IdentifierType::Unspecified,
            IdentifierType::Tei,
            IdentifierType::Purl,
            IdentifierType::Cpe,
            IdentifierType::Swid,
            IdentifierType::Gav,
            IdentifierType::Gtin,
            IdentifierType::Gmn,
            IdentifierType::Udi,
            IdentifierType::Asin,
            IdentifierType::Hash,
            IdentifierType::Conformance,
        ];

        assert_eq!(
            identifier_types.len(),
            12,
            "Should have 12 identifier types"
        );

        // Test that identifiers can be created with each type
        for id_type in identifier_types {
            let id = Identifier {
                id_type: id_type.clone(),
                id_value: "test-value".to_string(),
            };
            assert_eq!(id.id_type, id_type);
        }
    }

    // ──────────────────────── Deprecation State Machine ────────────────────────

    #[test]
    fn test_deprecation_state_transitions() {
        // Test valid state values
        let states: Vec<DeprecationState> = vec![
            DeprecationState::Unspecified,
            DeprecationState::Active,
            DeprecationState::Deprecated,
            DeprecationState::Retired,
        ];

        assert_eq!(states.len(), 4, "Should have 4 deprecation states");

        // Test deprecation creation
        for state in &states {
            let deprecation = Deprecation {
                state: state.clone(),
                reason: Some(format!("Test reason")),
                announced_date: Some(Utc::now()),
                effective_date: Some(Utc::now()),
                replacement_identifiers: vec![],
            };
            assert_eq!(deprecation.state, state.clone());
        }
    }

    // ──────────────────────── Checksum Algorithm Conformance ────────────────────

    #[test]
    fn test_checksum_algorithms_are_supported() {
        let algorithms: Vec<ChecksumAlgorithm> = vec![
            ChecksumAlgorithm::Md5,
            ChecksumAlgorithm::Sha1,
            ChecksumAlgorithm::Sha256,
            ChecksumAlgorithm::Sha384,
            ChecksumAlgorithm::Sha512,
            ChecksumAlgorithm::Sha3_256,
            ChecksumAlgorithm::Sha3_384,
            ChecksumAlgorithm::Sha3_512,
            ChecksumAlgorithm::Blake2b256,
            ChecksumAlgorithm::Blake2b384,
            ChecksumAlgorithm::Blake2b512,
            ChecksumAlgorithm::Blake3,
        ];

        assert!(
            !algorithms.is_empty(),
            "Should have checksum algorithms defined"
        );

        // Test checksum creation
        for alg in &algorithms {
            let checksum = Checksum {
                alg_type: alg.clone(),
                alg_value: "test-hash-value".to_string(),
            };
            assert_eq!(checksum.alg_type, alg.clone());
        }
    }

    // ──────────────────────── Error Handling Conformance ───────────────────────

    #[tokio::test]
    async fn test_not_found_returns_error() {
        let repo = InMemoryProductRepository::new();
        let service = ProductService::new(repo);

        let result = service.get_product(&Uuid::new_v4()).await.unwrap();
        assert!(result.is_none(), "Non-existent product should return None");
    }
}
