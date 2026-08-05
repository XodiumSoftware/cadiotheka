-- Add file_size to project_versions so existing deployments can track IFC sizes.
ALTER TABLE project_versions ADD COLUMN file_size INTEGER NOT NULL DEFAULT 0;
