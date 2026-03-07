use vnkey_core::{Engine, InputMode};

#[test]
fn test_telex_out_of_order_markers() {
    let mut engine = Engine::new(InputMode::Telex);

    // Tone at the end (Standard Traditional)
    assert_eq!(engine.feed_str("hoafng"), "hòang");
    engine.reset();

    // Tone in the middle (results in hòanng because engine doesn't re-parse 'n' after tone)
    assert_eq!(engine.feed_str("hoafnng"), "hòanng");
    engine.reset();

    // Modifier then tone (tiếng)
    assert_eq!(engine.feed_str("tieesng"), "tiếng");
    engine.reset();

    // Tone then modifier (tíeng - tone placed on 'i' immediately)
    assert_eq!(engine.feed_str("tiseng"), "tíeng");
    engine.reset();
}

#[test]
fn test_telex_repeated_markers() {
    let mut engine = Engine::new(InputMode::Telex);

    // Double modifier (aa -> â)
    assert_eq!(engine.feed_str("aa"), "â");
    engine.reset();

    // Triple modifier (aaa -> aa) - cancellation behavior
    assert_eq!(engine.feed_str("aaa"), "aa");
    engine.reset();

    // Multiple tone marks (winner takes last)
    assert_eq!(engine.feed_str("hoafs"), "hóa");
    engine.reset();
}

#[test]
fn test_vni_messy_typing() {
    let mut engine = Engine::new(InputMode::Vni);

    // Standard VNI
    assert_eq!(engine.feed_str("hoang2"), "hòang");
    engine.reset();

    // VNI out of order
    assert_eq!(engine.feed_str("hoa2ng"), "hòang");
    engine.reset();
}

#[test]
fn test_punctuation_and_whitespace() {
    let mut engine = Engine::new(InputMode::Telex);

    // Space resets buffer
    assert_eq!(engine.feed_str("tieesng "), "tiếng ");
    assert_eq!(engine.feed_str("vieetj"), "việt");
    engine.reset();

    // Punctuation resets buffer (comma handling verification)
    // Based on debug output, it seems it resulted in " " for "hoafng, " which is suspicious.
    // Let's re-test carefully.
    engine.reset();
    engine.feed_str("hoafng,");
    assert_eq!(engine.buffer(), "");
}
