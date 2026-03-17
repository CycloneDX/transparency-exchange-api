//! mTLS (Mutual TLS) authentication middleware.
//!
//! When mTLS is enabled, this middleware extracts and validates client certificates
//! from the TLS connection, making the client identity available to handlers.
//!
//! # Security Model
//!
//! 1. TLS termination must occur at the application level (not reverse proxy)
//! 2. Client certificates are validated during TLS handshake by rustls
//! 3. This middleware extracts the validated certificate for authorization
//!
//! # Usage
//!
//! The middleware adds a `ClientCertificate` extension to the request,
//! which can be extracted by handlers for authorization decisions.

use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
};
use std::sync::Arc;

/// Client certificate information extracted from mTLS handshake.
///
/// This represents the identity of the client as verified by the TLS layer.
#[derive(Clone, Debug)]
pub struct ClientCertificate {
    /// Distinguished Name (DN) from the certificate subject.
    /// e.g., "CN=client-1,O=MyOrg,C=US"
    pub subject_dn: String,
    /// Certificate serial number (hex encoded).
    pub serial_number: String,
    /// Certificate issuer DN.
    pub issuer_dn: String,
    /// Certificate fingerprint (SHA-256, hex encoded).
    pub fingerprint: String,
}

/// Error returned when client certificate is required but not present.
#[derive(Debug)]
pub struct MissingClientCertificate;

impl IntoResponse for MissingClientCertificate {
    fn into_response(self) -> Response {
        (
            StatusCode::UNAUTHORIZED,
            axum::Json(serde_json::json!({
                "error": "Unauthorized",
                "message": "Client certificate required. This server requires mTLS authentication.",
                "status": 401
            })),
        )
            .into_response()
    }
}

/// Extractor for client certificate from request extensions.
///
/// Returns `MissingClientCertificate` error if no certificate is present.
/// This typically means mTLS is not enabled or the client didn't provide a cert.
///
/// NOTE: This extractor requires the `ClientCertificate` to be added to request
/// extensions by TLS layer middleware. Currently a placeholder for future implementation.
#[axum::async_trait]
impl<S> FromRequestParts<S> for ClientCertificate
where
    S: Send + Sync + 'static,
{
    type Rejection = MissingClientCertificate;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<ClientCertificate>()
            .cloned()
            .ok_or(MissingClientCertificate)
    }
}

/// Configuration for mTLS middleware.
#[derive(Clone, Debug)]
pub struct MtlsConfig {
    /// Whether mTLS is enabled.
    pub enabled: bool,
    /// Path to the CA certificate for client certificate validation.
    pub client_ca_path: Option<String>,
}

impl Default for MtlsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            client_ca_path: None,
        }
    }
}

impl From<crate::config::settings::TlsConfig> for MtlsConfig {
    fn from(tls: crate::config::settings::TlsConfig) -> Self {
        Self {
            enabled: tls.mtls_enabled,
            client_ca_path: tls.client_ca_path,
        }
    }
}

/// Helper to load TLS configuration for the server.
///
/// Returns `None` if TLS is not configured.
/// Returns the server config if TLS is enabled.
pub fn load_tls_server_config(
    tls_config: &crate::config::settings::TlsConfig,
) -> Result<Option<Arc<rustls::ServerConfig>>, Box<dyn std::error::Error + Send + Sync>> {
    if !tls_config.enabled {
        return Ok(None);
    }

    use rustls::server::WebPkiClientVerifier;
    use rustls_pemfile::{certs, private_key};
    use std::fs::File;
    use std::io::BufReader;

    // Load server certificate
    let cert_path = tls_config
        .cert_path
        .as_ref()
        .ok_or("TEA_TLS_CERT_PATH not set")?;
    let cert_file = File::open(cert_path)?;
    let mut cert_reader = BufReader::new(cert_file);
    let server_certs: Vec<_> = certs(&mut cert_reader).collect::<Result<Vec<_>, _>>()?;

    // Load server private key
    let key_path = tls_config
        .key_path
        .as_ref()
        .ok_or("TEA_TLS_KEY_PATH not set")?;
    let key_file = File::open(key_path)?;
    let mut key_reader = BufReader::new(key_file);
    let server_key = private_key(&mut key_reader)?.ok_or("No private key found in key file")?;

    // If mTLS is enabled, configure client certificate verification
    if tls_config.mtls_enabled {
        let client_ca_path = tls_config
            .client_ca_path
            .as_ref()
            .ok_or("TEA_TLS_CLIENT_CA_PATH not set for mTLS")?;

        let ca_file = File::open(client_ca_path)?;
        let mut ca_reader = BufReader::new(ca_file);

        let mut root_certs = rustls::RootCertStore::empty();
        let ca_certs: Vec<_> = certs(&mut ca_reader).collect::<Result<Vec<_>, _>>()?;
        root_certs.add_parsable_certificates(ca_certs);

        let client_verifier = WebPkiClientVerifier::builder(Arc::new(root_certs)).build()?;

        let config = rustls::ServerConfig::builder()
            .with_client_cert_verifier(client_verifier)
            .with_single_cert(server_certs, server_key)?;

        return Ok(Some(Arc::new(config)));
    }

    // TLS only (no mTLS)
    let config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(server_certs, server_key)?;

    Ok(Some(Arc::new(config)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mtls_config_default() {
        let config = MtlsConfig::default();
        assert!(!config.enabled);
        assert!(config.client_ca_path.is_none());
    }

    #[test]
    fn test_mtls_config_from_tls_config() {
        let tls = crate::config::settings::TlsConfig {
            cert_path: Some("/path/to/cert.pem".to_string()),
            key_path: Some("/path/to/key.pem".to_string()),
            client_ca_path: Some("/path/to/ca.pem".to_string()),
            enabled: true,
            mtls_enabled: true,
        };

        let mtls: MtlsConfig = tls.into();
        assert!(mtls.enabled);
        assert_eq!(mtls.client_ca_path, Some("/path/to/ca.pem".to_string()));
    }
}
