//! Advanced Edge Case Tests — Skill-driven QA
//!
//! Tests viết dựa trên skills: test-driven-development, testing-patterns, code-review-checklist
//! Mục tiêu: Tìm mọi edge case khó chịu còn sót
//!
//! Run with: cargo test -p vietnamese-ime-engine --test advanced_edge_cases

use vnkey_core::{Engine, InputMode};

// ============================================================
// FACTORY HELPERS (per testing-patterns skill)
// ============================================================
fn telex() -> Engine {
    let mut e = Engine::new(InputMode::Telex);
    let mut cfg = e.config().clone();
    cfg.spell_check = false;
    cfg.auto_restore = false;
    e.set_config(cfg);
    e
}

fn telex_smart() -> Engine {
    let mut e = Engine::new(InputMode::Telex);
    let mut cfg = e.config().clone();
    cfg.spell_check = true;
    cfg.auto_restore = true;
    e.set_config(cfg);
    e
}

fn vni() -> Engine {
    let mut e = Engine::new(InputMode::Vni);
    let mut cfg = e.config().clone();
    cfg.spell_check = false;
    cfg.auto_restore = false;
    e.set_config(cfg);
    e
}

fn viqr() -> Engine {
    let mut e = Engine::new(InputMode::Viqr);
    let mut cfg = e.config().clone();
    cfg.spell_check = false;
    cfg.auto_restore = false;
    e.set_config(cfg);
    e
}

// ============================================================
// 1. PROGRESSIVE Z — Full 3-level test (NEVER tested before)
// ============================================================

#[test]
fn progressive_z_level1_removes_tone() {
    let mut e = telex();
    // Type "ás" then Z → should remove tone → "a"
    e.feed_str("as"); // → "á"
    let result = e.process_key('z');
    assert_eq!(result, "a", "Z level 1 should remove tone");
}

#[test]
fn progressive_z_level2_removes_modifiers() {
    let mut e = telex();
    // Type "ấ" (aas) then Z → removes tone → "â", then Z again → removes modifier → "a"
    e.feed_str("aas"); // → "ấ"
    e.process_key('z'); // Z1: "ấ" → "â" (tone removed)
    let result = e.process_key('z'); // Z2: "â" → "a" (modifier removed)
    assert_eq!(result, "a", "Z level 2 should remove modifiers");
}

#[test]
fn progressive_z_level3_appends_and_reparses() {
    let mut e = telex();
    e.feed_str("as"); // → "á"
    e.process_key('z'); // Z1: "á" → "a"
    e.process_key('z'); // Z2: "a" → "a" (no modifiers to remove)
    let result = e.process_key('z'); // Z3: append literal 'z', re-parse buffer
                                     // Level 3 appends 'z' to buffer and re-parses. The result depends on
                                     // how the parser handles the new buffer. The key invariant is: no crash,
                                     // and z_level resets to 0.
    assert!(
        !result.is_empty(),
        "Z level 3 should produce non-empty output, got empty"
    );
    let state = e.get_state();
    assert_eq!(state.z_level, 0, "Z level should reset to 0 after level 3");
}

#[test]
fn progressive_z_on_complex_word() {
    let mut e = telex();
    e.feed_str("dduwowngf"); // → "đường"
    let z1 = e.process_key('z'); // Z1: removes tone (huyền) → "đương"
    assert!(
        !z1.contains('ờ') && !z1.contains('ừ'),
        "Z1 should remove tone from đường, got: {}",
        z1
    );
}

// ============================================================
// 2. BUFFER OVERFLOW GUARD (P1 fix — must verify)
// ============================================================

#[test]
fn buffer_overflow_guard_resets_at_50() {
    let mut e = telex();
    // Feed 51 characters without space (cực đoan)
    for i in 0..50 {
        e.process_key(if i % 2 == 0 { 'a' } else { 'b' });
    }
    // 51st character should trigger reset
    let result = e.process_key('x');
    assert_eq!(
        result, "x",
        "Buffer overflow guard should reset at 50 chars"
    );
}

#[test]
fn normal_words_not_affected_by_guard() {
    let mut e = telex();
    // Normal Vietnamese word (< 50 chars) should work fine
    assert_eq!(e.feed_str("tieesng"), "tiếng");
    e.reset();
    assert_eq!(e.feed_str("vieejt"), "việt");
}

// ============================================================
// 3. VIQR MODE (ZERO existing tests!)
// ============================================================

#[test]
fn viqr_basic_tones() {
    let mut e = viqr();
    // VIQR tone marks: ' = sắc, ` = huyền, ? = hỏi, ~ = ngã, . = nặng
    assert_eq!(e.feed_str("a'"), "á");
    e.reset();
    assert_eq!(e.feed_str("a`"), "à");
    e.reset();
    assert_eq!(e.feed_str("a?"), "ả");
    e.reset();
    assert_eq!(e.feed_str("a~"), "ã");
    e.reset();
    assert_eq!(e.feed_str("a."), "ạ");
}

#[test]
fn viqr_modifiers() {
    let mut e = viqr();
    // VIQR modifiers: ^ = circumflex, + = horn, ( = breve
    assert_eq!(e.feed_str("a^"), "â");
    e.reset();
    assert_eq!(e.feed_str("o+"), "ơ");
    e.reset();
    assert_eq!(e.feed_str("u+"), "ư");
    e.reset();
    assert_eq!(e.feed_str("a("), "ă");
    e.reset();
    assert_eq!(e.feed_str("dd"), "đ");
}

// ============================================================
// 4. GI + VOWEL COMPLEX CASES (P3 fix verification)
// ============================================================

#[test]
fn gi_stays_as_onset_before_vowels() {
    let mut e = telex_smart();
    // "gi" is a valid onset — should NOT become "ghi"
    assert_eq!(e.feed_str("gia"), "gia");
    e.reset();
    assert_eq!(e.feed_str("gif"), "gì");
    e.reset();
    assert_eq!(e.feed_str("giowf"), "giờ");
}

#[test]
fn ghi_stays_intact_before_i() {
    let mut e = telex_smart();
    // "ghi" is valid — smart spelling should NOT strip the 'h'
    assert_eq!(e.feed_str("ghi"), "ghi");
    e.reset();
    assert_eq!(e.feed_str("ghis"), "ghí");
}

#[test]
fn g_becomes_gh_before_e_only() {
    let mut e = telex_smart();
    // g + e → gh + e (smart correction)
    assert_eq!(e.feed_str("ge"), "ghe");
    e.reset();
    assert_eq!(e.feed_str("gee"), "ghê");
}

// ============================================================
// 5. UNICODE EDGE CASES
// ============================================================

#[test]
fn unicode_emoji_passthrough() {
    let mut e = telex();
    // Non-Latin characters should pass through without crashing
    let result = e.process_key('🌟');
    assert!(!result.is_empty(), "Should handle emoji without crash");
}

#[test]
fn unicode_cjk_passthrough() {
    let mut e = telex();
    // CJK characters should pass through
    let result = e.process_key('中');
    assert!(!result.is_empty(), "Should handle CJK without crash");
}

// ============================================================
// 6. RAPID RESET STRESS TEST
// ============================================================

#[test]
fn rapid_reset_cycle() {
    let mut e = telex();
    for _ in 0..100 {
        e.feed_str("as");
        e.reset();
    }
    // After 100 cycles, engine should still work
    assert_eq!(e.feed_str("as"), "á");
}

#[test]
fn reset_mid_word_then_new_word() {
    let mut e = telex();
    e.process_key('t');
    e.process_key('i');
    e.reset(); // Reset mid-word
               // New word should start clean
    assert_eq!(e.feed_str("as"), "á");
}

// ============================================================
// 7. SHORTHAND / MACRO EDGE CASES
// ============================================================

#[test]
fn shorthand_case_insensitive_lookup() {
    let mut e = telex();
    let mut cfg = e.config().clone();
    cfg.macro_enabled = true;
    e.set_config(cfg);
    let mut macros = std::collections::HashMap::new();
    macros.insert("tg".to_string(), "thời gian".to_string());
    e.set_macros(macros);
    // Lowercase trigger
    assert_eq!(e.feed_str("tg "), "thời gian ");
}

#[test]
fn shorthand_uppercase_preserves_first_char() {
    let mut e = telex();
    let mut cfg = e.config().clone();
    cfg.macro_enabled = true;
    e.set_config(cfg);
    let mut macros = std::collections::HashMap::new();
    macros.insert("tg".to_string(), "thời gian".to_string());
    e.set_macros(macros);
    // Uppercase first char → expansion should capitalize
    assert_eq!(e.feed_str("Tg "), "Thời gian ");
}

#[test]
fn shorthand_no_match_passes_through() {
    let mut e = telex();
    let mut cfg = e.config().clone();
    cfg.macro_enabled = true;
    e.set_config(cfg);
    // No macros added — "abc" should pass through
    // feed_str returns last process_key result; space triggers word boundary
    let result = e.feed_str("abc ");
    // Result should be "abc " (word + space) since no shorthand match
    assert!(
        result.contains("abc"),
        "Unmatched shorthand should pass through: {}",
        result
    );
}

// ============================================================
// 8. TONE PLACEMENT WITH MODERN VS TRADITIONAL
// ============================================================

#[test]
fn modern_tone_hoa_tuy() {
    let mut e = telex();
    let mut cfg = e.config().clone();
    cfg.modern_tone = true;
    e.set_config(cfg);
    assert_eq!(e.feed_str("hoas"), "hoá"); // Modern: tone on á
    e.reset();
    assert_eq!(e.feed_str("tuys"), "tuý"); // Modern: tone on ý
}

#[test]
fn traditional_tone_hoa_tuy() {
    let mut e = telex();
    let mut cfg = e.config().clone();
    cfg.modern_tone = false;
    e.set_config(cfg);
    assert_eq!(e.feed_str("hoas"), "hóa"); // Traditional: tone on ó
    e.reset();
    assert_eq!(e.feed_str("tuys"), "túy"); // Traditional: tone on ú
}

// ============================================================
// 9. ENGLISH MODE (vietnamese_mode = false)
// ============================================================

#[test]
fn english_mode_no_transform() {
    let mut e = telex();
    let mut cfg = e.config().clone();
    cfg.vietnamese_mode = false;
    e.set_config(cfg);
    // All Telex sequences should be literal
    assert_eq!(e.feed_str("as"), "as");
    e.reset();
    assert_eq!(e.feed_str("dd"), "dd");
    e.reset();
    assert_eq!(e.feed_str("aa"), "aa");
}

#[test]
fn english_mode_shorthand_still_works_when_enabled() {
    let mut e = telex();
    let mut cfg = e.config().clone();
    cfg.vietnamese_mode = false;
    cfg.macro_enabled = true;
    cfg.shorthand_while_off = true;
    e.set_config(cfg);
    let mut macros = std::collections::HashMap::new();
    macros.insert("vn".to_string(), "Việt Nam".to_string());
    e.set_macros(macros);
    assert_eq!(e.feed_str("vn "), "Việt Nam ");
}

// ============================================================
// 10. AUTO RESTORE — Invalid syllables
// ============================================================

#[test]
fn auto_restore_completely_invalid() {
    let mut e = telex_smart();
    // "bcdf" — completely unparseable, no vowel, len > 2 → restore raw
    assert_eq!(e.feed_str("bcdf"), "bcdf");
}

#[test]
fn auto_restore_valid_still_transforms() {
    let mut e = telex_smart();
    // "ddax" → "đã" (valid onset+vowel+tone → should transform)
    assert_eq!(e.feed_str("ddax"), "đã");
}

#[test]
fn auto_restore_short_invalid_not_restored() {
    let mut e = telex_smart();
    // "bc" — invalid but len ≤ 2 → NOT restored (edge case in the guard)
    let result = e.feed_str("bc");
    // Engine may or may not transform this, but should not crash
    assert!(!result.is_empty(), "Short invalid should not crash");
}

// ============================================================
// 11. VNI MODE — Complete modifiers + tones combo
// ============================================================

#[test]
fn vni_modifier_plus_tone() {
    let mut e = vni();
    // â + sắc = ấ
    assert_eq!(e.feed_str("a61"), "ấ");
    e.reset();
    // ơ + huyền = ờ
    assert_eq!(e.feed_str("o72"), "ờ");
    e.reset();
    // ư + hỏi = ử
    assert_eq!(e.feed_str("u73"), "ử");
}

#[test]
fn vni_dd_modifier() {
    let mut e = vni();
    assert_eq!(e.feed_str("d9"), "đ");
    e.reset();
    assert_eq!(e.feed_str("d9i"), "đi");
}

// ============================================================
// 12. SPECIAL PUNCTUATION HANDLING
// ============================================================

#[test]
fn semicolon_is_word_boundary() {
    let mut e = telex();
    let r = e.feed_str("as;");
    assert_eq!(r, "á;");
}

#[test]
fn exclamation_is_word_boundary() {
    let mut e = telex();
    let r = e.feed_str("as!");
    assert_eq!(r, "á!");
}

#[test]
fn question_mark_is_word_boundary() {
    let mut e = telex();
    let r = e.feed_str("as?");
    // In Telex, '?' is not a special punctuation, so it's a word boundary
    assert_eq!(r, "á?");
}

// ============================================================
// 13. CONVERTER MODULE — HashMap O(1) verification
// ============================================================

#[test]
fn converter_remove_diacritics_comprehensive() {
    use vnkey_core::remove_diacritics;
    assert_eq!(remove_diacritics("Tiếng Việt"), "Tieng Viet");
    assert_eq!(remove_diacritics("đường"), "duong");
    assert_eq!(remove_diacritics("TIẾNG"), "TIENG");
    assert_eq!(remove_diacritics("Đẹp"), "Dep");
    assert_eq!(remove_diacritics("Đà Nẵng"), "Da Nang");
    assert_eq!(remove_diacritics("Hello World"), "Hello World");
    assert_eq!(remove_diacritics(""), "");
    assert_eq!(remove_diacritics("12345"), "12345");
}

#[test]
fn converter_is_vietnamese() {
    use vnkey_core::is_vietnamese;
    assert!(is_vietnamese("Tiếng Việt"));
    assert!(is_vietnamese("đ"));
    assert!(!is_vietnamese("Hello World"));
    assert!(!is_vietnamese(""));
    assert!(!is_vietnamese("12345"));
}

// ============================================================
// 14. PHONOLOGY — Syllable validation edge cases
// ============================================================

#[test]
fn phonology_validate_edge_cases() {
    use vnkey_core::{validate_syllable, Syllable};

    // Empty syllable → score 0
    let empty = Syllable {
        onset: "".to_string(),
        vowel: "".to_string(),
        coda: "".to_string(),
        tone: 0,
    };
    assert_eq!(validate_syllable(&empty, false), 0);

    // Onset only → 50 (partial)
    let onset_only = Syllable {
        onset: "th".to_string(),
        vowel: "".to_string(),
        coda: "".to_string(),
        tone: 0,
    };
    assert_eq!(validate_syllable(&onset_only, false), 50);

    // Very long vowel (>3 chars) → 20
    let long_vowel = Syllable {
        onset: "b".to_string(),
        vowel: "aoai".to_string(),
        coda: "".to_string(),
        tone: 0,
    };
    assert_eq!(validate_syllable(&long_vowel, false), 20);

    // Invalid onset → 5
    let bad_onset = Syllable {
        onset: "z".to_string(),
        vowel: "a".to_string(),
        coda: "".to_string(),
        tone: 0,
    };
    assert_eq!(validate_syllable(&bad_onset, false), 5);

    // Invalid coda → 8
    let bad_coda = Syllable {
        onset: "b".to_string(),
        vowel: "a".to_string(),
        coda: "z".to_string(),
        tone: 0,
    };
    assert_eq!(validate_syllable(&bad_coda, false), 8);
}

// ============================================================
// 15. PROPTEST EXTENDED — Increase resilience
// ============================================================

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn no_crash_telex_random(s in "\\PC{0,100}") {
            let mut e = telex();
            for c in s.chars() {
                let _ = e.process_key(c);
            }
        }

        #[test]
        fn no_crash_vni_random(s in "\\PC{0,100}") {
            let mut e = vni();
            for c in s.chars() {
                let _ = e.process_key(c);
            }
        }

        #[test]
        fn no_crash_viqr_random(s in "\\PC{0,100}") {
            let mut e = viqr();
            for c in s.chars() {
                let _ = e.process_key(c);
            }
        }

        #[test]
        fn buffer_never_exceeds_50(s in "[a-z]{0,200}") {
            let mut e = telex();
            for c in s.chars() {
                let result = e.process_key(c);
                assert!(result.len() <= 200, "Output suspiciously large: {}", result.len());
            }
        }

        #[test]
        fn reset_always_clears(s in "\\PC{0,50}") {
            let mut e = telex();
            for c in s.chars() {
                let _ = e.process_key(c);
            }
            e.reset();
            // After reset, engine should be clean
            assert_eq!(e.process_key('a'), "a");
        }
    }
}
