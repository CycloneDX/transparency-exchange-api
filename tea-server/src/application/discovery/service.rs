use serde::{Deserialize, Serialize};

use crate::domain::common::error::DomainError;

#[derive(Serialize, Deserialize)]
pub struct WellKnownResponse {
    #[serde(rename = "schemaVersion")]
    pub schema_version: i32,
    pub endpoints: Vec<Endpoint>,
}

#[derive(Serialize, Deserialize)]
pub struct Endpoint {
    pub url: String,
    pub versions: Vec<String>,
    pub priority: Option<i32>,
    pub description: Option<String>,
}

#[derive(Default)]
pub struct DiscoveryApplicationService;

impl DiscoveryApplicationService {
    pub fn new() -> Self {
        Self
    }

    pub async fn discover(&self, tei: &str) -> Result<Option<String>, DomainError> {
        // M3 fix: use next_back() instead of last() on the DoubleEndedIterator
        // returned by split() — avoids needlessly iterating the entire iterator.
        if let Some(uuid_part) = tei
            .strip_prefix("urn:tei:uuid:")
            .and_then(|s| s.split(':').next_back())
        {
            if uuid::Uuid::parse_str(uuid_part).is_ok() {
                return Ok(Some(uuid_part.to_string()));
            }
        }
        Ok(None)
    }

    pub async fn get_well_known(&self) -> Result<WellKnownResponse, DomainError> {
        let base_url =
            std::env::var("TEA_SERVER_URL").unwrap_or_else(|_| "http://localhost:8734".to_string());
        Ok(WellKnownResponse {
            schema_version: 1,
            endpoints: vec![Endpoint {
                url: format!("{base_url}/v1"),
                versions: vec!["1.0.0".to_string()],
                priority: Some(1),
                description: Some("Transparency Exchange API v1".to_string()),
            }],
        })
    }
}
