//! gRPC Publisher Service stub.
//!
//! TODO: This module is stubbed pending proto generation setup.
//! The gRPC service requires generated protobuf types from tonic-build.
//! Currently proto compilation is skipped due to missing well-known proto dependencies.

use tonic::{Request, Response, Status};

/// Stub gRPC service placeholder.
///
/// This struct exists to satisfy module exports but does not provide
/// functional gRPC endpoints. Once proto generation is configured,
/// this will implement the full Publisher trait.
pub struct PublisherGrpcService<A> {
    #[allow(dead_code)]
    app_service: A,
}

impl<A> PublisherGrpcService<A> {
    pub fn new(app_service: A) -> Self {
        Self { app_service }
    }
}

/// Placeholder trait for Publisher gRPC service.
/// This will be replaced by the generated trait from tonic-build.
#[allow(dead_code)]
#[tonic::async_trait]
pub trait Publisher: Send + Sync + 'static {
    async fn create_product(&self, _request: Request<()>) -> Result<Response<()>, Status> {
        Err(Status::unimplemented(
            "gRPC service not yet implemented - proto generation required",
        ))
    }
}

// Stub implementation for the placeholder trait
#[tonic::async_trait]
impl<A: Send + Sync + 'static> Publisher for PublisherGrpcService<A> {
    async fn create_product(&self, _request: Request<()>) -> Result<Response<()>, Status> {
        Err(Status::unimplemented(
            "gRPC service not yet implemented - proto generation required",
        ))
    }
}
