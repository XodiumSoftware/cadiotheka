-- Remove the platforms concept entirely. Cadiotheka only supports IFC files for
-- now, so per-project and per-version platform lists are no longer needed.

-- Drop the platforms JSON array column from projects.
ALTER TABLE projects DROP COLUMN platforms;

-- Drop the platforms JSON array column from project_versions.
ALTER TABLE project_versions DROP COLUMN platforms;

-- Drop the seed table holding supported CAD platforms.
DROP TABLE IF EXISTS platforms;
