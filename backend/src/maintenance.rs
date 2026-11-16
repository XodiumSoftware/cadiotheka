//! Scheduled maintenance sweeps for data whose expiry is not event-driven.
//!
//! Most KV entries carry a TTL, but TTL expiry alone leaves a residue the cron
//! job cleans up: expired keys linger as tombstones that still appear in
//! `list()` results until they are deleted, so listing costs drift upward over
//! time. A daily sweep deletes those tombstones and any key whose owner account
//! no longer exists.

use worker::{Env, Result, console_log};

use crate::utils::{db_env, kv_env, now_utc};

/// KV prefix for OAuth state, written by the login initiation handler.
const OAUTH_STATE_PREFIX: &str = "oauth_state:";
/// KV prefix for WebAuthn registration challenges, keyed by account id.
const PASSKEY_REGISTER_PREFIX: &str = "passkey_register:";
/// KV prefix for WebAuthn login challenges, keyed by a random state token.
const PASSKEY_LOGIN_PREFIX: &str = "passkey_login:";

/// Summary of a completed sweep, logged for observability.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct SweepSummary {
    /// Keys deleted because they had expired.
    pub(crate) expired_deleted: usize,
    /// Keys deleted because their account no longer exists.
    pub(crate) orphaned_deleted: usize,
    /// Keys whose deletion failed; these are retried on the next run.
    pub(crate) failures: usize,
}

/// Runs the daily maintenance sweep.
///
/// Failures are counted rather than returned so a single bad key does not
/// abort the remaining work; the next scheduled run retries what was left.
pub(crate) async fn run_sweep(env: &Env) -> Result<SweepSummary> {
    let kv = kv_env(env)?;
    let mut summary = SweepSummary::default();
    let account_ids = fetch_existing_account_ids(env).await?;

    for prefix in [
        OAUTH_STATE_PREFIX,
        PASSKEY_REGISTER_PREFIX,
        PASSKEY_LOGIN_PREFIX,
    ] {
        let result = sweep_prefix(&kv, prefix, &account_ids).await;
        summary.expired_deleted += result.expired_deleted;
        summary.orphaned_deleted += result.orphaned_deleted;
        summary.failures += result.failures;
    }

    Ok(summary)
}

/// Returns the set of account ids that still exist in D1.
///
/// Pages through the table because D1 bounds a single result set; truncating
/// this set would make the sweep delete live entries.
async fn fetch_existing_account_ids(env: &Env) -> Result<std::collections::HashSet<String>> {
    const PAGE_SIZE: u32 = 500;
    let db = db_env(env)?;
    let mut ids = std::collections::HashSet::new();
    let mut offset: u32 = 0;

    loop {
        let rows = db
            .prepare("SELECT id FROM accounts ORDER BY id LIMIT ?1 OFFSET ?2")
            .bind(&[PAGE_SIZE.into(), offset.into()])?
            .all()
            .await?
            .results::<IdRow>()?;
        let fetched = rows.len();
        ids.extend(rows.into_iter().map(|row| row.id));
        if fetched < PAGE_SIZE as usize {
            break;
        }
        offset += PAGE_SIZE;
    }

    Ok(ids)
}

/// Minimal account row returned by the existence check.
#[derive(serde::Deserialize)]
struct IdRow {
    id: String,
}

/// Deletes expired or account-less entries under `prefix`.
async fn sweep_prefix(
    kv: &worker::KvStore,
    prefix: &str,
    account_ids: &std::collections::HashSet<String>,
) -> SweepSummary {
    let mut summary = SweepSummary::default();
    let mut cursor: Option<String> = None;

    loop {
        let mut builder = kv.list();
        if let Some(cursor) = cursor.clone() {
            builder = builder.cursor(cursor);
        }
        let listing = match builder.prefix(prefix.to_string()).execute().await {
            Ok(listing) => listing,
            Err(err) => {
                console_log!("sweep: failed to list KV prefix {prefix:?}: {err:?}");
                summary.failures += 1;
                return summary;
            }
        };

        for key in &listing.keys {
            let expired = key
                .expiration
                .is_some_and(|expiry| expiry < now_utc().unix_timestamp().unsigned_abs());
            let orphaned = is_orphaned(&key.name, prefix, account_ids);

            if !expired && !orphaned {
                continue;
            }

            match kv.delete(&key.name).await {
                Ok(()) if expired => summary.expired_deleted += 1,
                Ok(()) => summary.orphaned_deleted += 1,
                Err(err) => {
                    console_log!("sweep: failed to delete KV key {}: {err:?}", key.name);
                    summary.failures += 1;
                }
            }
        }

        cursor = listing.cursor.filter(|cursor| !cursor.is_empty());
        if cursor.is_none() {
            break;
        }
    }

    summary
}

/// Returns whether a KV key is keyed by an account that no longer exists.
///
/// Only registration challenges use the account id as their key suffix; OAuth
/// state and login challenges use opaque tokens and cannot be attributed.
fn is_orphaned(key: &str, prefix: &str, account_ids: &std::collections::HashSet<String>) -> bool {
    if prefix != PASSKEY_REGISTER_PREFIX {
        return false;
    }
    match key.strip_prefix(prefix) {
        Some(account_id) if !account_id.is_empty() => !account_ids.contains(account_id),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn ids(values: &[&str]) -> HashSet<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }

    #[test]
    fn register_challenge_for_missing_account_is_orphaned() {
        let existing = ids(&["acc-1"]);
        assert!(is_orphaned(
            "passkey_register:acc-2",
            PASSKEY_REGISTER_PREFIX,
            &existing
        ));
    }

    #[test]
    fn register_challenge_for_existing_account_is_kept() {
        let existing = ids(&["acc-1"]);
        assert!(!is_orphaned(
            "passkey_register:acc-1",
            PASSKEY_REGISTER_PREFIX,
            &existing
        ));
    }

    #[test]
    fn login_challenge_is_never_treated_as_orphaned() {
        let existing = ids(&["acc-1"]);
        assert!(!is_orphaned(
            "passkey_login:random-token",
            PASSKEY_LOGIN_PREFIX,
            &existing
        ));
    }

    #[test]
    fn oauth_state_is_never_treated_as_orphaned() {
        let existing = ids(&["acc-1"]);
        assert!(!is_orphaned(
            "oauth_state:abc",
            OAUTH_STATE_PREFIX,
            &existing
        ));
    }

    #[test]
    fn empty_account_suffix_is_not_orphaned() {
        let existing = ids(&["acc-1"]);
        assert!(!is_orphaned(
            "passkey_register:",
            PASSKEY_REGISTER_PREFIX,
            &existing
        ));
    }

    #[test]
    fn key_without_prefix_is_not_orphaned() {
        let existing = ids(&["acc-1"]);
        assert!(!is_orphaned(
            "unrelated",
            PASSKEY_REGISTER_PREFIX,
            &existing
        ));
    }
}
