use three_d_asset::Srgba;

/// Alpha value used for stored highlight colors (fully opaque).
const HIGHLIGHT_ALPHA: u8 = 255;

/// Converts an [`Srgba`] color to a 6-digit hex string (e.g. `#ffc800`).
pub fn srgba_to_hex(color: Srgba) -> String {
    format!("#{:02x}{:02x}{:02x}", color.r, color.g, color.b)
}

/// Parses a 6-digit hex color string into [`Srgba`].
///
/// Accepts strings with or without a leading `#`. Returns `None` for invalid input.
pub fn hex_to_srgba(hex: &str) -> Option<Srgba> {
    let hex = hex.strip_prefix('#').unwrap_or(hex);
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some(Srgba::new(r, g, b, HIGHLIGHT_ALPHA))
}

/// Computes a simple black-or-white foreground color that contrasts with the
/// given background color.
///
/// Uses the relative luminance formula for sRGB.
pub fn contrast_color(background: Srgba) -> String {
    let luminance = 0.299 * f64::from(background.r)
        + 0.587 * f64::from(background.g)
        + 0.114 * f64::from(background.b);
    if luminance > 128.0 {
        "#000000".to_string()
    } else {
        "#ffffff".to_string()
    }
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
    palette[usize::try_from(hash).unwrap_or(0) % palette.len()]
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
    fn srgba_to_hex_returns_six_digit_hex() {
        assert_eq!(srgba_to_hex(Srgba::new(255, 200, 0, 255)), "#ffc800");
        assert_eq!(srgba_to_hex(Srgba::new(0, 0, 0, 255)), "#000000");
        assert_eq!(srgba_to_hex(Srgba::new(255, 255, 255, 255)), "#ffffff");
    }

    #[test]
    fn hex_to_srgba_parses_with_and_without_hash() {
        assert_eq!(hex_to_srgba("#ffc800"), Some(Srgba::new(255, 200, 0, 255)));
        assert_eq!(hex_to_srgba("ffc800"), Some(Srgba::new(255, 200, 0, 255)));
        assert_eq!(hex_to_srgba("#000000"), Some(Srgba::new(0, 0, 0, 255)));
    }

    #[test]
    fn hex_to_srgba_rejects_invalid_input() {
        assert_eq!(hex_to_srgba(""), None);
        assert_eq!(hex_to_srgba("#gg0000"), None);
        assert_eq!(hex_to_srgba("#fff"), None);
    }

    #[test]
    fn srgba_to_hex_and_hex_to_srgba_round_trip() {
        let original = Srgba::new(255, 200, 0, 255);
        assert_eq!(hex_to_srgba(&srgba_to_hex(original)), Some(original));
    }

    #[test]
    fn contrast_color_returns_black_or_white() {
        assert_eq!(contrast_color(Srgba::new(255, 255, 255, 255)), "#000000");
        assert_eq!(contrast_color(Srgba::new(0, 0, 0, 255)), "#ffffff");
        assert_eq!(contrast_color(Srgba::new(145, 65, 172, 255)), "#ffffff");
        assert_eq!(contrast_color(Srgba::new(200, 200, 200, 255)), "#000000");
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
    fn language_color_returns_expected() {
        assert_eq!(language_color("Rust"), "bg-[#dea584]");
        assert_eq!(language_color("UnknownLang"), "bg-base-content/50");
    }
}
