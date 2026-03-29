#![allow(clippy::result_large_err)]

use chrono::{DateTime, Utc};
use prost_types::Timestamp;
use tonic::Status;
use uuid::Uuid;

use crate::domain::artifact::entity::{
    Artifact, ArtifactFormat, ArtifactType, Subject, SubjectType,
};
use crate::domain::collection::entity::{
    Collection, CollectionScope, UpdateReason as DomainUpdateReason,
};
use crate::domain::common::checksum::{Checksum, ChecksumAlgorithm};
use crate::domain::common::deprecation::{Deprecation, DeprecationState as DomainDeprecationState};
use crate::domain::common::error::DomainError;
use crate::domain::common::identifier::{Identifier, IdentifierType as DomainIdentifierType};
use crate::domain::common::pagination::{Page, PaginationParams};
use crate::domain::component::entity::{
    Component, ComponentRelease, ComponentType as DomainComponentType, Distribution, LicenseInfo,
    LicenseType,
};
use crate::domain::product::entity::{
    ComponentRef as ProductReleaseComponentRef, Contact, ContactType, Product, ProductRelease,
    Vendor,
};
use crate::gen::tea::v1 as proto;

pub(super) fn parse_uuid(value: &str, field: &str) -> Result<Uuid, Status> {
    Uuid::parse_str(value)
        .map_err(|_| Status::invalid_argument(format!("{field} must be a valid UUID")))
}

pub(super) fn pagination_from_proto(page: Option<proto::PageRequest>) -> PaginationParams {
    let page = page.unwrap_or(proto::PageRequest {
        page_size: 50,
        page_token: String::new(),
    });

    let limit = if page.page_size <= 0 {
        50
    } else {
        page.page_size as usize
    };

    let offset = if page.page_token.trim().is_empty() {
        0
    } else {
        page.page_token.parse::<usize>().unwrap_or(0)
    };

    PaginationParams { limit, offset }
}

pub(super) fn page_response<T>(page: &Page<T>) -> proto::PageResponse {
    let next_offset = page.offset + page.items.len();
    let next_page_token = if next_offset < page.total {
        next_offset.to_string()
    } else {
        String::new()
    };

    proto::PageResponse {
        next_page_token,
        total_count: Some(page.total as i64),
    }
}

pub(super) fn domain_error_to_status(error: DomainError) -> Status {
    match error {
        DomainError::Validation(message) => Status::invalid_argument(message),
        DomainError::NotFound(message) => Status::not_found(message),
        DomainError::Conflict(message) => Status::already_exists(message),
        DomainError::Repository(error) => {
            tracing::error!(error = %error, "gRPC repository error");
            Status::internal("internal server error")
        }
    }
}

pub(super) fn product_to_proto(product: Product) -> proto::Product {
    proto::Product {
        uuid: product.uuid.to_string(),
        name: product.name,
        description: product.description.unwrap_or_default(),
        identifiers: product
            .identifiers
            .into_iter()
            .map(identifier_to_proto)
            .collect(),
        vendor: Some(vendor_to_proto(product.vendor)),
        created_date: timestamp(product.created_date),
        modified_date: timestamp(product.modified_date),
        homepage_url: product.homepage_url.unwrap_or_default(),
        documentation_url: product.documentation_url.unwrap_or_default(),
        vcs_url: product.vcs_url.unwrap_or_default(),
        deprecation: product.deprecation.map(deprecation_to_proto),
    }
}

pub(super) fn component_to_proto(component: Component) -> proto::Component {
    proto::Component {
        uuid: component.uuid.to_string(),
        name: component.name,
        description: component.description.unwrap_or_default(),
        identifiers: component
            .identifiers
            .into_iter()
            .map(identifier_to_proto)
            .collect(),
        created_date: timestamp(component.created_date),
        modified_date: timestamp(component.modified_date),
        component_type: component_type_to_proto(&component.component_type),
        licenses: component
            .licenses
            .into_iter()
            .map(license_to_proto)
            .collect(),
        publisher: component.publisher.unwrap_or_default(),
        homepage_url: component.homepage_url.unwrap_or_default(),
        vcs_url: component.vcs_url.unwrap_or_default(),
        deprecation: component.deprecation.map(deprecation_to_proto),
    }
}

pub(super) fn component_release_to_proto(release: ComponentRelease) -> proto::ComponentRelease {
    proto::ComponentRelease {
        uuid: release.uuid.to_string(),
        component: release.component_uuid.to_string(),
        version: release.version,
        created_date: None,
        release_date: release.release_date.and_then(timestamp),
        pre_release: release.pre_release,
        identifiers: release
            .identifiers
            .into_iter()
            .map(identifier_to_proto)
            .collect(),
        distributions: release
            .distributions
            .into_iter()
            .map(distribution_to_proto)
            .collect(),
        deprecation: None,
    }
}

pub(super) fn product_release_to_proto(release: ProductRelease) -> proto::ProductRelease {
    proto::ProductRelease {
        uuid: release.uuid.to_string(),
        product: Some(release.product_uuid.to_string()),
        version: release.version,
        created_date: timestamp(release.created_date),
        release_date: release.release_date.and_then(timestamp),
        pre_release: release.pre_release,
        identifiers: release
            .identifiers
            .into_iter()
            .map(identifier_to_proto)
            .collect(),
        components: release
            .components
            .into_iter()
            .map(product_release_component_ref_to_proto)
            .collect(),
        lifecycle_status: None,
        deprecation: None,
    }
}

pub(super) fn artifact_to_proto(artifact: Artifact) -> proto::Artifact {
    proto::Artifact {
        uuid: artifact.uuid.to_string(),
        name: artifact.name,
        r#type: artifact_type_to_proto(&artifact.type_),
        component_distributions: artifact.component_distributions,
        formats: artifact
            .formats
            .into_iter()
            .map(artifact_format_to_proto)
            .collect(),
        created_date: timestamp(artifact.created_date),
        description: artifact.description.unwrap_or_default(),
        subject: artifact.subject.map(subject_to_proto),
        deprecation: artifact.deprecation.map(deprecation_to_proto),
    }
}

pub(super) fn artifact_stub(uuid: Uuid) -> proto::Artifact {
    proto::Artifact {
        uuid: uuid.to_string(),
        name: String::new(),
        r#type: proto::ArtifactType::Unspecified as i32,
        component_distributions: vec![],
        formats: vec![],
        created_date: None,
        description: String::new(),
        subject: None,
        deprecation: None,
    }
}

pub(super) fn collection_to_proto(
    collection: Collection,
    artifacts: Vec<proto::Artifact>,
) -> proto::Collection {
    proto::Collection {
        uuid: collection.uuid.to_string(),
        version: collection.version,
        date: timestamp(collection.date),
        belongs_to: collection_scope_to_proto(&collection.belongs_to),
        update_reason: Some(update_reason_to_proto(&collection.update_reason)),
        artifacts,
        signature: None,
        created_date: timestamp(collection.created_date),
        deprecation: collection.deprecation.map(deprecation_to_proto),
        conformance_vectors: vec![],
    }
}

pub(super) fn collection_version_info(
    collection: &Collection,
    artifact_count: usize,
) -> proto::CollectionVersionInfo {
    proto::CollectionVersionInfo {
        uuid: collection.uuid.to_string(),
        version: collection.version,
        date: timestamp(collection.date),
        update_reason: Some(update_reason_to_proto(&collection.update_reason)),
        artifact_count: artifact_count as i32,
        is_signed: false,
    }
}

pub(super) fn timestamp(dt: DateTime<Utc>) -> Option<Timestamp> {
    Some(Timestamp {
        seconds: dt.timestamp(),
        nanos: dt.timestamp_subsec_nanos() as i32,
    })
}

pub(super) fn within_range(value: Option<DateTime<Utc>>, range: Option<&proto::DateRange>) -> bool {
    let Some(range) = range else {
        return true;
    };
    let Some(value) = value else {
        return false;
    };

    if let Some(start) = range.start.as_ref().and_then(proto_timestamp_to_chrono) {
        if value < start {
            return false;
        }
    }

    if let Some(end) = range.end.as_ref().and_then(proto_timestamp_to_chrono) {
        if value >= end {
            return false;
        }
    }

    true
}

pub(super) fn identifier_matches(domain: &Identifier, needle: &proto::Identifier) -> bool {
    identifier_to_proto(domain.clone()).id_type == needle.id_type
        && domain.id_value == needle.id_value
}

fn proto_timestamp_to_chrono(timestamp: &Timestamp) -> Option<DateTime<Utc>> {
    DateTime::<Utc>::from_timestamp(timestamp.seconds, timestamp.nanos as u32)
}

fn vendor_to_proto(vendor: Vendor) -> proto::Vendor {
    proto::Vendor {
        name: vendor.name,
        uuid: vendor.uuid.map(|uuid| uuid.to_string()),
        url: vendor.url.unwrap_or_default(),
        contacts: vendor.contacts.into_iter().map(contact_to_proto).collect(),
    }
}

fn contact_to_proto(contact: Contact) -> proto::Contact {
    match contact.type_ {
        ContactType::Email => proto::Contact {
            name: String::new(),
            email: contact.value,
            phone: String::new(),
        },
        ContactType::Phone => proto::Contact {
            name: String::new(),
            email: String::new(),
            phone: contact.value,
        },
        ContactType::Url | ContactType::Other | ContactType::Unspecified => proto::Contact {
            name: contact.value,
            email: String::new(),
            phone: String::new(),
        },
    }
}

fn license_to_proto(license: LicenseInfo) -> proto::LicenseInfo {
    let (spdx_id, name) = match license.license_type {
        LicenseType::Spdx => (license.license_id, String::new()),
        LicenseType::Other | LicenseType::Unspecified => (String::new(), license.license_id),
    };

    proto::LicenseInfo {
        spdx_id,
        name,
        url: license.url.unwrap_or_default(),
    }
}

fn distribution_to_proto(distribution: Distribution) -> proto::Distribution {
    proto::Distribution {
        distribution_type: distribution.distribution_type,
        description: distribution.description.unwrap_or_default(),
        identifiers: distribution
            .identifiers
            .into_iter()
            .map(identifier_to_proto)
            .collect(),
        url: distribution.url.unwrap_or_default(),
        signature_url: distribution.signature_url.unwrap_or_default(),
        checksums: distribution
            .checksums
            .into_iter()
            .map(checksum_to_proto)
            .collect(),
        size_bytes: None,
        mime_type: String::new(),
    }
}

fn product_release_component_ref_to_proto(
    component: ProductReleaseComponentRef,
) -> proto::ComponentRef {
    proto::ComponentRef {
        uuid: component.component_uuid.to_string(),
        release: Some(component.release_uuid.to_string()),
    }
}

fn artifact_format_to_proto(format: ArtifactFormat) -> proto::ArtifactFormat {
    proto::ArtifactFormat {
        mime_type: format.mime_type,
        description: format.description.unwrap_or_default(),
        url: format.url,
        signature_url: format.signature_url.unwrap_or_default(),
        checksums: format
            .checksums
            .into_iter()
            .map(checksum_to_proto)
            .collect(),
        size_bytes: format.size_bytes,
        encoding: format.encoding.unwrap_or_default(),
        spec_version: format.spec_version.unwrap_or_default(),
    }
}

fn subject_to_proto(subject: Subject) -> proto::ArtifactSubject {
    proto::ArtifactSubject {
        r#type: subject_type_to_proto(&subject.type_),
        identifiers: subject
            .identifiers
            .into_iter()
            .map(identifier_to_proto)
            .collect(),
        name: subject.name.unwrap_or_default(),
        version: subject.version.unwrap_or_default(),
    }
}

pub(super) fn checksum_to_proto(checksum: Checksum) -> proto::Checksum {
    proto::Checksum {
        alg_type: checksum_algorithm_to_proto(&checksum.alg_type),
        alg_value: checksum.alg_value,
    }
}

fn identifier_to_proto(identifier: Identifier) -> proto::Identifier {
    proto::Identifier {
        id_type: identifier_type_to_proto(&identifier.id_type),
        id_value: identifier.id_value,
    }
}

fn deprecation_to_proto(deprecation: Deprecation) -> proto::Deprecation {
    let successor_uuid = deprecation
        .replacement_identifiers
        .first()
        .and_then(|identifier| Uuid::parse_str(&identifier.id_value).ok())
        .map(|uuid| uuid.to_string());

    proto::Deprecation {
        state: deprecation_state_to_proto(&deprecation.state),
        notice: deprecation.reason.unwrap_or_default(),
        sunset_date: deprecation.effective_date.and_then(timestamp),
        successor_uuid,
    }
}

fn identifier_type_to_proto(id_type: &DomainIdentifierType) -> i32 {
    match id_type {
        DomainIdentifierType::Unspecified => proto::IdentifierType::Unspecified as i32,
        DomainIdentifierType::Tei => proto::IdentifierType::Tei as i32,
        DomainIdentifierType::Purl => proto::IdentifierType::Purl as i32,
        DomainIdentifierType::Cpe => proto::IdentifierType::Cpe as i32,
        DomainIdentifierType::Swid => proto::IdentifierType::Swid as i32,
        DomainIdentifierType::Gav => proto::IdentifierType::Gav as i32,
        DomainIdentifierType::Gtin => proto::IdentifierType::Gtin as i32,
        DomainIdentifierType::Gmn => proto::IdentifierType::Gmn as i32,
        DomainIdentifierType::Udi => proto::IdentifierType::Udi as i32,
        DomainIdentifierType::Asin => proto::IdentifierType::Asin as i32,
        DomainIdentifierType::Hash => proto::IdentifierType::Hash as i32,
        DomainIdentifierType::Conformance => proto::IdentifierType::Unspecified as i32,
    }
}

fn checksum_algorithm_to_proto(algorithm: &ChecksumAlgorithm) -> i32 {
    match algorithm {
        ChecksumAlgorithm::Unspecified => proto::ChecksumAlgorithm::Unspecified as i32,
        ChecksumAlgorithm::Md5 => proto::ChecksumAlgorithm::Md5 as i32,
        ChecksumAlgorithm::Sha1 => proto::ChecksumAlgorithm::Sha1 as i32,
        ChecksumAlgorithm::Sha256 => proto::ChecksumAlgorithm::Sha256 as i32,
        ChecksumAlgorithm::Sha384 => proto::ChecksumAlgorithm::Sha384 as i32,
        ChecksumAlgorithm::Sha512 => proto::ChecksumAlgorithm::Sha512 as i32,
        ChecksumAlgorithm::Sha3_256 => proto::ChecksumAlgorithm::Sha3256 as i32,
        ChecksumAlgorithm::Sha3_384 => proto::ChecksumAlgorithm::Sha3384 as i32,
        ChecksumAlgorithm::Sha3_512 => proto::ChecksumAlgorithm::Sha3512 as i32,
        ChecksumAlgorithm::Blake2b256 => proto::ChecksumAlgorithm::Blake2b256 as i32,
        ChecksumAlgorithm::Blake2b384 => proto::ChecksumAlgorithm::Blake2b384 as i32,
        ChecksumAlgorithm::Blake2b512 => proto::ChecksumAlgorithm::Blake2b512 as i32,
        ChecksumAlgorithm::Blake3 => proto::ChecksumAlgorithm::Blake3 as i32,
    }
}

fn component_type_to_proto(component_type: &DomainComponentType) -> i32 {
    match component_type {
        DomainComponentType::Unspecified => proto::ComponentType::Unspecified as i32,
        DomainComponentType::Application => proto::ComponentType::Application as i32,
        DomainComponentType::Framework => proto::ComponentType::Framework as i32,
        DomainComponentType::Library => proto::ComponentType::Library as i32,
        DomainComponentType::Container => proto::ComponentType::Container as i32,
        DomainComponentType::OperatingSystem => proto::ComponentType::OperatingSystem as i32,
        DomainComponentType::Device => proto::ComponentType::Device as i32,
        DomainComponentType::File => proto::ComponentType::File as i32,
        DomainComponentType::Firmware => proto::ComponentType::Firmware as i32,
        DomainComponentType::Other => proto::ComponentType::Unspecified as i32,
    }
}

fn artifact_type_to_proto(artifact_type: &ArtifactType) -> i32 {
    match artifact_type {
        ArtifactType::Unspecified => proto::ArtifactType::Unspecified as i32,
        ArtifactType::Attestation => proto::ArtifactType::Attestation as i32,
        ArtifactType::Bom => proto::ArtifactType::Bom as i32,
        ArtifactType::BuildMeta => proto::ArtifactType::BuildMeta as i32,
        ArtifactType::Certification => proto::ArtifactType::Certification as i32,
        ArtifactType::Formulation => proto::ArtifactType::Formulation as i32,
        ArtifactType::License => proto::ArtifactType::License as i32,
        ArtifactType::ReleaseNotes => proto::ArtifactType::ReleaseNotes as i32,
        ArtifactType::SecurityTxt => proto::ArtifactType::SecurityTxt as i32,
        ArtifactType::ThreatModel => proto::ArtifactType::ThreatModel as i32,
        ArtifactType::Vulnerabilities => proto::ArtifactType::Vulnerabilities as i32,
        ArtifactType::Cle => proto::ArtifactType::Cle as i32,
        ArtifactType::Cdxa => proto::ArtifactType::Cdxa as i32,
        ArtifactType::Cbom => proto::ArtifactType::Cbom as i32,
        ArtifactType::ModelCard => proto::ArtifactType::ModelCard as i32,
        ArtifactType::StaticAnalysis => proto::ArtifactType::StaticAnalysis as i32,
        ArtifactType::DynamicAnalysis => proto::ArtifactType::DynamicAnalysis as i32,
        ArtifactType::PentestReport => proto::ArtifactType::PentestReport as i32,
        ArtifactType::RiskAssessment => proto::ArtifactType::RiskAssessment as i32,
        ArtifactType::Poam => proto::ArtifactType::Poam as i32,
        ArtifactType::QualityMetrics => proto::ArtifactType::QualityMetrics as i32,
        ArtifactType::Harness => proto::ArtifactType::Harness as i32,
        ArtifactType::Conformance => proto::ArtifactType::Conformance as i32,
        ArtifactType::Other => proto::ArtifactType::Other as i32,
    }
}

fn subject_type_to_proto(subject_type: &SubjectType) -> i32 {
    match subject_type {
        SubjectType::Unspecified => proto::SubjectType::Unspecified as i32,
        SubjectType::Component => proto::SubjectType::Component as i32,
        SubjectType::Product => proto::SubjectType::Product as i32,
        SubjectType::Service => proto::SubjectType::Service as i32,
        SubjectType::Organization => proto::SubjectType::Organization as i32,
        SubjectType::Build => proto::SubjectType::Build as i32,
    }
}

fn collection_scope_to_proto(scope: &CollectionScope) -> i32 {
    match scope {
        CollectionScope::Unspecified => proto::CollectionScope::Unspecified as i32,
        CollectionScope::Release => proto::CollectionScope::Release as i32,
        CollectionScope::ProductRelease => proto::CollectionScope::ProductRelease as i32,
    }
}

fn update_reason_to_proto(reason: &DomainUpdateReason) -> proto::UpdateReason {
    let reason_type = match reason {
        DomainUpdateReason::Unspecified => proto::UpdateReasonType::Unspecified as i32,
        DomainUpdateReason::InitialRelease => proto::UpdateReasonType::InitialRelease as i32,
        DomainUpdateReason::VexUpdated => proto::UpdateReasonType::VexUpdated as i32,
        DomainUpdateReason::ArtifactUpdated => proto::UpdateReasonType::ArtifactUpdated as i32,
        DomainUpdateReason::ArtifactRemoved => proto::UpdateReasonType::ArtifactRemoved as i32,
        DomainUpdateReason::ArtifactAdded => proto::UpdateReasonType::ArtifactAdded as i32,
    };

    proto::UpdateReason {
        r#type: reason_type,
        comment: String::new(),
        affected_artifact_uuids: vec![],
    }
}

fn deprecation_state_to_proto(state: &DomainDeprecationState) -> i32 {
    match state {
        DomainDeprecationState::Unspecified => proto::DeprecationState::Unspecified as i32,
        DomainDeprecationState::Active => proto::DeprecationState::Active as i32,
        DomainDeprecationState::Deprecated => proto::DeprecationState::Deprecated as i32,
        DomainDeprecationState::Retired => proto::DeprecationState::Sunset as i32,
    }
}
