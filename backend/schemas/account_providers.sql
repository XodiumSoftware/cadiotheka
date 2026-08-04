CREATE TABLE IF NOT EXISTS account_providers (
    account_id TEXT NOT NULL,
    provider TEXT NOT NULL,
    provider_id TEXT NOT NULL,
    created_at TEXT NOT NULL,
    PRIMARY KEY (provider, provider_id),
    FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE
);
