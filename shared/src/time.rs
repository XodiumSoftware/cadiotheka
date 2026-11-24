//! Current-time access shared by the frontend and backend.
//!
//! Both crates compile to WebAssembly, where `std::time::SystemTime` is not
//! available, so the JavaScript `Date` API is the only clock.

/// Returns the current UTC time using the JavaScript `Date` API.
#[must_use]
pub fn now_utc() -> time::OffsetDateTime {
    let millis = js_sys::Date::now();
    time::OffsetDateTime::UNIX_EPOCH + time::Duration::seconds_f64(millis / 1_000.0)
}

#[cfg(all(test, target_arch = "wasm32"))]
mod tests {
    use super::*;

    #[test]
    fn now_utc_is_not_the_unix_epoch() {
        assert!(now_utc() > time::OffsetDateTime::UNIX_EPOCH);
    }
}
