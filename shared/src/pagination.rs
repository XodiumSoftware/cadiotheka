//! Pagination limits shared by the backend list endpoints and the frontend
//! fetchers.
//!
//! The backend clamps every list query to [`MAX_LIST_LIMIT`], so a client that
//! wants to sweep a whole collection must page in windows no larger than that
//! value.

/// Default page size for list endpoints when no `limit` query parameter is
/// given.
pub const DEFAULT_LIST_LIMIT: u32 = 100;

/// Maximum page size accepted by list endpoints. Bounds each D1 query so a
/// single request can never scan the whole table.
pub const MAX_LIST_LIMIT: u32 = 500;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_limit_is_within_the_maximum() {
        const { assert!(DEFAULT_LIST_LIMIT <= MAX_LIST_LIMIT) };
    }
}
