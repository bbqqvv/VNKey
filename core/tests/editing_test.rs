use vnkey_core::{Engine, InputMode};

#[test]
fn test_backspace_basic() {
    let mut engine = Engine::new(InputMode::Telex);

    // Type "tiếng"
    engine.feed_str("tieesng");
    assert_eq!(engine.buffer(), "tieesng");
    assert_eq!(engine.get_state().transformed, "tiếng");

    // Backspace 'g' -> "tiens" (raw) -> "tiến" (transformed)
    assert!(engine.process_backspace());
    assert_eq!(engine.buffer(), "tieesn");
    assert_eq!(engine.get_state().transformed, "tiến");

    // Backspace 'n' -> "tiees" (raw) -> "tiế" (transformed)
    assert!(engine.process_backspace());
    assert_eq!(engine.buffer(), "tiees");
    assert_eq!(engine.get_state().transformed, "tiế");

    // Backspace 's' (tone) -> "tiee" (raw) -> "tiê" (transformed)
    assert!(engine.process_backspace());
    assert_eq!(engine.buffer(), "tiee");
    assert_eq!(engine.get_state().transformed, "tiê");
}

#[test]
fn test_backspace_exhaustion() {
    let mut engine = Engine::new(InputMode::Telex);
    engine.feed_str("abc");
    
    assert!(engine.process_backspace()); // c
    assert!(engine.process_backspace()); // b
    assert!(engine.process_backspace()); // a
    assert!(!engine.process_backspace()); // Empty - should return false
}

#[test]
fn test_state_snapshot_consistency() {
    let mut engine = Engine::new(InputMode::Telex);
    engine.feed_str("hoafng");
    
    let state1 = engine.get_state();
    assert_eq!(state1.buffer, "hoafng");
    assert_eq!(state1.transformed, "hòang");
    
    // Type more
    engine.feed_str(" ");
    let state2 = engine.get_state();
    assert_eq!(state2.buffer, ""); // Reset after space
    assert_eq!(state2.transformed, ""); 
}
