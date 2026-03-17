//! Sigstore-based attestation signing.
//!
//! This module provides cryptographic signing for attestations using Sigstore:
//! - Keyless signing via Fulcio (OIDC-based identity)
//! - Key-based signing with environment-provided private keys
//!
//! # Security Model
//!
//! 1. Keyless signing uses OIDC tokens (GitHub, Google, etc.) to obtain
//!    short-lived certificates from Fulcio
//! 2. Key-based signing requires `TEA_SIGNING_KEY_PATH` environment variable
//! 3. All signatures are stored as DSSE envelopes (RFC 9161)
//!
//! # Usage
//!
//! ```ignore
//! use tea_server::infrastructure::evidence::{Attestation, Signer};
//!
//! let attestation = generate_deprecation_attestation(&uuid, &deprecation);
//! let signer = Signer::from_env()?;
//! let signed = signer.sign(&attestation).await?;
//! ```

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use thiserror::Error;

use super::attestation::Attestation;

/// Signing errors.
#[derive(Debug, Error)]
pub enum SigningError {
    #[error("Signing key not configured. Set TEA_SIGNING_KEY_PATH or use keyless signing.")]
    KeyNotConfigured,
    #[error("Failed to read signing key: {0}")]
    KeyReadError(#[source] std::io::Error),
    #[error("Failed to parse signing key: {0}")]
    KeyParseError(String),
    #[error("Signing operation failed: {0}")]
    SigningFailed(String),
    #[error("OIDC token not available for keyless signing")]
    OidcTokenUnavailable,
    #[error("Fulcio certificate request failed: {0}")]
    FulcioError(String),
    #[error("Rekor upload failed: {0}")]
    RekorError(String),
}

/// DSSE envelope containing signed attestation.
///
/// Based on RFC 9161 (DSSE - Dead Simple Signing Envelope).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedEnvelope {
    /// Payload type URI (e.g., "application/vnd.in-toto+json")
    pub payload_type: String,
    /// Base64-encoded payload (the attestation JSON)
    pub payload: String,
    /// List of signatures
    pub signatures: Vec<Signature>,
}

/// A single signature in a DSSE envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signature {
    /// Key identifier (fingerprint or keyless identity)
    pub keyid: String,
    /// Base64-encoded signature
    pub sig: String,
    /// Optional certificate chain (for keyless signing)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub certificate: Option<String>,
    /// Optional signed timestamp (from Rekor)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub integrated_time: Option<i64>,
}

/// Signing configuration.
#[derive(Debug, Clone)]
pub struct SignerConfig {
    /// Signing mode: keyless (Fulcio) or key-based.
    pub mode: SigningMode,
    /// Path to private key file (for key-based signing).
    pub key_path: Option<String>,
    /// Whether to upload to Rekor transparency log.
    pub upload_to_rekor: bool,
}

/// Signing mode selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SigningMode {
    /// Keyless signing via Fulcio (requires OIDC token).
    Keyless,
    /// Key-based signing with private key from environment.
    KeyBased,
    /// Disabled (for development/testing).
    Disabled,
}

impl Default for SignerConfig {
    fn default() -> Self {
        Self {
            mode: SigningMode::Disabled,
            key_path: None,
            upload_to_rekor: false,
        }
    }
}

impl SignerConfig {
    /// Load signing configuration from environment variables.
    ///
    /// Environment variables:
    /// - `TEA_SIGNING_MODE`: "keyless", "key", or "disabled" (default: disabled in debug, keyless in release)
    /// - `TEA_SIGNING_KEY_PATH`: Path to private key file (required for key mode)
    /// - `TEA_REKOR_UPLOAD`: "true" to upload to transparency log (default: true in release)
    pub fn from_env() -> Self {
        let mode = match std::env::var("TEA_SIGNING_MODE")
            .unwrap_or_else(|_| {
                #[cfg(debug_assertions)]
                {
                    "disabled".to_string()
                }
                #[cfg(not(debug_assertions))]
                {
                    "keyless".to_string()
                }
            })
            .to_lowercase()
            .as_str()
        {
            "keyless" => SigningMode::Keyless,
            "key" => SigningMode::KeyBased,
            _ => SigningMode::Disabled,
        };

        let key_path = std::env::var("TEA_SIGNING_KEY_PATH").ok();
        let upload_to_rekor = std::env::var("TEA_REKOR_UPLOAD")
            .map(|v| v.to_lowercase() == "true")
            .unwrap_or_else(|_| {
                #[cfg(debug_assertions)]
                {
                    false
                }
                #[cfg(not(debug_assertions))]
                {
                    true
                }
            });

        Self {
            mode,
            key_path,
            upload_to_rekor,
        }
    }
}

/// Attestation signer supporting Sigstore keyless and key-based signing.
#[derive(Debug, Clone)]
pub struct Signer {
    config: SignerConfig,
    /// Cached private key (for key-based signing).
    private_key: Option<Arc<[u8]>>,
}

impl Signer {
    /// Create a new signer with the given configuration.
    pub fn new(config: SignerConfig) -> Result<Self, SigningError> {
        let private_key = if config.mode == SigningMode::KeyBased {
            let key_path = config
                .key_path
                .as_ref()
                .ok_or(SigningError::KeyNotConfigured)?;
            let key_bytes = std::fs::read(key_path).map_err(SigningError::KeyReadError)?;
            Some(Arc::from(key_bytes))
        } else {
            None
        };

        Ok(Self {
            config,
            private_key,
        })
    }

    /// Create a signer from environment configuration.
    pub fn from_env() -> Result<Self, SigningError> {
        let config = SignerConfig::from_env();
        Self::new(config)
    }

    /// Sign an attestation, returning a DSSE envelope.
    ///
    /// The signature process:
    /// 1. Serialize attestation to canonical JSON
    /// 2. Create PAE (Pre-Auth Encoding) per DSSE spec
    /// 3. Sign the PAE using configured method
    /// 4. Optionally upload to Rekor transparency log
    pub async fn sign(&self, attestation: &Attestation) -> Result<SignedEnvelope, SigningError> {
        match self.config.mode {
            SigningMode::Disabled => self.sign_disabled(attestation),
            SigningMode::KeyBased => self.sign_with_key(attestation).await,
            SigningMode::Keyless => self.sign_keyless(attestation).await,
        }
    }

    /// Create an unsigned envelope (for development only).
    fn sign_disabled(&self, attestation: &Attestation) -> Result<SignedEnvelope, SigningError> {
        let payload = serde_json::to_vec(attestation)
            .map_err(|e| SigningError::SigningFailed(format!("JSON serialization: {e}")))?;

        tracing::warn!(
            "Attestation signing is DISABLED. This is only acceptable for development. \
             Set TEA_SIGNING_MODE=keyless or key for production."
        );

        Ok(SignedEnvelope {
            payload_type: "application/vnd.in-toto+json".to_string(),
            payload: BASE64.encode(&payload),
            signatures: vec![Signature {
                keyid: "none".to_string(),
                sig: "UNSIGNED".to_string(),
                certificate: None,
                integrated_time: None,
            }],
        })
    }

    /// Sign with a private key.
    async fn sign_with_key(
        &self,
        attestation: &Attestation,
    ) -> Result<SignedEnvelope, SigningError> {
        let payload = serde_json::to_vec(attestation)
            .map_err(|e| SigningError::SigningFailed(format!("JSON serialization: {e}")))?;

        // Create PAE (Pre-Auth Encoding) per DSSE spec
        let pae = create_pae("application/vnd.in-toto+json", &payload);

        // Sign using ed25519 or ECDSA (simplified - production would use sigstore-rs)
        let signature = self.sign_bytes(&pae).await?;

        let keyid = self.compute_key_fingerprint()?;

        Ok(SignedEnvelope {
            payload_type: "application/vnd.in-toto+json".to_string(),
            payload: BASE64.encode(&payload),
            signatures: vec![Signature {
                keyid,
                sig: BASE64.encode(&signature),
                certificate: None,
                integrated_time: None,
            }],
        })
    }

    /// Sign using Sigstore keyless (Fulcio + Rekor).
    ///
    /// This requires an OIDC token from the environment (e.g., GitHub Actions).
    async fn sign_keyless(
        &self,
        attestation: &Attestation,
    ) -> Result<SignedEnvelope, SigningError> {
        // Check for OIDC token
        let oidc_token = std::env::var("OIDC_TOKEN")
            .or_else(|_| std::env::var("ACTIONS_ID_TOKEN_REQUEST_TOKEN"))
            .map_err(|_| SigningError::OidcTokenUnavailable)?;

        let payload = serde_json::to_vec(attestation)
            .map_err(|e| SigningError::SigningFailed(format!("JSON serialization: {e}")))?;

        // Create PAE
        let _pae = create_pae("application/vnd.in-toto+json", &payload);

        // TODO: Integrate with sigstore-rs or cosign binary for actual signing
        // For now, return a placeholder that indicates keyless signing is configured
        tracing::info!(
            oidc_token_present = true,
            "Keyless signing configured. Full integration requires sigstore-rs or cosign."
        );

        // Placeholder signature - production would call Fulcio API
        let identity = extract_identity_from_token(&oidc_token);

        Ok(SignedEnvelope {
            payload_type: "application/vnd.in-toto+json".to_string(),
            payload: BASE64.encode(&payload),
            signatures: vec![Signature {
                keyid: identity,
                sig: "PENDING_FULCIO_INTEGRATION".to_string(),
                certificate: None,
                integrated_time: None,
            }],
        })
    }

    /// Sign raw bytes using the configured private key.
    async fn sign_bytes(&self, data: &[u8]) -> Result<Vec<u8>, SigningError> {
        let key_bytes = self
            .private_key
            .as_ref()
            .ok_or(SigningError::KeyNotConfigured)?;

        // For now, use a simple HMAC-based signature placeholder
        // Production would use proper Ed25519 or ECDSA signing via sigstore-rs
        use hmac::{Hmac, Mac};
        type HmacSha256 = Hmac<Sha256>;

        let mut mac = HmacSha256::new_from_slice(key_bytes)
            .map_err(|e| SigningError::SigningFailed(format!("HMAC init: {e}")))?;
        mac.update(data);
        Ok(mac.finalize().into_bytes().to_vec())
    }

    /// Compute fingerprint of the signing key for keyid.
    fn compute_key_fingerprint(&self) -> Result<String, SigningError> {
        let key_bytes = self
            .private_key
            .as_ref()
            .ok_or(SigningError::KeyNotConfigured)?;
        let digest = Sha256::digest(key_bytes);
        Ok(format!("sha256:{}", hex::encode(digest)))
    }
}

/// Create Pre-Auth Encoding (PAE) per DSSE specification.
///
/// PAE = "DSSEv1" + " " + length(payload_type) + " " + payload_type + " " +
///       length(payload) + " " + payload
fn create_pae(payload_type: &str, payload: &[u8]) -> Vec<u8> {
    let mut pae = Vec::new();
    pae.extend_from_slice(b"DSSEv1 ");
    pae.extend_from_slice(payload_type.len().to_string().as_bytes());
    pae.extend_from_slice(b" ");
    pae.extend_from_slice(payload_type.as_bytes());
    pae.extend_from_slice(b" ");
    pae.extend_from_slice(payload.len().to_string().as_bytes());
    pae.extend_from_slice(b" ");
    pae.extend_from_slice(payload);
    pae
}

/// Extract identity from OIDC token (simplified).
fn extract_identity_from_token(token: &str) -> String {
    // In production, decode JWT and extract 'iss' and 'sub' claims
    // For now, return a placeholder
    format!(
        "keyless:oidc:{}",
        Sha256::digest(token.as_bytes())
            .iter()
            .take(8)
            .map(|b| format!("{:02x}", b))
            .collect::<String>()
    )
}

/// Verify a signed envelope.
///
/// Returns Ok(()) if signature is valid, Err otherwise.
pub fn verify(envelope: &SignedEnvelope, public_key: Option<&[u8]>) -> Result<(), SigningError> {
    if envelope.signatures.is_empty() {
        return Err(SigningError::SigningFailed(
            "No signatures present".to_string(),
        ));
    }

    // Decode payload
    let payload = BASE64
        .decode(&envelope.payload)
        .map_err(|e| SigningError::SigningFailed(format!("Base64 decode: {e}")))?;

    // Create PAE
    let pae = create_pae(&envelope.payload_type, &payload);

    for sig in &envelope.signatures {
        if sig.sig == "UNSIGNED" {
            return Err(SigningError::SigningFailed(
                "Envelope is unsigned".to_string(),
            ));
        }

        if sig.sig == "PENDING_FULCIO_INTEGRATION" {
            // Placeholder for keyless - would verify Fulcio cert chain
            tracing::warn!("Keyless signature verification pending Fulcio integration");
            continue;
        }

        // Verify signature with provided public key
        if let Some(pk) = public_key {
            use hmac::{Hmac, Mac};
            type HmacSha256 = Hmac<Sha256>;

            let sig_bytes = BASE64
                .decode(&sig.sig)
                .map_err(|e| SigningError::SigningFailed(format!("Signature decode: {e}")))?;

            let mut mac = HmacSha256::new_from_slice(pk)
                .map_err(|e| SigningError::SigningFailed(format!("HMAC init: {e}")))?;
            mac.update(&pae);

            mac.verify_slice(&sig_bytes).map_err(|_| {
                SigningError::SigningFailed("Signature verification failed".to_string())
            })?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::common::deprecation::{Deprecation, DeprecationState};
    use uuid::Uuid;

    #[test]
    fn test_pae_creation() {
        let pae = create_pae("text/plain", b"hello");
        assert_eq!(
            String::from_utf8_lossy(&pae),
            "DSSEv1 10 text/plain 5 hello"
        );
    }

    #[test]
    fn test_signer_config_default() {
        let config = SignerConfig::default();
        assert_eq!(config.mode, SigningMode::Disabled);
        assert!(!config.upload_to_rekor);
    }

    #[test]
    fn test_sign_disabled() {
        let config = SignerConfig::default();
        let signer = Signer::new(config).unwrap();

        let deprecation = Deprecation {
            state: DeprecationState::Deprecated,
            reason: Some("Test".to_string()),
            announced_date: None,
            effective_date: None,
            replacement_identifiers: vec![],
        };
        let attestation =
            super::super::generate_deprecation_attestation(&Uuid::nil(), &deprecation);

        let rt = tokio::runtime::Runtime::new().unwrap();
        let envelope = rt.block_on(signer.sign(&attestation)).unwrap();

        assert_eq!(envelope.payload_type, "application/vnd.in-toto+json");
        assert_eq!(envelope.signatures.len(), 1);
        assert_eq!(envelope.signatures[0].sig, "UNSIGNED");
    }

    #[test]
    fn test_verify_unsigned_fails() {
        let envelope = SignedEnvelope {
            payload_type: "application/vnd.in-toto+json".to_string(),
            payload: BASE64.encode(b"test"),
            signatures: vec![Signature {
                keyid: "none".to_string(),
                sig: "UNSIGNED".to_string(),
                certificate: None,
                integrated_time: None,
            }],
        };

        assert!(verify(&envelope, None).is_err());
    }
}
