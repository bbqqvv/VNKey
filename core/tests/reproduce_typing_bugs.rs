use vnkey_core::{Engine, InputMode};

#[test]
fn reproduce_w_repetition_bug() {
    let mut engine = Engine::new(InputMode::Telex);
    
    // w -> ư (Correct)
    assert_eq!(engine.feed_str("w"), "ư");
    
    engine.reset();
    // ww -> w (Correct - 2nd w cancels ư and outputs literal w)
    assert_eq!(engine.feed_str("ww"), "w");
    
    engine.reset();
    // 3rd w -> ww (1st->ư, 2nd->w, 3rd->ww)
    assert_eq!(engine.feed_str("www"), "ww", "3rd w should produce ww");

    engine.reset();
    // 4th w -> www? Based on pattern: w, ww, www...
    assert_eq!(engine.feed_str("wwww"), "www");
}

#[test]
fn reproduce_modifier_cancellation_bug() {
    let mut engine = Engine::new(InputMode::Telex);
    
    // Current buggy behavior: aaa -> âa
    // Expected standard behavior: aaa -> aa (double-tap a to cancel â)
    assert_eq!(engine.feed_str("aaa"), "aa", "aaa should cancel and produce aa, not âa");
}

#[test]
fn reproduce_ee_cancellation_bug() {
    let mut engine = Engine::new(InputMode::Telex);
    assert_eq!(engine.feed_str("eee"), "ee", "eee should cancel and produce ee, not êe");
}

#[test]
fn reproduce_oo_cancellation_bug() {
    let mut engine = Engine::new(InputMode::Telex);
    assert_eq!(engine.feed_str("ooo"), "oo", "ooo should cancel and produce oo, not ôo");
}

#[test]
fn reproduce_dd_cancellation_bug() {
    let mut engine = Engine::new(InputMode::Telex);
    assert_eq!(engine.feed_str("ddd"), "dd", "ddd should cancel and produce dd, not đd");
}
