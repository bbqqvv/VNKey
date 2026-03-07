/// VNI Input Method — tone extraction and modifier application
///
/// Implements the VNI input rules:
/// - Tone keys: 1=sắc, 2=huyền, 3=hỏi, 4=ngã, 5=nặng, 0=remove tone
/// - Modifiers: a6=â, e6=ê, o6=ô, d9=đ, a8=ă, o7=ơ, u7=ư

use crate::syllable::is_vowel;
use crate::unicode_map::VNI_MODIFIERS;

/// Extract the tone mark from a VNI input string.
///
/// Returns (core_without_tone_keys, tone_index).
/// Number keys are only consumed when there's already a vowel in the core.
pub fn extract_tone(input: &str) -> (String, u8) {
    let mut core = String::new();
    let mut tone: u8 = 0;

    for c in input.chars() {
        let has_vowel = core.chars().any(|ch| is_vowel(ch));

        if has_vowel && "12345".contains(c) {
            tone = c.to_digit(10).unwrap() as u8;
        } else if has_vowel && c == '0' {
            tone = 0;
        } else {
            core.push(c);
        }
    }

    (core, tone)
}

/// Apply VNI modifier rules to a core string.
pub fn apply_modifiers(input: &str) -> String {
    let mut res = input.to_string();
    for &(from, to) in VNI_MODIFIERS {
        res = res.replace(from, to);
    }
    res
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_tone_sac() {
        let (core, tone) = extract_tone("a1");
        assert_eq!(core, "a");
        assert_eq!(tone, 1);
    }

    #[test]
    fn test_extract_tone_no_vowel() {
        // '1' without vowel → literal
        let (core, tone) = extract_tone("1");
        assert_eq!(core, "1");
        assert_eq!(tone, 0);
    }

    #[test]
    fn test_extract_tone_remove() {
        let (core, tone) = extract_tone("a10");
        assert_eq!(core, "a");
        assert_eq!(tone, 0);
    }

    #[test]
    fn test_modifiers_a6() {
        assert_eq!(apply_modifiers("a6"), "â");
    }

    #[test]
    fn test_modifiers_d9() {
        assert_eq!(apply_modifiers("d9"), "đ");
    }
}
