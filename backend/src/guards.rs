use worker::{Request, Response, Result, RouteContext};

use crate::api::accounts::Account;
use crate::api::session::require_account;
use crate::api::turnstile::verify_turnstile_token;
use crate::utils::check_rate_limit;

/// Outcome of running authentication guards.
///
/// Guards either allow the handler to continue with an authenticated account,
/// or short-circuit the request with a response (e.g. rate limit exceeded).
pub enum GuardOutcome {
    /// The guard passed and the caller may proceed with the account.
    Account(Account),
    /// The guard failed; the caller should return this response immediately.
    Response(Response),
}

/// Verifies rate limit and session, returning the authenticated account when
/// both pass.
///
/// This is useful for mutation handlers that should be rate-limited and require
/// a logged-in user but do not need Turnstile verification.
pub async fn require_auth_with_rate_limit(
    req: &Request,
    ctx: &RouteContext<()>,
    namespace: &str,
) -> Result<GuardOutcome> {
    if let Some(response) = check_rate_limit(req, ctx, namespace).await? {
        return Ok(GuardOutcome::Response(response));
    }
    Ok(GuardOutcome::Account(require_account(req, ctx).await?))
}

/// Verifies rate limit, Turnstile token, and session, returning the
/// authenticated account when all pass.
///
/// This is useful for project creation and other high-value mutations that need
/// bot protection in addition to rate limiting and authentication.
pub async fn require_auth_with_turnstile_and_rate_limit(
    req: &mut Request,
    ctx: &RouteContext<()>,
    namespace: &str,
) -> Result<GuardOutcome> {
    if let Some(response) = check_rate_limit(req, ctx, namespace).await? {
        return Ok(GuardOutcome::Response(response));
    }
    if let Some(response) = verify_turnstile_token(req, ctx).await? {
        return Ok(GuardOutcome::Response(response));
    }
    Ok(GuardOutcome::Account(require_account(req, ctx).await?))
}
