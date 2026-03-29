-- Migration: 20260328000000_enable_versioned_collections
-- Allow multiple collection versions with the same logical UUID and keep
-- artifact memberships version-specific.

CREATE SEQUENCE IF NOT EXISTS tea_collections_row_id_seq;

ALTER TABLE tea_collections
    ADD COLUMN IF NOT EXISTS row_id BIGINT;

ALTER TABLE tea_collections
    ALTER COLUMN row_id SET DEFAULT nextval('tea_collections_row_id_seq');

UPDATE tea_collections
SET row_id = nextval('tea_collections_row_id_seq')
WHERE row_id IS NULL;

ALTER TABLE tea_collections
    ALTER COLUMN row_id SET NOT NULL;

ALTER TABLE tea_collections
    ADD COLUMN IF NOT EXISTS name VARCHAR(512) NOT NULL DEFAULT '',
    ADD COLUMN IF NOT EXISTS dependencies JSONB;

ALTER TABLE tea_collections
    DROP CONSTRAINT IF EXISTS tea_collections_uuid_version_key;

ALTER TABLE tea_collections
    ADD CONSTRAINT tea_collections_uuid_version_key UNIQUE (uuid, version);

ALTER TABLE collection_artifacts
    ADD COLUMN IF NOT EXISTS collection_version INTEGER;

UPDATE collection_artifacts AS ca
SET collection_version = tc.version
FROM tea_collections AS tc
WHERE ca.collection_uuid = tc.uuid
  AND ca.collection_version IS NULL;

INSERT INTO collection_artifacts (collection_uuid, collection_version, artifact_uuid, position, added_date)
SELECT
    tc.uuid,
    tc.version,
    artifact.uuid::uuid,
    artifact.position - 1,
    COALESCE(tc.modified_date, tc.created_date, NOW())
FROM tea_collections AS tc
CROSS JOIN LATERAL jsonb_array_elements_text(COALESCE(tc.artifacts, '[]'::jsonb))
    WITH ORDINALITY AS artifact(uuid, position)
ON CONFLICT DO NOTHING;

ALTER TABLE collection_artifacts
    DROP CONSTRAINT IF EXISTS collection_artifacts_collection_uuid_fkey;

ALTER TABLE collection_artifacts
    DROP CONSTRAINT IF EXISTS collection_artifacts_collection_fkey;

ALTER TABLE collection_artifacts
    DROP CONSTRAINT IF EXISTS collection_artifacts_pkey;

ALTER TABLE collection_artifacts
    ALTER COLUMN collection_version SET NOT NULL;

ALTER TABLE collection_artifacts
    ADD CONSTRAINT collection_artifacts_pkey
        PRIMARY KEY (collection_uuid, collection_version, artifact_uuid);

ALTER TABLE collection_artifacts
    ADD CONSTRAINT collection_artifacts_collection_fkey
        FOREIGN KEY (collection_uuid, collection_version)
        REFERENCES tea_collections (uuid, version)
        ON DELETE CASCADE;

CREATE INDEX IF NOT EXISTS idx_collection_artifacts_collection_version
    ON collection_artifacts (collection_uuid, collection_version);

ALTER TABLE tea_collections
    DROP CONSTRAINT IF EXISTS tea_collections_pkey;

ALTER TABLE tea_collections
    ADD CONSTRAINT tea_collections_pkey PRIMARY KEY (row_id);

CREATE INDEX IF NOT EXISTS idx_tea_collections_uuid_version
    ON tea_collections (uuid, version DESC);
