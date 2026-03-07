use vnkey_core::{Engine, InputMode};

#[test]
fn test_trace_bugs() {
    use std::fs::File;
    use std::io::Write;
    let mut log = File::create("detailed_trace.txt").unwrap();

    let mut e = Engine::new(InputMode::Telex);

    let keys = vec!['h', 'o', 'a', ' '];
    writeln!(log, "--- Trace: 'hoa ' (lowercase) ---").unwrap();
    for k in keys {
        let res = e.process_key(k);
        writeln!(
            log,
            "Key: '{}' -> Res: '{}', Buffer: '{}', CaseMap: {:?}",
            k,
            res,
            e.buffer(),
            e.case_map()
        )
        .unwrap();
    }

    let keys_cap = vec!['H', 'o', 'a', ' '];
    writeln!(log, "\n--- Trace: 'Hoa ' (Capitalized) ---").unwrap();
    e.reset();
    for k in keys_cap {
        let res = e.process_key(k);
        writeln!(
            log,
            "Key: '{}' -> Res: '{}', Buffer: '{}', CaseMap: {:?}",
            k,
            res,
            e.buffer(),
            e.case_map()
        )
        .unwrap();
    }

    let keys_ww = vec!['w', 'w', 'w'];
    writeln!(log, "\n--- Trace: 'www' ---").unwrap();
    e.reset();
    for k in keys_ww {
        let res = e.process_key(k);
        writeln!(
            log,
            "Key: '{}' -> Res: '{}', Buffer: '{}', CaseMap: {:?}",
            k,
            res,
            e.buffer(),
            e.case_map()
        )
        .unwrap();
    }
}
