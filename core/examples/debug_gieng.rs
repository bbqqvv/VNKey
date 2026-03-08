use vnkey_core::{Engine, EngineConfig, InputMode};

fn main() {
    let mut e = Engine::new(InputMode::Telex);
    let mut cfg = EngineConfig::default();
    cfg.spell_check = false;
    cfg.auto_restore = false;
    e.set_config(cfg);

    let res = e.feed_str("aaa");
    println!("'aaa' -> '{}'", res);

    e.reset();
    let res = e.feed_str("ass");
    println!("'ass' -> '{}'", res);
}
