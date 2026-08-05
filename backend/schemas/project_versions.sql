CREATE TABLE IF NOT EXISTS project_versions (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    filename TEXT NOT NULL,
    ifc_key TEXT NOT NULL,
    state TEXT NOT NULL DEFAULT 'undefined',
    created_at TEXT NOT NULL,
    file_size INTEGER NOT NULL DEFAULT 0,
    version TEXT NOT NULL DEFAULT '',
    platform TEXT NOT NULL DEFAULT '',
    downloads INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);
