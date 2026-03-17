//! HTTP middleware for TEA server.
//!
//! Provides cross-cutting concerns:
//! - Rate limiting
//! - Authentication/Authorization
//! - Request logging

pub mod rate_limit;

pub use rate_limit::{RateLimitConfig, RateLimitError, RateLimiter};
