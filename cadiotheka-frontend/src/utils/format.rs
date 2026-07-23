use thousands::Separable;

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

/// Formats a non-negative integer with SI suffixes for compact display.
pub fn format_number(value: u64) -> String {
    if value < 1_000 {
        return value.to_string();
    }

    let mut scales = human_format::Scales::SI();
    scales.with_suffixes(vec!["", "k", "M", "B"]);

    #[allow(clippy::cast_precision_loss)]
    let value_f64 = value as f64;

    human_format::Formatter::new()
        .with_decimals(1)
        .with_separator("")
        .with_scales(scales)
        .format(value_f64)
}

/// Returns the full integer with thousands separators (e.g. "1.234.567").
pub fn format_number_full(value: u64) -> String {
    value.separate_with_dots()
}

/// Returns a human-readable relative age string such as "2 weeks ago".
pub fn format_time_ago(timestamp: time::OffsetDateTime) -> String {
    format_duration_ago(now_utc() - timestamp)
}

/// Returns a full timestamp formatted as "dd/mm/yyyy at hh:mm".
pub fn format_time_full(timestamp: time::OffsetDateTime) -> String {
    let format = time::macros::format_description!("[day]/[month]/[year] at [hour]:[minute]");
    timestamp.format(&format).unwrap_or_default()
}

/// Formats a duration as a relative age string.
///
/// Negative durations are clamped to zero so timestamps in the future do not
/// produce confusing output such as "-5 seconds ago".
fn format_duration_ago(duration: time::Duration) -> String {
    let seconds = duration.max(time::Duration::ZERO).whole_seconds();
    #[allow(clippy::cast_sign_loss)]
    let std_duration = std::time::Duration::from_secs(seconds as u64);

    timeago::Formatter::new().too_low("0").convert(std_duration)
}

/// Returns the current UTC time using the JavaScript `Date` API.
fn now_utc() -> time::OffsetDateTime {
    let millis = js_sys::Date::now();
    time::OffsetDateTime::UNIX_EPOCH + time::Duration::seconds_f64(millis / 1_000.0)
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
            format_duration_ago(time::Duration::days(61)),
            "2 months ago"
        );
        assert_eq!(
            format_duration_ago(time::Duration::days(732)),
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
        assert_eq!(format_duration_ago(time::Duration::days(31)), "1 month ago");
        assert_eq!(format_duration_ago(time::Duration::days(366)), "1 year ago");
    }

    #[test]
    fn clean_sha_only_strips_trailing_dirty() {
        assert_eq!(clean_sha("abc123-dirty-foo"), "abc123-dirty-foo");
        assert_eq!(clean_sha(""), "");
        assert_eq!(clean_sha("-dirty"), "");
        assert_eq!(clean_sha("abc123-dirty-dirty"), "abc123-dirty");
    }

    #[test]
    fn clean_sha_removes_dirty_suffix() {
        assert_eq!(clean_sha("abc123-dirty"), "abc123");
        assert_eq!(clean_sha("abc123"), "abc123");
    }

    #[test]
    fn test_placeholder_letter() {
        assert_eq!(placeholder_letter("Blender"), "B");
        assert_eq!(placeholder_letter("freecad"), "F");
        assert_eq!(placeholder_letter(""), "?");
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
}
