//! Tests specific to VNKey typing rules and requested edge cases (vnkey-rust-typing-test-suite)

use vnkey_core::{Engine, InputMode};

// ============================================================
// HELPER (Task 1.2) - Utility to simulate typing
// ============================================================
// The engine's feed_str method naturally acts as simulate_typing,
// but we wrap it here for clarity as requested.
fn simulate_typing(engine: &mut Engine, input: &str) -> String {
    engine.feed_str(input)
}

fn create_engine() -> Engine {
    let mut e = Engine::new(InputMode::Telex);
    let mut cfg = e.config().clone();
    cfg.spell_check = false;
    cfg.auto_restore = false;
    cfg.modern_tone = true; // Use modern tone placement (e.g. oà instead of òa)
    e.set_config(cfg);
    e
}

// ============================================================
// 2. BASIC TYPING RULES
// ============================================================

// Task 2.1: Basic Tones
#[test]
fn basic_tones() {
    let cases = [
        ("as", "á"), ("af", "à"), ("ar", "ả"), ("ax", "ã"), ("aj", "ạ"),
    ];
    for (input, expected) in cases {
        let mut e = create_engine();
        assert_eq!(simulate_typing(&mut e, input), expected, "Failed basic tone: {}", input);
    }
}

// Task 2.2: Basic Modifiers (Vowels/Consonants)
#[test]
fn basic_modifiers() {
    let cases = [
        ("aa", "â"), ("ee", "ê"), ("oo", "ô"), ("dd", "đ"), ("aw", "ă"), ("ow", "ơ"), ("uw", "ư"),
    ];
    for (input, expected) in cases {
        let mut e = create_engine();
        assert_eq!(simulate_typing(&mut e, input), expected, "Failed modifier: {}", input);
    }
}

// Task 2.3: Remove tone by repeating tone key
#[test]
fn double_tap_remove_tone() {
    let cases = [
        ("ass", "as"), ("aff", "af"), ("arr", "ar"), ("axx", "ax"), ("ajj", "aj"),
    ];
    for (input, expected) in cases {
        let mut e = create_engine();
        assert_eq!(simulate_typing(&mut e, input), expected, "Failed double-tap remove: {}", input);
    }
}

// ============================================================
// 3. EDGE CASES & TONE PLACEMENT
// ============================================================

// Task 3.1 & 3.2 & 3.3 & 3.4 & 3.5: Specific word sequences
#[test]
fn edge_cases_words() {
    let cases = [
        ("tieesng", "tiếng"),
        ("hoafng", "hoàng"),
        ("ddawng", "đăng"),
        ("nguyeen", "nguyên"),
        ("nghieenj", "nghiện"),
    ];
    for (input, expected) in cases {
        let mut e = create_engine();
        assert_eq!(simulate_typing(&mut e, input), expected, "Failed word test: {}", input);
    }
}

// Task 3.6: Correct placement logic
#[test]
fn tone_placement_logic() {
    let mut e = create_engine();
    // testing "oa" + "f" -> "oà" (modern placement)
    assert_eq!(simulate_typing(&mut e, "hoafng"), "hoàng");
    
    e.reset();
    // Thuy -> tone goes to y if modern
    assert_eq!(simulate_typing(&mut e, "thuys"), "thuý");
}

// Task 3.7: Advanced Smart W cases (User requested)
#[test]
fn smart_w_advanced_cases() {
    let mut e = create_engine();
    
    // Case: 'w' alone -> 'ư' (Standard Smart W)
    let res = simulate_typing(&mut e, "w");
    println!("'w' -> '{}'", res);
    assert_eq!(res, "ư"); e.reset();
    
    // Case: 'w' followed by vowel -> literal 'w' + vowel (some engines do this, but let's check current engine)
    // Based on apply_modifiers, "wa" -> "ư" processed then "a" added? Or "wa" -> "w" + "a"?
    // Let's test what the engine actually does.
    
    // Case: 'w' after consonant (Smart ư/ă)
    let res = simulate_typing(&mut e, "tw");
    println!("'tw' -> '{}'", res);
    assert_eq!(res, "tư");
    e.reset();
    let res = simulate_typing(&mut e, "hw");
    println!("'hw' -> '{}'", res);
    assert_eq!(res, "hư");
    e.reset();
    let res = simulate_typing(&mut e, "sw");
    println!("'sw' -> '{}'", res);
    assert_eq!(res, "sư");
    e.reset();
    
    // Case: 'w' as modifier for vowels
    let res = simulate_typing(&mut e, "aw");
    println!("'aw' -> '{}'", res);
    assert_eq!(res, "ă");
    e.reset();
    let res = simulate_typing(&mut e, "uw");
    println!("'uw' -> '{}'", res);
    assert_eq!(res, "ư"); e.reset();
    let res = simulate_typing(&mut e, "ow");
    println!("'ow' -> '{}'", res);
    assert_eq!(res, "ơ");
    e.reset();
    let res = simulate_typing(&mut e, "uow");
    println!("'uow' -> '{}'", res);
    assert_eq!(res, "ươ");
    e.reset();
    
    // Case: 'w' at end of complex words
    let res = simulate_typing(&mut e, "thuw");
    println!("'thuw' -> '{}'", res);
    assert_eq!(res, "thư");
    e.reset();
    let res = simulate_typing(&mut e, "thawngs");
    println!("'thawngs' -> '{}'", res);
    assert_eq!(res, "thắng");
    e.reset();
    
    // Case: Double 'w' -> literal 'w'
    let res = simulate_typing(&mut e, "ww");
    println!("'ww' -> '{}'", res);
    assert_eq!(res, "w");
    e.reset();

    // New Case: Triple 'w' -> literal 'ww'
    let res = simulate_typing(&mut e, "www");
    assert_eq!(res, "ww");
    e.reset();
    
    // Case: 'uww' -> 'uw' (literal w)
    let res = simulate_typing(&mut e, "uww");
    println!("'uww' -> '{}'", res);
    assert_eq!(res, "uw");
    e.reset();
    
    // Case: Capital 'W' -> 'Ư' (preserving case)
    let res = simulate_typing(&mut e, "W");
    println!("'W' -> '{}'", res);
    assert_eq!(res, "Ư");
    e.reset();
    let res = simulate_typing(&mut e, "TW");
    println!("'TW' -> '{}'", res);
    assert_eq!(res, "TƯ");
    e.reset();
}
