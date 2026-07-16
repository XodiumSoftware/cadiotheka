use leptos::wasm_bindgen::JsCast;
use leptos::wasm_bindgen::JsValue;
use leptos::wasm_bindgen::closure::Closure;

/// Add a listener to the browser `window` and automatically remove it when
/// the surrounding effect is cleaned up.
///
/// Returns `None` if the listener could not be registered.
pub fn window_event_listener<E, F>(event: &'static str, mut handler: F) -> Option<()>
where
    E: JsCast + 'static,
    F: FnMut(E) + 'static,
{
    let window = leptos::web_sys::window()?;
    let closure = Closure::wrap(Box::new(move |ev: leptos::web_sys::Event| {
        if let Ok(typed) = ev.dyn_into::<E>() {
            handler(typed);
        }
    }) as Box<dyn FnMut(_)>);

    // Transfer the closure to JavaScript ownership. The listener is removed in
    // `on_cleanup`; once detached, the JS function becomes unreachable and is
    // collected, freeing the associated Rust closure.
    let function: js_sys::Function = closure.as_ref().unchecked_ref::<js_sys::Function>().clone();
    if let Err(err) = window.add_event_listener_with_callback(event, &function) {
        leptos::web_sys::console::warn_1(&JsValue::from_str(&format!(
            "Failed to add window '{event}' event listener: {err:?}"
        )));
        return None;
    }
    std::mem::forget(closure);

    leptos::prelude::on_cleanup(move || {
        if let Some(window) = leptos::web_sys::window()
            && let Err(err) = window.remove_event_listener_with_callback(event, &function)
        {
            leptos::web_sys::console::warn_1(&JsValue::from_str(&format!(
                "Failed to remove window '{event}' event listener: {err:?}"
            )));
        }
    });

    Some(())
}

/// Strip a `-dirty` suffix from a Git SHA, if present.
pub fn clean_sha(sha: &str) -> &str {
    sha.strip_suffix("-dirty").unwrap_or(sha)
}

/// Returns the uppercase first letter of a string, or `?` if empty.
pub fn placeholder_letter(title: &str) -> String {
    title
        .chars()
        .next()
        .unwrap_or('?')
        .to_uppercase()
        .to_string()
}

/// Returns a deterministic Tailwind background color class from a string.
pub fn placeholder_color(title: &str) -> &'static str {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let palette: [&'static str; 8] = [
        "bg-red-500",
        "bg-orange-500",
        "bg-yellow-500",
        "bg-green-500",
        "bg-cyan-500",
        "bg-blue-500",
        "bg-purple-500",
        "bg-pink-500",
    ];

    let mut hasher = DefaultHasher::new();
    title.hash(&mut hasher);
    let hash = hasher.finish();
    palette[(hash as usize) % palette.len()]
}

/// Formats a non-negative integer with SI suffixes for compact display.
pub fn format_number(value: u64) -> String {
    match value {
        0..=999 => value.to_string(),
        1_000..=999_999 => format!("{:.1}k", value as f64 / 1_000.0),
        1_000_000..=999_999_999 => format!("{:.1}M", value as f64 / 1_000_000.0),
        _ => format!("{:.1}B", value as f64 / 1_000_000_000.0),
    }
}

/// Returns the full integer with thousands separators (e.g. "1.234.567").
pub fn format_number_full(value: u64) -> String {
    let raw = value.to_string();
    let mut result = String::new();
    for (i, ch) in raw.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push('.');
        }
        result.push(ch);
    }
    result.chars().rev().collect()
}

/// Returns a human-readable relative age string such as "2 weeks ago".
pub fn format_time_ago(timestamp: time::OffsetDateTime) -> String {
    format_duration_ago(now_utc() - timestamp)
}

/// Formats a duration as a relative age string.
///
/// Negative durations are clamped to zero so timestamps in the future do not
/// produce confusing output such as "-5 seconds ago".
fn format_duration_ago(duration: time::Duration) -> String {
    let seconds = duration.max(time::Duration::ZERO).whole_seconds();

    let value = if seconds < 60 {
        (seconds, "second")
    } else if seconds < 3_600 {
        (duration.whole_minutes(), "minute")
    } else if seconds < 86_400 {
        (duration.whole_hours(), "hour")
    } else if seconds < 604_800 {
        (duration.whole_days(), "day")
    } else if seconds < 2_592_000 {
        (duration.whole_weeks(), "week")
    } else if seconds < 31_536_000 {
        (duration.whole_days() / 30, "month")
    } else {
        (duration.whole_days() / 365, "year")
    };

    let (count, unit) = value;
    if count == 1 {
        format!("1 {unit} ago")
    } else {
        format!("{count} {unit}s ago")
    }
}

/// Returns a full timestamp formatted as "dd/mm/yyyy at hh:mm".
pub fn format_time_full(timestamp: time::OffsetDateTime) -> String {
    let format = time::macros::format_description!("[day]/[month]/[year] at [hour]:[minute]");
    timestamp.format(&format).unwrap_or_default()
}

/// Returns the current UTC time using the JavaScript `Date` API.
fn now_utc() -> time::OffsetDateTime {
    let millis = js_sys::Date::now();
    let seconds = (millis / 1_000.0) as i64;
    let nanos = ((millis % 1_000.0) * 1_000_000.0) as i32;
    time::OffsetDateTime::from_unix_timestamp(seconds).unwrap_or(time::OffsetDateTime::UNIX_EPOCH)
        + time::Duration::nanoseconds(nanos.into())
}

/// Base URL for backend API requests.
///
/// In release builds the frontend is served from `cadiotheka.com` and talks to
/// `api.cadiotheka.com/data`. In debug builds Trunk proxies `/data/` to the
/// local backend, so a relative path is used.
pub const fn api_base_url() -> &'static str {
    if cfg!(debug_assertions) {
        "/data"
    } else {
        "https://api.cadiotheka.com/data"
    }
}

/// Returns the full URL for a backend API path.
pub fn api_url(path: &str) -> String {
    let base = api_base_url();
    if path.starts_with('/') {
        format!("{base}{path}")
    } else {
        format!("{base}/{path}")
    }
}

/// Returns the full URL for an auth endpoint (session/profile/logout).
///
/// Auth is served from the same origin as the frontend because session cookies
/// must be readable by the frontend's domain.
pub fn auth_url(path: &str) -> String {
    let base = "/auth";
    if path.starts_with('/') {
        format!("{base}{path}")
    } else {
        format!("{base}/{path}")
    }
}

/// Returns the full URL for an OAuth login provider endpoint.
pub fn login_url(provider: &str) -> String {
    let base = "/login";
    if provider.starts_with('/') {
        format!("{base}{provider}")
    } else {
        format!("{base}/{provider}")
    }
}

/// Return a Tailwind color class for a programming language name.
///
/// Unknown languages fall back to a neutral base-content badge.
pub fn language_color(language: &str) -> &'static str {
    match language {
        "Rust" => "bg-[#dea584]",
        "TypeScript" => "bg-[#3178c6]",
        "JavaScript" => "bg-[#f1e05a]",
        "Python" => "bg-[#3572A5]",
        "HTML" => "bg-[#e34c26]",
        "CSS" => "bg-[#563d7c]",
        "Java" | "java" => "bg-[#b07219]",
        "Go" => "bg-[#00ADD8]",
        "C" => "bg-[#555555]",
        "C++" => "bg-[#f34b7d]",
        "Kotlin" => "bg-[#A97BFF]",
        _ => "bg-base-content/50",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_duration_ago_clamps_negative_durations() {
        assert_eq!(
            format_duration_ago(time::Duration::seconds(-5)),
            "0 seconds ago"
        );
        assert_eq!(
            format_duration_ago(time::Duration::hours(-24)),
            "0 seconds ago"
        );
    }

    #[test]
    fn format_number_compact() {
        assert_eq!(format_number(0), "0");
        assert_eq!(format_number(999), "999");
        assert_eq!(format_number(1_000), "1.0k");
        assert_eq!(format_number(1_500), "1.5k");
        assert_eq!(format_number(1_000_000), "1.0M");
        assert_eq!(format_number(1_500_000), "1.5M");
        assert_eq!(format_number(1_000_000_000), "1.0B");
        assert_eq!(format_number(2_500_000_000), "2.5B");
    }

    #[test]
    fn format_number_full_separates_thousands() {
        assert_eq!(format_number_full(0), "0");
        assert_eq!(format_number_full(1_000), "1.000");
        assert_eq!(format_number_full(1_234_567), "1.234.567");
        assert_eq!(format_number_full(12_345_678_901), "12.345.678.901");
    }

    #[test]
    fn format_duration_ago_handles_all_units() {
        assert_eq!(
            format_duration_ago(time::Duration::seconds(45)),
            "45 seconds ago"
        );
        assert_eq!(
            format_duration_ago(time::Duration::seconds(1)),
            "1 second ago"
        );
        assert_eq!(
            format_duration_ago(time::Duration::minutes(5)),
            "5 minutes ago"
        );
        assert_eq!(format_duration_ago(time::Duration::hours(3)), "3 hours ago");
        assert_eq!(format_duration_ago(time::Duration::days(2)), "2 days ago");
        assert_eq!(format_duration_ago(time::Duration::weeks(2)), "2 weeks ago");
        assert_eq!(
            format_duration_ago(time::Duration::days(60)),
            "2 months ago"
        );
        assert_eq!(
            format_duration_ago(time::Duration::days(730)),
            "2 years ago"
        );
    }

    #[test]
    fn format_number_compact_boundary_cases() {
        assert_eq!(format_number(100), "100");
        assert_eq!(format_number(999_499), "999.5k");
        assert_eq!(format_number(999_499_999), "999.5M");
        assert_eq!(format_number(999_499_999_999), "999.5B");
    }

    #[test]
    fn format_number_full_handles_single_digits_and_large_numbers() {
        assert_eq!(format_number_full(1), "1");
        assert_eq!(format_number_full(12), "12");
        assert_eq!(format_number_full(123), "123");
        assert_eq!(format_number_full(1_000_000_000_000), "1.000.000.000.000");
    }

    #[test]
    fn format_duration_ago_boundary_units() {
        assert_eq!(
            format_duration_ago(time::Duration::seconds(0)),
            "0 seconds ago"
        );
        assert_eq!(
            format_duration_ago(time::Duration::seconds(59)),
            "59 seconds ago"
        );
        assert_eq!(
            format_duration_ago(time::Duration::minutes(1)),
            "1 minute ago"
        );
        assert_eq!(format_duration_ago(time::Duration::hours(1)), "1 hour ago");
        assert_eq!(format_duration_ago(time::Duration::days(1)), "1 day ago");
        assert_eq!(format_duration_ago(time::Duration::weeks(1)), "1 week ago");
        assert_eq!(format_duration_ago(time::Duration::days(30)), "1 month ago");
        assert_eq!(format_duration_ago(time::Duration::days(365)), "1 year ago");
    }

    #[test]
    fn test_placeholder_letter() {
        assert_eq!(placeholder_letter("Blender"), "B");
        assert_eq!(placeholder_letter("freecad"), "F");
        assert_eq!(placeholder_letter(""), "?");
    }

    #[test]
    fn test_placeholder_color_is_deterministic() {
        let a = placeholder_color("abc");
        let b = placeholder_color("abc");
        assert_eq!(a, b);
    }

    #[test]
    fn language_color_case_sensitive_and_variants() {
        assert_eq!(language_color("Java"), "bg-[#b07219]");
        assert_eq!(language_color("java"), "bg-[#b07219]");
        assert_eq!(language_color("Kotlin"), "bg-[#A97BFF]");
        assert_eq!(language_color("Go"), "bg-[#00ADD8]");
        assert_eq!(language_color(""), "bg-base-content/50");
    }

    #[test]
    fn clean_sha_only_strips_trailing_dirty() {
        assert_eq!(clean_sha("abc123-dirty-foo"), "abc123-dirty-foo");
        assert_eq!(clean_sha(""), "");
        assert_eq!(clean_sha("-dirty"), "");
        assert_eq!(clean_sha("abc123-dirty-dirty"), "abc123-dirty");
    }

    #[test]
    fn format_time_full_known_timestamp() {
        let timestamp = time::OffsetDateTime::from_unix_timestamp(0).unwrap();
        assert_eq!(format_time_full(timestamp), "01/01/1970 at 00:00");
    }

    #[test]
    fn format_time_full_rounds_down_minutes() {
        let timestamp = time::OffsetDateTime::from_unix_timestamp(90).unwrap();
        assert_eq!(format_time_full(timestamp), "01/01/1970 at 00:01");
    }

    #[test]
    fn language_color_returns_expected() {
        assert_eq!(language_color("Rust"), "bg-[#dea584]");
        assert_eq!(language_color("UnknownLang"), "bg-base-content/50");
    }

    #[test]
    fn clean_sha_removes_dirty_suffix() {
        assert_eq!(clean_sha("abc123-dirty"), "abc123");
        assert_eq!(clean_sha("abc123"), "abc123");
    }
}
