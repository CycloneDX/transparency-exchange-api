pub mod attestation;
pub mod signer;

pub use attestation::{
    generate_deprecation_attestation, generate_signed_deprecation_attestation, Attestation, Subject,
};
pub use signer::{verify, Signature, SignedEnvelope, Signer, SignerConfig, SigningMode};
