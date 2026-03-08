use vnkey_core::{Engine, EngineConfig, InputMode};

fn setup_engine(mode: InputMode) -> Engine {
    let mut e = Engine::new(mode);
    let mut cfg = EngineConfig::default();
    cfg.spell_check = true;
    cfg.auto_restore = true;
    e.set_config(cfg);
    e
}

#[test]
fn test_tone_sweep() {
    let mut e = setup_engine(InputMode::Telex);

    let cases = vec![
        // Basic gi/qu
        ("giaf", "gi\u{00E0}"),
        ("gias", "gi\u{00E1}"),
        ("gioir", "gi\u{1ECF}i"),
        ("giuowngf", "gi\u{1B0}\u{1EDD}ng"),
        ("giuwax", "gi\u{1EEF}a"),
        ("giuax", "gi\u{169}a"),
        ("quaf", "qu\u{00E0}"),
        ("quar", "qu\u{1EA3}"),
        ("quets", "qu\u{00E9}t"),
        ("queets", "qu\u{1EBF}t"),
        ("quyts", "qu\u{00FD}t"),
        ("quowf", "qu\u{1EDD}"),
        ("quangr", "qu\u{1EA3}ng"),
        // Specific complex cases
        ("gieesng", "gi\u{1EBF}ng"),
        ("quoocs", "qu\u{1ED1}c"),
        ("khuyur", "khu\u{1EF7}u"),
        ("ngoanwf", "ngo\u{1EB1}n"),
        ("hoaf", "h\u{00F2}a"),
        ("thuys", "th\u{00FA}y"),
        ("nguyeexn", "nguy\u{1EC5}n"),
        ("tuyeens", "tuy\u{1EBF}n"),
        ("uaanf", "u\u{1EA7}n"),
        ("oais", "o\u{00E1}i"),
        // Casing
        ("NGUYEEXN", "NGUY\u{1EC4}N"),
        ("Nguyeexn", "Nguy\u{1EC5}n"),
    ];

    for (input, expected) in cases {
        e.reset();
        let result = e.feed_str(input);
        assert_eq!(result, expected, "Failed for input: {}", input);
    }
}

#[test]
fn test_telex_cancellation() {
    let mut e = setup_engine(InputMode::Telex);
    assert_eq!(e.feed_str("aa"), "\u{00E2}");
    e.reset();
    assert_eq!(e.feed_str("aaa"), "aa");
    // as -> á, ass -> as
    e.reset();
    assert_eq!(e.feed_str("ass"), "as");
}

#[test]
fn test_modern_vs_traditional_tone() {
    let mut e = setup_engine(InputMode::Telex);
    // Traditional
    assert_eq!(e.feed_str("hoaf"), "h\u{00F2}a");
    // Modern
    e.reset();
    let mut cfg = e.config().clone();
    cfg.modern_tone = true;
    e.set_config(cfg);
    assert_eq!(e.feed_str("hoaf"), "ho\u{00E0}");
}
