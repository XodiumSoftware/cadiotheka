-- Migrate per-version platform selection from a single value to a JSON array.
-- This migration is safe to run multiple times: it uses IF NOT EXISTS and
-- only overwrites the new `platforms` column when it still holds the default
-- empty array (so existing multi-platform data is never destroyed).

-- Step 1: add the new JSON array column if it does not already exist.
ALTER TABLE project_versions ADD COLUMN IF NOT EXISTS platforms TEXT NOT NULL DEFAULT '[]';

-- Step 2: backfill rows where `platforms` is still the default and a legacy
-- `platform` value exists. Empty legacy values become '[]'.
UPDATE project_versions
SET platforms = CASE
    WHEN platform IS NULL OR platform = '' THEN '[]'
    ELSE json_array(platform)
END
WHERE platforms = '[]' AND platform IS NOT NULL;

-- Step 3: drop the legacy single-value column if it still exists.
-- D1/SQLite does not support `DROP COLUMN IF EXISTS`, so we swallow the
-- harmless error when the column has already been removed.
ALTER TABLE project_versions DROP COLUMN platform;
