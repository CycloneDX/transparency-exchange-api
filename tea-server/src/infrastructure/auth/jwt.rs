// SA-01/02: Real JWT Bearer authentication middleware.
//
// Replaces the stub BearerAuthInterceptor that accepted any non-empty string.
// Uses HS256 by default; secret is read from TEA_JWT_SECRET env var at startup
// (must be ≥32 bytes — validated in main.rs).
//
// Usage (in main.rs):
//   use crate::infrastructure::auth::jwt::require_auth;
//   let v1_write_routes = Router::new()
//       ...
//       .route_layer(middleware::from_fn(require_auth));

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Json, Response},
};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject — typically a user ID or service account identifier.
    pub sub: String,
    /// Expiration timestamp (Unix seconds). Required.
    pub exp: usize,
    /// Optional: issuer claim, validated if `TEA_JWT_ISSUER` env var is set.
    pub iss: Option<String>,
    /// Optional: audience claim, validated against the configured TEA audience.
    #[serde(default)]
    pub aud: Option<AudienceClaim>,
    /// Optional OAuth-style space-delimited scopes.
    #[serde(default)]
    pub scope: Option<String>,
    /// Optional fine-grained permissions.
    #[serde(default)]
    pub permissions: Vec<String>,
    /// Optional coarse role name.
    #[serde(default)]
    pub role: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AudienceClaim {
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum AuthError {
    MissingAuthorization,
    MissingSecret,
    Expired,
    InvalidAudience,
    InvalidToken,
    InsufficientPrivileges,
}

/// Extract the `Authorization: Bearer <token>` header from a request,
/// returning `None` if absent or malformed.
fn extract_bearer(req: &Request) -> Option<&str> {
    req.headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(parse_bearer_header)
}

pub fn parse_bearer_header(value: &str) -> Option<&str> {
    value.strip_prefix("Bearer ")
}

fn unauthorized(message: &'static str) -> Response {
    (
        StatusCode::UNAUTHORIZED,
        [("WWW-Authenticate", "Bearer realm=\"TEA API\"")],
        Json(json!({
            "error": "Unauthorized",
            "message": message,
            "status": 401,
        })),
    )
        .into_response()
}

impl AuthError {
    fn message(self) -> &'static str {
        match self {
            Self::MissingAuthorization => "Missing Authorization header",
            Self::MissingSecret => "Server misconfigured — auth secret unavailable",
            Self::Expired => "Token has expired",
            Self::InvalidAudience => "Token audience is not accepted",
            Self::InvalidToken => "Invalid token",
            Self::InsufficientPrivileges => "Token lacks required write privileges",
        }
    }
}

fn configured_audience() -> String {
    std::env::var("TEA_JWT_AUDIENCE").unwrap_or_else(|_| "tea-api".to_string())
}

fn configured_write_scope() -> String {
    std::env::var("TEA_JWT_WRITE_SCOPE").unwrap_or_else(|_| "tea:write".to_string())
}

fn configured_write_role() -> String {
    std::env::var("TEA_JWT_WRITE_ROLE").unwrap_or_else(|_| "tea-writer".to_string())
}

#[cfg(test)]
fn audience_matches(claim: &AudienceClaim, expected: &str) -> bool {
    match claim {
        AudienceClaim::Single(value) => value == expected,
        AudienceClaim::Multiple(values) => values.iter().any(|value| value == expected),
    }
}

fn token_has_write_access(claims: &Claims, required_scope: &str, required_role: &str) -> bool {
    let scope_matches = claims
        .scope
        .as_deref()
        .map(|scope| scope.split_whitespace().any(|item| item == required_scope))
        .unwrap_or(false);
    let permission_matches = claims.permissions.iter().any(|item| item == required_scope);
    let role_matches = claims
        .role
        .as_deref()
        .map(|role| role == required_role)
        .unwrap_or(false);

    scope_matches || permission_matches || role_matches
}

pub fn authorize_bearer_token(token: &str) -> Result<Claims, AuthError> {
    let secret = std::env::var("TEA_JWT_SECRET").map_err(|_| AuthError::MissingSecret)?;
    let audience = configured_audience();
    let write_scope = configured_write_scope();
    let write_role = configured_write_role();

    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;
    validation.set_audience(&[audience.as_str()]);

    if let Ok(issuer) = std::env::var("TEA_JWT_ISSUER") {
        validation.set_issuer(&[issuer]);
    }

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map_err(|err| match err.kind() {
        jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::Expired,
        jsonwebtoken::errors::ErrorKind::InvalidAudience => AuthError::InvalidAudience,
        _ => AuthError::InvalidToken,
    })?;

    if !token_has_write_access(&token_data.claims, &write_scope, &write_role) {
        return Err(AuthError::InsufficientPrivileges);
    }

    Ok(token_data.claims)
}

/// Axum middleware: validates a JWT Bearer token on every request.
///
/// The secret is loaded from the `TEA_JWT_SECRET` environment variable.
/// Returns `401 Unauthorized` if the token is absent, expired, or malformed.
pub async fn require_auth(mut req: Request, next: Next) -> Result<Response, Response> {
    let token = extract_bearer(&req)
        .ok_or_else(|| unauthorized(AuthError::MissingAuthorization.message()))?;
    let claims = authorize_bearer_token(token).map_err(|error| unauthorized(error.message()))?;

    req.extensions_mut().insert(claims);

    Ok(next.run(req).await)
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{encode, EncodingKey, Header};
    use std::sync::{Mutex, OnceLock};

    static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    fn env_lock() -> &'static Mutex<()> {
        ENV_LOCK.get_or_init(|| Mutex::new(()))
    }

    fn with_auth_env<F>(test: F)
    where
        F: FnOnce(),
    {
        let _guard = env_lock().lock().unwrap();
        let saved = [
            ("TEA_JWT_SECRET", std::env::var("TEA_JWT_SECRET").ok()),
            ("TEA_JWT_AUDIENCE", std::env::var("TEA_JWT_AUDIENCE").ok()),
            (
                "TEA_JWT_WRITE_SCOPE",
                std::env::var("TEA_JWT_WRITE_SCOPE").ok(),
            ),
            (
                "TEA_JWT_WRITE_ROLE",
                std::env::var("TEA_JWT_WRITE_ROLE").ok(),
            ),
            ("TEA_JWT_ISSUER", std::env::var("TEA_JWT_ISSUER").ok()),
        ];

        std::env::set_var("TEA_JWT_SECRET", "dev-only-insecure-secret-32-bytes--");
        std::env::set_var("TEA_JWT_AUDIENCE", "tea-api");
        std::env::set_var("TEA_JWT_WRITE_SCOPE", "tea:write");
        std::env::set_var("TEA_JWT_WRITE_ROLE", "tea-writer");
        std::env::set_var("TEA_JWT_ISSUER", "issuer");

        test();

        for (key, value) in saved {
            match value {
                Some(value) => std::env::set_var(key, value),
                None => std::env::remove_var(key),
            }
        }
    }

    fn encode_claims(claims: &Claims) -> String {
        encode(
            &Header::default(),
            claims,
            &EncodingKey::from_secret(b"dev-only-insecure-secret-32-bytes--"),
        )
        .unwrap()
    }

    fn claims() -> Claims {
        Claims {
            sub: "user-123".to_string(),
            exp: usize::MAX,
            iss: Some("issuer".to_string()),
            aud: Some(AudienceClaim::Single("tea-api".to_string())),
            scope: Some("tea:write tea:read".to_string()),
            permissions: vec![],
            role: None,
        }
    }

    #[test]
    fn audience_matches_single_and_multiple_claims() {
        assert!(audience_matches(
            &AudienceClaim::Single("tea-api".to_string()),
            "tea-api"
        ));
        assert!(audience_matches(
            &AudienceClaim::Multiple(vec!["other".to_string(), "tea-api".to_string()]),
            "tea-api"
        ));
        assert!(!audience_matches(
            &AudienceClaim::Single("other".to_string()),
            "tea-api"
        ));
    }

    #[test]
    fn write_access_accepts_scope_permission_or_role() {
        let base = claims();
        assert!(token_has_write_access(&base, "tea:write", "tea-writer"));

        let via_permission = Claims {
            scope: None,
            permissions: vec!["tea:write".to_string()],
            ..base.clone()
        };
        assert!(token_has_write_access(
            &via_permission,
            "tea:write",
            "tea-writer"
        ));

        let via_role = Claims {
            scope: None,
            permissions: vec![],
            role: Some("tea-writer".to_string()),
            ..base.clone()
        };
        assert!(token_has_write_access(&via_role, "tea:write", "tea-writer"));
    }

    #[test]
    fn write_access_rejects_tokens_without_required_claims() {
        let claims = Claims {
            scope: Some("tea:read".to_string()),
            permissions: vec!["other".to_string()],
            role: Some("viewer".to_string()),
            ..claims()
        };

        assert!(!token_has_write_access(&claims, "tea:write", "tea-writer"));
    }

    #[test]
    fn parse_bearer_header_requires_prefix() {
        assert_eq!(parse_bearer_header("Bearer token-123"), Some("token-123"));
        assert_eq!(parse_bearer_header("token-123"), None);
    }

    #[test]
    fn authorize_bearer_token_accepts_valid_write_token() {
        with_auth_env(|| {
            let token = encode_claims(&claims());
            let parsed = authorize_bearer_token(&token).unwrap();
            assert_eq!(parsed.sub, "user-123");
        });
    }

    #[test]
    fn authorize_bearer_token_rejects_missing_write_privilege() {
        with_auth_env(|| {
            let token = encode_claims(&Claims {
                scope: Some("tea:read".to_string()),
                permissions: vec![],
                role: Some("viewer".to_string()),
                ..claims()
            });

            let error = authorize_bearer_token(&token).unwrap_err();
            assert_eq!(error, AuthError::InsufficientPrivileges);
        });
    }
}
