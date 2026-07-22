-- Migration: add OAuth provider linking support.
-- Run this against your D1 database (e.g. via the Cloudflare dashboard SQL editor
-- or `wrangler d1 execute cadiotheka --file=...`).

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

-- 3. Remove the now-redundant provider columns from the accounts table.
ALTER TABLE accounts DROP COLUMN provider;
ALTER TABLE accounts DROP COLUMN provider_id;
