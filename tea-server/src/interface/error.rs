use axum::{
    async_trait,
    extract::{rejection::JsonRejection, FromRequest, Request},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::de::DeserializeOwned;
use serde_json::json;

use crate::domain::common::error::DomainError;

/// Application-level error type that bridges domain errors to HTTP responses.
#[derive(Debug)]
pub struct AppError(pub DomainError);

impl From<DomainError> for AppError {
    fn from(err: DomainError) -> Self {
        AppError(err)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self.0 {
            DomainError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            DomainError::Conflict(msg) => (StatusCode::CONFLICT, msg.clone()),
            DomainError::Validation(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg.clone()),
            DomainError::Repository(repo_err) => {
                tracing::error!(error = %repo_err, "Repository error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "An internal error occurred. Please try again later.".to_string(),
                )
            }
        };

        let body = Json(json!({
            "error": status.canonical_reason().unwrap_or("Error"),
            "message": message,
            "status": status.as_u16(),
        }));

        (status, body).into_response()
    }
}

/// Custom JSON extractor that converts serde/axum deserialization errors into
/// the same structured error format as AppError, instead of leaking raw serde
/// messages like "missing field `name` at line 1 column 2".
pub struct AppJson<T>(pub T);

#[async_trait]
impl<T, S> FromRequest<S> for AppJson<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        match Json::<T>::from_request(req, state).await {
            Ok(Json(value)) => Ok(AppJson(value)),
            Err(rejection) => {
                let (status, message) = match &rejection {
                    JsonRejection::JsonDataError(e) => (
                        StatusCode::UNPROCESSABLE_ENTITY,
                        e.body_text()
                            .trim_start_matches(
                                "Failed to deserialize the JSON body into the target type: ",
                            )
                            .to_string(),
                    ),
                    JsonRejection::JsonSyntaxError(_) => {
                        (StatusCode::BAD_REQUEST, "Invalid JSON syntax".to_string())
                    }
                    JsonRejection::MissingJsonContentType(_) => (
                        StatusCode::UNSUPPORTED_MEDIA_TYPE,
                        "Content-Type must be application/json".to_string(),
                    ),
                    _ => (StatusCode::BAD_REQUEST, "Bad request".to_string()),
                };

                Err((
                    status,
                    Json(json!({
                        "error": status.canonical_reason().unwrap_or("Error"),
                        "message": message,
                        "status": status.as_u16(),
                    })),
                )
                    .into_response())
            }
        }
    }
}
