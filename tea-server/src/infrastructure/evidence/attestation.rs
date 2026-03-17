use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use super::signer::{SignedEnvelope, Signer, SigningError};
use crate::domain::common::deprecation::Deprecation;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attestation {
    #[serde(rename = "_type")]
    pub type_: String,
    pub subject: Vec<Subject>,
    #[serde(rename = "predicateType")]
    pub predicate_type: String,
    pub predicate: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subject {
    pub name: String,
    /// SA-12: Maps algorithm name → hex digest. Currently SHA-256 of the
    /// canonical JSON predicate — must be replaced with a signed content
    /// digest of the actual resource before production use.
    pub digest: std::collections::HashMap<String, String>,
}

/// SA-12: Generate a deprecation attestation with a real SHA-256 subject digest.
///
/// The digest is over the canonical JSON of the predicate payload, providing
/// tamper-evidence for the attestation content. A production implementation
/// must sign the full attestation with a key from a trusted CA (e.g., Sigstore).
pub fn generate_deprecation_attestation(uuid: &Uuid, deprecation: &Deprecation) -> Attestation {
    let predicate = serde_json::json!({
        "deprecation": {
            "state": match deprecation.state {
                crate::domain::common::deprecation::DeprecationState::Active => "active",
                crate::domain::common::deprecation::DeprecationState::Deprecated => "deprecated",
                crate::domain::common::deprecation::DeprecationState::Retired => "retired",
                _ => "unspecified",
            },
            "reason": deprecation.reason,
            "replacement_identifiers": deprecation.replacement_identifiers
                .iter()
                .map(|id| id.id_value.clone())
                .collect::<Vec<_>>(),
            "effective_date": deprecation.effective_date.map(|d| d.to_rfc3339()).unwrap_or_default(),
        }
    });

    // SA-12: Compute SHA-256 digest of the canonical predicate JSON
    let predicate_bytes = serde_json::to_vec(&predicate).unwrap_or_default();
    let digest_hex = hex::encode(Sha256::digest(&predicate_bytes));
    let mut digest_map = std::collections::HashMap::new();
    digest_map.insert("sha256".to_string(), digest_hex);

    let subject = Subject {
        name: format!("urn:uuid:{uuid}"),
        digest: digest_map,
    };

    Attestation {
        type_: "https://in-toto.io/attestation/deprecation/v0.1".to_string(),
        subject: vec![subject],
        predicate_type: "https://cyclonedx.org/attestation/deprecation/v0.1".to_string(),
        predicate,
    }
}

/// Generate and sign a deprecation attestation.
///
/// This is the preferred API for production use. It:
/// 1. Creates the attestation with subject digest
/// 2. Signs it using the configured Sigstore signer
/// 3. Returns a DSSE envelope ready for storage/transmission
pub async fn generate_signed_deprecation_attestation(
    uuid: &Uuid,
    deprecation: &Deprecation,
    signer: &Signer,
) -> Result<SignedEnvelope, SigningError> {
    let attestation = generate_deprecation_attestation(uuid, deprecation);
    signer.sign(&attestation).await
}
