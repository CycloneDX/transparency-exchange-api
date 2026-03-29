#![allow(clippy::result_large_err)]

use std::collections::HashMap;

use tonic::{Request, Response, Status};

use crate::gen::tea::v1::{self as proto, discovery_service_server::DiscoveryService};
use crate::infrastructure::middleware::rate_limit::RateLimitConfig;

pub struct DiscoveryGrpcService {
    base_url: String,
    api_base_url: String,
}

impl DiscoveryGrpcService {
    pub fn new(base_url: impl Into<String>) -> Self {
        let base_url = base_url.into().trim_end_matches('/').to_string();
        let api_base_url = format!("{base_url}/v1");
        Self {
            base_url,
            api_base_url,
        }
    }
}

#[tonic::async_trait]
impl DiscoveryService for DiscoveryGrpcService {
    async fn discover(
        &self,
        request: Request<proto::DiscoverRequest>,
    ) -> Result<Response<proto::DiscoverResponse>, Status> {
        let tei = request.into_inner().tei;
        let parsed = parse_tei(&tei)?;
        if parsed.r#type != proto::TeiType::Uuid as i32 {
            return Err(Status::unimplemented(
                "only UUID-based TEIs are currently supported by the reference server",
            ));
        }

        Ok(Response::new(proto::DiscoverResponse {
            product_release_uuid: parsed.unique_id.clone(),
            server_url: self.base_url.clone(),
            supported_versions: vec!["1.0.0".to_string()],
            tei: tei.clone(),
            parsed_tei: Some(parsed.clone()),
            all_teis: vec![tei],
            identifiers: vec![],
            product_release: Some(proto::ProductReleaseMetadata {
                uuid: parsed.unique_id,
                version: String::new(),
                product_name: String::new(),
                vendor_name: String::new(),
                release_date: None,
                artifact_count: 0,
            }),
            authentication_required: false,
            auth_methods: vec![],
        }))
    }

    async fn get_well_known(
        &self,
        _request: Request<proto::GetWellKnownRequest>,
    ) -> Result<Response<proto::WellKnownResponse>, Status> {
        Ok(Response::new(proto::WellKnownResponse {
            schema_version: 1,
            endpoints: vec![proto::Endpoint {
                url: self.api_base_url.clone(),
                versions: vec!["1.0.0".to_string()],
                priority: 1.0,
            }],
        }))
    }

    async fn health_check(
        &self,
        request: Request<proto::HealthCheckRequest>,
    ) -> Result<Response<proto::HealthCheckResponse>, Status> {
        let service = request.into_inner().service;
        let mut details = HashMap::new();
        details.insert("base_url".to_string(), self.base_url.clone());
        details.insert(
            "service".to_string(),
            if service.trim().is_empty() {
                "overall".to_string()
            } else {
                service
            },
        );

        Ok(Response::new(proto::HealthCheckResponse {
            status: proto::HealthStatus::Serving as i32,
            details,
        }))
    }

    async fn get_server_info(
        &self,
        _request: Request<proto::GetServerInfoRequest>,
    ) -> Result<Response<proto::ServerInfo>, Status> {
        let rate_limit = RateLimitConfig::from_env();

        Ok(Response::new(proto::ServerInfo {
            name: "tea-server".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            spec_versions: vec!["1.0.0".to_string()],
            description: "Reference implementation of the Transparency Exchange API".to_string(),
            operator_contact: String::new(),
            tos_url: String::new(),
            privacy_url: String::new(),
            documentation_url: "https://cyclonedx.github.io/transparency-exchange-api/".to_string(),
            capabilities: Some(proto::ServerCapabilities {
                consumer_api: true,
                publisher_api: false,
                insights_api: false,
                collection_signing: false,
                sigstore_integration: false,
                checksum_search: true,
                cel_queries: false,
                streaming_downloads: false,
                artifact_types: vec![],
            }),
            rate_limits: Some(proto::RateLimitInfo {
                unauthenticated_rpm: rate_limit.requests_per_minute as i32,
                authenticated_rpm: rate_limit.requests_per_minute as i32,
                max_bandwidth_bps: 0,
            }),
        }))
    }
}

fn parse_tei(tei: &str) -> Result<proto::ParsedTei, Status> {
    let parts = tei.split(':').collect::<Vec<_>>();
    if parts.len() < 5 || parts[0] != "urn" || parts[1] != "tei" {
        return Err(Status::invalid_argument(
            "tei must be a valid urn:tei:* identifier",
        ));
    }

    let tei_type = match parts[2] {
        "uuid" => proto::TeiType::Uuid,
        "purl" => proto::TeiType::Purl,
        "swid" => proto::TeiType::Swid,
        "hash" => proto::TeiType::Hash,
        "eanupc" => proto::TeiType::Eanupc,
        "gtin" => proto::TeiType::Gtin,
        "asin" => proto::TeiType::Asin,
        "udi" => proto::TeiType::Udi,
        _ => return Err(Status::invalid_argument("unsupported TEI type")),
    };

    Ok(proto::ParsedTei {
        raw: tei.to_string(),
        r#type: tei_type as i32,
        domain: parts[3].to_string(),
        unique_id: parts[4..].join(":"),
    })
}

#[cfg(test)]
mod tests {
    use tonic::Request;

    use crate::gen::tea::v1::discovery_service_server::DiscoveryService;

    use super::DiscoveryGrpcService;

    #[tokio::test]
    async fn discover_resolves_uuid_tei() {
        let service = DiscoveryGrpcService::new("https://tea.example.com");
        let response = service
            .discover(Request::new(crate::gen::tea::v1::DiscoverRequest {
                tei: "urn:tei:uuid:tea.example.com:123e4567-e89b-12d3-a456-426614174000"
                    .to_string(),
            }))
            .await
            .unwrap()
            .into_inner();

        assert_eq!(
            response.product_release_uuid,
            "123e4567-e89b-12d3-a456-426614174000"
        );
        assert_eq!(response.server_url, "https://tea.example.com");
        assert!(response.parsed_tei.is_some());
    }

    #[tokio::test]
    async fn get_well_known_uses_v1_endpoint() {
        let service = DiscoveryGrpcService::new("https://tea.example.com/");
        let response = service
            .get_well_known(Request::new(crate::gen::tea::v1::GetWellKnownRequest {}))
            .await
            .unwrap()
            .into_inner();

        assert_eq!(response.schema_version, 1);
        assert_eq!(response.endpoints.len(), 1);
        assert_eq!(response.endpoints[0].url, "https://tea.example.com/v1");
    }
}
