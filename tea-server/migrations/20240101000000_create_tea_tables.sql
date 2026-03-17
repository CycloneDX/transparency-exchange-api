-- Create TEA database schema
-- Migration: 20240101000000_create_tea_tables

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- TEA Product table
CREATE TABLE tea_products (
    uuid UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(512) NOT NULL,
    description TEXT,
    identifiers JSONB,
    vendor_name VARCHAR(512),
    vendor_uuid UUID,
    vendor_url VARCHAR(2048),
    vendor_contacts JSONB,
    created_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    modified_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    homepage_url VARCHAR(2048),
    documentation_url VARCHAR(2048),
    vcs_url VARCHAR(2048)
);

-- TEA Product Release table
CREATE TABLE tea_product_releases (
    uuid UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    product_uuid UUID NOT NULL REFERENCES tea_products(uuid) ON DELETE CASCADE,
    version VARCHAR(256) NOT NULL,
    release_date TIMESTAMPTZ,
    pre_release BOOLEAN NOT NULL DEFAULT FALSE,
    identifiers JSONB,
    created_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    modified_date TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Component Reference table (many-to-many between product releases and components)
CREATE TABLE component_references (
    product_release_uuid UUID NOT NULL REFERENCES tea_product_releases(uuid) ON DELETE CASCADE,
    component_uuid UUID NOT NULL,
    release_uuid UUID NOT NULL,
    PRIMARY KEY (product_release_uuid, component_uuid, release_uuid)
);

-- TEA Component table
CREATE TABLE tea_components (
    uuid UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(512) NOT NULL,
    description TEXT,
    identifiers JSONB,
    component_type VARCHAR(64),
    licenses JSONB,
    publisher VARCHAR(512),
    created_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    modified_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    homepage_url VARCHAR(2048),
    vcs_url VARCHAR(2048)
);

-- TEA Component Release table
CREATE TABLE tea_component_releases (
    uuid UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    component_uuid UUID NOT NULL REFERENCES tea_components(uuid) ON DELETE CASCADE,
    version VARCHAR(256) NOT NULL,
    release_date TIMESTAMPTZ,
    pre_release BOOLEAN NOT NULL DEFAULT FALSE,
    identifiers JSONB,
    created_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    modified_date TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Distribution table
CREATE TABLE distributions (
    component_release_uuid UUID NOT NULL REFERENCES tea_component_releases(uuid) ON DELETE CASCADE,
    distribution_type VARCHAR(256) NOT NULL,
    description TEXT,
    identifiers JSONB,
    url VARCHAR(2048),
    signature_url VARCHAR(2048),
    checksums JSONB NOT NULL,
    PRIMARY KEY (component_release_uuid, distribution_type)
);

-- TEA Collection table
CREATE TABLE tea_collections (
    uuid UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    version INTEGER NOT NULL,
    date TIMESTAMPTZ NOT NULL,
    belongs_to VARCHAR(32) NOT NULL,
    update_reason VARCHAR(64) NOT NULL,
    artifacts JSONB,
    created_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    modified_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (uuid, version)
);

-- TEA Artifact table
CREATE TABLE tea_artifacts (
    uuid UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(512) NOT NULL,
    type VARCHAR(64) NOT NULL,
    component_distributions JSONB,
    formats JSONB NOT NULL,
    created_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    description TEXT,
    subject JSONB
);

-- Indexes for performance
CREATE INDEX idx_tea_products_name ON tea_products(name);
CREATE INDEX idx_tea_product_releases_product_uuid ON tea_product_releases(product_uuid);
CREATE INDEX idx_tea_product_releases_version ON tea_product_releases(version);
CREATE INDEX idx_tea_components_name ON tea_components(name);
CREATE INDEX idx_tea_component_releases_component_uuid ON tea_component_releases(component_uuid);
CREATE INDEX idx_tea_component_releases_version ON tea_component_releases(version);
CREATE INDEX idx_tea_collections_uuid_version ON tea_collections(uuid, version);
CREATE INDEX idx_tea_artifacts_name ON tea_artifacts(name);
CREATE INDEX idx_tea_artifacts_type ON tea_artifacts(type);
