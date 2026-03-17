-- Migration: 20240102000000_add_deprecation_and_fk
-- Add deprecation columns, fix FK integrity, and remove redundant constraints

-- 1. Add deprecation columns to tea_products
ALTER TABLE tea_products
    ADD COLUMN IF NOT EXISTS deprecation_reason       TEXT,
    ADD COLUMN IF NOT EXISTS deprecated_by            VARCHAR(256),
    ADD COLUMN IF NOT EXISTS deprecated_date          TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS deprecated_successor_url VARCHAR(2048),
    ADD COLUMN IF NOT EXISTS deprecation_state        VARCHAR(32) NOT NULL DEFAULT 'ACTIVE';

-- 2. Add deprecation columns to tea_components
ALTER TABLE tea_components
    ADD COLUMN IF NOT EXISTS deprecation_reason       TEXT,
    ADD COLUMN IF NOT EXISTS deprecated_by            VARCHAR(256),
    ADD COLUMN IF NOT EXISTS deprecated_date          TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS deprecated_successor_url VARCHAR(2048),
    ADD COLUMN IF NOT EXISTS deprecation_state        VARCHAR(32) NOT NULL DEFAULT 'ACTIVE';

-- 3. Add deprecation columns to tea_collections
ALTER TABLE tea_collections
    ADD COLUMN IF NOT EXISTS deprecation_reason       TEXT,
    ADD COLUMN IF NOT EXISTS deprecated_by            VARCHAR(256),
    ADD COLUMN IF NOT EXISTS deprecated_date          TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS deprecated_successor_url VARCHAR(2048),
    ADD COLUMN IF NOT EXISTS deprecation_state        VARCHAR(32) NOT NULL DEFAULT 'ACTIVE';

-- 4. Add deprecation columns to tea_artifacts
ALTER TABLE tea_artifacts
    ADD COLUMN IF NOT EXISTS deprecation_reason       TEXT,
    ADD COLUMN IF NOT EXISTS deprecated_by            VARCHAR(256),
    ADD COLUMN IF NOT EXISTS deprecated_date          TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS deprecated_successor_url VARCHAR(2048),
    ADD COLUMN IF NOT EXISTS deprecation_state        VARCHAR(32) NOT NULL DEFAULT 'ACTIVE';

-- 5. Fix component_references: add FK constraint that was missing
--    First drop the old broken PK, re-add with proper FK
ALTER TABLE component_references
    ADD CONSTRAINT fk_component_references_component
        FOREIGN KEY (component_uuid) REFERENCES tea_components (uuid) ON DELETE CASCADE;

-- 6. Remove redundant UNIQUE (uuid, version) from tea_collections — uuid is already the PK
ALTER TABLE tea_collections
    DROP CONSTRAINT IF EXISTS tea_collections_uuid_version_key;

-- 7. Create a proper collection_artifacts join table (replaces artifacts JSONB blob in tea_collections)
CREATE TABLE IF NOT EXISTS collection_artifacts (
    collection_uuid UUID NOT NULL REFERENCES tea_collections (uuid) ON DELETE CASCADE,
    artifact_uuid   UUID NOT NULL REFERENCES tea_artifacts (uuid) ON DELETE RESTRICT,
    position        INTEGER NOT NULL DEFAULT 0,
    added_date      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (collection_uuid, artifact_uuid)
);

-- 8. Create index for efficient lookups on new join table
CREATE INDEX IF NOT EXISTS idx_collection_artifacts_collection ON collection_artifacts (collection_uuid);
CREATE INDEX IF NOT EXISTS idx_collection_artifacts_artifact   ON collection_artifacts (artifact_uuid);

-- 9. Add index on deprecation_state for efficient "active only" queries
CREATE INDEX IF NOT EXISTS idx_tea_products_deprecation_state   ON tea_products   (deprecation_state);
CREATE INDEX IF NOT EXISTS idx_tea_components_deprecation_state ON tea_components (deprecation_state);
