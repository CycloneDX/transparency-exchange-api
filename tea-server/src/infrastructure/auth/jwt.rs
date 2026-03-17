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

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// Subject — typically a user ID or service account identifier.
    pub sub: String,
    /// Expiration timestamp (Unix seconds). Required.
    pub exp: usize,
    /// Optional: issuer claim, validated if `TEA_JWT_ISSUER` env var is set.
    pub iss: Option<String>,
}

/// Extract the `Authorization: Bearer <token>` header from a request,
/// returning `None` if absent or malformed.
fn extract_bearer(req: &Request) -> Option<&str> {
    req.headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
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

/// Axum middleware: validates a JWT Bearer token on every request.
///
/// The secret is loaded from the `TEA_JWT_SECRET` environment variable.
/// Returns `401 Unauthorized` if the token is absent, expired, or malformed.
pub async fn require_auth(req: Request, next: Next) -> Result<Response, Response> {
    let token = extract_bearer(&req).ok_or_else(|| unauthorized("Missing Authorization header"))?;

    let secret = std::env::var("TEA_JWT_SECRET")
        .map_err(|_| unauthorized("Server misconfigured — auth secret unavailable"))?;

    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;

    // Optional issuer validation
    if let Ok(issuer) = std::env::var("TEA_JWT_ISSUER") {
        validation.set_issuer(&[issuer]);
    }

    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map_err(|err| match err.kind() {
        jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
            unauthorized("Token has expired")
        }
        _ => unauthorized("Invalid token"),
    })?;

    Ok(next.run(req).await)
}
