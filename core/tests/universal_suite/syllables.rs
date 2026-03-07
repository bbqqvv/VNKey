pub const ONSETS: &[&str] = &[
    "b", "c", "ch", "d", "đ", "g", "gh", "gi", "h", "k", "kh", "l", "m", "n", "ng", "ngh", "nh", "p",
    "ph", "q", "r", "s", "t", "th", "tr", "v", "x", "",
];

pub const VOWEL_CLUSTERS: &[&str] = &[
    "a", "ă", "â", "e", "ê", "i", "o", "ô", "ơ", "u", "ư", "y", "ai", "ao", "au", "ay", "âu", "ây",
    "eo", "ia", "iê", "iu", "oa", "oă", "oe", "oi", "ôi", "ơi", "ua", "uâ", "uê", "ui", "uô", "uơ",
    "ưa", "ươ", "uy", "uya", "uyê",
];

pub const CODAS: &[&str] = &["", "c", "ch", "m", "n", "ng", "nh", "p", "t"];

pub const TONES: &[(&str, u8)] = &[
    ("", 0),  // Ngang
    ("s", 1), // Sắc
    ("f", 2), // Huyền
    ("r", 3), // Hỏi
    ("x", 4), // Ngã
    ("j", 5), // Nặng
];

pub struct Syllable {
    pub onset: String,
    pub vowel: String,
    pub coda: String,
    pub tone: u8,
}

impl Syllable {
    pub fn is_valid(&self) -> bool {
        if self.vowel.is_empty() {
            return false;
        }

        // 1. Initial Consonant Rules
        let first_v = self.vowel.chars().next().unwrap_or(' ');
        let is_front = matches!(first_v, 'i' | 'e' | 'ê' | 'y' | 'i' | 'y');

        match self.onset.as_str() {
            "gh" | "ngh" | "k" => {
                if !is_front {
                    return false;
                }
            }
            "g" | "ng" | "c" => {
                // "gi" is allowed, but for others, back consonants go with back vowels
                if is_front && self.onset != "gi" {
                    return false;
                }
            }
            "q" => {
                // Q must be followed by U
                if !self.vowel.starts_with('u') {
                    return false;
                }
            }
            "p" => {
                // P only starts the onset in few loan words
                if !matches!(self.vowel.as_str(), "i" | "o" | "ô" | "u") {
                    return false;
                }
            }
            _ => {}
        }

        // 2. Vowel Cluster & Coda Rules
        // - y Cluster often restricted
        if self.vowel.starts_with('y') && !self.onset.is_empty() {
            if !matches!(
                self.onset.as_str(),
                "h" | "k" | "l" | "m" | "n" | "q" | "t" | "v"
            ) {
                return false;
            }
        }

        // - Stop codas (c, ch, p, t) only take Sắc/Nặng
        if matches!(self.coda.as_str(), "c" | "ch" | "p" | "t") {
            if self.tone != 1 && self.tone != 5 {
                return false;
            }
        }

        // - 'ch' and 'nh' codas only follow front vowels (a, e, ê, i)
        if matches!(self.coda.as_str(), "ch" | "nh") {
            let last_v = self.vowel.chars().last().unwrap_or(' ');
            if !matches!(last_v, 'a' | 'e' | 'ê' | 'i') {
                return false;
            }
        }

        // 3. Ambiguity Handling: uơ vs ươ
        // In Telex, uow is usually ươ. uơ only follows q in this generator's context
        // to avoid ambiguity with ươ (e.g. huow -> hươ vs huơ).
        if self.vowel == "uơ" {
            if self.onset != "q" {
                return false;
            }
        }
        if self.vowel == "ươ" {
            if self.onset == "q" {
                return false;
            }
        }

        true
    }
}
