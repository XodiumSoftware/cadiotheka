-- Migrate existing single IFC models into the versions table.
INSERT OR IGNORE INTO project_versions (id, project_id, filename, ifc_key, state, created_at, file_size)
SELECT
    lower(hex(randomblob(16))),
    p.id,
    'model.ifc',
    p.ifc_url,
    'undefined',
    p.timestamp,
    0
FROM projects p
WHERE p.ifc_url IS NOT NULL AND p.ifc_url != '';
