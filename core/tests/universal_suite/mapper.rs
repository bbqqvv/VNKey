use super::syllables::{Syllable, TONES};

pub fn syllable_to_telex(s: &Syllable) -> String {
    let mut keys = String::new();

    // 1. Onset
    keys.push_str(&map_to_telex_keys(&s.onset));

    // 2. Vowel
    keys.push_str(&map_to_telex_keys(&s.vowel));

    // 3. Tone (placed after vowel cluster as in user examples)
    if s.tone > 0 {
        for &(key, val) in TONES {
            if val == s.tone {
                keys.push_str(key);
                break;
            }
        }
    }

    // 4. Coda
    keys.push_str(&map_to_telex_keys(&s.coda));

    keys
}

fn map_to_telex_keys(input: &str) -> String {
    let mut res = String::new();
    for c in input.chars() {
        match c {
            'â' => res.push_str("aa"),
            'ă' => res.push_str("aw"),
            'ê' => res.push_str("ee"),
            'ô' => res.push_str("oo"),
            'ơ' => res.push_str("ow"),
            'ư' => res.push_str("uw"),
            'đ' => res.push_str("dd"),
            _ => res.push(c),
        }
    }
    res
}
