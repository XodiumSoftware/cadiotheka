CREATE TABLE IF NOT EXISTS project_versions (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    filename TEXT NOT NULL,
    ifc_key TEXT NOT NULL,
    state TEXT NOT NULL DEFAULT 'undefined',
    created_at TEXT NOT NULL,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

-- Migrate existing single IFC models into the versions table.
INSERT OR IGNORE INTO project_versions (id, project_id, filename, ifc_key, state, created_at)
SELECT
    lower(hex(randomblob(16))),
    p.id,
    'model.ifc',
    p.ifc_url,
    'undefined',
    p.timestamp
FROM projects p
WHERE p.ifc_url IS NOT NULL AND p.ifc_url != '';
