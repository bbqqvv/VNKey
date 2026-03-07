use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct TrieNode {
    pub children: HashMap<char, TrieNode>,
    pub is_end_of_word: bool,
}

/// Represents a word suggestion with a confidence score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    pub word: String,
    pub score: f64,
}

#[derive(Serialize, Deserialize, Default)]
pub struct Dictionary {
    root: TrieNode,
    /// Store pairs of words with (word, frequency, last_used_timestamp)
    bigrams: HashMap<String, Vec<(String, u32, u64)>>,
    /// Store triples of words: "tôi đang" -> [("học", 10, ts), ("làm", 5, ts)]
    /// Key format: "word1|word2"
    trigrams: HashMap<String, Vec<(String, u32, u64)>>,
}

impl Dictionary {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, word: &str) {
        let mut current = &mut self.root;
        for c in word.to_lowercase().chars() {
            current = current.children.entry(c).or_default();
        }
        current.is_end_of_word = true;
    }

    /// Check if a word exists in the dictionary
    pub fn contains(&self, word: &str) -> bool {
        let mut current = &self.root;
        for c in word.to_lowercase().chars() {
            if let Some(node) = current.children.get(&c) {
                current = node;
            } else {
                return false;
            }
        }
        current.is_end_of_word
    }

    /// Load words from a dictionary file (one word per line)
    pub fn load_from_file(&mut self, path: &str) -> std::io::Result<()> {
        use std::fs::File;
        use std::io::{BufRead, BufReader};

        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for word in reader.lines().map_while(Result::ok) {
            let trimmed = word.trim();
            if !trimmed.is_empty() {
                self.insert(trimmed);
            }
        }
        Ok(())
    }

    fn get_now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    /// Record usage of a word sequence (can be 1 prev word or 2 prev words)
    pub fn record_usage(&mut self, context: &[&str], current: &str) {
        let now = Self::get_now();
        let current_low = current.to_lowercase();
        
        // Update Bigram (context[last])
        if let Some(&prev) = context.last() {
            let prev_low = prev.to_lowercase();
            let entries = self.bigrams.entry(prev_low).or_default();
            if let Some(entry) = entries.iter_mut().find(|(w, _, _)| w == &current_low) {
                entry.1 += 1;
                entry.2 = now;
            } else {
                entries.push((current_low.clone(), 1, now));
            }
            let now_ts = Self::get_now();
            entries.sort_by(|a, b| {
                let score_b = Self::calculate_score_internal(b, now_ts);
                let score_a = Self::calculate_score_internal(a, now_ts);
                score_b.partial_cmp(&score_a).unwrap()
            });
            if entries.len() > 20 { entries.truncate(20); }
        }

        // Update Trigram (context[last-1], context[last])
        if context.len() >= 2 {
            let w1 = context[context.len()-2].to_lowercase();
            let w2 = context[context.len()-1].to_lowercase();
            let key = format!("{}|{}", w1, w2);
            let entries = self.trigrams.entry(key).or_default();
            if let Some(entry) = entries.iter_mut().find(|(w, _, _)| w == &current_low) {
                entry.1 += 1;
                entry.2 = now;
            } else {
                entries.push((current_low, 1, now));
            }
            let now_ts = Self::get_now();
            entries.sort_by(|a, b| {
                let score_b = Self::calculate_score_internal(b, now_ts);
                let score_a = Self::calculate_score_internal(a, now_ts);
                score_b.partial_cmp(&score_a).unwrap()
            });
            if entries.len() > 10 { entries.truncate(10); }
        }
    }

    /// Calculate confidence score based on frequency and recency
    /// Calculate confidence score based on frequency and recency
    pub fn calculate_score_internal(entry: &(String, u32, u64), now: u64) -> f64 {
        let freq = entry.1 as f64;
        let last_used = entry.2;
        
        // Simple decay: score = freq / (1 + log10(1 + age_in_minutes))
        let age = if now > last_used { (now - last_used) as f64 } else { 0.0 };
        let age_mins = age / 60.0;
        freq / (1.0 + (1.0 + age_mins).log10())
    }

    /// Predict next words based on 1 or 2 previous words
    pub fn predict_next_word(&self, context: &[&str]) -> Vec<Suggestion> {
        let mut predictions: Vec<Suggestion> = Vec::new();

        let now = Self::get_now();

        // 1. Try Trigram if we have 2 words
        if context.len() >= 2 {
            let w1 = context[context.len()-2].to_lowercase();
            let w2 = context[context.len()-1].to_lowercase();
            let key = format!("{}|{}", w1, w2);
            if let Some(entries) = self.trigrams.get(&key) {
                for entry in entries.iter().take(3) {
                    predictions.push(Suggestion {
                        word: entry.0.clone(),
                        score: Self::calculate_score_internal(entry, now) * 1.5, // Boost trigrams
                    });
                }
            }
        }

        // 2. Try Bigram for the last word
        if let Some(&prev) = context.last() {
            let prev_low = prev.to_lowercase();
            if let Some(entries) = self.bigrams.get(&prev_low) {
                for entry in entries.iter().take(5) {
                    if !predictions.iter().any(|p| p.word == entry.0) {
                        predictions.push(Suggestion {
                            word: entry.0.clone(),
                            score: Self::calculate_score_internal(entry, now),
                        });
                    }
                }
            }
        }

        predictions.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        predictions.truncate(5);
        predictions
    }

    pub fn find_completions(&self, prefix: &str) -> Vec<Suggestion> {
        let mut current = &self.root;
        for c in prefix.chars() {
            if let Some(node) = current.children.get(&c) {
                current = node;
            } else {
                return Vec::new();
            }
        }

        let mut results = Vec::new();
        self.collect_words(current, prefix.to_string(), &mut results);
        results.into_iter().map(|w| Suggestion { word: w, score: 1.0 }).collect()
    }

    fn collect_words(&self, node: &TrieNode, prefix: String, results: &mut Vec<String>) {
        if results.len() >= 5 {
            return; // Limit to 5 suggestions
        }

        if node.is_end_of_word {
            results.push(prefix.clone());
        }

        // Sort keys to have deterministic results
        let mut keys: Vec<&char> = node.children.keys().collect();
        keys.sort();

        for &c in keys {
            let mut new_prefix = prefix.clone();
            new_prefix.push(c);
            self.collect_words(&node.children[&c], new_prefix, results);
            if results.len() >= 5 {
                break;
            }
        }
    }

    /// Load a basic set of common Vietnamese words and bigrams
    pub fn load_common_words(&mut self) {
        let common_words = vec![
            "anh", "ăn", "âm", "áp", "ác", "ai", "ao", "âu",
            "ba", "bà", "bố", "mẹ", "em", "chị", "ông", "bà",
            "con", "cái", "của", "cho", "chư", "chưa", "chẳng", "chỉ",
            "đi", "đến", "đang", "đã", "được", "đây", "đó", "đâu",
            "em", "ép", "êm", "ế", "ếch",
            "không", "khá", "khi", "khác", "khó", "khu", "khuyên",
            "là", "làm", "lên", "lấy", "lại", "luôn", "lòng",
            "mình", "mà", "mới", "mỗi", "muốn", "mang", "mắt",
            "người", "ngày", "nghĩ", "nghe", "ngon", "ngọt", "ngu",
            "ở", "ơi", "ông", "ô", "ố",
            "phải", "phần", "phát", "phòng", "phố", "phương",
            "quá", "qua", "quanh", "quan", "quận", "quyết",
            "rằng", "rồi", "rất", "ra", "riêng", "rừng",
            "sẽ", "sau", "sang", "sáng", "sống", "sao",
            "ta", "tôi", "tới", "từ", "tại", "thấy", "theo", "thì",
            "uống", "uốn", "u", "ú",
            "và", "với", "về", "vừa", "việc", "vẫn", "vậy",
            "xem", "xuống", "xong", "xanh", "xấu", "xinh",
            "yêu", "yếu", "ý", "yên",
            "việt", "nam", "tiếng", "học", "tập", "chơi", "ngủ", "nghỉ",
            "trường", "lớp", "bạn", "thầy", "cô", "giáo", "viên",
            "cộng", "hòa", "xã", "hội", "chủ", "nghĩa",
        ];

        for word in common_words {
            self.insert(word);
        }

        // Default bigrams for "Hay hơn" effect
        let default_bigrams = vec![
            ("việt", vec!["nam", "vị", "hàn"]),
            ("cộng", vec!["hòa"]),
            ("hòa", vec!["bình"]),
            ("tiếng", vec!["việt", "anh", "pháp"]),
            ("học", vec!["tập", "sinh", "vấn"]),
            ("người", vec!["việt", "đẹp", "dân"]),
            ("đang", vec!["làm", "chơi", "học"]),
            ("chào", vec!["anh", "chị", "em", "bạn"]),
            ("ăn", vec!["cơm", "phở", "bún"]),
            ("tôi", vec!["đang", "là", "muốn", "nghĩ"]),
        ];

        for (prev, nexts) in default_bigrams {
            for next in nexts {
                self.record_usage(&[prev], next);
            }
        }
    }
}
