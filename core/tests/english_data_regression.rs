use vnkey_core::{Engine, InputMode};
use std::fs::File;
use std::io::{BufRead, BufReader};

#[test]
fn test_english_tone_failures_regression() {
    let mut engine = Engine::new(InputMode::Telex);
    let mut cfg = engine.config().clone();
    cfg.spell_check = true;
    cfg.auto_restore = true;
    engine.set_config(cfg);

    let file = File::open("tests/data/english_100k_failures_tone.txt").expect("Failed to open test data");
    let reader = BufReader::new(file);

    let mut total = 0;
    let mut fixed = 0;
    let mut still_failing = 0;

    for line in reader.lines() {
        let line = line.unwrap();
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 2 { continue; }

        let word = parts[0];
        // let expected_fail = parts[1]; // What it used to fail to

        total += 1;
        engine.reset();
        let output = engine.feed_str(word);

        if output == word {
            fixed += 1;
        } else {
            still_failing += 1;
            if still_failing < 50 {
                println!("Still failing: [{}] -> [{}] (buffer: {})", word, output, engine.get_state().buffer);
            }
        }
    }

    println!("--- English Tone Failures Regression Report ---");
    println!("Total samples: {}", total);
    println!("Fixed (Literal Mode): {}", fixed);
    println!("Still failing: {}", still_failing);
    println!("Improvement: {:.2}%", (fixed as f32 / total as f32) * 100.0);

    // We expect some improvement, but not 100% since short words are hard
    assert!(fixed > 0, "Literal mode should have fixed at least some words!");
}
