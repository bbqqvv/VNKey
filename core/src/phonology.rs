/// Phonology — Vietnamese linguistic and phonological rules.
///
/// This module implements "Phonotactics" — the study of the rules governing 
/// the combinations of phonemes in the Vietnamese language.
/// 
/// A "Smart" IME must understand if a syllable is linguistically valid.

use crate::syllable::Syllable;

/// Valid Vietnamese Onsets (Âm đầu) - Sorted for binary search
pub const ONSETS: &[&str] = &[
    "", "b", "c", "ch", "d", "g", "gh", "gi", "h", "k", "kh", "l", "m", "n", 
    "ng", "ngh", "nh", "p", "ph", "qu", "r", "s", "t", "th", "tr", "v", "x", "đ"
];

/// Valid Vietnamese Codas (Âm cuối) - Sorted for binary search
pub const CODAS: &[&str] = &[
    "", "c", "ch", "i", "m", "n", "ng", "nh", "o", "p", "t", "u", "y"
];

/// Valid Vietnamese Vowel Clusters (Vần) - Sorted for binary search
pub const VOWEL_CLUSTERS: &[&str] = &[
    "a", "ai", "ao", "au", "ay", "e", "eo", "i", "ia", "iê", "o", "oa", "oai", "oao", "oay", "oe", "oi", "oo", "oă", "u", "ua", "uê", "ui", "uôi", "uôn", "uông", "uô", "uy", "uya", "uyê", "uân", "uâng", "uất", "uơ", "uâ", "uă", "y", "ya", "yê", "yêu", "â", "âu", "ây", "ă", "ê", "êu", "ô", "ôi", "ông", "ôn", "ơ", "ơi", "ơm", "ơn", "ơng", "ư", "ưa", "ươi", "ươn", "ương", "ươ", "ưu"
];

/// Intermediate Vowel Clusters - valid during typing but not as final words
pub const INTERMEDIATE_VOWEL_CLUSTERS: &[&str] = &[
    "ie", "uo", "ue", "uâ"
];

/// Check if an onset is linguistically valid.
pub fn is_valid_onset(onset: &str, allow_foreign: bool) -> bool {
    let lower = onset.to_lowercase();
    if ONSETS.binary_search(&lower.as_str()).is_ok() {
        return true;
    }
    // Optimization: Allow 'ww', 'dd', etc. as they might be intermediate Telex states
    if matches!(lower.as_str(), "ww" | "dd" | "ss" | "ff" | "rr" | "xx" | "jj") {
        return true;
    }
    if allow_foreign {
        return matches!(lower.as_str(), "z" | "w" | "j" | "f");
    }
    false
}

/// Check if a coda is linguistically valid.
pub fn is_valid_coda(coda: &str, allow_foreign: bool) -> bool {
    let lower = coda.to_lowercase();
    if CODAS.binary_search(&lower.as_str()).is_ok() {
        return true;
    }
    if allow_foreign {
        return matches!(lower.as_str(), "z" | "w" | "j" | "f");
    }
    false
}

/// Check if the combination of Onset and Vowel is valid.
/// Vietnamese spelling rules:
/// - 'k', 'gh', 'ngh' only stand before 'i', 'e', 'ê', 'iê/yê'.
/// - 'c', 'g', 'ng' stand before 'a', 'o', 'ô', 'ơ', 'u', 'ư'.
pub fn is_valid_spelling(onset: &str, vowel: &str) -> bool {
    let lo = onset.to_lowercase();
    let lv = vowel.to_lowercase();
    
    if lv.is_empty() { return true; }
    
    let first_v = lv.chars().next().unwrap();
    
    match lo.as_str() {
        "k" | "gh" | "ngh" => {
            // Must be followed by front vowels: i, e, ê
            matches!(first_v, 'i' | 'e' | 'ê' | 'y')
        },
        "c" | "g" | "ng" => {
            // Cannot be followed by front vowels (except some loanwords, but standard VN rules say NO)
            !matches!(first_v, 'i' | 'e' | 'ê')
        },
        "qu" => {
            // Usually not followed by 'u' (except maybe "quu" in some rare cases, but mostly no)
            first_v != 'u'
        },
        _ => true
    }
}

/// Detailed Syllable Integrity Check.
/// Returns a score from 0 to 100.
pub fn validate_syllable(syl: &Syllable, allow_foreign: bool) -> u8 {
    if !is_valid_onset(&syl.onset, allow_foreign) { 
        #[cfg(test)] println!("Invalid onset: '{}'", syl.onset);
        return 5; // Very low score for invalid onset
    }
    if !is_valid_coda(&syl.coda, allow_foreign) { 
        #[cfg(test)] println!("Invalid coda: '{}'", syl.coda);
        return 8; // Low score for invalid coda (typo?)
    }
    if !is_valid_spelling(&syl.onset, &syl.vowel) { return 10; } // Spelling mistake (standard)
    
    if syl.vowel.is_empty() {
        if syl.onset.is_empty() { return 0; }
        return 50; // Partial syllable (just onset)
    }

    // Check for impossible vowel clusters (e.g., "aoa" is invalid, "oao" is valid)
    let lower_vowel = syl.vowel.to_lowercase();
    
    // P13: Strict validation for perfection
    if VOWEL_CLUSTERS.contains(&lower_vowel.as_str()) {
        return 100;
    }

    // P13: Leniency for intermediate typing states (ie, uo, ue)
    if INTERMEDIATE_VOWEL_CLUSTERS.contains(&lower_vowel.as_str()) {
        return 50; 
    }

    // P13: Very long vowels are suspicious but common in English (e.g. "beautiful")
    if lower_vowel.chars().count() > 3 {
        #[cfg(test)] println!("Long vowel cluster: '{}'", lower_vowel);
        return 20;
    }

    #[cfg(test)] println!("Invalid vowel cluster: '{}'", lower_vowel);
    2 // Completely invalid vowel cluster
}

/// Check if the syllable is 100% linguistically perfect.
pub fn is_perfect(syl: &Syllable, allow_foreign: bool) -> bool {
    validate_syllable(syl, allow_foreign) == 100
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syllable::Syllable;

    #[test]
    fn test_onset_validation() {
        assert!(is_valid_onset("ngh", false));
        assert!(is_valid_onset("tr", false));
        assert!(!is_valid_onset("z", false)); // 'z' is not a standard VN onset
        assert!(is_valid_onset("z", true));  // 'z' is valid if allowed
        assert!(is_valid_onset("", false)); // Empty is valid (start with vowel)
    }

    #[test]
    fn test_spelling_rules() {
        // k/gh/ngh rules
        assert!(is_valid_spelling("k", "i"));
        assert!(is_valid_spelling("gh", "ê"));
        assert!(is_valid_spelling("ngh", "e"));
        assert!(!is_valid_spelling("k", "a"));
        assert!(!is_valid_spelling("gh", "o"));
        assert!(!is_valid_spelling("ngh", "u"));

        // c/g/ng rules
        assert!(is_valid_spelling("c", "a"));
        assert!(is_valid_spelling("g", "ô"));
        assert!(is_valid_spelling("ng", "ư"));
        assert!(!is_valid_spelling("c", "i"));
        assert!(!is_valid_spelling("g", "ê"));
        assert!(!is_valid_spelling("ng", "e"));
    }

    #[test]
    fn test_validity_score() {
        let perfect = Syllable {
            onset: "ngh".to_string(),
            vowel: "iê".to_string(),
            coda: "p".to_string(),
            tone: 1,
        };
        assert_eq!(validate_syllable(&perfect, false), 100);

        let typo = Syllable {
            onset: "k".to_string(),
            vowel: "a".to_string(),
            coda: "n".to_string(),
            tone: 0,
        };
        assert_eq!(validate_syllable(&typo, false), 10);

        let invalid_coda = Syllable {
            onset: "b".to_string(),
            vowel: "a".to_string(),
            coda: "z".to_string(),
            tone: 0,
        };
        // P13: Invalid coda now returns 8 (suspicious but not garbage)
        assert_eq!(validate_syllable(&invalid_coda, false), 8);
        assert_eq!(validate_syllable(&invalid_coda, true), 100); // Valid with foreign allow
    }
}
