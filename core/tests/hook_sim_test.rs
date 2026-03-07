/// Simulates the exact keyboard hook flow from hook.rs
/// to find the root cause of typing bugs in real app.
use vnkey_core::{Engine, InputMode};

struct HookSim {
    engine: Engine,
    cw: String, // current_word from hook.rs
}

impl HookSim {
    fn new() -> Self {
        let mut engine = Engine::new(InputMode::Telex);
        let mut cfg = engine.config().clone();
        cfg.spell_check = false;
        cfg.auto_restore = false;
        engine.set_config(cfg);
        HookSim { engine, cw: String::new() }
    }
    
    fn press(&mut self, ch: char) -> String {
        let new_word = self.engine.process_key(ch);
        let should_clear = self.engine.get_state().buffer.is_empty();
        
        let mut base_word = self.cw.clone();
        base_word.push(ch);
        
        if new_word != base_word {
            let cpx = self.cw.chars().zip(new_word.chars()).take_while(|(a,b)| a==b).count();
            let bs = self.cw.chars().count() - cpx;
            let txt: String = new_word.chars().skip(cpx).collect();
            println!("  '{}': cw='{}' -> new='{}' | SEND bs={} txt='{}'", ch, self.cw, new_word, bs, txt);
        } else {
            println!("  '{}': cw='{}' -> new='{}' | PASS", ch, self.cw, new_word);
        }
        
        if should_clear { self.cw.clear(); } else { self.cw = new_word.clone(); }
        new_word
    }
    
    fn type_str(&mut self, s: &str) -> String {
        let mut last = String::new();
        for ch in s.chars() {
            last = self.press(ch);
        }
        last
    }
}

#[test]
fn test_oo_gives_o_circumflex() {
    let mut sim = HookSim::new();
    println!("=== oo -> ô ===");
    let r = sim.type_str("oo");
    assert_eq!(sim.cw, "ô", "oo should produce ô, got '{}'", sim.cw);
}

#[test]
fn test_ooo_cancellation() {
    let mut sim = HookSim::new();
    println!("=== ooo -> oo ===");
    sim.type_str("ooo");
    assert_eq!(sim.cw, "oo", "ooo should produce oo, got '{}'", sim.cw);
}

#[test]
fn test_aa_gives_a_circumflex() {
    let mut sim = HookSim::new();
    println!("=== aa -> â ===");
    sim.type_str("aa");
    assert_eq!(sim.cw, "â", "aa should produce â, got '{}'", sim.cw);
}

#[test]
fn test_ee_gives_e_circumflex() {
    let mut sim = HookSim::new();
    println!("=== ee -> ê ===");
    sim.type_str("ee");
    assert_eq!(sim.cw, "ê", "ee should produce ê, got '{}'", sim.cw);
}

#[test]
fn test_dd_gives_d_stroke() {
    let mut sim = HookSim::new();
    println!("=== dd -> đ ===");
    sim.type_str("dd");
    assert_eq!(sim.cw, "đ", "dd should produce đ, got '{}'", sim.cw);
}

#[test]
fn test_w_first_press_gives_u_horn() {
    let mut sim = HookSim::new();
    println!("=== w -> ư ===");
    sim.press('w');
    assert_eq!(sim.cw, "ư", "w should produce ư, got '{}'", sim.cw);
}

#[test]
fn test_ww_gives_w() {
    let mut sim = HookSim::new();
    println!("=== ww -> w ===");
    sim.type_str("ww");
    assert_eq!(sim.cw, "w", "ww should produce w, got '{}'", sim.cw);
}

#[test]
fn test_www_gives_ww() {
    let mut sim = HookSim::new();
    println!("=== www -> ww ===");
    sim.type_str("www");
    assert_eq!(sim.cw, "ww", "www should produce ww, got '{}'", sim.cw);
}

#[test]
fn test_tw_gives_tu_horn() {
    let mut sim = HookSim::new();
    println!("=== tw -> tư ===");
    sim.type_str("tw");
    assert_eq!(sim.cw, "tư", "tw should produce tư, got '{}'", sim.cw);
}

#[test]
fn test_aw_gives_a_breve() {
    let mut sim = HookSim::new();
    println!("=== aw -> ă ===");
    sim.type_str("aw");
    assert_eq!(sim.cw, "ă", "aw should produce ă, got '{}'", sim.cw);
}

#[test]
fn test_ow_gives_o_horn() {
    let mut sim = HookSim::new();
    println!("=== ow -> ơ ===");
    sim.type_str("ow");
    assert_eq!(sim.cw, "ơ", "ow should produce ơ, got '{}'", sim.cw);
}

#[test]
fn test_uw_gives_u_horn() {
    let mut sim = HookSim::new();
    println!("=== uw -> ư ===");
    sim.type_str("uw");
    assert_eq!(sim.cw, "ư", "uw should produce ư, got '{}'", sim.cw);
}

#[test]
fn test_vieejt() {
    let mut sim = HookSim::new();
    println!("=== vieejt -> việt ===");
    sim.type_str("vieejt");
    assert_eq!(sim.cw, "việt");
}

#[test]
fn test_nguowif() {
    let mut sim = HookSim::new();
    println!("=== nguowif -> người ===");
    sim.type_str("nguowif");
    assert_eq!(sim.cw, "người");
}

#[test]
fn test_ddax() {
    let mut sim = HookSim::new();
    println!("=== ddax -> đã ===");
    sim.type_str("ddax");
    assert_eq!(sim.cw, "đã");
}

#[test]
fn test_tieesng() {
    let mut sim = HookSim::new();
    println!("=== tieesng -> tiếng ===");
    sim.type_str("tieesng");
    assert_eq!(sim.cw, "tiếng");
}

#[test]
fn test_huongw() {
    let mut sim = HookSim::new();
    println!("=== huongw -> hương ===");
    sim.type_str("huongw");
    assert_eq!(sim.cw, "hương");
}

#[test]
fn test_chuongw() {
    let mut sim = HookSim::new();
    println!("=== chuongw -> chương ===");
    sim.type_str("chuongw");
    assert_eq!(sim.cw, "chương");
}

#[test]
fn test_xin_space_chaof() {
    let mut sim = HookSim::new();
    println!("=== xin chaof -> chào ===");
    sim.type_str("xin ");
    sim.type_str("chaof");
    assert_eq!(sim.cw, "chào");
}

// This is the critical test - does typing random produce ư flood?
#[test]
fn test_no_unexpected_u_horn() {
    let mut sim = HookSim::new();
    println!("=== abcdef -> no ư ===");
    for ch in "abcdef".chars() {
        let word = sim.press(ch);
        assert!(!word.contains('ư'), "Unexpected ư in '{}' after key '{}'", word, ch);
    }
}
