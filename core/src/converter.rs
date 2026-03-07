/// Vietnamese Encoding Converter — chuyển mã tiếng Việt
///
/// Provides utilities for Vietnamese text processing:
/// - Remove diacritics (Tiếng Việt → Tieng Viet)
/// - Detect Vietnamese text
/// - Unicode normalization
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::OnceLock;

/// Supported Vietnamese encodings — aligned with UniKey 4.6
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum VnEncoding {
    /// Unicode — the standard (NFC composed form)
    #[default]
    Unicode,
    /// TCVN3 (ABC) — legacy encoding, single-byte
    Tcvn3,
    /// VNI Windows — legacy encoding
    VniWindows,
    /// VIQR — diacritics as ASCII sequences
    Viqr,
    /// Vietnamese locale CP 1258
    VnLocaleCP1258,
    /// Unicode tổ hợp (NFD decomposed form)
    UnicodeComposite,
    /// UTF-8 Literal
    Utf8Literal,
    /// NCR Decimal (&#xxxx;)
    NcrDecimal,
    /// NCR Hex (&#xHHHH;)
    NcrHex,
    /// Unicode C String (\uHHHH)
    UnicodeCString,
}


/// Raw map data for Vietnamese characters to their base (unaccented) form
const VN_CHAR_MAP: &[(char, char)] = &[
    // a variants
    ('á', 'a'), ('à', 'a'), ('ả', 'a'), ('ã', 'a'), ('ạ', 'a'),
    ('â', 'a'), ('ầ', 'a'), ('ấ', 'a'), ('ẩ', 'a'), ('ẫ', 'a'), ('ậ', 'a'),
    ('ă', 'a'), ('ằ', 'a'), ('ắ', 'a'), ('ẳ', 'a'), ('ẵ', 'a'), ('ặ', 'a'),
    // e variants
    ('é', 'e'), ('è', 'e'), ('ẻ', 'e'), ('ẽ', 'e'), ('ẹ', 'e'),
    ('ê', 'e'), ('ề', 'e'), ('ế', 'e'), ('ể', 'e'), ('ễ', 'e'), ('ệ', 'e'),
    // i variants
    ('í', 'i'), ('ì', 'i'), ('ỉ', 'i'), ('ĩ', 'i'), ('ị', 'i'),
    // o variants
    ('ó', 'o'), ('ò', 'o'), ('ỏ', 'o'), ('õ', 'o'), ('ọ', 'o'),
    ('ô', 'o'), ('ồ', 'o'), ('ố', 'o'), ('ổ', 'o'), ('ỗ', 'o'), ('ộ', 'o'),
    ('ơ', 'o'), ('ờ', 'o'), ('ớ', 'o'), ('ở', 'o'), ('ỡ', 'o'), ('ợ', 'o'),
    // u variants
    ('ú', 'u'), ('ù', 'u'), ('ủ', 'u'), ('ũ', 'u'), ('ụ', 'u'),
    ('ư', 'u'), ('ừ', 'u'), ('ứ', 'u'), ('ử', 'u'), ('ữ', 'u'), ('ự', 'u'),
    // y variants
    ('ý', 'y'), ('ỳ', 'y'), ('ỷ', 'y'), ('ỹ', 'y'), ('ỵ', 'y'),
    // d
    ('đ', 'd'),
];

/// P2 FIX: HashMap for O(1) lookups instead of linear scan O(n=67)
/// Uses std::sync::OnceLock (stable since Rust 1.70) — zero external deps
fn vn_char_map() -> &'static HashMap<char, char> {
    static MAP: OnceLock<HashMap<char, char>> = OnceLock::new();
    MAP.get_or_init(|| VN_CHAR_MAP.iter().cloned().collect())
}

/// All Vietnamese diacritical characters (lowercase)
const VN_DIACRITICS: &str = "àáảãạâầấẩẫậăằắẳẵặèéẻẽẹêềếểễệìíỉĩịòóỏõọôồốổỗộơờớởỡợùúủũụưừứửữựỳýỷỹỵđ";

/// Remove all Vietnamese diacritics from a string.
///
/// Useful for search normalization, URL slugs, etc.
/// Example: "Tiếng Việt" → "Tieng Viet"
pub fn remove_diacritics(input: &str) -> String {
    let map = vn_char_map();
    input.chars().map(|c| {
        let lc = c.to_lowercase().next().unwrap_or(c);
        if let Some(&base) = map.get(&lc) {
            if c.is_uppercase() {
                base.to_uppercase().next().unwrap_or(base)
            } else {
                base
            }
        } else {
            c // No mapping: keep original
        }
    }).collect()
}

/// Detect if a string contains Vietnamese diacritical characters
pub fn is_vietnamese(input: &str) -> bool {
    input.chars().any(|c| {
        let lc = c.to_lowercase().next().unwrap_or(c);
        VN_DIACRITICS.contains(lc)
    })
}

/// Count Vietnamese characters in a string
pub fn count_vietnamese_chars(input: &str) -> usize {
    input.chars().filter(|c| {
        let lc = c.to_lowercase().next().unwrap_or(*c);
        VN_DIACRITICS.contains(lc)
    }).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_diacritics() {
        assert_eq!(remove_diacritics("Tiếng Việt"), "Tieng Viet");
        assert_eq!(remove_diacritics("đường"), "duong");
        assert_eq!(remove_diacritics("Hello"), "Hello");
        assert_eq!(remove_diacritics("Đà Nẵng"), "Da Nang");
    }

    #[test]
    fn test_remove_diacritics_preserves_case() {
        assert_eq!(remove_diacritics("TIẾNG"), "TIENG");
        assert_eq!(remove_diacritics("Đẹp"), "Dep");
    }

    #[test]
    fn test_is_vietnamese() {
        assert!(is_vietnamese("Tiếng Việt"));
        assert!(is_vietnamese("Đà Nẵng"));
        assert!(is_vietnamese("một"));
        assert!(!is_vietnamese("Hello World"));
        assert!(!is_vietnamese("12345"));
    }

    #[test]
    fn test_count_vietnamese_chars() {
        assert_eq!(count_vietnamese_chars("Tiếng Việt"), 2); // ế, ệ
        assert_eq!(count_vietnamese_chars("Hello"), 0);
    }

    #[test]
    fn test_all_vowels_covered() {
        // Verify every Vietnamese diacritical char maps to a base
        for c in VN_DIACRITICS.chars() {
            let result = remove_diacritics(&c.to_string());
            assert!(
                result.chars().all(|r| r.is_ascii_alphabetic()),
                "Character '{}' did not map to ASCII: '{}'", c, result
            );
        }
    }
}
