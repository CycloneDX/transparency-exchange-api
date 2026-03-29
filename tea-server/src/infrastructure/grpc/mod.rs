//! gRPC infrastructure for TEA server.
//!
//! Discovery/consumer read handlers and the currently supported publisher
//! write handlers are implemented against the generated TEA protobuf surface.
//! Remaining publisher RPCs fail closed with `UNIMPLEMENTED` until their
//! backing storage and domain semantics are ready.

mod consumer;
mod conversions;
mod discovery;
mod interceptor;
mod publisher;

pub use consumer::ConsumerGrpcService;
pub use discovery::DiscoveryGrpcService;
pub use interceptor::publisher_auth_interceptor;
pub use publisher::PublisherGrpcService;
