CREATE TABLE IF NOT EXISTS projects (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    author TEXT NOT NULL,
    author_id TEXT NOT NULL,
    author_username TEXT NOT NULL,
    collaborator_ids TEXT NOT NULL DEFAULT '[]',
    description TEXT NOT NULL DEFAULT '',
    extended_desc TEXT NOT NULL DEFAULT '',
    tags TEXT NOT NULL DEFAULT '[]',
    supported_platforms TEXT NOT NULL DEFAULT '[]',
    downloads INTEGER NOT NULL DEFAULT 0,
    favorites TEXT NOT NULL DEFAULT '[]',
    timestamp TEXT NOT NULL,
    ifc_url TEXT
);
