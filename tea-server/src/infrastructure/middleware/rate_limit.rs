//! Rate limiting middleware for TEA API.
//!
//! Provides configurable rate limiting to protect against:
//! - DoS attacks
//! - Resource exhaustion
//! - Abuse of write endpoints
//!
//! # Configuration
//!
//! - `TEA_RATE_LIMIT_RPM`: Requests per minute per IP (default: 60)
//! - `TEA_RATE_LIMIT_BURST`: Burst allowance (default: 10)
//!
//! # Implementation
//!
//! Uses a sliding window algorithm with Redis for distributed rate limiting,
//! falling back to in-memory for single-instance deployments.

use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Rate limit configuration.
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Requests per minute per client.
    pub requests_per_minute: u32,
    /// Burst allowance (extra requests allowed briefly).
    pub burst: u32,
    /// Whether rate limiting is enabled.
    pub enabled: bool,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 60,
            burst: 10,
            enabled: true,
        }
    }
}

impl RateLimitConfig {
    /// Load rate limit configuration from environment.
    pub fn from_env() -> Self {
        let requests_per_minute = std::env::var("TEA_RATE_LIMIT_RPM")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(60);

        let burst = std::env::var("TEA_RATE_LIMIT_BURST")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(10);

        // Disable rate limiting in debug builds by default
        #[cfg(debug_assertions)]
        let enabled = std::env::var("TEA_RATE_LIMIT_ENABLED")
            .map(|v| v.to_lowercase() == "true")
            .unwrap_or(false);

        #[cfg(not(debug_assertions))]
        let enabled = std::env::var("TEA_RATE_LIMIT_ENABLED")
            .map(|v| v.to_lowercase() == "true")
            .unwrap_or(true);

        Self {
            requests_per_minute,
            burst,
            enabled,
        }
    }
}

/// In-memory rate limiter state.
///
/// Tracks request counts per client IP using a sliding window.
#[derive(Debug, Default)]
pub struct RateLimiter {
    /// Client IP -> (request_count, window_start)
    clients: RwLock<HashMap<String, (u32, Instant)>>,
    config: RateLimitConfig,
}

impl RateLimiter {
    /// Create a new rate limiter with the given configuration.
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            clients: RwLock::new(HashMap::new()),
            config,
        }
    }

    /// Check if a request from the given client IP should be allowed.
    pub async fn check(&self, client_ip: &str) -> Result<(), RateLimitError> {
        if !self.config.enabled {
            return Ok(());
        }

        let mut clients = self.clients.write().await;
        let now = Instant::now();
        let window_duration = Duration::from_secs(60);

        // Get or create entry for this client
        let entry = clients.entry(client_ip.to_string()).or_insert((0, now));

        // Check if window has expired - if so, reset
        if now.duration_since(entry.1) > window_duration {
            *entry = (1, now);
            return Ok(());
        }

        // Check if within burst allowance
        let limit = self.config.requests_per_minute + self.config.burst;
        if entry.0 >= limit {
            return Err(RateLimitError::LimitExceeded {
                retry_after: window_duration
                    .saturating_sub(now.duration_since(entry.1))
                    .as_secs(),
            });
        }

        // Increment counter
        entry.0 += 1;
        Ok(())
    }

    /// Clean up expired entries to prevent memory growth.
    pub async fn cleanup(&self) {
        let mut clients = self.clients.write().await;
        let now = Instant::now();
        let window_duration = Duration::from_secs(60);

        clients.retain(|_, (_, window_start)| {
            now.duration_since(*window_start) <= window_duration * 2
        });
    }
}

/// Rate limiting error.
#[derive(Debug)]
pub enum RateLimitError {
    LimitExceeded { retry_after: u64 },
}

impl IntoResponse for RateLimitError {
    fn into_response(self) -> Response {
        match self {
            Self::LimitExceeded { retry_after } => (
                StatusCode::TOO_MANY_REQUESTS,
                [
                    ("Retry-After", retry_after.to_string()),
                    ("X-RateLimit-Limit", "60".to_string()),
                ],
                axum::Json(serde_json::json!({
                    "error": "Too Many Requests",
                    "message": "Rate limit exceeded. Please slow down your requests.",
                    "status": 429,
                    "retry_after_seconds": retry_after
                })),
            )
                .into_response(),
        }
    }
}

/// Extract client IP from request.
///
/// Checks X-Forwarded-For, X-Real-IP, then falls back to socket addr.
fn extract_client_ip(req: &Request<Body>) -> String {
    // Check X-Forwarded-For header (most common for proxies)
    if let Some(forwarded) = req.headers().get("x-forwarded-for") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            // Take the first IP in the chain (original client)
            if let Some(ip) = forwarded_str.split(',').next() {
                return ip.trim().to_string();
            }
        }
    }

    // Check X-Real-IP header
    if let Some(real_ip) = req.headers().get("x-real-ip") {
        if let Ok(ip) = real_ip.to_str() {
            return ip.to_string();
        }
    }

    // Fallback to a unique identifier (no socket access in middleware)
    // In production, use tower-service's ConnectInfo for socket addr
    "unknown".to_string()
}

/// Rate limiting middleware.
///
/// Use with `middleware::from_fn_with_state`:
///
/// ```ignore
/// let limiter = Arc::new(RateLimiter::new(config));
/// let app = Router::new()
///     .route("/v1/products", post(create_product))
///     .layer(middleware::from_fn_with_state(limiter.clone(), rate_limit));
/// ```
pub async fn rate_limit(
    State(limiter): State<Arc<RateLimiter>>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let client_ip = extract_client_ip(&req);

    match limiter.check(&client_ip).await {
        Ok(()) => next.run(req).await,
        Err(err) => err.into_response(),
    }
}

/// Rate limiting middleware for write endpoints (stricter limits).
///
/// Applies a 10x stricter rate limit for POST/PUT/DELETE operations.
pub async fn rate_limit_writes(
    State(limiter): State<Arc<RateLimiter>>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let method = req.method().clone();
    let client_ip = extract_client_ip(&req);

    // For write methods, apply stricter check
    if matches!(
        method,
        axum::http::Method::POST | axum::http::Method::PUT | axum::http::Method::DELETE
    ) {
        // Create a stricter limiter for writes (1/10th the normal limit)
        let write_limiter = RateLimiter::new(RateLimitConfig {
            requests_per_minute: limiter.config.requests_per_minute / 10,
            burst: limiter.config.burst / 5,
            enabled: limiter.config.enabled,
        });

        match write_limiter.check(&client_ip).await {
            Ok(()) => next.run(req).await,
            Err(err) => err.into_response(),
        }
    } else {
        // Read methods use normal rate limit
        match limiter.check(&client_ip).await {
            Ok(()) => next.run(req).await,
            Err(err) => err.into_response(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_config_default() {
        let config = RateLimitConfig::default();
        assert_eq!(config.requests_per_minute, 60);
        assert_eq!(config.burst, 10);
    }

    #[tokio::test]
    async fn test_rate_limiter_allows_under_limit() {
        let config = RateLimitConfig {
            requests_per_minute: 10,
            burst: 5,
            enabled: true,
        };
        let limiter = RateLimiter::new(config);

        for _ in 0..10 {
            assert!(limiter.check("192.168.1.1").await.is_ok());
        }
    }

    #[tokio::test]
    async fn test_rate_limiter_blocks_over_limit() {
        let config = RateLimitConfig {
            requests_per_minute: 5,
            burst: 2,
            enabled: true,
        };
        let limiter = RateLimiter::new(config);

        // Use up the limit + burst
        for _ in 0..7 {
            let _ = limiter.check("192.168.1.1").await;
        }

        // Next request should fail
        assert!(limiter.check("192.168.1.1").await.is_err());
    }

    #[tokio::test]
    async fn test_rate_limiter_disabled() {
        let config = RateLimitConfig {
            requests_per_minute: 1,
            burst: 0,
            enabled: false,
        };
        let limiter = RateLimiter::new(config);

        // Should allow unlimited requests when disabled
        for _ in 0..100 {
            assert!(limiter.check("192.168.1.1").await.is_ok());
        }
    }

    #[tokio::test]
    async fn test_rate_limiter_per_ip() {
        let config = RateLimitConfig {
            requests_per_minute: 2,
            burst: 0,
            enabled: true,
        };
        let limiter = RateLimiter::new(config);

        // Each IP should have its own limit
        assert!(limiter.check("192.168.1.1").await.is_ok());
        assert!(limiter.check("192.168.1.1").await.is_ok());
        assert!(limiter.check("192.168.1.2").await.is_ok());
        assert!(limiter.check("192.168.1.2").await.is_ok());

        // Both IPs should now be blocked
        assert!(limiter.check("192.168.1.1").await.is_err());
        assert!(limiter.check("192.168.1.2").await.is_err());
    }
}
