-- Add per-version metadata used by the redesigned versions table.
ALTER TABLE project_versions ADD COLUMN version TEXT NOT NULL DEFAULT '';
ALTER TABLE project_versions ADD COLUMN platform TEXT NOT NULL DEFAULT '';
ALTER TABLE project_versions ADD COLUMN downloads INTEGER NOT NULL DEFAULT 0;
