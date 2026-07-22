-- Migration: add OAuth provider linking support.
-- Run this against your D1 database (e.g. via the Cloudflare dashboard SQL editor
-- or `wrangler d1 execute cadiotheka --file=...`).
-- Wrangler wraps the whole file in a transaction automatically.

-- 1. Create the table that stores every provider identity linked to an account.
CREATE TABLE IF NOT EXISTS account_providers (
    account_id TEXT NOT NULL,
    provider TEXT NOT NULL,
    provider_id TEXT NOT NULL,
    created_at TEXT NOT NULL,
    PRIMARY KEY (provider, provider_id),
    FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE
);

-- 2. Backfill the linked-providers table with the primary provider identity from
-- every existing account, so lookups work for all current users.
INSERT OR IGNORE INTO account_providers (account_id, provider, provider_id, created_at)
SELECT id, provider, provider_id, created_at
FROM accounts;

-- 3. Recreate the accounts table without the provider/provider_id columns.
--    SQLite does not support dropping columns that have UNIQUE constraints, so
--    we copy the data into a new schema and swap the tables.
CREATE TABLE accounts_new (
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

INSERT INTO accounts_new (id, username, display_name, email, role, bio, avatar_url, created_at, verified)
SELECT id, username, display_name, email, role, bio, avatar_url, created_at, verified
FROM accounts;

DROP TABLE accounts;
ALTER TABLE accounts_new RENAME TO accounts;
