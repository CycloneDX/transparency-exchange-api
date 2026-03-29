/// Comprehensive unit tests for all 4 domain services.
///
/// These tests exercise the in-memory repositories directly (no Postgres required)
/// and cover: validation failures, not-found scenarios, deprecation, and list operations.
#[cfg(test)]
mod product_service_tests {
    use crate::domain::common::deprecation::{Deprecation, DeprecationState};
    use crate::domain::common::error::DomainError;
    use crate::domain::common::pagination::PaginationParams;
    use crate::domain::product::entity::{Product, ProductRelease, Vendor};
    use crate::domain::product::service::ProductService;
    use crate::infrastructure::persistence::memory::product_repository::InMemoryProductRepository;
    use chrono::Utc;
    use uuid::Uuid;

    fn make_service() -> ProductService<InMemoryProductRepository> {
        ProductService::new(InMemoryProductRepository::new())
    }

    fn valid_product() -> Product {
        Product {
            uuid: Uuid::nil(),
            name: "ACME Widget".to_string(),
            description: Some("A widget".to_string()),
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

    fn valid_release(product_uuid: Uuid) -> ProductRelease {
        ProductRelease {
            uuid: Uuid::nil(),
            product_uuid,
            version: "1.0.0".to_string(),
            created_date: Utc::now(),
            modified_date: Utc::now(),
            release_date: Some(Utc::now()),
            pre_release: false,
            identifiers: vec![],
            components: vec![],
        }
    }

    #[tokio::test]
    async fn create_product_empty_name_returns_validation_error() {
        let svc = make_service();
        let mut product = valid_product();
        product.name = "".to_string();
        let result = svc.create_product(product).await;
        assert!(matches!(result, Err(DomainError::Validation(_))));
    }

    #[tokio::test]
    async fn create_product_whitespace_name_returns_validation_error() {
        let svc = make_service();
        let mut product = valid_product();
        product.name = "   ".to_string();
        let result = svc.create_product(product).await;
        assert!(matches!(result, Err(DomainError::Validation(_))));
    }

    #[tokio::test]
    async fn create_product_empty_vendor_name_returns_validation_error() {
        let svc = make_service();
        let mut product = valid_product();
        product.vendor.name = "".to_string();
        let result = svc.create_product(product).await;
        assert!(matches!(result, Err(DomainError::Validation(_))));
    }

    #[tokio::test]
    async fn create_product_invalid_homepage_url_returns_validation_error() {
        let svc = make_service();
        let mut product = valid_product();
        product.homepage_url = Some("not-a-url".to_string());
        let result = svc.create_product(product).await;
        assert!(matches!(result, Err(DomainError::Validation(_))));
    }

    #[tokio::test]
    async fn create_product_ftp_url_returns_validation_error() {
        let svc = make_service();
        let mut product = valid_product();
        product.vcs_url = Some("ftp://example.com".to_string());
        let result = svc.create_product(product).await;
        assert!(matches!(result, Err(DomainError::Validation(_))));
    }

    #[tokio::test]
    async fn create_product_valid_https_url_succeeds() {
        let svc = make_service();
        let mut product = valid_product();
        product.homepage_url = Some("https://example.com".to_string());
        let result = svc.create_product(product).await;
        assert!(result.is_ok());
        let created = result.unwrap();
        // Server assigns a fresh UUID
        assert_ne!(created.uuid, Uuid::nil());
    }

    #[tokio::test]
    async fn create_product_assigns_new_uuid() {
        let svc = make_service();
        let product = valid_product();
        let created = svc.create_product(product).await.unwrap();
        assert_ne!(created.uuid, Uuid::nil());
    }

    #[tokio::test]
    async fn create_product_server_sets_timestamps() {
        let svc = make_service();
        let product = valid_product();
        let created = svc.create_product(product).await.unwrap();
        // Timestamps must be set (non-zero epoch)
        assert!(created.created_date.timestamp() > 0);
        assert!(created.modified_date.timestamp() > 0);
    }

    #[tokio::test]
    async fn get_product_not_found_returns_none() {
        let svc = make_service();
        let uuid = Uuid::new_v4();
        let result = svc.get_product(&uuid).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn get_product_after_create_returns_product() {
        let svc = make_service();
        let product = valid_product();
        let created = svc.create_product(product).await.unwrap();
        let fetched = svc.get_product(&created.uuid).await.unwrap();
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().uuid, created.uuid);
    }

    #[tokio::test]
    async fn list_products_empty_when_no_products() {
        let svc = make_service();
        let result = svc
            .list_products(PaginationParams::default())
            .await
            .unwrap();
        assert!(result.items.is_empty());
    }

    #[tokio::test]
    async fn list_products_returns_all_created_products() {
        let svc = make_service();
        svc.create_product(valid_product()).await.unwrap();
        svc.create_product({
            let mut p = valid_product();
            p.name = "Other Product".to_string();
            p
        })
        .await
        .unwrap();
        let list = svc
            .list_products(PaginationParams::default())
            .await
            .unwrap();
        assert_eq!(list.items.len(), 2);
    }

    #[tokio::test]
    async fn deprecate_product_not_found_returns_not_found_error() {
        let svc = make_service();
        let uuid = Uuid::new_v4();
        let deprecation = Deprecation {
            state: DeprecationState::Deprecated,
            reason: Some("EOL".to_string()),
            announced_date: None,
            effective_date: None,
            replacement_identifiers: vec![],
        };
        let result = svc.deprecate_product(&uuid, deprecation).await;
        assert!(matches!(result, Err(DomainError::NotFound(_))));
    }

    #[tokio::test]
    async fn deprecate_product_sets_deprecation() {
        let svc = make_service();
        let created = svc.create_product(valid_product()).await.unwrap();
        let deprecation = Deprecation {
            state: DeprecationState::Deprecated,
            reason: Some("EOL".to_string()),
            announced_date: None,
            effective_date: None,
            replacement_identifiers: vec![],
        };
        let deprecated = svc
            .deprecate_product(&created.uuid, deprecation)
            .await
            .unwrap();
        assert!(deprecated.deprecation.is_some());
        assert_eq!(
            deprecated.deprecation.unwrap().state,
            DeprecationState::Deprecated
        );
    }

    #[tokio::test]
    async fn create_product_release_requires_product_uuid() {
        let svc = make_service();
        let mut release = valid_release(Uuid::nil());
        release.product_uuid = Uuid::nil();

        let result = svc.create_release(release).await;
        assert!(matches!(result, Err(DomainError::Validation(_))));
    }

    #[tokio::test]
    async fn create_product_release_assigns_uuid_and_lists_by_product() {
        let svc = make_service();
        let product = svc.create_product(valid_product()).await.unwrap();
        let release = svc
            .create_release(valid_release(product.uuid))
            .await
            .unwrap();

        assert_ne!(release.uuid, Uuid::nil());
        let releases = svc.list_releases(&product.uuid).await.unwrap();
        assert_eq!(releases.len(), 1);
        assert_eq!(releases[0].version, "1.0.0");
    }
}

#[cfg(test)]
mod artifact_service_tests {
    use crate::domain::artifact::entity::{Artifact, ArtifactFormat, ArtifactType};
    use crate::domain::artifact::service::ArtifactService;
    use crate::domain::common::error::DomainError;
    use crate::infrastructure::persistence::memory::artifact_repository::InMemoryArtifactRepository;
    use chrono::Utc;
    use uuid::Uuid;

    fn make_service() -> ArtifactService<InMemoryArtifactRepository> {
        ArtifactService::new(InMemoryArtifactRepository::new())
    }

    fn valid_artifact() -> Artifact {
        Artifact {
            uuid: Uuid::nil(),
            name: "sbom.json".to_string(),
            type_: ArtifactType::Bom,
            component_distributions: vec![],
            formats: vec![ArtifactFormat {
                mime_type: "application/json".to_string(),
                description: None,
                url: "https://example.com/sbom.json".to_string(),
                signature_url: None,
                checksums: vec![],
                size_bytes: None,
                encoding: None,
                spec_version: None,
            }],
            created_date: Utc::now(),
            modified_date: Utc::now(),
            description: None,
            subject: None,
            deprecation: None,
            dependencies: vec![],
        }
    }

    #[tokio::test]
    async fn create_artifact_empty_name_fails() {
        let svc = make_service();
        let mut artifact = valid_artifact();
        artifact.name = "".to_string();
        let result = svc.create_artifact(artifact).await;
        assert!(matches!(result, Err(DomainError::Validation(_))));
    }

    #[tokio::test]
    async fn create_artifact_empty_formats_fails() {
        let svc = make_service();
        let mut artifact = valid_artifact();
        artifact.formats = vec![];
        let result = svc.create_artifact(artifact).await;
        assert!(matches!(result, Err(DomainError::Validation(_))));
    }

    #[tokio::test]
    async fn create_artifact_invalid_format_url_fails() {
        let svc = make_service();
        let mut artifact = valid_artifact();
        artifact.formats[0].url = "not-a-url".to_string();
        let result = svc.create_artifact(artifact).await;
        assert!(matches!(result, Err(DomainError::Validation(_))));
    }

    #[tokio::test]
    async fn create_artifact_valid_succeeds_and_assigns_uuid() {
        let svc = make_service();
        let artifact = valid_artifact();
        let created = svc.create_artifact(artifact).await.unwrap();
        assert_ne!(created.uuid, Uuid::nil());
    }

    #[tokio::test]
    async fn get_artifact_not_found_returns_none() {
        let svc = make_service();
        assert!(svc.get_artifact(&Uuid::new_v4()).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn deprecate_artifact_not_found_returns_not_found_error() {
        use crate::domain::common::deprecation::{Deprecation, DeprecationState};
        let svc = make_service();
        let dep = Deprecation {
            state: DeprecationState::Deprecated,
            reason: None,
            announced_date: None,
            effective_date: None,
            replacement_identifiers: vec![],
        };
        let result = svc.deprecate_artifact(&Uuid::new_v4(), dep).await;
        assert!(matches!(result, Err(DomainError::NotFound(_))));
    }
}

#[cfg(test)]
mod collection_service_tests {
    use crate::domain::collection::entity::{Collection, CollectionScope, UpdateReason};
    use crate::domain::collection::service::CollectionService;
    use crate::domain::common::error::DomainError;
    use crate::infrastructure::persistence::memory::collection_repository::InMemoryCollectionRepository;
    use chrono::Utc;
    use uuid::Uuid;

    fn make_service() -> CollectionService<InMemoryCollectionRepository> {
        CollectionService::new(InMemoryCollectionRepository::new())
    }

    fn valid_collection() -> Collection {
        Collection {
            uuid: Uuid::nil(),
            name: "Release 1.0".to_string(),
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

    #[tokio::test]
    async fn create_collection_empty_name_fails() {
        let svc = make_service();
        let mut col = valid_collection();
        col.name = "".to_string();
        assert!(matches!(
            svc.create_collection(col).await,
            Err(DomainError::Validation(_))
        ));
    }

    #[tokio::test]
    async fn create_collection_version_zero_fails() {
        let svc = make_service();
        let mut col = valid_collection();
        col.version = 0;
        assert!(matches!(
            svc.create_collection(col).await,
            Err(DomainError::Validation(_))
        ));
    }

    #[tokio::test]
    async fn create_collection_succeeds_and_assigns_uuid() {
        let svc = make_service();
        let created = svc.create_collection(valid_collection()).await.unwrap();
        assert_ne!(created.uuid, Uuid::nil());
    }

    #[tokio::test]
    async fn deprecate_collection_not_found_returns_not_found() {
        use crate::domain::common::deprecation::{Deprecation, DeprecationState};
        let svc = make_service();
        let dep = Deprecation {
            state: DeprecationState::Deprecated,
            reason: None,
            announced_date: None,
            effective_date: None,
            replacement_identifiers: vec![],
        };
        let result = svc.deprecate_collection(&Uuid::new_v4(), dep).await;
        assert!(matches!(result, Err(DomainError::NotFound(_))));
    }
}

#[cfg(test)]
mod component_service_tests {
    use crate::domain::common::error::DomainError;
    use crate::domain::component::entity::{Component, ComponentType};
    use crate::domain::component::service::ComponentService;
    use crate::infrastructure::persistence::memory::component_repository::InMemoryComponentRepository;
    use chrono::Utc;
    use uuid::Uuid;

    fn make_service() -> ComponentService<InMemoryComponentRepository> {
        ComponentService::new(InMemoryComponentRepository::new())
    }

    fn valid_component() -> Component {
        Component {
            uuid: Uuid::nil(),
            name: "log4j".to_string(),
            description: None,
            identifiers: vec![],
            component_type: ComponentType::Library,
            licenses: vec![],
            publisher: Some("Apache".to_string()),
            homepage_url: None,
            vcs_url: None,
            created_date: Utc::now(),
            modified_date: Utc::now(),
            deprecation: None,
            dependencies: vec![],
        }
    }

    #[tokio::test]
    async fn create_component_empty_name_fails() {
        let svc = make_service();
        let mut comp = valid_component();
        comp.name = "".to_string();
        assert!(matches!(
            svc.create_component(comp).await,
            Err(DomainError::Validation(_))
        ));
    }

    #[tokio::test]
    async fn create_component_invalid_homepage_url_fails() {
        let svc = make_service();
        let mut comp = valid_component();
        comp.homepage_url = Some("bad-url".to_string());
        assert!(matches!(
            svc.create_component(comp).await,
            Err(DomainError::Validation(_))
        ));
    }

    #[tokio::test]
    async fn create_component_valid_succeeds_and_assigns_uuid() {
        let svc = make_service();
        let created = svc.create_component(valid_component()).await.unwrap();
        assert_ne!(created.uuid, Uuid::nil());
    }

    #[tokio::test]
    async fn get_component_not_found_returns_none() {
        let svc = make_service();
        assert!(svc.get_component(&Uuid::new_v4()).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn deprecate_component_not_found_returns_not_found() {
        use crate::domain::common::deprecation::{Deprecation, DeprecationState};
        let svc = make_service();
        let dep = Deprecation {
            state: DeprecationState::Deprecated,
            reason: None,
            announced_date: None,
            effective_date: None,
            replacement_identifiers: vec![],
        };
        assert!(matches!(
            svc.deprecate_component(&Uuid::new_v4(), dep).await,
            Err(DomainError::NotFound(_))
        ));
    }
}

#[cfg(test)]
mod in_memory_repo_tests {
    use crate::domain::common::error::RepositoryError;
    use crate::domain::product::entity::{Product, Vendor};
    use crate::domain::product::repository::ProductRepository;
    use crate::infrastructure::persistence::memory::product_repository::InMemoryProductRepository;
    use chrono::Utc;
    use uuid::Uuid;

    fn make_repo() -> InMemoryProductRepository {
        InMemoryProductRepository::default()
    }

    fn test_product() -> Product {
        Product {
            uuid: Uuid::new_v4(),
            name: "Test".to_string(),
            description: None,
            identifiers: vec![],
            vendor: Vendor {
                name: "Tester Corp".to_string(),
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

    #[tokio::test]
    async fn update_non_existent_returns_not_found() {
        let repo = make_repo();
        let product = test_product();
        let result = repo.update(&product).await;
        assert!(matches!(result, Err(RepositoryError::NotFound)));
    }

    #[tokio::test]
    async fn save_then_update_succeeds() {
        let repo = make_repo();
        let mut product = test_product();
        repo.save(&product).await.unwrap();
        product.name = "Updated".to_string();
        let result = repo.update(&product).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn find_by_uuid_returns_none_for_missing() {
        let repo = make_repo();
        let result = repo.find_by_uuid(&Uuid::new_v4()).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn find_by_uuid_returns_saved_product() {
        let repo = make_repo();
        let product = test_product();
        let uuid = product.uuid;
        repo.save(&product).await.unwrap();
        let fetched = repo.find_by_uuid(&uuid).await.unwrap();
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().uuid, uuid);
    }

    #[tokio::test]
    async fn default_impl_works() {
        // M2 fix: Default impl should be available
        let _repo = InMemoryProductRepository::default();
    }
}
