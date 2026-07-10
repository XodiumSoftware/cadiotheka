//! General-purpose utility helpers for Cadiotheka.

/// Collection of reusable pure helper functions.
#[derive(Debug, Default, Clone, Copy)]
pub struct Utils;

impl Utils {
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
        Self::format_duration_ago(Self::now_utc() - timestamp)
    }

    /// Formats a duration as a relative age string.
    fn format_duration_ago(duration: time::Duration) -> String {
        let seconds = duration.whole_seconds();

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
        timestamp.format(format).unwrap_or_default()
    }

    /// Returns the current UTC time using the JavaScript `Date` API.
    fn now_utc() -> time::OffsetDateTime {
        let millis = js_sys::Date::now();
        let seconds = (millis / 1_000.0) as i64;
        let nanos = ((millis % 1_000.0) * 1_000_000.0) as i32;
        time::OffsetDateTime::from_unix_timestamp(seconds)
            .unwrap_or(time::OffsetDateTime::UNIX_EPOCH)
            + time::Duration::nanoseconds(nanos.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_number_compact() {
        assert_eq!(Utils::format_number(0), "0");
        assert_eq!(Utils::format_number(999), "999");
        assert_eq!(Utils::format_number(1_000), "1.0k");
        assert_eq!(Utils::format_number(1_500), "1.5k");
        assert_eq!(Utils::format_number(1_000_000), "1.0M");
        assert_eq!(Utils::format_number(1_500_000), "1.5M");
        assert_eq!(Utils::format_number(1_000_000_000), "1.0B");
        assert_eq!(Utils::format_number(2_500_000_000), "2.5B");
    }

    #[test]
    fn format_number_full_separates_thousands() {
        assert_eq!(Utils::format_number_full(0), "0");
        assert_eq!(Utils::format_number_full(1_000), "1.000");
        assert_eq!(Utils::format_number_full(1_234_567), "1.234.567");
        assert_eq!(Utils::format_number_full(12_345_678_901), "12.345.678.901");
    }

    #[test]
    fn format_duration_ago_handles_all_units() {
        assert_eq!(
            Utils::format_duration_ago(time::Duration::seconds(45)),
            "45 seconds ago"
        );
        assert_eq!(
            Utils::format_duration_ago(time::Duration::seconds(1)),
            "1 second ago"
        );
        assert_eq!(
            Utils::format_duration_ago(time::Duration::minutes(5)),
            "5 minutes ago"
        );
        assert_eq!(
            Utils::format_duration_ago(time::Duration::hours(3)),
            "3 hours ago"
        );
        assert_eq!(
            Utils::format_duration_ago(time::Duration::days(2)),
            "2 days ago"
        );
        assert_eq!(
            Utils::format_duration_ago(time::Duration::weeks(2)),
            "2 weeks ago"
        );
        assert_eq!(
            Utils::format_duration_ago(time::Duration::days(60)),
            "2 months ago"
        );
        assert_eq!(
            Utils::format_duration_ago(time::Duration::days(730)),
            "2 years ago"
        );
    }

    #[test]
    fn format_time_full_known_timestamp() {
        let timestamp = time::OffsetDateTime::from_unix_timestamp(0).unwrap();
        assert_eq!(Utils::format_time_full(timestamp), "01/01/1970 at 00:00");
    }
}
