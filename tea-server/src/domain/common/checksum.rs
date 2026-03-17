use serde::{Deserialize, Serialize};

/// Cryptographic hash algorithm for checksums.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ChecksumAlgorithm {
    Unspecified,
    Md5,
    Sha1,
    Sha256,
    Sha384,
    Sha512,
    Sha3_256,
    Sha3_384,
    Sha3_512,
    Blake2b256,
    Blake2b384,
    Blake2b512,
    Blake3,
}

/// A cryptographic checksum.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Checksum {
    #[serde(rename = "algType")]
    pub alg_type: ChecksumAlgorithm,
    #[serde(rename = "algValue")]
    pub alg_value: String,
}
