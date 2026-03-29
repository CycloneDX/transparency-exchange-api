-- Migration: 20240105000000_align_postgres_adapters
-- Align the historical Postgres schema with the repository adapters that the
-- live server can now use when TEA_PERSISTENCE_BACKEND=postgres.

ALTER TABLE tea_collections
    ADD COLUMN IF NOT EXISTS name VARCHAR(512) NOT NULL DEFAULT '',
    ADD COLUMN IF NOT EXISTS dependencies JSONB;

ALTER TABLE tea_artifacts
    ADD COLUMN IF NOT EXISTS modified_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ADD COLUMN IF NOT EXISTS identifiers JSONB;

ALTER TABLE tea_component_releases
    ADD COLUMN IF NOT EXISTS distributions JSONB;
