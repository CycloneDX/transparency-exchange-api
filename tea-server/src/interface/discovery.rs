use axum::{extract::State, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// TEA server discovery index, served at `GET /.well-known/tea`.
///
/// Clients use this to discover available API versions and endpoints
/// before making any TEA API calls, per TEA spec §3 (Discovery).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeaServerIndex {
    pub spec_version: String,
    pub server_url: String,
    pub supported_versions: Vec<String>,
    pub endpoints: Vec<TeaEndpoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeaEndpoint {
    pub version: String,
    pub url: String,
    pub description: Option<String>,
    pub active: bool,
}

/// SA-15: Uses AppState.base_url — no longer re-reads env var on every request.
pub async fn well_known_tea(
    State(base_url): State<Arc<String>>,
) -> impl IntoResponse {
    let index = TeaServerIndex {
        spec_version: "1.0.0".to_string(),
        server_url: base_url.as_ref().clone(),
        supported_versions: vec!["1.0".to_string()],
        endpoints: vec![TeaEndpoint {
            version: "1.0".to_string(),
            url: format!("{base_url}/v1"),
            description: Some(
                "Transparency Exchange API v1 — products, components, artifacts, collections"
                    .to_string(),
            ),
            active: true,
        }],
    };

    Json(index)
}
