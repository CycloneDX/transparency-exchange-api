//! HTTP API infrastructure for TEA server.
//!
//! Provides REST API routes that complement the gRPC API.

mod routes;

pub use routes::create_router;
