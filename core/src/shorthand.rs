/// Shorthand / Auto-text Module — tốc ký
///
/// Provides fast text expansion: type an abbreviation → get full text.
/// Supports import/export shorthand lists as JSON.
use std::collections::HashMap;

/// The shorthand dictionary
#[derive(Debug, Clone)]
pub struct ShorthandDict {
    entries: HashMap<String, String>,
}

impl ShorthandDict {
    /// Create an empty dictionary
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Create with built-in Vietnamese shorthand defaults
    pub fn with_defaults() -> Self {
        let mut dict = Self::new();
        // Common abbreviations
        let defaults = [
            ("dc", "được"),
            ("kg", "không"),
            ("ntn", "như thế nào"),
            ("ns", "nói"),
            ("nc", "nước"),
            ("trc", "trước"),
            ("ms", "mới"),
            ("nma", "nhưng mà"),
            ("bt", "bình thường"),
            ("mk", "mình"),
            ("bn", "bạn"),
            ("cx", "cũng"),
            ("ng", "người"),
            ("gd", "gia đình"),
            ("hk", "không"),
            ("r", "rồi"),
            ("vs", "với"),
            ("vn", "Việt Nam"),
            ("hn", "Hà Nội"),
            ("sg", "Sài Gòn"),
            ("tks", "cảm ơn"),
            ("vd", "ví dụ"),
            ("th", "thôi"),
        ];
        for (k, v) in defaults {
            dict.add(k, v);
        }
        dict
    }

    /// Add or update an entry
    pub fn add(&mut self, abbreviation: &str, expansion: &str) {
        self.entries
            .insert(abbreviation.to_string(), expansion.to_string());
    }

    /// Remove an entry
    pub fn remove(&mut self, abbreviation: &str) -> bool {
        self.entries.remove(abbreviation).is_some()
    }

    /// Look up an abbreviation
    pub fn lookup(&self, abbreviation: &str) -> Option<&str> {
        self.entries.get(abbreviation).map(|s| s.as_str())
    }

    /// Get all entries as a sorted vector
    pub fn entries(&self) -> Vec<(&str, &str)> {
        let mut v: Vec<_> = self
            .entries
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        v.sort_by_key(|(k, _)| *k);
        v
    }

    /// Number of entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the dictionary is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Export to JSON string
    pub fn export_json(&self) -> String {
        let mut json = String::from("{\n");
        let entries = self.entries();
        for (i, (k, v)) in entries.iter().enumerate() {
            // Manual JSON to avoid serde dependency
            json.push_str(&format!("  \"{}\": \"{}\"", escape_json(k), escape_json(v)));
            if i < entries.len() - 1 {
                json.push(',');
            }
            json.push('\n');
        }
        json.push('}');
        json
    }

    /// Import from JSON string
    pub fn import_json(json: &str) -> Result<Self, String> {
        let mut dict = Self::new();
        // Simple JSON parser (no serde dependency)
        let trimmed = json.trim();
        if !trimmed.starts_with('{') || !trimmed.ends_with('}') {
            return Err("Invalid JSON: must be an object".to_string());
        }
        let inner = &trimmed[1..trimmed.len() - 1];
        for line in inner.lines() {
            let line = line.trim().trim_end_matches(',');
            if line.is_empty() {
                continue;
            }
            // Parse "key": "value"
            let parts: Vec<&str> = line.splitn(2, ':').collect();
            if parts.len() != 2 {
                continue;
            }
            let key = parts[0].trim().trim_matches('"');
            let val = parts[1].trim().trim_matches('"');
            if !key.is_empty() && !val.is_empty() {
                dict.add(key, val);
            }
        }
        Ok(dict)
    }

    /// Export to CSV string (tab-separated)
    pub fn export_csv(&self) -> String {
        let mut csv = String::from("abbreviation\texpansion\n");
        for (k, v) in self.entries() {
            csv.push_str(&format!("{}\t{}\n", k, v));
        }
        csv
    }

    /// Import from CSV string (tab-separated)
    pub fn import_csv(csv: &str) -> Result<Self, String> {
        let mut dict = Self::new();
        for (i, line) in csv.lines().enumerate() {
            if i == 0 {
                continue;
            } // skip header
            let parts: Vec<&str> = line.splitn(2, '\t').collect();
            if parts.len() == 2 && !parts[0].is_empty() {
                dict.add(parts[0], parts[1]);
            }
        }
        Ok(dict)
    }
}

impl Default for ShorthandDict {
    fn default() -> Self {
        Self::new()
    }
}

/// Escape special characters for JSON string
fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\t', "\\t")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_lookup() {
        let dict = ShorthandDict::with_defaults();
        assert_eq!(dict.lookup("dc"), Some("được"));
        assert_eq!(dict.lookup("kg"), Some("không"));
        assert_eq!(dict.lookup("xyz"), None);
    }

    #[test]
    fn test_add_remove() {
        let mut dict = ShorthandDict::new();
        dict.add("tl", "trả lời");
        assert_eq!(dict.lookup("tl"), Some("trả lời"));
        dict.remove("tl");
        assert_eq!(dict.lookup("tl"), None);
    }

    #[test]
    fn test_json_roundtrip() {
        let mut dict = ShorthandDict::new();
        dict.add("dc", "được");
        dict.add("kg", "không");
        let json = dict.export_json();
        let dict2 = ShorthandDict::import_json(&json).unwrap();
        assert_eq!(dict2.lookup("dc"), Some("được"));
        assert_eq!(dict2.lookup("kg"), Some("không"));
    }

    #[test]
    fn test_csv_roundtrip() {
        let mut dict = ShorthandDict::new();
        dict.add("dc", "được");
        dict.add("kg", "không");
        let csv = dict.export_csv();
        let dict2 = ShorthandDict::import_csv(&csv).unwrap();
        assert_eq!(dict2.lookup("dc"), Some("được"));
        assert_eq!(dict2.lookup("kg"), Some("không"));
    }

    #[test]
    fn test_defaults_count() {
        let dict = ShorthandDict::with_defaults();
        assert!(dict.len() >= 20);
    }
}
