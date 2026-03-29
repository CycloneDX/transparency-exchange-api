// SA-16: Settings struct properly implemented — loads from environment variables
// at startup with validation. Replaces the dead-code stub.
//
// Secrets (database_url, jwt_secret) are intentionally redacted in Debug output.

use std::fmt;
use url::Url;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PersistenceBackend {
    Memory,
    Postgres,
}

impl PersistenceBackend {
    fn from_env() -> Self {
        match std::env::var("TEA_PERSISTENCE_BACKEND")
            .unwrap_or_else(|_| "memory".to_string())
            .trim()
            .to_ascii_lowercase()
            .as_str()
        {
            "memory" => Self::Memory,
            "postgres" => Self::Postgres,
            other => panic!("TEA_PERSISTENCE_BACKEND must be 'memory' or 'postgres', got: {other}"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GrpcConfig {
    pub enabled: bool,
    pub port: u16,
    pub publisher_enabled: bool,
}

/// Server configuration loaded from environment at startup.
///
/// All fields are validated before the server begins accepting connections.
pub struct Settings {
    pub persistence_backend: PersistenceBackend,
    pub database_url: String,
    pub seaweedfs_endpoint: String,
    pub seaweedfs_bucket: String,
    /// JWT signing secret. Minimum 32 bytes enforced at load time.
    pub jwt_secret: String,
    /// Server base URL for discovery responses.
    pub server_url: String,
    /// Listening port.
    pub port: u16,
    /// Allowed CORS origins (comma-separated). Empty = no CORS header set.
    pub allowed_origins: Vec<String>,
    /// Per-request server timeout in seconds.
    pub request_timeout_secs: u64,
    /// Background cleanup interval for in-memory rate limiter state.
    pub rate_limit_cleanup_secs: u64,
    /// gRPC listener configuration.
    pub grpc: GrpcConfig,
    /// TLS configuration for HTTPS and mTLS.
    pub tls: TlsConfig,
}

/// TLS configuration for server security.
///
/// Supports:
/// - TLS termination (HTTPS)
/// - mTLS (mutual TLS) with client certificate validation
///
/// When `client_ca_path` is set, clients must present valid certificates
/// signed by the CA for authentication.
#[derive(Clone, Debug, Default)]
pub struct TlsConfig {
    /// Path to server certificate file (PEM format).
    /// Required for TLS. Environment: TEA_TLS_CERT_PATH
    pub cert_path: Option<String>,
    /// Path to server private key file (PEM format).
    /// Required for TLS. Environment: TEA_TLS_KEY_PATH
    pub key_path: Option<String>,
    /// Path to CA certificate for client certificate validation.
    /// When set, enables mTLS (mutual TLS). Environment: TEA_TLS_CLIENT_CA_PATH
    pub client_ca_path: Option<String>,
    /// Whether TLS is enabled (true if cert_path and key_path are both set).
    pub enabled: bool,
    /// Whether mTLS is enabled (true if client_ca_path is also set).
    pub mtls_enabled: bool,
}

// SA-08: Redact credentials from Debug output so they don't appear in logs.
impl fmt::Debug for Settings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Settings")
            .field("persistence_backend", &self.persistence_backend)
            .field("database_url", &"[REDACTED]")
            .field("jwt_secret", &"[REDACTED]")
            .field("seaweedfs_endpoint", &self.seaweedfs_endpoint)
            .field("seaweedfs_bucket", &self.seaweedfs_bucket)
            .field("server_url", &self.server_url)
            .field("port", &self.port)
            .field("request_timeout_secs", &self.request_timeout_secs)
            .field("rate_limit_cleanup_secs", &self.rate_limit_cleanup_secs)
            .field("grpc", &self.grpc)
            .field("tls", &self.tls)
            .finish()
    }
}

fn parse_bool_env(name: &str, default: bool) -> bool {
    match std::env::var(name) {
        Ok(value) => match value.trim().to_ascii_lowercase().as_str() {
            "1" | "true" | "yes" | "on" => true,
            "0" | "false" | "no" | "off" => false,
            other => panic!("{name} must be a boolean value, got: {other}"),
        },
        Err(_) => default,
    }
}

fn is_local_host(host: &str) -> bool {
    matches!(host, "localhost" | "127.0.0.1" | "::1")
}

fn validate_public_url(field: &str, value: &str, require_https: bool) {
    let parsed = Url::parse(value)
        .unwrap_or_else(|_| panic!("{field} must be an absolute http/https URL, got: {value}"));
    let scheme = parsed.scheme();
    let host = parsed.host_str().unwrap_or_default();

    if scheme != "http" && scheme != "https" {
        panic!("{field} must use http or https, got: {value}");
    }

    if require_https && scheme != "https" && !is_local_host(host) {
        panic!("{field} must use https outside local development, got: {value}");
    }
}

fn load_database_url() -> String {
    if let Ok(url) = std::env::var("TEA_DATABASE_URL") {
        return url;
    }

    if let Ok(url) = std::env::var("DATABASE_URL") {
        tracing::warn!(
            "DATABASE_URL is deprecated for tea-server runtime configuration; \
             use TEA_DATABASE_URL instead"
        );
        return url;
    }

    String::new()
}

/// SA-08: Load and validate all settings from environment variables.
///
/// Panics at startup if required variables are missing or invalid, rather
/// than failing silently at runtime.
///
/// SECURITY: In release builds, TEA_JWT_SECRET is mandatory.
/// In debug builds only, a dev fallback is provided for local development.
pub fn load() -> Settings {
    let persistence_backend = PersistenceBackend::from_env();
    let jwt_secret = std::env::var("TEA_JWT_SECRET").unwrap_or_else(|_| {
        #[cfg(debug_assertions)]
        {
            tracing::warn!(
                "TEA_JWT_SECRET not set — using insecure dev-only default. \
                 This fallback is ONLY available in debug builds."
            );
            "dev-only-insecure-secret-32-bytes--".to_string()
        }
        #[cfg(not(debug_assertions))]
        {
            panic!(
                "TEA_JWT_SECRET environment variable is required in release builds. \
                 Generate a secure secret with: openssl rand -hex 32"
            );
        }
    });

    if jwt_secret.len() < 32 {
        panic!(
            "TEA_JWT_SECRET must be at least 32 bytes. Got {} bytes. \
             Generate one with: openssl rand -hex 32",
            jwt_secret.len()
        );
    }

    let port_str = std::env::var("TEA_PORT").unwrap_or_else(|_| "8734".to_string());
    let port: u16 = port_str
        .parse()
        .unwrap_or_else(|_| panic!("TEA_PORT must be a valid port number, got: {port_str}"));

    let allowed_origins = std::env::var("TEA_ALLOWED_ORIGINS")
        .unwrap_or_default()
        .split(',')
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim().to_string())
        .collect::<Vec<_>>();

    let request_timeout_secs = std::env::var("TEA_REQUEST_TIMEOUT_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(30);
    let rate_limit_cleanup_secs = std::env::var("TEA_RATE_LIMIT_CLEANUP_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(300);
    let grpc_enabled = parse_bool_env("TEA_GRPC_ENABLED", false);
    let grpc_port_str = std::env::var("TEA_GRPC_PORT").unwrap_or_else(|_| "50051".to_string());
    let grpc_port: u16 = grpc_port_str.parse().unwrap_or_else(|_| {
        panic!("TEA_GRPC_PORT must be a valid port number, got: {grpc_port_str}")
    });
    let grpc_publisher_enabled = parse_bool_env("TEA_GRPC_PUBLISHER_ENABLED", false);

    // ── TLS Configuration ────────────────────────────────────────────────────
    let cert_path = std::env::var("TEA_TLS_CERT_PATH").ok();
    let key_path = std::env::var("TEA_TLS_KEY_PATH").ok();
    let client_ca_path = std::env::var("TEA_TLS_CLIENT_CA_PATH").ok();

    // Validate TLS configuration consistency
    let tls_enabled = cert_path.is_some() && key_path.is_some();
    if cert_path.is_some() != key_path.is_some() {
        panic!(
            "TEA_TLS_CERT_PATH and TEA_TLS_KEY_PATH must both be set for TLS. \
             Got cert={}, key={}",
            cert_path.is_some(),
            key_path.is_some()
        );
    }

    let mtls_enabled = tls_enabled && client_ca_path.is_some();
    if mtls_enabled {
        tracing::info!(
            client_ca = ?client_ca_path,
            "mTLS enabled - clients must present valid certificates"
        );
    } else if tls_enabled {
        tracing::info!("TLS enabled - HTTPS server without client certificate validation");
    }

    let tls = TlsConfig {
        cert_path,
        key_path,
        client_ca_path,
        enabled: tls_enabled,
        mtls_enabled,
    };

    let server_url =
        std::env::var("TEA_SERVER_URL").unwrap_or_else(|_| "http://localhost:8734".to_string());
    validate_public_url("TEA_SERVER_URL", &server_url, cfg!(not(debug_assertions)));

    if tls.enabled && !server_url.starts_with("https://") {
        panic!("TEA_SERVER_URL must use https when TLS is enabled");
    }

    for origin in &allowed_origins {
        if origin == "*" {
            panic!("TEA_ALLOWED_ORIGINS must not contain wildcard '*'");
        }
        validate_public_url("TEA_ALLOWED_ORIGINS", origin, cfg!(not(debug_assertions)));
    }

    if request_timeout_secs == 0 {
        panic!("TEA_REQUEST_TIMEOUT_SECS must be greater than 0");
    }

    if rate_limit_cleanup_secs == 0 {
        panic!("TEA_RATE_LIMIT_CLEANUP_SECS must be greater than 0");
    }

    if grpc_enabled && grpc_port == port {
        panic!("TEA_GRPC_PORT must differ from TEA_PORT when gRPC is enabled");
    }

    if grpc_publisher_enabled && !grpc_enabled {
        panic!("TEA_GRPC_PUBLISHER_ENABLED requires TEA_GRPC_ENABLED=true");
    }

    let database_url = load_database_url();
    if persistence_backend == PersistenceBackend::Postgres && database_url.trim().is_empty() {
        panic!("TEA_DATABASE_URL is required when TEA_PERSISTENCE_BACKEND=postgres");
    }

    Settings {
        persistence_backend,
        database_url,
        seaweedfs_endpoint: std::env::var("SEAWEEDFS_ENDPOINT").unwrap_or_default(),
        seaweedfs_bucket: std::env::var("SEAWEEDFS_BUCKET").unwrap_or_default(),
        jwt_secret,
        server_url,
        port,
        allowed_origins,
        request_timeout_secs,
        rate_limit_cleanup_secs,
        grpc: GrpcConfig {
            enabled: grpc_enabled,
            port: grpc_port,
            publisher_enabled: grpc_publisher_enabled,
        },
        tls,
    }
}

#[cfg(test)]
mod tests {
    use super::{load, load_database_url, parse_bool_env};
    use std::sync::{Mutex, OnceLock};

    static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    fn env_lock() -> &'static Mutex<()> {
        ENV_LOCK.get_or_init(|| Mutex::new(()))
    }

    fn with_env_vars<F>(vars: &[(&str, Option<&str>)], test: F)
    where
        F: FnOnce(),
    {
        let _guard = env_lock().lock().unwrap();
        let saved = vars
            .iter()
            .map(|(key, _)| ((*key).to_string(), std::env::var(key).ok()))
            .collect::<Vec<_>>();

        for (key, value) in vars {
            match value {
                Some(value) => std::env::set_var(key, value),
                None => std::env::remove_var(key),
            }
        }

        test();

        for (key, value) in saved {
            match value {
                Some(value) => std::env::set_var(&key, value),
                None => std::env::remove_var(&key),
            }
        }
    }

    #[test]
    fn load_database_url_prefers_tea_database_url() {
        with_env_vars(
            &[
                ("TEA_DATABASE_URL", Some("postgres://canonical")),
                ("DATABASE_URL", Some("postgres://legacy")),
            ],
            || {
                assert_eq!(load_database_url(), "postgres://canonical");
            },
        );
    }

    #[test]
    fn load_database_url_falls_back_to_legacy_database_url() {
        with_env_vars(
            &[
                ("TEA_DATABASE_URL", None),
                ("DATABASE_URL", Some("postgres://legacy")),
            ],
            || {
                assert_eq!(load_database_url(), "postgres://legacy");
            },
        );
    }

    #[test]
    fn load_uses_canonical_database_url_for_settings() {
        with_env_vars(
            &[
                ("TEA_PERSISTENCE_BACKEND", Some("memory")),
                ("TEA_DATABASE_URL", Some("postgres://canonical")),
                ("DATABASE_URL", Some("postgres://legacy")),
                (
                    "TEA_JWT_SECRET",
                    Some("dev-only-insecure-secret-32-bytes--"),
                ),
                ("TEA_SERVER_URL", Some("http://localhost:8734")),
                ("TEA_ALLOWED_ORIGINS", None),
                ("TEA_TLS_CERT_PATH", None),
                ("TEA_TLS_KEY_PATH", None),
                ("TEA_TLS_CLIENT_CA_PATH", None),
            ],
            || {
                let settings = load();
                assert_eq!(settings.database_url, "postgres://canonical");
            },
        );
    }

    #[test]
    fn load_requires_database_url_for_postgres_backend() {
        with_env_vars(
            &[
                ("TEA_PERSISTENCE_BACKEND", Some("postgres")),
                ("TEA_DATABASE_URL", None),
                ("DATABASE_URL", None),
                (
                    "TEA_JWT_SECRET",
                    Some("dev-only-insecure-secret-32-bytes--"),
                ),
                ("TEA_SERVER_URL", Some("http://localhost:8734")),
                ("TEA_ALLOWED_ORIGINS", None),
                ("TEA_TLS_CERT_PATH", None),
                ("TEA_TLS_KEY_PATH", None),
                ("TEA_TLS_CLIENT_CA_PATH", None),
            ],
            || {
                let panic = std::panic::catch_unwind(load).expect_err("load should panic");
                let message = panic_message(panic);
                assert!(message.contains("TEA_DATABASE_URL is required"));
            },
        );
    }

    #[test]
    fn parse_bool_env_accepts_truthy_and_falsey_values() {
        with_env_vars(
            &[
                ("TEA_BOOL_TRUE", Some("true")),
                ("TEA_BOOL_FALSE", Some("off")),
            ],
            || {
                assert!(parse_bool_env("TEA_BOOL_TRUE", false));
                assert!(!parse_bool_env("TEA_BOOL_FALSE", true));
                assert!(parse_bool_env("TEA_BOOL_MISSING", true));
            },
        );
    }

    #[test]
    fn load_exposes_grpc_defaults() {
        with_env_vars(
            &[
                ("TEA_PERSISTENCE_BACKEND", Some("memory")),
                ("TEA_DATABASE_URL", None),
                ("DATABASE_URL", None),
                (
                    "TEA_JWT_SECRET",
                    Some("dev-only-insecure-secret-32-bytes--"),
                ),
                ("TEA_SERVER_URL", Some("http://localhost:8734")),
                ("TEA_ALLOWED_ORIGINS", None),
                ("TEA_TLS_CERT_PATH", None),
                ("TEA_TLS_KEY_PATH", None),
                ("TEA_TLS_CLIENT_CA_PATH", None),
                ("TEA_GRPC_ENABLED", None),
                ("TEA_GRPC_PORT", None),
                ("TEA_GRPC_PUBLISHER_ENABLED", None),
            ],
            || {
                let settings = load();
                assert!(!settings.grpc.enabled);
                assert_eq!(settings.grpc.port, 50051);
                assert!(!settings.grpc.publisher_enabled);
            },
        );
    }

    fn panic_message(err: Box<dyn std::any::Any + Send>) -> String {
        if let Some(msg) = err.downcast_ref::<&str>() {
            (*msg).to_string()
        } else if let Some(msg) = err.downcast_ref::<String>() {
            msg.clone()
        } else {
            "unknown panic".to_string()
        }
    }
}
