use vnkey_core::{Engine, InputMode};

#[test]
fn test_reproduce_bugs() {
    use std::fs::File;
    use std::io::Write;
    let mut log = File::create("actual_test_results.txt").unwrap();

    let mut e = Engine::new(InputMode::Telex);

    writeln!(log, "--- Testing 'w' logic ---").unwrap();
    // standalone 'w' -> 'ư'
    e.reset();
    writeln!(log, "w  -> expected 'ư', got '{}'", e.process_key('w')).unwrap();

    // 'ww' -> 'w'
    e.reset();
    e.process_key('w');
    writeln!(log, "ww -> expected 'w', got '{}'", e.process_key('w')).unwrap();

    // 'www' -> 'ww'
    e.reset();
    e.process_key('w');
    e.process_key('w');
    writeln!(log, "www -> expected 'ww', got '{}'", e.process_key('w')).unwrap();

    writeln!(log, "\n--- Testing Casing logic ---").unwrap();
    // Case: 'Hoa '
    e.reset();
    e.process_key('H');
    e.process_key('o');
    e.process_key('a');
    let res = e.process_key(' ');
    writeln!(log, "Hoa_space -> expected 'Hoa ', got '{}'", res).unwrap();

    // Case: 'HOA '
    e.reset();
    e.process_key('H');
    e.process_key('O');
    e.process_key('A');
    let res = e.process_key(' ');
    writeln!(log, "HOA_space -> expected 'HOA ', got '{}'", res).unwrap();

    // Case: 'Hoas '
    e.reset();
    e.process_key('H');
    e.process_key('o');
    e.process_key('a');
    e.process_key('s');
    let res = e.process_key(' ');
    writeln!(
        log,
        "Hoas_space -> expected 'Hoá ' or 'Hóa ', got '{}'",
        res
    )
    .unwrap();
    // Case: 'viEejt'
    e.reset();
    let res = e.feed_str("viEejt");
    writeln!(log, "viEejt -> expected 'việt', got '{}'", res).unwrap();
}
