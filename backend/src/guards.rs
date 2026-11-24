use worker::{Request, Response, Result, RouteContext};

use crate::api::accounts::Account;
use crate::api::session::read_session;
use crate::utils::{RateLimitNamespace, check_rate_limit, unauthorized};

/// Outcome of running authentication guards.
///
/// Guards either allow the handler to continue with an authenticated account,
/// or short-circuit the request with a response (e.g. rate limit exceeded).
pub(crate) enum GuardOutcome {
    /// The guard passed and the caller may proceed with the account.
    Account(Account),
    /// The guard failed; the caller should return this response immediately.
    Response(Response),
}

/// Requires a valid session, returning the authenticated account when one
/// exists.
///
/// Unauthenticated requests short-circuit with a `401 Unauthorized` response
/// the caller should return immediately. This is used for mutation handlers
/// that need authentication but no rate limiting.
pub(crate) async fn require_auth(req: &Request, ctx: &RouteContext<()>) -> Result<GuardOutcome> {
    Ok(match read_session(req, ctx).await? {
        Some(account) => GuardOutcome::Account(account),
        None => GuardOutcome::Response(unauthorized("Unauthorized")?),
    })
}

/// Verifies rate limit and session, returning the authenticated account when
/// both pass.
///
/// This is useful for mutation handlers that should be rate-limited and require
/// a logged-in user.
pub(crate) async fn require_auth_with_rate_limit(
    req: &Request,
    ctx: &RouteContext<()>,
    namespace: RateLimitNamespace,
) -> Result<GuardOutcome> {
    if let Some(response) = check_rate_limit(req, ctx, namespace).await? {
        return Ok(GuardOutcome::Response(response));
    }
    require_auth(req, ctx).await
}
