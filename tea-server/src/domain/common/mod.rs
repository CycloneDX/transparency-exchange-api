pub mod checksum;
pub mod deprecation;
pub mod error;
pub mod identifier;
pub mod pagination;
pub mod validation;

// Serde default for DateTime<Utc> fields that must be server-assigned.
// Using a named function avoids the double-evaluation serde does when
// `default = "chrono::Utc::now"` is used directly in attributes.
pub fn now() -> chrono::DateTime<chrono::Utc> {
    chrono::Utc::now()
}
