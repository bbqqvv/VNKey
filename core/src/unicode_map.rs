/// Vietnamese IME Constants and Unicode Tone Mapping
///
/// This module contains all the static data tables used by the engine:
/// tone marks, modifier rules, and vowel classification.

/// Tone mark mapping: base vowel → (sắc, huyền, hỏi, ngã, nặng)
/// Using escape sequences to ensure NFC (Precomposed) and avoid encoding issues.
pub const TONE_MAP: &[(char, &str)] = &[
    ('a', "\u{00E1}\u{00E0}\u{1EA3}\u{00E3}\u{1EA1}"),
    ('e', "\u{00E9}\u{00E8}\u{1EBB}\u{1EBD}\u{1EB9}"),
    ('i', "\u{00ED}\u{00EC}\u{1EC9}\u{0129}\u{1ECB}"),
    ('o', "\u{00F3}\u{00F2}\u{1ECF}\u{00F5}\u{1ECD}"),
    ('u', "\u{00FA}\u{00F9}\u{1EE7}\u{0169}\u{1EE5}"),
    ('y', "\u{00FD}\u{1EF3}\u{1EF7}\u{1EF9}\u{1EF5}"),
    ('\u{00E2}', "\u{1EA5}\u{1EA7}\u{1EA9}\u{1EAB}\u{1EAD}"),
    ('\u{00EA}', "\u{1EBF}\u{1EC1}\u{1EC3}\u{1EC5}\u{1EC7}"),
    ('\u{00F4}', "\u{1ED1}\u{1ED3}\u{1ED5}\u{1ED7}\u{1ED9}"),
    ('\u{0103}', "\u{1EAF}\u{1EB1}\u{1EB3}\u{1EB5}\u{1EB7}"),
    ('\u{01A1}', "\u{1EDB}\u{1EDD}\u{1EDF}\u{1EE1}\u{1EE3}"),
    ('\u{01B0}', "\u{1EE9}\u{1EEB}\u{1EED}\u{1EEF}\u{1EF1}"),
];

/// All Vietnamese vowel characters (base + modified)
pub const VOWELS: &str = "aeiouy\u{00E2}\u{00EA}\u{00F4}\u{0103}\u{01A1}\u{01B0}";

/// All Vietnamese vowel variants (including all tone marks) — for robust syllable parsing
pub const ALL_VOWELS: &str = "aeiouy\u{00E2}\u{00EA}\u{00F4}\u{0103}\u{01A1}\u{01B0}\
\u{00E1}\u{00E0}\u{1EA3}\u{00E3}\u{1EA1}\u{00E2}\u{1EA7}\u{1EA5}\u{1EA9}\u{1EAB}\u{1EAD}\
\u{0103}\u{1EB1}\u{1EAF}\u{1EB3}\u{1EB5}\u{1EB7}\u{00E9}\u{00E8}\u{1EBB}\u{1EBD}\u{1EB9}\u{00EA}\u{1EC1}\u{1EBF}\u{1EC3}\u{1EC5}\u{1EC7}\
\u{00ED}\u{00EC}\u{1EC9}\u{0129}\u{1ECB}\u{00F3}\u{00F2}\u{1ECF}\u{00F5}\u{1ECD}\u{00F4}\u{1ED3}\u{1ED1}\u{1ED5}\u{1ED7}\u{1ED9}\
\u{01A1}\u{1EDD}\u{1EDB}\u{1EDF}\u{1EE1}\u{1EE3}\u{00FA}\u{00F9}\u{1EE7}\u{0169}\u{1EE5}\u{01B0}\u{1EEB}\u{1EE9}\u{1EED}\u{1EEF}\u{1EF1}\
\u{1EF3}\u{00FD}\u{1EF7}\u{1EF9}\u{1EF5}\
\u{00C1}\u{00C0}\u{1EA2}\u{00C3}\u{1EA0}\u{00C2}\u{1EA6}\u{1EA4}\u{1EA8}\u{1EAA}\u{1EAC}\u{0102}\u{1EB0}\u{1EAE}\u{1EB2}\u{1EB4}\u{1EB6}\
\u{00C9}\u{00C8}\u{1EBA}\u{1EBC}\u{1EB8}\u{00CA}\u{1EC0}\u{1EBE}\u{1EC2}\u{1EC4}\u{1EC6}\u{00CD}\u{00CC}\u{1EC8}\u{0128}\u{1ECA}\
\u{00D3}\u{00D2}\u{1ECE}\u{00D5}\u{1ECC}\u{00D4}\u{1ED2}\u{1ED0}\u{1ED4}\u{1ED6}\u{1ED8}\u{01A0}\u{1EDC}\u{1EDA}\u{1EDE}\u{1EE0}\u{1EE2}\
\u{00DA}\u{00D9}\u{1EE6}\u{0168}\u{1EE4}\u{01AF}\u{1EEA}\u{1EE8}\u{1EEC}\u{1EEE}\u{1EF0}\u{1EF2}\u{00DD}\u{1EF6}\u{1EF8}\u{1EF4}";

/// Characters that indicate a modified vowel (circumflex, breve, horn)
pub const MODIFIED_VOWELS: &str = "\u{00E2}\u{00EA}\u{00F4}\u{0103}\u{01A1}\u{01B0}\u{00C2}\u{00CA}\u{00D4}\u{0102}\u{01A0}\u{01AF}";

/// Telex modifier rules: input sequence → replacement character
pub const TELEX_MODIFIERS: &[(&str, &str)] = &[
    ("aa", "\u{00E2}"),
    ("ee", "\u{00EA}"),
    ("oo", "\u{00F4}"),
    ("dd", "\u{0111}"),
    ("aw", "\u{0103}"),
    ("ow", "\u{01A1}"),
    ("uw", "\u{01B0}"),
];

/// VNI modifier rules: input sequence → replacement character
pub const VNI_MODIFIERS: &[(&str, &str)] = &[
    ("a6", "\u{00E2}"),
    ("e6", "\u{00EA}"),
    ("o6", "\u{00F4}"),
    ("d9", "\u{0111}"),
    ("a8", "\u{0103}"),
    ("o7", "\u{01A1}"),
    ("u7", "\u{01B0}"),
];

/// Telex tone key mapping: key → tone index (1-5)
pub const TELEX_TONE_KEYS: &[(char, u8)] = &[
    ('s', 1), // sắc
    ('f', 2), // huyền
    ('r', 3), // hỏi
    ('x', 4), // ngã
    ('j', 5), // nặng
];
