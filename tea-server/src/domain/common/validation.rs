use url::Url;

use super::error::DomainError;

/// Validates that a string field is not empty or whitespace-only.
pub fn validate_non_empty(field: &str, val: &str) -> Result<(), DomainError> {
    if val.trim().is_empty() {
        Err(DomainError::Validation(format!(
            "Field `{field}` must not be empty"
        )))
    } else {
        Ok(())
    }
}

/// Validates that a string is a valid absolute URL (must have http/https scheme).
pub fn validate_url(field: &str, val: &str) -> Result<(), DomainError> {
    match Url::parse(val) {
        Ok(url) if url.scheme() == "https" || url.scheme() == "http" => Ok(()),
        _ => Err(DomainError::Validation(format!(
            "Field `{field}` must be a valid http or https URL, got: `{val}`"
        ))),
    }
}

/// Validates that a string field does not exceed `max` bytes.
/// Prevents oversized inputs from exhausting downstream storage or processing.
pub fn validate_max_len(field: &str, val: &str, max: usize) -> Result<(), DomainError> {
    if val.len() > max {
        Err(DomainError::Validation(format!(
            "Field `{field}` exceeds maximum length of {max} characters"
        )))
    } else {
        Ok(())
    }
}

/// Validates that an integer field is non-negative.
pub fn validate_non_negative(field: &str, val: i32) -> Result<(), DomainError> {
    if val < 0 {
        Err(DomainError::Validation(format!(
            "Field `{field}` must be non-negative, got {val}"
        )))
    } else {
        Ok(())
    }
}

/// Validates an optional URL field — passes if the value is None.
pub fn validate_optional_url(field: &str, val: &Option<String>) -> Result<(), DomainError> {
    match val {
        Some(url) => validate_url(field, url),
        None => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_string_fails_non_empty_check() {
        assert!(validate_non_empty("name", "").is_err());
        assert!(validate_non_empty("name", "   ").is_err());
    }

    #[test]
    fn non_empty_string_passes() {
        assert!(validate_non_empty("name", "ACME Corp").is_ok());
    }

    #[test]
    fn invalid_url_fails() {
        assert!(validate_url("homepage_url", "not-a-url").is_err());
        assert!(validate_url("homepage_url", "ftp://example.com").is_err());
    }

    #[test]
    fn valid_http_url_passes() {
        assert!(validate_url("homepage_url", "https://example.com").is_ok());
        assert!(validate_url("homepage_url", "http://localhost:8080/path").is_ok());
    }

    #[test]
    fn optional_url_none_passes() {
        assert!(validate_optional_url("homepage_url", &None).is_ok());
    }

    #[test]
    fn optional_url_invalid_fails() {
        assert!(validate_optional_url("homepage_url", &Some("bad".to_string())).is_err());
    }
}
