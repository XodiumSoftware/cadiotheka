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
