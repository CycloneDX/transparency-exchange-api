-- Migration: 20240103000000_add_audit_trail
-- Add audit trail columns for accountability (who created/modified records)

-- 1. Add audit columns to tea_products
ALTER TABLE tea_products
    ADD COLUMN IF NOT EXISTS created_by VARCHAR(256),
    ADD COLUMN IF NOT EXISTS modified_by VARCHAR(256);

-- 2. Add audit columns to tea_components
ALTER TABLE tea_components
    ADD COLUMN IF NOT EXISTS created_by VARCHAR(256),
    ADD COLUMN IF NOT EXISTS modified_by VARCHAR(256);

-- 3. Add audit columns to tea_collections
ALTER TABLE tea_collections
    ADD COLUMN IF NOT EXISTS created_by VARCHAR(256),
    ADD COLUMN IF NOT EXISTS modified_by VARCHAR(256);

-- 4. Add audit columns to tea_artifacts
ALTER TABLE tea_artifacts
    ADD COLUMN IF NOT EXISTS created_by VARCHAR(256),
    ADD COLUMN IF NOT EXISTS modified_by VARCHAR(256);

-- 5. Add audit columns to tea_product_releases
ALTER TABLE tea_product_releases
    ADD COLUMN IF NOT EXISTS created_by VARCHAR(256),
    ADD COLUMN IF NOT EXISTS modified_by VARCHAR(256);

-- 6. Add audit columns to tea_component_releases
ALTER TABLE tea_component_releases
    ADD COLUMN IF NOT EXISTS created_by VARCHAR(256),
    ADD COLUMN IF NOT EXISTS modified_by VARCHAR(256);

-- 7. Create indexes for audit queries (e.g., "show all changes by user X")
CREATE INDEX IF NOT EXISTS idx_tea_products_created_by ON tea_products(created_by);
CREATE INDEX IF NOT EXISTS idx_tea_components_created_by ON tea_components(created_by);
CREATE INDEX IF NOT EXISTS idx_tea_collections_created_by ON tea_collections(created_by);
CREATE INDEX IF NOT EXISTS idx_tea_artifacts_created_by ON tea_artifacts(created_by);

-- 8. Add comment documenting the audit trail
COMMENT ON COLUMN tea_products.created_by IS 'Actor who created this record (from JWT subject or mTLS client cert DN)';
COMMENT ON COLUMN tea_products.modified_by IS 'Actor who last modified this record';
COMMENT ON COLUMN tea_components.created_by IS 'Actor who created this record';
COMMENT ON COLUMN tea_components.modified_by IS 'Actor who last modified this record';
COMMENT ON COLUMN tea_collections.created_by IS 'Actor who created this record';
COMMENT ON COLUMN tea_collections.modified_by IS 'Actor who last modified this record';
COMMENT ON COLUMN tea_artifacts.created_by IS 'Actor who created this record';
COMMENT ON COLUMN tea_artifacts.modified_by IS 'Actor who last modified this record';
