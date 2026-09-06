-- Add indexes backing the hot lookup paths: per-account provider resolution
-- in account queries and per-project version listings.
CREATE INDEX IF NOT EXISTS idx_account_providers_account_id ON account_providers (account_id);
CREATE INDEX IF NOT EXISTS idx_project_versions_project_id ON project_versions (project_id);
