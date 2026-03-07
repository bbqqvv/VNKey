use proptest::prelude::*;
use vnkey_core::{Engine, InputMode};

proptest! {
    #![proptest_config(ProptestConfig::with_cases(5000))] // Run 5000 cases per test

    #[test]
    fn test_telex_stability_fuzz(s in "\\PC*") {
        let mut engine = Engine::new(InputMode::Telex);
        let mut config = engine.config().clone();
        config.spell_check = false;
        config.auto_restore = false;
        engine.set_config(config);

        // Ensure that engine never panics given ANY Unicode string
        let _ = engine.feed_str(&s);
    }

    #[test]
    fn test_vni_stability_fuzz(s in "\\PC*") {
        let mut engine = Engine::new(InputMode::Vni);
        let mut config = engine.config().clone();
        config.spell_check = false;
        config.auto_restore = false;
        engine.set_config(config);

        let _ = engine.feed_str(&s);
    }
}

#[test]
fn test_mixed_mode_stability() {
    let mut engine = Engine::new(InputMode::Telex);
    // Simulate switching modes and configuration mid-stream
    engine.feed_str("typing some telex...");
    engine.set_mode(InputMode::Vni);
    engine.feed_str("12345");
    engine.set_mode(InputMode::Telex);
    engine.feed_str("aaeeoo");

    let mut config = engine.config().clone();
    config.spell_check = true;
    engine.set_config(config);
    engine.feed_str(" testing spell check...");

    // No panic is success
}
