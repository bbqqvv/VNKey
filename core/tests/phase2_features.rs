//! Tests for Phase 2 features:
//! - Free-form/Late-binding modifiers (Chuongw → Chương)
//! - Auto-fix tone position when vowels change

use vnkey_core::{Engine, InputMode};

fn telex() -> Engine {
    let mut e = Engine::new(InputMode::Telex);
    let mut cfg = e.config().clone();
    cfg.spell_check = false;
    cfg.auto_restore = false;
    cfg.modern_tone = true;
    e.set_config(cfg);
    e
}

// ============================================================
// FREE-FORM MODIFIERS — 'w' at end of word
// ============================================================

#[test]
fn freeform_chuongw_to_chuong() {
    let mut e = telex();
    // "chuongw" → w at end applies horn to "uo" pair → "chương"
    assert_eq!(e.feed_str("chuongw"), "chương");
}

#[test]
fn freeform_muonw() {
    let mut e = telex();
    // "muonw" → w applies to "uo" → "mươn"
    assert_eq!(e.feed_str("muonw"), "mươn");
}

#[test]
fn freeform_luongw() {
    let mut e = telex();
    assert_eq!(e.feed_str("luongw"), "lương");
}

#[test]
fn freeform_huongw() {
    let mut e = telex();
    assert_eq!(e.feed_str("huongw"), "hương");
}

#[test]
fn freeform_standard_aw_still_works() {
    // Standard position modifier should still work
    let mut e = telex();
    assert_eq!(e.feed_str("awn"), "ăn");
    e.reset();
    assert_eq!(e.feed_str("ow"), "ơ");
    e.reset();
    assert_eq!(e.feed_str("uw"), "ư");
}

#[test]
fn freeform_w_with_tone() {
    let mut e = telex();
    // "chuongw" → "chương"
    assert_eq!(e.feed_str("chuongw"), "chương");
    // Add 's' for sắc → tone on ươ pair
    let r = e.process_key('s');
    // ươ pair: both modified, tone goes on first (ư)
    assert!(
        r.contains('ứ') || r.contains('ớ'),
        "Expected tone on ươ pair, got: {}",
        r
    );
}

// ============================================================
// AUTO-FIX TONE POSITION when vowels change
// ============================================================

#[test]
fn autofix_tone_moves_with_new_vowel() {
    let mut e = telex();
    // Type "ba" + sắc → "bá"
    assert_eq!(e.process_key('b'), "b");
    assert_eq!(e.process_key('a'), "ba");
    assert_eq!(e.process_key('s'), "bá");
    // Add 'n' → "bán" (tone stays on 'a')
    assert_eq!(e.process_key('n'), "bán");
}

#[test]
fn autofix_tone_on_progressive_vowel_change() {
    let mut e = telex();
    // Type "ho" + huyền → "hò" (tone on 'o')
    assert_eq!(e.feed_str("hof"), "hò");
    // Continue typing 'a' → "hoa" now has 2 vowels
    // Tone should re-position based on rules (oa = tone on 'a' in modern)
    let r = e.process_key('a');
    assert_eq!(r, "hoà"); // Modern: tone on 'a'
}

#[test]
fn autofix_tone_when_modifier_changes_vowel() {
    let mut e = telex();
    // Type "tho" + sắc → "thó"
    assert_eq!(e.feed_str("thos"), "thó");
    // Add 'o' → "oo" modifier → "thô" + sắc → "thố"
    let r = e.process_key('o');
    assert_eq!(r, "thố");
}

#[test]
fn autofix_ee_modifier_keeps_tone() {
    let mut e = telex();
    // Type "the" + sắc → "thé"
    assert_eq!(e.feed_str("thes"), "thé");
    // Add 'e' → "ee" modifier → "thê" + sắc → "thế"
    let r = e.process_key('e');
    assert_eq!(r, "thế");
}

// ============================================================
// EDGE CASES
// ============================================================

#[test]
fn freeform_no_applicable_vowel() {
    let mut e = telex();
    // "thw" → no vowel to modify with 'w', fallback to Smart W → "thư"
    assert_eq!(e.feed_str("thw"), "thư");
}

#[test]
fn freeform_dd_still_works() {
    let mut e = telex();
    assert_eq!(e.feed_str("dduongw"), "đương");
}
