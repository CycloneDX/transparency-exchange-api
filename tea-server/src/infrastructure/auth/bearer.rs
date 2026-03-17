pub struct BearerAuthInterceptor;

impl BearerAuthInterceptor {
    pub fn validate(token: &str) -> Result<(), String> {
        if token.is_empty() {
            Err("Empty token".to_string())
        } else {
            Ok(())
        }
    }
}
