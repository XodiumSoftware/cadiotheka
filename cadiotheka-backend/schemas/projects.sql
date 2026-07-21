CREATE TABLE IF NOT EXISTS projects (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    author TEXT NOT NULL,
    author_id TEXT NOT NULL,
    author_username TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    extended_desc TEXT NOT NULL DEFAULT '',
    tags TEXT NOT NULL DEFAULT '[]',
    supported_platforms TEXT NOT NULL DEFAULT '[]',
    downloads INTEGER NOT NULL DEFAULT 0,
    favorites INTEGER NOT NULL DEFAULT 0,
    timestamp TEXT NOT NULL,
    icon_url TEXT
);
