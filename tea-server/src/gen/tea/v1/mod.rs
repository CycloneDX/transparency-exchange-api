//! Generated TEA v1 protobuf types.
//!
//! These are stub implementations until proto compilation is fully configured.

pub mod publisher_server {
    use tonic::async_trait;

    /// Generated trait for the Publisher gRPC service.
    #[async_trait]
    pub trait Publisher: Send + Sync + 'static {
        // Product operations
        async fn create_product(
            &self,
            request: tonic::Request<super::CreateProductRequest>,
        ) -> Result<tonic::Response<super::CreateProductResponse>, tonic::Status>;
        
        async fn get_product(
            &self,
            request: tonic::Request<super::GetProductRequest>,
        ) -> Result<tonic::Response<super::GetProductResponse>, tonic::Status>;
        
        async fn update_product(
            &self,
            request: tonic::Request<super::UpdateProductRequest>,
        ) -> Result<tonic::Response<super::UpdateProductResponse>, tonic::Status>;
        
        async fn delete_product(
            &self,
            request: tonic::Request<super::DeleteProductRequest>,
        ) -> Result<tonic::Response<super::DeleteProductResponse>, tonic::Status>;
        
        async fn deprecate_product(
            &self,
            request: tonic::Request<super::DeprecateProductRequest>,
        ) -> Result<tonic::Response<super::DeprecateProductResponse>, tonic::Status>;

        // Component operations
        async fn create_component(
            &self,
            request: tonic::Request<super::CreateComponentRequest>,
        ) -> Result<tonic::Response<super::CreateComponentResponse>, tonic::Status>;
        
        async fn get_component(
            &self,
            request: tonic::Request<super::GetComponentRequest>,
        ) -> Result<tonic::Response<super::GetComponentResponse>, tonic::Status>;
        
        async fn update_component(
            &self,
            request: tonic::Request<super::UpdateComponentRequest>,
        ) -> Result<tonic::Response<super::UpdateComponentResponse>, tonic::Status>;
        
        async fn delete_component(
            &self,
            request: tonic::Request<super::DeleteComponentRequest>,
        ) -> Result<tonic::Response<super::DeleteComponentResponse>, tonic::Status>;
        
        async fn deprecate_component(
            &self,
            request: tonic::Request<super::DeprecateComponentRequest>,
        ) -> Result<tonic::Response<super::DeprecateComponentResponse>, tonic::Status>;

        // Artifact operations
        async fn create_artifact(
            &self,
            request: tonic::Request<super::CreateArtifactRequest>,
        ) -> Result<tonic::Response<super::CreateArtifactResponse>, tonic::Status>;
        
        async fn get_artifact(
            &self,
            request: tonic::Request<super::GetArtifactRequest>,
        ) -> Result<tonic::Response<super::GetArtifactResponse>, tonic::Status>;
        
        async fn update_artifact(
            &self,
            request: tonic::Request<super::UpdateArtifactRequest>,
        ) -> Result<tonic::Response<super::UpdateArtifactResponse>, tonic::Status>;
        
        async fn delete_artifact(
            &self,
            request: tonic::Request<super::DeleteArtifactRequest>,
        ) -> Result<tonic::Response<super::DeleteArtifactResponse>, tonic::Status>;
        
        async fn deprecate_artifact(
            &self,
            request: tonic::Request<super::DeprecateArtifactRequest>,
        ) -> Result<tonic::Response<super::DeprecateArtifactResponse>, tonic::Status>;

        // Collection operations
        async fn create_collection(
            &self,
            request: tonic::Request<super::CreateCollectionRequest>,
        ) -> Result<tonic::Response<super::CreateCollectionResponse>, tonic::Status>;
        
        async fn get_collection(
            &self,
            request: tonic::Request<super::GetCollectionRequest>,
        ) -> Result<tonic::Response<super::GetCollectionResponse>, tonic::Status>;
        
        async fn update_collection(
            &self,
            request: tonic::Request<super::UpdateCollectionRequest>,
        ) -> Result<tonic::Response<super::UpdateCollectionResponse>, tonic::Status>;
        
        async fn delete_collection(
            &self,
            request: tonic::Request<super::DeleteCollectionRequest>,
        ) -> Result<tonic::Response<super::DeleteCollectionResponse>, tonic::Status>;
        
        async fn deprecate_collection(
            &self,
            request: tonic::Request<super::DeprecateCollectionRequest>,
        ) -> Result<tonic::Response<super::DeprecateCollectionResponse>, tonic::Status>;
    }
}

// ─── Product Types ──────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Product {
    #[prost(string, tag = "1")]
    pub uuid: String,
    #[prost(string, tag = "2")]
    pub name: String,
    #[prost(string, optional, tag = "3")]
    pub description: Option<String>,
    #[prost(message, optional, tag = "4")]
    pub vendor: Option<Vendor>,
    #[prost(string, optional, tag = "5")]
    pub homepage_url: Option<String>,
    #[prost(string, optional, tag = "6")]
    pub documentation_url: Option<String>,
    #[prost(string, optional, tag = "7")]
    pub vcs_url: Option<String>,
    #[prost(message, repeated, tag = "8")]
    pub identifiers: Vec<Identifier>,
    #[prost(message, optional, tag = "9")]
    pub deprecation: Option<Deprecation>,
    #[prost(message, repeated, tag = "10")]
    pub dependencies: Vec<Identifier>,
    #[prost(message, optional, tag = "11")]
    pub created_date: Option<::prost_types::Timestamp>,
    #[prost(message, optional, tag = "12")]
    pub modified_date: Option<::prost_types::Timestamp>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Vendor {
    #[prost(string, tag = "1")]
    pub name: String,
    #[prost(string, optional, tag = "2")]
    pub uuid: Option<String>,
    #[prost(string, optional, tag = "3")]
    pub url: Option<String>,
    #[prost(message, repeated, tag = "4")]
    pub contacts: Vec<Contact>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Contact {
    #[prost(string, optional, tag = "1")]
    pub name: Option<String>,
    #[prost(string, optional, tag = "2")]
    pub email: Option<String>,
    #[prost(string, optional, tag = "3")]
    pub phone: Option<String>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateProductRequest {
    #[prost(message, optional, tag = "1")]
    pub product: Option<Product>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateProductResponse {
    #[prost(message, optional, tag = "1")]
    pub product: Option<Product>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetProductRequest {
    #[prost(string, tag = "1")]
    pub uuid: String,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetProductResponse {
    #[prost(message, optional, tag = "1")]
    pub product: Option<Product>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateProductRequest {
    #[prost(string, tag = "1")]
    pub uuid: String,
    #[prost(message, optional, tag = "2")]
    pub product: Option<Product>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateProductResponse {
    #[prost(message, optional, tag = "1")]
    pub product: Option<Product>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteProductRequest {
    #[prost(string, tag = "1")]
    pub uuid: String,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteProductResponse {}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeprecateProductRequest {
    #[prost(string, tag = "1")]
    pub uuid: String,
    #[prost(message, optional, tag = "2")]
    pub deprecation: Option<Deprecation>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeprecateProductResponse {
    #[prost(message, optional, tag = "1")]
    pub product: Option<Product>,
}

// ─── Component Types ────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Component {
    #[prost(string, tag = "1")]
    pub uuid: String,
    #[prost(string, tag = "2")]
    pub name: String,
    #[prost(string, optional, tag = "3")]
    pub description: Option<String>,
    #[prost(string, optional, tag = "4")]
    pub homepage_url: Option<String>,
    #[prost(string, optional, tag = "5")]
    pub vcs_url: Option<String>,
    #[prost(message, optional, tag = "6")]
    pub deprecation: Option<Deprecation>,
    #[prost(message, optional, tag = "7")]
    pub created_date: Option<::prost_types::Timestamp>,
    #[prost(message, optional, tag = "8")]
    pub modified_date: Option<::prost_types::Timestamp>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateComponentRequest {
    #[prost(message, optional, tag = "1")]
    pub component: Option<Component>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateComponentResponse {
    #[prost(message, optional, tag = "1")]
    pub component: Option<Component>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetComponentRequest {
    #[prost(string, tag = "1")]
    pub uuid: String,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetComponentResponse {
    #[prost(message, optional, tag = "1")]
    pub component: Option<Component>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateComponentRequest {
    #[prost(string, tag = "1")]
    pub uuid: String,
    #[prost(message, optional, tag = "2")]
    pub component: Option<Component>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateComponentResponse {
    #[prost(message, optional, tag = "1")]
    pub component: Option<Component>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteComponentRequest {
    #[prost(string, tag = "1")]
    pub uuid: String,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteComponentResponse {}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeprecateComponentRequest {
    #[prost(string, tag = "1")]
    pub uuid: String,
    #[prost(message, optional, tag = "2")]
    pub deprecation: Option<Deprecation>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeprecateComponentResponse {
    #[prost(message, optional, tag = "1")]
    pub component: Option<Component>,
}

// ─── Artifact Types ──────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Artifact {
    #[prost(string, tag = "1")]
    pub uuid: String,
    #[prost(string, tag = "2")]
    pub name: String,
    #[prost(string, optional, tag = "3")]
    pub description: Option<String>,
    #[prost(enumeration = "ArtifactType", tag = "4")]
    pub r#type: i32,
    #[prost(message, repeated, tag = "5")]
    pub formats: Vec<ArtifactFormat>,
    #[prost(message, optional, tag = "6")]
    pub deprecation: Option<Deprecation>,
    #[prost(message, optional, tag = "7")]
    pub created_date: Option<::prost_types::Timestamp>,
    #[prost(message, optional, tag = "8")]
    pub modified_date: Option<::prost_types::Timestamp>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ArtifactFormat {
    #[prost(string, tag = "1")]
    pub mime_type: String,
    #[prost(string, optional, tag = "2")]
    pub description: Option<String>,
    #[prost(string, tag = "3")]
    pub url: String,
    #[prost(string, optional, tag = "4")]
    pub signature_url: Option<String>,
    #[prost(message, repeated, tag = "5")]
    pub checksums: Vec<Checksum>,
    #[prost(int64, optional, tag = "6")]
    pub size_bytes: Option<i64>,
    #[prost(string, optional, tag = "7")]
    pub encoding: Option<String>,
    #[prost(string, optional, tag = "8")]
    pub spec_version: Option<String>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Checksum {
    #[prost(enumeration = "ChecksumAlgorithm", tag = "1")]
    pub alg_type: i32,
    #[prost(string, tag = "2")]
    pub alg_value: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum ArtifactType {
    Unspecified = 0,
    Attestation = 1,
    Bom = 2,
    BuildMeta = 3,
    Certification = 4,
    Formulation = 5,
    License = 6,
    ReleaseNotes = 7,
    SecurityTxt = 8,
    ThreatModel = 9,
    Vulnerabilities = 10,
    Cle = 11,
    Cdxa = 12,
    Cbom = 13,
    ModelCard = 14,
    StaticAnalysis = 15,
    DynamicAnalysis = 16,
    PentestReport = 17,
    RiskAssessment = 18,
    Poam = 19,
    QualityMetrics = 20,
    Harness = 21,
    Conformance = 22,
    Other = 23,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum ChecksumAlgorithm {
    Unspecified = 0,
    Md5 = 1,
    Sha1 = 2,
    Sha256 = 3,
    Sha384 = 4,
    Sha512 = 5,
    Sha3_256 = 6,
    Sha3_384 = 7,
    Sha3_512 = 8,
    Blake2b256 = 9,
    Blake2b384 = 10,
    Blake2b512 = 11,
    Blake3 = 12,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateArtifactRequest {
    #[prost(message, optional, tag = "1")]
    pub artifact: Option<Artifact>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateArtifactResponse {
    #[prost(message, optional, tag = "1")]
    pub artifact: Option<Artifact>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetArtifactRequest {
    #[prost(string, tag = "1")]
    pub uuid: String,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetArtifactResponse {
    #[prost(message, optional, tag = "1")]
    pub artifact: Option<Artifact>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateArtifactRequest {
    #[prost(string, tag = "1")]
    pub uuid: String,
    #[prost(message, optional, tag = "2")]
    pub artifact: Option<Artifact>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateArtifactResponse {
    #[prost(message, optional, tag = "1")]
    pub artifact: Option<Artifact>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteArtifactRequest {
    #[prost(string, tag = "1")]
    pub uuid: String,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteArtifactResponse {}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeprecateArtifactRequest {
    #[prost(string, tag = "1")]
    pub uuid: String,
    #[prost(message, optional, tag = "2")]
    pub deprecation: Option<Deprecation>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeprecateArtifactResponse {
    #[prost(message, optional, tag = "1")]
    pub artifact: Option<Artifact>,
}

// ─── Collection Types ────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Collection {
    #[prost(string, tag = "1")]
    pub uuid: String,
    #[prost(string, tag = "2")]
    pub name: String,
    #[prost(string, optional, tag = "3")]
    pub description: Option<String>,
    #[prost(int32, tag = "4")]
    pub version: i32,
    #[prost(enumeration = "CollectionScope", tag = "5")]
    pub belongs_to: i32,
    #[prost(enumeration = "UpdateReason", tag = "6")]
    pub update_reason: i32,
    #[prost(string, repeated, tag = "7")]
    pub artifacts: Vec<String>,
    #[prost(message, optional, tag = "8")]
    pub deprecation: Option<Deprecation>,
    #[prost(message, optional, tag = "9")]
    pub created_date: Option<::prost_types::Timestamp>,
    #[prost(message, optional, tag = "10")]
    pub modified_date: Option<::prost_types::Timestamp>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum CollectionScope {
    Unspecified = 0,
    Release = 1,
    ProductRelease = 2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum UpdateReason {
    Unspecified = 0,
    InitialRelease = 1,
    VexUpdated = 2,
    ArtifactUpdated = 3,
    ArtifactRemoved = 4,
    ArtifactAdded = 5,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateCollectionRequest {
    #[prost(message, optional, tag = "1")]
    pub collection: Option<Collection>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateCollectionResponse {
    #[prost(message, optional, tag = "1")]
    pub collection: Option<Collection>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetCollectionRequest {
    #[prost(string, tag = "1")]
    pub uuid: String,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetCollectionResponse {
    #[prost(message, optional, tag = "1")]
    pub collection: Option<Collection>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateCollectionRequest {
    #[prost(string, tag = "1")]
    pub uuid: String,
    #[prost(message, optional, tag = "2")]
    pub collection: Option<Collection>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateCollectionResponse {
    #[prost(message, optional, tag = "1")]
    pub collection: Option<Collection>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteCollectionRequest {
    #[prost(string, tag = "1")]
    pub uuid: String,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteCollectionResponse {}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeprecateCollectionRequest {
    #[prost(string, tag = "1")]
    pub uuid: String,
    #[prost(message, optional, tag = "2")]
    pub deprecation: Option<Deprecation>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeprecateCollectionResponse {
    #[prost(message, optional, tag = "1")]
    pub collection: Option<Collection>,
}

// ─── Common Types ────────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Identifier {
    #[prost(enumeration = "IdentifierType", tag = "1")]
    pub id_type: i32,
    #[prost(string, tag = "2")]
    pub id_value: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum IdentifierType {
    Unspecified = 0,
    Tei = 1,
    Purl = 2,
    Cpe = 3,
    Swid = 4,
    Gav = 5,
    Gtin = 6,
    Gmn = 7,
    Udi = 8,
    Asin = 9,
    Hash = 10,
    Conformance = 11,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Deprecation {
    #[prost(enumeration = "DeprecationState", tag = "1")]
    pub state: i32,
    #[prost(string, optional, tag = "2")]
    pub reason: Option<String>,
    #[prost(message, optional, tag = "3")]
    pub announced_date: Option<::prost_types::Timestamp>,
    #[prost(message, optional, tag = "4")]
    pub effective_date: Option<::prost_types::Timestamp>,
    #[prost(message, repeated, tag = "5")]
    pub replacement_identifiers: Vec<Identifier>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum DeprecationState {
    Unspecified = 0,
    Active = 1,
    Deprecated = 2,
    Retired = 3,
}
