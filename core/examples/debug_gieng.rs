use vnkey_core::{Engine, EngineConfig, InputMode};

fn main() {
    let mut e = Engine::new(InputMode::Telex);
    let mut cfg = EngineConfig::default();
    cfg.spell_check = true;
    cfg.auto_restore = true;
    e.set_config(cfg);

    let res = e.feed_str("ass");
    println!("Word: {}", res);
}
