-- Migration: 20240104000000_add_soft_delete
-- Add soft delete pattern for recoverable deletion
--
-- Soft delete allows records to be "deleted" without data loss,
-- enabling audit trails, recovery, and compliance requirements.
-- Queries should filter by deleted_at IS NULL for active records.

-- 1. Add soft delete columns to tea_products
ALTER TABLE tea_products
    ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS deleted_by VARCHAR(256);

-- 2. Add soft delete columns to tea_components
ALTER TABLE tea_components
    ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS deleted_by VARCHAR(256);

-- 3. Add soft delete columns to tea_collections
ALTER TABLE tea_collections
    ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS deleted_by VARCHAR(256);

-- 4. Add soft delete columns to tea_artifacts
ALTER TABLE tea_artifacts
    ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS deleted_by VARCHAR(256);

-- 5. Add soft delete columns to tea_product_releases
ALTER TABLE tea_product_releases
    ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS deleted_by VARCHAR(256);

-- 6. Add soft delete columns to tea_component_releases
ALTER TABLE tea_component_releases
    ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS deleted_by VARCHAR(256);

-- 7. Create partial indexes for active (non-deleted) records
-- These indexes exclude soft-deleted records for common query patterns
CREATE INDEX IF NOT EXISTS idx_tea_products_active ON tea_products(uuid) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_tea_products_name_active ON tea_products(name) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_tea_components_active ON tea_components(uuid) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_tea_components_name_active ON tea_components(name) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_tea_collections_active ON tea_collections(uuid) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_tea_artifacts_active ON tea_artifacts(uuid) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_tea_artifacts_name_active ON tea_artifacts(name) WHERE deleted_at IS NULL;

-- 8. Create index for deleted records (for admin/purge queries)
CREATE INDEX IF NOT EXISTS idx_tea_products_deleted_at ON tea_products(deleted_at) WHERE deleted_at IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_tea_components_deleted_at ON tea_components(deleted_at) WHERE deleted_at IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_tea_collections_deleted_at ON tea_collections(deleted_at) WHERE deleted_at IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_tea_artifacts_deleted_at ON tea_artifacts(deleted_at) WHERE deleted_at IS NOT NULL;

-- 9. Add comments documenting soft delete behavior
COMMENT ON COLUMN tea_products.deleted_at IS 'Timestamp when this record was soft-deleted. NULL means active.';
COMMENT ON COLUMN tea_products.deleted_by IS 'Actor who soft-deleted this record (JWT subject or mTLS client cert DN)';
COMMENT ON COLUMN tea_components.deleted_at IS 'Timestamp when this record was soft-deleted. NULL means active.';
COMMENT ON COLUMN tea_components.deleted_by IS 'Actor who soft-deleted this record';
COMMENT ON COLUMN tea_collections.deleted_at IS 'Timestamp when this record was soft-deleted. NULL means active.';
COMMENT ON COLUMN tea_collections.deleted_by IS 'Actor who soft-deleted this record';
COMMENT ON COLUMN tea_artifacts.deleted_at IS 'Timestamp when this record was soft-deleted. NULL means active.';
COMMENT ON COLUMN tea_artifacts.deleted_by IS 'Actor who soft-deleted this record';

-- 10. Create a view for active products (convenience for queries)
CREATE OR REPLACE VIEW active_products AS
SELECT * FROM tea_products WHERE deleted_at IS NULL;

CREATE OR REPLACE VIEW active_components AS
SELECT * FROM tea_components WHERE deleted_at IS NULL;

CREATE OR REPLACE VIEW active_collections AS
SELECT * FROM tea_collections WHERE deleted_at IS NULL;

CREATE OR REPLACE VIEW active_artifacts AS
SELECT * FROM tea_artifacts WHERE deleted_at IS NULL;
