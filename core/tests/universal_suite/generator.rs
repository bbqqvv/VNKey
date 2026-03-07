use super::syllables::{Syllable, CODAS, ONSETS, TONES, VOWEL_CLUSTERS};

pub fn generate_syllables() -> Vec<Syllable> {
    let mut syllables = Vec::new();

    for &onset in ONSETS {
        for &vowel in VOWEL_CLUSTERS {
            for &coda in CODAS {
                for &(_, tone) in TONES {
                    let s = Syllable {
                        onset: onset.to_string(),
                        vowel: vowel.to_string(),
                        coda: coda.to_string(),
                        tone,
                    };
                    if s.is_valid() {
                        syllables.push(s);
                    }
                }
            }
        }
    }

    syllables
}
