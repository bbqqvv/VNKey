/// Tone Placement — Vietnamese tone mark positioning algorithm
///
/// Rules for placing tone marks on the correct vowel:
/// 1. Single vowel → place on that vowel
/// 2. Two vowels:
///    - If either has a modifier (â, ê, ô, ă, ơ, ư) → place on modified one
///    - Special pairs (oa, oe, uy, ue, uo) → place on second
///    - "qu" + vowel pair → place on second vowel
///    - "gi" + "ia" → place on second vowel
///    - Otherwise → place on first
/// 3. Three+ vowels → place on second (middle)

use crate::unicode_map::{TONE_MAP, MODIFIED_VOWELS};
use crate::syllable::is_vowel;

/// Tone mark identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToneMark {
    None  = 0,
    Sac   = 1, // sắc (acute)
    Huyen = 2, // huyền (grave)
    Hoi   = 3, // hỏi (hook above)
    Nga   = 4, // ngã (tilde)
    Nang  = 5, // nặng (dot below)
}

impl ToneMark {
    /// Convert from u8 index (0-5) to ToneMark
    pub fn from_index(idx: u8) -> Self {
        match idx {
            1 => ToneMark::Sac,
            2 => ToneMark::Huyen,
            3 => ToneMark::Hoi,
            4 => ToneMark::Nga,
            5 => ToneMark::Nang,
            _ => ToneMark::None,
        }
    }

    /// Convert to u8 index
    pub fn as_index(&self) -> u8 {
        *self as u8
    }
}

/// Check if a character has a vowel modifier (circumflex, breve, horn)
fn has_modifier(c: char) -> bool {
    let lc = c.to_lowercase().next().unwrap_or(c);
    MODIFIED_VOWELS.contains(lc)
}

/// Apply a tone mark to a word, returning the toned string.
///
/// Returns `None` if no suitable vowel is found.
pub fn place_tone(word: &str, tone: u8) -> Option<String> {
    if tone == 0 { return Some(word.to_string()); }

    let mut chars: Vec<char> = word.chars().collect();

    // Collect vowel indices
    let vowel_indices: Vec<usize> = chars.iter()
        .enumerate()
        .filter(|(_, &c)| is_vowel(c))
        .map(|(i, _)| i)
        .collect();

    if vowel_indices.is_empty() { return None; }

    // Helper: lowercase a char (handles both ASCII and Vietnamese)
    let lower = |c: char| -> char {
        c.to_lowercase().next().unwrap_or(c)
    };

    // Determine which vowel gets the tone
    let target = if vowel_indices.len() == 1 {
        vowel_indices[0]
    } else if vowel_indices.len() == 2 {
        let pair: String = vowel_indices.iter()
            .map(|&i| lower(chars[i]))
            .collect();
        let lower_word = word.to_lowercase();

        if (lower_word.starts_with("qu") && ["ua", "uâ", "uơ", "uô"].contains(&pair.as_str()))
            || (lower_word.starts_with("gi") && pair == "ia")
        {
            vowel_indices[1]
        } else if has_modifier(chars[vowel_indices[0]]) && has_modifier(chars[vowel_indices[1]]) {
            // Nếu cả 2 đều có modifier (ươ), ưu tiên cái thứ 2 (ươ -> ườn, ướt)
            vowel_indices[1]
        } else if has_modifier(chars[vowel_indices[0]]) {
            vowel_indices[0]
        } else if has_modifier(chars[vowel_indices[1]]) {
            vowel_indices[1]
        } else if ["oa", "oe", "uy", "ue", "uo", "uô"].contains(&pair.as_str()) {
            // place_tone default is traditional
            if pair == "uy" || pair == "uô" || chars.len() > vowel_indices[1] + 1 { 
                // if uy, uô OR if there's a coda consonant (e.g. oán, uý, uốn) -> second vowel
                vowel_indices[1]
            } else {
                vowel_indices[0] // hòa, xòe
            }
        } else {
            vowel_indices[0]
        }
    } else {
        // 3+ vowels: tone goes on the middle one
        vowel_indices[1]
    };

    // Look up the toned character
    let target_char = chars[target];
    let target_lower = lower(target_char);
    for &(base, toned_str) in TONE_MAP {
        if target_lower == base {
            let replacement = toned_str.chars().nth((tone - 1) as usize)?;
            chars[target] = if target_char.is_uppercase() {
                replacement.to_uppercase().next().unwrap_or(replacement)
            } else {
                replacement
            };
            return Some(chars.into_iter().collect());
        }
    }

    None
}

/// Apply a tone mark with configurable placement style.
///
/// Modern (mới): "hoà", "loà" — tone on last vowel in oa/oe
/// Traditional (cũ): "hòa", "lòa" — tone on first vowel in oa/oe
pub fn place_tone_with_style(
    word: &str,
    tone: u8,
    style: crate::config::TonePlacement,
) -> Option<String> {
    if tone == 0 { return Some(word.to_string()); }

    let mut chars: Vec<char> = word.chars().collect();

    let vowel_indices: Vec<usize> = chars.iter()
        .enumerate()
        .filter(|(_, &c)| is_vowel(c))
        .map(|(i, _)| i)
        .collect();

    if vowel_indices.is_empty() { return None; }

    let lower = |c: char| -> char {
        c.to_lowercase().next().unwrap_or(c)
    };

    let target = if vowel_indices.len() == 1 {
        vowel_indices[0]
    } else if vowel_indices.len() == 2 {
        let pair: String = vowel_indices.iter()
            .map(|&i| lower(chars[i]))
            .collect();
        let lower_word = word.to_lowercase();

        if (lower_word.starts_with("qu") && ["ua", "uâ", "uơ", "uô"].contains(&pair.as_str()))
            || (lower_word.starts_with("gi") && pair == "ia")
        {
            vowel_indices[1]
        } else if has_modifier(chars[vowel_indices[0]]) && has_modifier(chars[vowel_indices[1]]) {
            vowel_indices[1]
        } else if has_modifier(chars[vowel_indices[0]]) {
            vowel_indices[0]
        } else if has_modifier(chars[vowel_indices[1]]) {
            vowel_indices[1]
        } else if ["oa", "oe", "uy", "ue", "uo", "uô"].contains(&pair.as_str()) {
            // Style-dependent: oa/oe/uy/ue/uo diphthongs
            if pair == "uô" || chars.len() > vowel_indices[1] + 1 {
                vowel_indices[1] // if it has a coda consonant, tone always goes to 2nd (oán, oét, uýnh, uốn)
            } else {
                match style {
                    crate::config::TonePlacement::Modern => vowel_indices[1],    // hoà, tuý, thuý, xoè
                    crate::config::TonePlacement::Traditional => vowel_indices[0], // hòa, túy, thúy, xòe
                }
            }
        } else {
            vowel_indices[0]
        }
    } else {
        vowel_indices[1]
    };

    let target_char = chars[target];
    let target_lower = lower(target_char);
    for &(base, toned_str) in TONE_MAP {
        if target_lower == base {
            let replacement = toned_str.chars().nth((tone - 1) as usize)?;
            chars[target] = if target_char.is_uppercase() {
                replacement.to_uppercase().next().unwrap_or(replacement)
            } else {
                replacement
            };
            return Some(chars.into_iter().collect());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tone_sac() {
        assert_eq!(place_tone("ba", 1), Some("bá".to_string()));
        assert_eq!(place_tone("an", 1), Some("án".to_string()));
    }

    #[test]
    fn test_tone_huyen() {
        assert_eq!(place_tone("ba", 2), Some("bà".to_string()));
    }

    #[test]
    fn test_tone_on_modified_vowel() {
        assert_eq!(place_tone("bân", 1), Some("bấn".to_string()));
        // Both ư and ơ are modified; tone goes on second modified vowel if coda exists: ươn → ướn
        assert_eq!(place_tone("\u{01B0}\u{01A1}n", 1), Some("\u{01B0}\u{1EDB}n".to_string()));
    }

    #[test]
    fn test_tone_oa_pair() {
        assert_eq!(place_tone("loan", 1), Some("loán".to_string()));
        
        // Default place_tone should be Traditional
        assert_eq!(place_tone("hoa", 2), Some("hòa".to_string()));
        
        // Modern Style
        assert_eq!(place_tone_with_style("hoa", 2, crate::config::TonePlacement::Modern), Some("hoà".to_string()));
    }

    #[test]
    fn test_tone_uy_always_second() {
        // Traditional style: túy (index 1 is 'u')
        // Modern style: tuý (index 2 is 'y')
        let t_style = crate::config::TonePlacement::Traditional;
        let m_style = crate::config::TonePlacement::Modern;
        
        assert_eq!(place_tone_with_style("tuy", 1, t_style).unwrap(), "t\u{00FA}y");
        assert_eq!(place_tone_with_style("tuy", 1, m_style).unwrap(), "tu\u{00FD}");
    }

    #[test]
    fn test_tone_three_vowels() {
        assert_eq!(place_tone("oai", 1), Some("oái".to_string()));
    }

    #[test]
    fn test_no_tone() {
        assert_eq!(place_tone("ban", 0), Some("ban".to_string()));
    }

    #[test]
    fn test_consonant_only() {
        assert_eq!(place_tone("th", 1), None);
    }
}
