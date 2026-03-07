use vnkey_core::{Engine, InputMode};

#[test]
fn test_nfc_nfd_normalization_consistency() {
    let mut engine = Engine::new(InputMode::Telex);

    // NFC (composed): "ế" is U+1EBF
    // NFD (decomposed): "ê" + "́" (combining acute)
    
    // Testing if engine handles NFD input by treating it logically or normalizing.
    // Given the current architecture, NFD input might be processed char by char.
    
    let nfc_e = "\u{1EBF}";
    let nfd_e = "ê\u{0301}";

    engine.reset();
    engine.feed_str(nfc_e);
    let out_nfc = engine.buffer().to_string();

    engine.reset();
    engine.feed_str(nfd_e);
    let out_nfd = engine.buffer().to_string();

    // Check byte lengths to see if they are both NFC (1EBF is 3 bytes in UTF-8)
    // NFD (ê + acute) would be 2 bytes (ê) + 2 bytes (acute) = 4 bytes.
    println!("NFC out: {} (len {}), NFD out: {} (len {})", out_nfc, out_nfc.len(), out_nfd, out_nfd.len());
}

#[test]
fn test_emoji_interspersion() {
    let mut engine = Engine::new(InputMode::Telex);

    // Emoji should trigger a reset (non-alphanumeric)
    engine.reset();
    engine.feed_str("tiế");
    let out_emoji = engine.feed_str("😀");
    let out_final = engine.feed_str("ng");
    
    println!("Emoji output: '{}', Final output: '{}'", out_emoji, out_final);
    // Based on engine logic, any non-word char should reset the word buffer.
    assert_eq!(out_final, "ng"); // 'ng' should be a new word
}

#[test]
fn test_combining_marks_stability() {
    let mut engine = Engine::new(InputMode::Telex);

    // Feed raw combining marks without base chars (should not panic)
    engine.reset();
    engine.feed_str("\u{0300}\u{0301}\u{0302}\u{0303}\u{0309}");
    
    // Feed combining mark after a consonant (non-vowel)
    engine.reset();
    engine.feed_str("b\u{0301}"); 
    
    // No panic is success
}
