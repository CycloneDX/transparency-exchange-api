//! gRPC infrastructure for TEA server.
//!
//! NOTE: This module is currently stubbed pending proto generation setup.
//! The gRPC service requires generated protobuf types which are not yet available.

mod publisher;

pub use publisher::PublisherGrpcService;
