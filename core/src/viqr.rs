/// VIQR Input Method — tone extraction and modifier application
///
/// Implements the VIQR (Vietnamese Quoted-Readable) input rules:
/// - Tone marks: ' = sắc, ` = huyền, ? = hỏi, ~ = ngã, . = nặng
/// - Modifiers: ^ = circumflex (â,ê,ô), + = horn (ơ,ư), ( = breve (ă), dd = đ
///
/// Example: Vie^.t Nam → Việt Nam

use crate::syllable::is_vowel;

/// VIQR tone key mapping
const VIQR_TONE_KEYS: &[(char, u8)] = &[
    ('\'', 1), // sắc
    ('`', 2),  // huyền
    ('?', 3),  // hỏi
    ('~', 4),  // ngã
    ('.', 5),  // nặng
];

/// Extract the tone mark from a VIQR input string.
pub fn extract_tone(input: &str) -> (String, u8) {
    let mut core = String::new();
    let mut tone: u8 = 0;

    for c in input.chars() {
        let mut is_tone = false;
        let has_vowel = core.chars().any(|ch| is_vowel(ch));

        if has_vowel {
            for &(key, tone_val) in VIQR_TONE_KEYS {
                if c == key {
                    tone = tone_val;
                    is_tone = true;
                    break;
                }
            }
        }

        if !is_tone {
            core.push(c);
        }
    }

    (core, tone)
}

/// Apply VIQR modifier rules to a core string.
pub fn apply_modifiers(input: &str) -> String {
    let mut res = input.to_string();

    // Circumflex: a^ → â, e^ → ê, o^ → ô
    res = res.replace("a^", "â").replace("e^", "ê").replace("o^", "ô");
    // Horn: o+ → ơ, u+ → ư
    res = res.replace("o+", "ơ").replace("u+", "ư");
    // Breve: a( → ă
    res = res.replace("a(", "ă");
    // Stroke: dd → đ
    res = res.replace("dd", "đ");

    res
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_viqr_tone_sac() {
        let (core, tone) = extract_tone("a'");
        assert_eq!(core, "a");
        assert_eq!(tone, 1);
    }

    #[test]
    fn test_viqr_tone_huyen() {
        let (core, tone) = extract_tone("a`");
        assert_eq!(core, "a");
        assert_eq!(tone, 2);
    }

    #[test]
    fn test_viqr_tone_no_vowel() {
        let (core, tone) = extract_tone("b'");
        assert_eq!(core, "b'");
        assert_eq!(tone, 0);
    }

    #[test]
    fn test_viqr_modifiers() {
        assert_eq!(apply_modifiers("a^"), "â");
        assert_eq!(apply_modifiers("e^"), "ê");
        assert_eq!(apply_modifiers("o+"), "ơ");
        assert_eq!(apply_modifiers("u+"), "ư");
        assert_eq!(apply_modifiers("a("), "ă");
        assert_eq!(apply_modifiers("dd"), "đ");
    }
}
