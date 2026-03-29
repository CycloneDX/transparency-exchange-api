#![allow(clippy::result_large_err)]

use tonic::{Request, Status};

use crate::infrastructure::auth::jwt::{authorize_bearer_token, parse_bearer_header, AuthError};

pub fn publisher_auth_interceptor(mut request: Request<()>) -> Result<Request<()>, Status> {
    let header = request
        .metadata()
        .get("authorization")
        .ok_or_else(|| auth_error_to_status(AuthError::MissingAuthorization))?;
    let header = header
        .to_str()
        .map_err(|_| auth_error_to_status(AuthError::InvalidToken))?;
    let token =
        parse_bearer_header(header).ok_or_else(|| auth_error_to_status(AuthError::InvalidToken))?;
    let claims = authorize_bearer_token(token).map_err(auth_error_to_status)?;
    request.extensions_mut().insert(claims);
    Ok(request)
}

pub fn auth_error_to_status(error: AuthError) -> Status {
    match error {
        AuthError::MissingAuthorization
        | AuthError::Expired
        | AuthError::InvalidAudience
        | AuthError::InvalidToken
        | AuthError::InsufficientPrivileges => Status::unauthenticated(error_message(error)),
        AuthError::MissingSecret => Status::internal(error_message(error)),
    }
}

fn error_message(error: AuthError) -> &'static str {
    match error {
        AuthError::MissingAuthorization => "missing authorization metadata",
        AuthError::MissingSecret => "server auth configuration is unavailable",
        AuthError::Expired => "token has expired",
        AuthError::InvalidAudience => "token audience is not accepted",
        AuthError::InvalidToken => "invalid bearer token",
        AuthError::InsufficientPrivileges => "token lacks required publisher privileges",
    }
}

#[cfg(test)]
mod tests {
    use super::publisher_auth_interceptor;
    use crate::infrastructure::auth::jwt::{AudienceClaim, Claims};
    use jsonwebtoken::{encode, EncodingKey, Header};
    use std::sync::{Mutex, OnceLock};
    use tonic::Request;

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

    fn token() -> String {
        encode(
            &Header::default(),
            &Claims {
                sub: "publisher".to_string(),
                exp: usize::MAX,
                iss: Some("issuer".to_string()),
                aud: Some(AudienceClaim::Single("tea-api".to_string())),
                scope: Some("tea:write".to_string()),
                permissions: vec![],
                role: None,
            },
            &EncodingKey::from_secret(b"dev-only-insecure-secret-32-bytes--"),
        )
        .unwrap()
    }

    #[test]
    fn interceptor_rejects_missing_header() {
        with_auth_env(|| {
            let error = publisher_auth_interceptor(Request::new(())).unwrap_err();
            assert_eq!(error.code(), tonic::Code::Unauthenticated);
        });
    }

    #[test]
    fn interceptor_inserts_claims_for_valid_token() {
        with_auth_env(|| {
            let mut request = Request::new(());
            let value = format!("Bearer {}", token()).parse().unwrap();
            request.metadata_mut().insert("authorization", value);

            let request = publisher_auth_interceptor(request).unwrap();
            let claims = request.extensions().get::<Claims>().unwrap();
            assert_eq!(claims.sub, "publisher");
        });
    }
}
