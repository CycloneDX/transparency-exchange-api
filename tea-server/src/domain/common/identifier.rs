use serde::{Deserialize, Serialize};

/// Type of identifier used to identify entities in the transparency ecosystem.
///
/// These identifier types correspond to the TEA specification and proto enum values.
/// The numeric values match the proto enum for direct conversion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[repr(i32)]
pub enum IdentifierType {
    Unspecified = 0,
    Tei = 1,
    Purl = 2,
    Cpe = 3,
    Swid = 4,
    Gav = 5,
    Gtin = 6,
    Gmn = 7,
    Udi = 8,
    Asin = 9,
    Hash = 10,
    Conformance = 11,
}

impl IdentifierType {
    /// Convert from proto i32 value to IdentifierType.
    /// Returns None for unknown values.
    pub fn from_proto(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::Unspecified),
            1 => Some(Self::Tei),
            2 => Some(Self::Purl),
            3 => Some(Self::Cpe),
            4 => Some(Self::Swid),
            5 => Some(Self::Gav),
            6 => Some(Self::Gtin),
            7 => Some(Self::Gmn),
            8 => Some(Self::Udi),
            9 => Some(Self::Asin),
            10 => Some(Self::Hash),
            11 => Some(Self::Conformance),
            _ => None,
        }
    }

    /// Convert to proto i32 value.
    pub fn to_proto(self) -> i32 {
        self as i32
    }

    /// Get human-readable name for the identifier type.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Unspecified => "unspecified",
            Self::Tei => "tei",
            Self::Purl => "purl",
            Self::Cpe => "cpe",
            Self::Swid => "swid",
            Self::Gav => "gav",
            Self::Gtin => "gtin",
            Self::Gmn => "gmn",
            Self::Udi => "udi",
            Self::Asin => "asin",
            Self::Hash => "hash",
            Self::Conformance => "conformance",
        }
    }
}

impl Default for IdentifierType {
    fn default() -> Self {
        Self::Unspecified
    }
}

/// An identifier for a TEA entity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Identifier {
    #[serde(rename = "idType")]
    pub id_type: IdentifierType,
    #[serde(rename = "idValue")]
    pub id_value: String,
}

impl Identifier {
    /// Create a new identifier with the given type and value.
    pub fn new(id_type: IdentifierType, id_value: String) -> Self {
        Self { id_type, id_value }
    }

    /// Create a PURL identifier.
    pub fn purl(value: impl Into<String>) -> Self {
        Self::new(IdentifierType::Purl, value.into())
    }

    /// Create a CPE identifier.
    pub fn cpe(value: impl Into<String>) -> Self {
        Self::new(IdentifierType::Cpe, value.into())
    }

    /// Create a SWID identifier.
    pub fn swid(value: impl Into<String>) -> Self {
        Self::new(IdentifierType::Swid, value.into())
    }

    /// Create a GAV (Maven) identifier.
    pub fn gav(value: impl Into<String>) -> Self {
        Self::new(IdentifierType::Gav, value.into())
    }

    /// Create a hash identifier.
    pub fn hash(value: impl Into<String>) -> Self {
        Self::new(IdentifierType::Hash, value.into())
    }

    /// Create a TEI identifier.
    pub fn tei(value: impl Into<String>) -> Self {
        Self::new(IdentifierType::Tei, value.into())
    }
}
