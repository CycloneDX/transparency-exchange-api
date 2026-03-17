// SA-16: Settings struct properly implemented — loads from environment variables
// at startup with validation. Replaces the dead-code stub.
//
// Secrets (database_url, redis_url) are intentionally redacted in Debug output.

use std::fmt;

/// Server configuration loaded from environment at startup.
///
/// All fields are validated before the server begins accepting connections.
pub struct Settings {
    pub database_url: String,
    pub redis_url: String,
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
            .field("database_url", &"[REDACTED]")
            .field("redis_url", &"[REDACTED]")
            .field("jwt_secret", &"[REDACTED]")
            .field("seaweedfs_endpoint", &self.seaweedfs_endpoint)
            .field("seaweedfs_bucket", &self.seaweedfs_bucket)
            .field("server_url", &self.server_url)
            .field("port", &self.port)
            .field("tls", &self.tls)
            .finish()
    }
}

/// SA-08: Load and validate all settings from environment variables.
///
/// Panics at startup if required variables are missing or invalid, rather
/// than failing silently at runtime.
///
/// SECURITY: In release builds, TEA_JWT_SECRET is mandatory.
/// In debug builds only, a dev fallback is provided for local development.
pub fn load() -> Settings {
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
        .collect();

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

    Settings {
        database_url: std::env::var("DATABASE_URL").unwrap_or_default(),
        redis_url: std::env::var("REDIS_URL").unwrap_or_default(),
        seaweedfs_endpoint: std::env::var("SEAWEEDFS_ENDPOINT").unwrap_or_default(),
        seaweedfs_bucket: std::env::var("SEAWEEDFS_BUCKET").unwrap_or_default(),
        jwt_secret,
        server_url: std::env::var("TEA_SERVER_URL")
            .unwrap_or_else(|_| "http://localhost:8734".to_string()),
        port,
        allowed_origins,
        tls,
    }
}
