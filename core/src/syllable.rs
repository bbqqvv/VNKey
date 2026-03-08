/// Syllable — Vietnamese phonetic syllable decomposition
///
/// A Vietnamese syllable is decomposed into:
/// - onset:  initial consonant(s) — e.g. "qu", "gi", "th", "ng"
/// - vowel:  vowel nucleus — e.g. "a", "iê", "ươ"
/// - coda:   final consonant(s) — e.g. "n", "ng", "ch", "t"
/// - tone:   tone mark index (0 = no tone, 1-5 = sắc/huyền/hỏi/ngã/nặng)
///
/// Represents a decomposed Vietnamese syllable.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Syllable {
    pub onset: String,
    pub vowel: String,
    pub coda: String,
    pub tone: u8,
}

impl Syllable {
    /// Create a new empty syllable
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if this syllable has any vowel content
    pub fn has_vowel(&self) -> bool {
        !self.vowel.is_empty()
    }

    /// Reconstruct the syllable into a string (without tone applied)
    pub fn to_base_string(&self) -> String {
        format!("{}{}{}", self.onset, self.vowel, self.coda)
    }
}

/// Check if a character is a Vietnamese vowel (including all accents)
pub fn is_vowel(c: char) -> bool {
    crate::unicode_map::ALL_VOWELS.contains(c)
}

/// Check if a character is a modified vowel (circumflex, breve, horn)
pub fn is_modified_vowel(c: char) -> bool {
    let lc = c.to_lowercase().next().unwrap_or(c);
    crate::unicode_map::MODIFIED_VOWELS.contains(lc)
}

/// Parse a transformed string into a Syllable struct.
///
/// Handles special Vietnamese onset rules:
/// - "qu" + vowel → onset="qu", vowel starts after 'u'
/// - "gi" + vowel → onset="gi", vowel starts after 'i'
///
/// Also handles vowel cluster normalization:
/// - Duplicate trailing vowels (e.g. "yy") → extra goes to coda
/// - Semi-vowel 'y' after 'a'/'â' diphthong → stays in vowel but handled by tone
pub fn parse(transformed: &str, tone: u8) -> Syllable {
    let mut syl = Syllable::new();
    syl.tone = tone;

    if transformed.is_empty() {
        return syl;
    }

    let chars: Vec<char> = transformed.chars().collect();
    let mut v_start = -1;
    let mut v_end = -1;

    // Find vowel cluster boundaries
    for (i, &c) in chars.iter().enumerate() {
        if is_vowel(c) {
            if v_start == -1 {
                v_start = i as i32;
            }
            v_end = i as i32;
        }
    }

    if v_start != -1 {
        let mut onset_idx = v_start as usize;
        let mut vowel_idx_start = v_start as usize;

        // Special onset cases: GI and QU
        // "gi" + vowel(s)
        let is_gi = chars[..vowel_idx_start]
            .iter()
            .collect::<String>()
            .to_lowercase()
            == "g"
            && chars[vowel_idx_start] == 'i';

        let is_qu = chars[..vowel_idx_start]
            .iter()
            .collect::<String>()
            .to_lowercase()
            == "q"
            && chars[vowel_idx_start] == 'u';

        if is_gi {
            if vowel_idx_start == v_end as usize {
                // Keep "g" + "i" as onset="g", vowel="i" if i is the only vowel
            } else {
                // "gi" + more vowels -> onset="gi", strip first 'i' from vowel
                onset_idx += 1;
                vowel_idx_start += 1;
            }
        } else if is_qu {
            if vowel_idx_start <= v_end as usize {
                onset_idx += 1;
                vowel_idx_start += 1;
            }
        }

        let mut has_inner_consonant = false;
        if chars[v_start as usize..=v_end as usize]
            .iter()
            .any(|&c| !is_vowel(c))
        {
            has_inner_consonant = true;
        }

        if has_inner_consonant {
            // Not a valid Vietnamese vowel cluster (monosyllabic)
            // e.g. "facebook" (a-e-o), "linux" (i-u)
            syl.onset = transformed.to_string();
            syl.vowel = String::new();
            syl.coda = String::new();
            return syl;
        }

        let onset: String = chars[..onset_idx].iter().collect();
        let mut vowel: String = chars[vowel_idx_start..=(v_end as usize)].iter().collect();
        let mut coda: String = chars[(v_end as usize + 1)..].iter().collect();

        // --- VOWEL CLUSTER NORMALIZATION ---
        // Duplicate trailing vowels: "ayy" -> vowel="ay", coda="y"
        let v_chars: Vec<char> = vowel.chars().collect();
        if v_chars.len() >= 2 {
            let last = v_chars[v_chars.len() - 1];
            let second_last = v_chars[v_chars.len() - 2];
            if last == second_last {
                vowel = v_chars[..v_chars.len() - 1].iter().collect();
                coda = format!("{}{}", last, coda);
            }
        }

        syl.onset = onset;
        syl.vowel = vowel;
        syl.coda = coda;
    } else {
        syl.onset = transformed.to_string();
    }

    syl
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let s = parse("ban", 0);
        assert_eq!(s.onset, "b");
        assert_eq!(s.vowel, "a");
        assert_eq!(s.coda, "n");
    }

    #[test]
    fn test_parse_qu() {
        let s = parse("quan", 0);
        assert_eq!(s.onset, "qu");
        assert_eq!(s.vowel, "a");
        assert_eq!(s.coda, "n");
    }

    #[test]
    fn test_parse_gi() {
        let s = parse("giao", 0);
        assert_eq!(s.onset, "gi");
        assert_eq!(s.vowel, "ao");
        assert_eq!(s.coda, "");
    }

    #[test]
    fn test_parse_gi_alone() {
        let s = parse("gi", 0);
        assert_eq!(s.onset, "g");
        assert_eq!(s.vowel, "i");
    }

    #[test]
    fn test_parse_consonant_only() {
        let s = parse("th", 0);
        assert_eq!(s.onset, "th");
        assert_eq!(s.vowel, "");
    }

    #[test]
    fn test_is_vowel() {
        assert!(is_vowel('a'));
        assert!(is_vowel('ư'));
        assert!(is_vowel('ơ'));
        assert!(!is_vowel('b'));
        assert!(!is_vowel('t'));
    }

    #[test]
    fn test_parse_duplicate_vowel_yy() {
        // "thayy" → vowel="ay", coda="y" (not vowel="ayy")
        let s = parse("thayy", 0);
        assert_eq!(s.onset, "th");
        assert_eq!(s.vowel, "ay");
        assert_eq!(s.coda, "y");
    }

    #[test]
    fn test_parse_duplicate_vowel_aa() {
        // After modifiers, "aa" becomes "â", so this tests raw "aa"
        let s = parse("baa", 0);
        assert_eq!(s.onset, "b");
        assert_eq!(s.vowel, "a");
        assert_eq!(s.coda, "a");
    }
}
