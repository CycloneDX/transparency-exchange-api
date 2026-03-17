#[cfg(test)]
mod tests {
    use testcontainers_modules::postgres::Postgres;
    use testcontainers_modules::testcontainers::runners::AsyncRunner;
    use testcontainers_modules::testcontainers::ContainerAsync;
    use sqlx::PgPool;
    use tea_server::infrastructure::persistence::postgres::product_repository::PostgresProductRepository;
    use tea_server::domain::product::service::ProductService;
    use tea_server::domain::product::entity::{Product, Vendor};
    use tea_server::domain::common::deprecation::{Deprecation, DeprecationState};
    use tea_server::domain::common::error::DomainError;
    use chrono::Utc;
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

    // M7 fix: return type correctly uses ContainerAsync<Postgres> — not the IntoContainerPort
    // trait object that caused the type mismatch in the previous testcontainers-modules version.
    async fn make_pool() -> (ContainerAsync<Postgres>, PgPool) {
        let container = Postgres::default()
            .start()
            .await
            .expect("failed to start postgres container");
        let connection_string = format!(
            "postgres://postgres:postgres@127.0.0.1:{}/postgres",
            container.get_host_port_ipv4(5432).await.expect("failed to get port")
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

        assert_eq!(retrieved.expect("product should exist").name, "Test Product");
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

        let deprecated = service
            .deprecate_product(&created.uuid, deprecation)
            .await
            .unwrap();

        // Fetch again from Postgres — deprecation must survive round-trip (C3 fix)
        let fetched = service.get_product(&created.uuid).await.unwrap().unwrap();
        assert!(fetched.deprecation.is_some(), "C3 check: deprecation state should persist in Postgres");
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
        repo.save(&product).await.expect("first save should succeed");

        let err = repo
            .save(&product) // same UUID
            .await
            .expect_err("duplicate UUID should return Conflict");

        assert!(
            matches!(err, RepositoryError::Conflict),
            "H4 check: expected Conflict, got: {err:?}"
        );
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
