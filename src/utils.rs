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
        let now = Self::now_utc();
        let duration = now - timestamp;
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
