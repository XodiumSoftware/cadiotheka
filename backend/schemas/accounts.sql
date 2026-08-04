CREATE TABLE IF NOT EXISTS accounts (
    id TEXT PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    email TEXT NOT NULL,
    role TEXT NOT NULL CHECK(role IN ('creator', 'admin')),
    bio TEXT NOT NULL DEFAULT '',
    avatar_url TEXT,
    created_at TEXT NOT NULL,
    verified INTEGER NOT NULL DEFAULT 0
);
