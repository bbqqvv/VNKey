//! Tests for new Phase 1 features:
//! - TelexEx mode (W ≠ Ư)
//! - VIQR input method
//! - Progressive Z (advanced tone removal)
//! - Tone placement styles (Modern vs Traditional)

use vnkey_core::{Engine, EngineConfig, InputMode, TonePlacement};

fn telex() -> Engine {
    let mut e = Engine::new(InputMode::Telex);
    let mut cfg = e.config().clone();
    cfg.spell_check = false;
    cfg.auto_restore = false;
    e.set_config(cfg);
    e
}
fn telex_ex() -> Engine {
    let mut e = Engine::new(InputMode::TelexEx);
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
// TELEX_EX: W does NOT produce Ư
// ============================================================

#[test]
fn telex_ex_w_stays_w() {
    let mut e = telex_ex();
    // In TelexEx, 'w' at word start stays as 'w'
    assert_eq!(e.process_key('w'), "w");
    e.reset();
    // 'w' after consonant also stays as 'w' (NOT ư)
    assert_eq!(e.feed_str("tw"), "tw");
}

#[test]
fn telex_ex_modifiers_still_work() {
    let mut e = telex_ex();
    // Standard modifiers (aw→ă, ow→ơ, uw→ư) should still work
    assert_eq!(e.feed_str("aw"), "ă");
    e.reset();
    assert_eq!(e.feed_str("ow"), "ơ");
    e.reset();
    assert_eq!(e.feed_str("uw"), "ư");
    e.reset();
    // dd→đ works
    assert_eq!(e.feed_str("dd"), "đ");
}

#[test]
fn telex_ex_tones_work() {
    let mut e = telex_ex();
    assert_eq!(e.feed_str("as"), "á");
    e.reset();
    assert_eq!(e.feed_str("af"), "à");
    e.reset();
    assert_eq!(e.feed_str("vieejt"), "việt");
}

#[test]
fn telex_vs_telex_ex_w_difference() {
    // Telex: tw → tư
    let mut e = telex();
    assert_eq!(e.feed_str("tw"), "tư");

    // TelexEx: tw → tw (literal)
    let mut e2 = telex_ex();
    assert_eq!(e2.feed_str("tw"), "tw");
}

// ============================================================
// VIQR INPUT METHOD
// ============================================================

#[test]
fn viqr_basic_tones() {
    let mut e = viqr();
    assert_eq!(e.feed_str("a'"), "á"); // sắc
    e.reset();
    assert_eq!(e.feed_str("a`"), "à"); // huyền
    e.reset();
    assert_eq!(e.feed_str("a?"), "ả"); // hỏi
    e.reset();
    assert_eq!(e.feed_str("a~"), "ã"); // ngã
    e.reset();
    assert_eq!(e.feed_str("a."), "ạ"); // nặng
}

#[test]
fn viqr_modifiers() {
    let mut e = viqr();
    assert_eq!(e.feed_str("a^"), "â"); // circumflex
    e.reset();
    assert_eq!(e.feed_str("e^"), "ê");
    e.reset();
    assert_eq!(e.feed_str("o^"), "ô");
    e.reset();
    assert_eq!(e.feed_str("o+"), "ơ"); // horn
    e.reset();
    assert_eq!(e.feed_str("u+"), "ư");
    e.reset();
    assert_eq!(e.feed_str("a("), "ă"); // breve
    e.reset();
    assert_eq!(e.feed_str("dd"), "đ"); // stroke
}

#[test]
fn viqr_combined() {
    let mut e = viqr();
    // â + tone sắc
    assert_eq!(e.feed_str("a^'"), "ấ");
    e.reset();
    // ê + tone huyền
    assert_eq!(e.feed_str("e^`"), "ề");
}

// ============================================================
// PROGRESSIVE Z — Advanced tone removal
// ============================================================

#[test]
fn progressive_z_level_1_remove_tone() {
    let mut e = telex();
    // Type "ás" then press Z → remove tone → "a"
    assert_eq!(e.process_key('a'), "a");
    assert_eq!(e.process_key('s'), "á");
    assert_eq!(e.process_key('z'), "a"); // Z level 1: tone removed
}

#[test]
fn progressive_z_level_2_remove_modifiers() {
    let mut e = telex();
    // Type "ấ" (â + sắc)
    assert_eq!(e.feed_str("aas"), "ấ");
    // Z level 1: remove tone → "â"
    let r1 = e.process_key('z');
    assert_eq!(r1, "â");
    // Z level 2: remove modifiers → "a"
    let r2 = e.process_key('z');
    assert_eq!(r2, "a");
}

#[test]
fn progressive_z_level_3_literal_z() {
    let mut e = telex();
    assert_eq!(e.feed_str("aas"), "ấ"); // â + sắc
                                        // Z level 1: remove tone → "â"
    let r1 = e.process_key('z');
    assert_eq!(r1, "â");
    // Z level 2: remove modifiers → "a"
    let r2 = e.process_key('z');
    assert_eq!(r2, "a");
    // Z level 3: literal z → buffer becomes "az"
    let r3 = e.process_key('z');
    assert_eq!(r3, "az");
}

#[test]
fn progressive_z_resets_on_non_z_key() {
    let mut e = telex();
    assert_eq!(e.feed_str("as"), "á");
    e.process_key('z'); // Remove tone → "a"
                        // Type 'n' → Z state resets, normal processing continues
    let r = e.process_key('n');
    assert_eq!(r, "an");
}

// ============================================================
// TONE PLACEMENT STYLE (Modern vs Traditional)
// ============================================================

#[test]
fn tone_placement_traditional_default() {
    // Default is now Traditional: tone on first vowel in oa/oe
    let mut e = telex();
    assert_eq!(e.feed_str("hoaf"), "hòa"); // Traditional: tone on 'o'
}

#[test]
fn tone_placement_modern() {
    let mut config = EngineConfig::default();
    config.modern_tone = true;
    let mut e = Engine::with_config(InputMode::Telex, config);
    let mut cfg = e.config().clone();
    cfg.spell_check = false;
    cfg.auto_restore = false;
    e.set_config(cfg);
    assert_eq!(e.feed_str("hoaf"), "hoà"); // Modern: tone on 'a'
}

#[test]
fn tone_placement_toggle_at_runtime() {
    let mut e = telex();

    // Start with Traditional (default)
    assert_eq!(e.feed_str("hoaf"), "hòa");

    // Switch to Modern
    e.set_tone_placement(true);
    assert_eq!(e.feed_str("hoaf"), "hoà");

    // Switch back to Traditional
    e.set_tone_placement(false);
    assert_eq!(e.feed_str("hoaf"), "hòa");
}

// ============================================================
// MODE SWITCHING — All 4 modes
// ============================================================

#[test]
fn switch_all_modes() {
    let mut e = Engine::new(InputMode::Telex);
    assert_eq!(e.feed_str("as"), "á");

    e.set_mode(InputMode::TelexEx);
    assert_eq!(e.feed_str("as"), "á");

    e.set_mode(InputMode::Vni);
    assert_eq!(e.feed_str("a1"), "á");

    e.set_mode(InputMode::Viqr);
    assert_eq!(e.feed_str("a'"), "á");
}
