//! Senior features test for VNKey
use vnkey_core::{Engine, InputMode};

#[test]
fn test_smart_spelling_k_gh_ngh() {
    let mut engine = Engine::new(InputMode::Telex);
    let mut cfg = engine.config().clone();
    cfg.spell_check = true;
    engine.set_config(cfg);

    // c -> k before i/e/ê
    assert_eq!(engine.feed_str("ci"), "ki");
    assert_eq!(engine.feed_str("ce"), "ke");
    assert_eq!(engine.feed_str("cee"), "kê");

    // g -> gh before e/ê
    assert_eq!(engine.feed_str("ge"), "ghe");
    assert_eq!(engine.feed_str("gee"), "ghê");

    // ng -> ngh before i/e/ê
    assert_eq!(engine.feed_str("ngi"), "nghi");
    assert_eq!(engine.feed_str("nge"), "nghe");
    assert_eq!(engine.feed_str("ngee"), "nghê");
}

#[test]
fn test_smart_spelling_reverse() {
    let mut engine = Engine::new(InputMode::Telex);
    let mut cfg = engine.config().clone();
    cfg.spell_check = true;
    engine.set_config(cfg);

    // k -> c before a/o/u
    assert_eq!(engine.feed_str("ka"), "ca");
    assert_eq!(engine.feed_str("ko"), "co");
    assert_eq!(engine.feed_str("ku"), "cu");

    // gh -> g before a/o
    assert_eq!(engine.feed_str("gha"), "ga");
    assert_eq!(engine.feed_str("gho"), "go");
}

#[test]
fn test_linguistic_validity_score() {
    let mut engine = Engine::new(InputMode::Telex);
    let mut cfg = engine.config().clone();
    cfg.spell_check = true;
    engine.set_config(cfg);

    // Perfect word
    engine.feed_str("tieengs");
    assert_eq!(engine.get_state().validity_score, 100);

    // Invalid word (non-VN onset) -> Score 5 in P13
    engine.reset();
    engine.feed_str("zi");
    assert_eq!(engine.get_state().validity_score, 5);

    // Spelling error (before smart fix, but smart fix should make it 100)
    engine.reset();
    engine.feed_str("ci"); // Engine fixes to "ki"
    assert_eq!(engine.get_state().transformed, "ki");
    assert_eq!(engine.get_state().validity_score, 100);
}
