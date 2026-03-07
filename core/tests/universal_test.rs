mod universal_suite;

use universal_suite::{generate_syllables, syllable_to_telex, Syllable};
use vnkey_core::{config, tone, Engine, InputMode};

#[test]
fn test_universal_telex_exhaustive() {
    let syllables = generate_syllables();
    println!("Generated {} valid syllables for testing.", syllables.len());

    let mut engine = Engine::new(InputMode::Telex);
    let mut config = engine.config().clone();
    config.spell_check = false; // Disable spell check for raw syllable testing
    config.auto_restore = false;
    engine.set_config(config);

    let mut pass_count = 0;

    for s in &syllables {
        engine.reset();
        let typing_sequence = syllable_to_telex(s);
        let result = engine.feed_str(&typing_sequence);

        let expected = calculate_expected(s, engine.config().modern_tone);

        if result != expected {
            println!(
                "FAIL: Syllable '{}{}{}' (Tone {}). Sequence: '{}'. Expected: '{}', Got: '{}'",
                s.onset, s.vowel, s.coda, s.tone, typing_sequence, expected, result
            );
        } else {
            pass_count += 1;
        }
    }

    println!(
        "Exhaustive Test: {}/{} passed.",
        pass_count,
        syllables.len()
    );
    assert_eq!(pass_count, syllables.len(), "Not all syllables passed!");
}

#[test]
fn test_telex_messy_patterns() {
    let mut engine = Engine::new(InputMode::Telex);
    let mut config = engine.config().clone();
    config.spell_check = false;
    engine.set_config(config);

    // 1. Triple modifier: aaa -> aa (Cancellation)
    engine.reset();
    let res = engine.feed_str("aaa");
    assert_eq!(res, "aa");

    // 2. Extra messy tone: tieesnng -> tiếnng
    engine.reset();
    let res = engine.feed_str("tieesnng");
    assert_eq!(res, "tiếnng");

    // 3. Repeated messy modifier/tone: dddaaawww -> ddaaaww (Cancellation and Cycle)
    engine.reset();
    let res = engine.feed_str("dddaaawww");
    assert_eq!(res, "ddaaaww");

    // 4. Tone on words with existing tone marks: nghieenjf -> nghiền
    engine.reset();
    let res = engine.feed_str("nghieenjf");
    assert_eq!(res, "nghiền");
}

fn calculate_expected(s: &Syllable, modern_tone: bool) -> String {
    let core = format!("{}{}", s.onset, s.vowel);
    let toned_core = if s.tone > 0 {
        tone::place_tone_with_style(
            &core,
            s.tone,
            if modern_tone {
                config::TonePlacement::Modern
            } else {
                config::TonePlacement::Traditional
            },
        )
        .unwrap_or(core)
    } else {
        core
    };
    format!("{}{}", toned_core, s.coda)
}
