use vnkey_core::{Engine, InputMode};

// ===== Case Preservation Tests =====
// Verifies that apply_case() correctly handles ALLCAPS/TitleCase/lowercase
// when output length differs from input length due to modifiers (aa→â, dd→đ)

#[test]
fn test_case_allcaps_viet() {
    let mut e = Engine::new(InputMode::Telex);
    let mut cfg = e.config().clone();
    cfg.spell_check = false;
    cfg.auto_restore = false;
    e.set_config(cfg);
    // VIEEJT → VIỆT (all uppercase preserved)
    assert_eq!(e.feed_str("VIEEJT"), "VIỆT");
}

#[test]
fn test_case_titlecase_viet() {
    let mut e = Engine::new(InputMode::Telex);
    let mut cfg = e.config().clone();
    cfg.spell_check = false;
    cfg.auto_restore = false;
    e.set_config(cfg);
    // Vieejt → Việt (first char upper, rest lower)
    assert_eq!(e.feed_str("Vieejt"), "Việt");
}

#[test]
fn test_case_lowercase_viet() {
    let mut e = Engine::new(InputMode::Telex);
    let mut cfg = e.config().clone();
    cfg.spell_check = false;
    cfg.auto_restore = false;
    e.set_config(cfg);
    // vieejt → việt (all lowercase)
    assert_eq!(e.feed_str("vieejt"), "việt");
}

#[test]
fn test_case_mixed_fallback_lowercase() {
    let mut e = Engine::new(InputMode::Telex);
    let mut cfg = e.config().clone();
    cfg.spell_check = false;
    cfg.auto_restore = false;
    e.set_config(cfg);
    // viEejt → việt (first char lowercase → entire output lowercase)
    assert_eq!(e.feed_str("viEejt"), "việt");
}

#[test]
fn test_case_allcaps_tieng() {
    let mut e = Engine::new(InputMode::Telex);
    let mut cfg = e.config().clone();
    cfg.spell_check = false;
    cfg.auto_restore = false;
    e.set_config(cfg);
    // TIEESNG → TIẾNG (all caps with tone)
    assert_eq!(e.feed_str("TIEESNG"), "TIẾNG");
}

#[test]
fn test_case_titlecase_with_modifier_and_tone() {
    let mut e = Engine::new(InputMode::Telex);
    let mut cfg = e.config().clone();
    cfg.spell_check = false;
    cfg.auto_restore = false;
    e.set_config(cfg);
    // DDawts → Đắt (DD→Đ, aw→ă, s=sắc)
    // Note: "DDawts" = D,D,a,w,t,s (6 chars input)
    // After modifiers: đắt (3 chars output)
    // case_map: [true, true, false, false, false, false]
    // is_all_upper = false, is_first_upper = true → TitleCase
    assert_eq!(e.feed_str("DDawts"), "Đắt");
}

#[test]
fn test_case_allcaps_with_dd_modifier() {
    let mut e = Engine::new(InputMode::Telex);
    let mut cfg = e.config().clone();
    cfg.spell_check = false;
    cfg.auto_restore = false;
    e.set_config(cfg);
    // DDAWTS → ĐẮT (all caps)
    assert_eq!(e.feed_str("DDAWTS"), "ĐẮT");
}

#[test]
fn test_case_single_char_uppercase() {
    let mut e = Engine::new(InputMode::Telex);
    let mut cfg = e.config().clone();
    cfg.spell_check = false;
    cfg.auto_restore = false;
    e.set_config(cfg);
    // Single uppercase char
    assert_eq!(e.process_key('A'), "A");
}

#[test]
fn test_case_single_char_lowercase() {
    let mut e = Engine::new(InputMode::Telex);
    let mut cfg = e.config().clone();
    cfg.spell_check = false;
    cfg.auto_restore = false;
    e.set_config(cfg);
    assert_eq!(e.process_key('a'), "a");
}

#[test]
fn test_case_titlecase_simple_tone() {
    let mut e = Engine::new(InputMode::Telex);
    let mut cfg = e.config().clone();
    cfg.spell_check = false;
    cfg.auto_restore = false;
    e.set_config(cfg);
    // "As" → "Á" (Title case preserved with tone)
    assert_eq!(e.feed_str("As"), "Á");
}

#[test]
fn test_case_allcaps_simple_tone() {
    let mut e = Engine::new(InputMode::Telex);
    let mut cfg = e.config().clone();
    cfg.spell_check = false;
    cfg.auto_restore = false;
    e.set_config(cfg);
    // "AS" → "Á" - both uppercase, output single char → uppercase
    assert_eq!(e.feed_str("AS"), "Á");
}

#[test]
fn test_case_no_regression_perfect_match() {
    let mut e = Engine::new(InputMode::Telex);
    let mut cfg = e.config().clone();
    cfg.spell_check = false;
    cfg.auto_restore = false;
    e.set_config(cfg);
    // "VNkey" → "VNkey" (perfect length match, char-by-char mapping)
    assert_eq!(e.feed_str("VNkey"), "VNkey");
    e.reset();
    assert_eq!(e.feed_str("Vnkey"), "Vnkey");
}
