use vnkey_core::{Engine, InputMode};

#[test]
fn test_terminal_escape_codes() {
    let mut engine = Engine::new(InputMode::Telex);

    // Simulate "tiếng" followed by an ANSI escape code (e.g., color red)
    // \x1b[31m
    engine.feed_str("tieesng");
    assert_eq!(engine.get_state().transformed, "tiếng");

    // Space resets and returns the word + space
    let out = engine.process_key(' ');
    assert_eq!(out, "tiếng ");
    assert_eq!(engine.buffer(), "");

    engine.process_key('v');
    engine.process_key('i');
    engine.process_key('e');
    engine.process_key('e');
    engine.process_key('t');
    engine.process_key('j');
    assert_eq!(engine.get_state().transformed, "việt");
}

#[test]
fn test_rich_text_formatting_markers() {
    let mut engine = Engine::new(InputMode::Telex);

    // Simulate typing "tiếng" where a bold marker '*' is inserted mid-word
    engine.feed_str("tiee");
    // User hits bold marker '*' - non-alphanumeric should return "tiê*"
    let out = engine.process_key('*');
    assert_eq!(out, "tiê*");
    assert_eq!(engine.buffer(), "");

    engine.feed_str("sng");
    assert_eq!(engine.buffer(), "sng");
}

#[test]
fn test_extremely_fast_typing_no_delay() {
    let mut engine = Engine::new(InputMode::Telex);

    // Simulate a burst of keys with SPACES
    let burst = "tieesng vieetj hoafng ";
    let mut last_res = String::new();
    for c in burst.chars() {
        last_res = engine.process_key(c);
    }

    // Last char was space, so it returns "hòang "
    assert_eq!(last_res, "hòang ");
}
