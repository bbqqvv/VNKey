//! # Vietnamese IME Engine
//!
//! A pure Rust Vietnamese input method engine supporting Telex and VNI.
//!
//! ## Design Philosophy
//! - **Zero OS dependencies** — works anywhere Rust compiles
//! - **Zero framework dependencies** — no Tauri, no Windows, no macOS
//! - **Pure logic** — receives characters, returns transformed text
//!
//! ## Usage
//! ```rust
//! use vnkey_core::{Engine, InputMode};
//!
//! let mut engine = Engine::new(InputMode::Telex);
//! assert_eq!(engine.process_key('v'), "v");
//! assert_eq!(engine.process_key('i'), "vi");
//! assert_eq!(engine.process_key('e'), "vie");
//! assert_eq!(engine.process_key('e'), "viê");
//! assert_eq!(engine.process_key('s'), "viế");
//! // ...
//! ```
//!
//! ## Architecture
//! ```text
//! [Any Input Source] → Engine::process_key(char) → String
//!                              ↑
//!           [telex.rs / vni.rs] + [tone.rs] + [syllable.rs]
//!                              ↑
//!                       [unicode_map.rs]
//! ```

use serde::{Serialize, Deserialize};

pub mod telex;
pub mod vni;
pub mod viqr;
pub mod syllable;
pub mod tone;
pub mod unicode_map;
pub mod config;
pub mod shorthand;
pub mod converter;
pub mod phonology;
pub mod error;
pub mod dictionary;
pub mod ffi;
#[cfg(windows)]
pub mod hook;

// Re-exports for convenience
pub use syllable::Syllable;
pub use tone::ToneMark;
pub use config::{EngineConfig, TonePlacement};
pub use shorthand::ShorthandDict;
pub use converter::{remove_diacritics, is_vietnamese, VnEncoding};
pub use phonology::{validate_syllable, is_perfect};
pub use error::{EngineError, EngineResult};
pub use dictionary::{Dictionary, Suggestion};

use std::collections::HashMap;

/// Input method mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    /// Standard Telex: aa=â, dd=đ, w=ư (Smart W), s/f/r/x/j for tones
    Telex,
    /// TelexEx: Like Telex but W does NOT produce Ư (w stays as w)
    TelexEx,
    /// VNI: Number keys for tones (1-5), number keys for modifiers
    Vni,
    /// VIQR: Punctuation marks for tones ('/`/?/~/.), ^/+/( for modifiers
    Viqr,
}

/// Represents the full internal state of the engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineState {
    pub transformed: String,
    pub buffer: String,
    pub onset: String,
    pub vowel: String,
    pub coda: String,
    pub tone: u8,
    pub z_level: u8,
    pub validity_score: u8,
    pub suggestions: Vec<Suggestion>,
}

/// Detailed diagnostic information for developer mode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticData {
    pub buffer: String,
    pub onset: String,
    pub vowel: String,
    pub coda: String,
    pub tone: u8,
    pub phonetic_score: u8,
    pub literal_mode: bool,
    pub z_level: u8,
    pub mode: String,
}

/// Vietnamese IME Engine — the brain of the input method.
///
/// Stateful: maintains an internal buffer for the current word being typed.
/// Call `reset()` when the user moves to a new word.
pub struct Engine {
    mode: InputMode,
    config: config::EngineConfig,
    buffer: String,
    current_syllable: Syllable,
    shorthand_dict: ShorthandDict,
    /// Tracks which positions in the original buffer were uppercase
    case_map: Vec<bool>,
    /// Progressive Z state: tracks how many times Z has been pressed
    /// 0 = normal, 1 = tone removed, 2 = modifiers removed, 3 = literal z
    z_level: u8,
    /// Snapshot of syllable before Z was pressed (for progressive undo)
    pre_z_syllable: Option<Syllable>,
    pre_z_buffer: Option<String>,
    /// The very last word that was committed (including the whitespace/punctuation)
    /// This is used to restore buffer if user presses Backspace immediately after space
    last_committed_word: String,
    /// The raw buffer of the last word before it was reset on word boundary
    last_committed_buffer: String,
    /// The case map of the last word before it was reset
    last_committed_case_map: Vec<bool>,
    /// Whether the next character should be capitalized
    capitalize_next: bool,
    /// Literal Mode: if true, bypass all Vietnamese transformations for this word.
    /// Triggered for long nonsense strings or foreign words.
    literal_mode: bool,
    dictionary: Dictionary,
}

impl Engine {
    /// Create a new engine with the specified input mode.
    pub fn new(mode: InputMode) -> Self {
        Self {
            mode,
            config: config::EngineConfig::default(),
            buffer: String::new(),
            current_syllable: Syllable::new(),
            shorthand_dict: ShorthandDict::with_defaults(),
            case_map: Vec::new(),
            z_level: 0,
            pre_z_syllable: None,
            pre_z_buffer: None,
            last_committed_word: String::new(),
            last_committed_buffer: String::new(),
            last_committed_case_map: Vec::new(),
            capitalize_next: false,
            literal_mode: false,
            dictionary: {
                let mut d = Dictionary::new();
                // Try to load the Vietnamese dictionary file
                // Path is relative to the core's source or the executable
                let _ = d.load_from_file("src/dictionaries/vi.dic");
                d
            },
        }
    }

    /// Create engine with custom config.
    pub fn with_config(mode: InputMode, config: config::EngineConfig) -> Self {
        let mut engine = Self::new(mode);
        engine.config = config;
        engine
    }

    /// Get the current internal state of the engine.
    pub fn get_state(&self) -> EngineState {
        let score = if self.config.spell_check && self.config.vietnamese_mode {
            phonology::validate_syllable(&self.current_syllable, self.config.allow_foreign_consonants)
        } else {
            100 // Default to 100 (perfect) if validation is disabled or in English mode
        };
        
        let transformed = if self.config.vietnamese_mode && !self.literal_mode {
            self.reconstruct()
        } else {
            self.apply_case(&self.buffer)
        };
        
        EngineState {
            transformed,
            buffer: self.buffer.clone(),
            onset: self.current_syllable.onset.clone(),
            vowel: self.current_syllable.vowel.clone(),
            coda: self.current_syllable.coda.clone(),
            tone: self.current_syllable.tone,
            z_level: self.z_level,
            validity_score: score,
            suggestions: Vec::new(),
        }
    }

    /// Get detailed diagnostic info for developer mode.
    pub fn get_diagnostic_info(&self) -> DiagnosticData {
        let score = validate_syllable(&self.current_syllable, self.config.allow_foreign_consonants);
        DiagnosticData {
            buffer: self.buffer.clone(),
            onset: self.current_syllable.onset.clone(),
            vowel: self.current_syllable.vowel.clone(),
            coda: self.current_syllable.coda.clone(),
            tone: self.current_syllable.tone,
            phonetic_score: score,
            literal_mode: self.literal_mode,
            z_level: self.z_level,
            mode: format!("{:?}", self.mode),
        }
    }

    /// Change the input mode. Resets the buffer.
    pub fn set_mode(&mut self, mode: InputMode) {
        self.mode = mode;
        self.reset();
    }

    /// Get the current input mode.
    pub fn mode(&self) -> InputMode {
        self.mode
    }

    /// Update configuration at runtime.
    pub fn set_config(&mut self, config: config::EngineConfig) {
        self.config = config;
    }

    /// Get current config reference.
    pub fn config(&self) -> &config::EngineConfig {
        &self.config
    }

    /// Replace the entire shorthand dictionary at runtime.
    pub fn set_macros(&mut self, macros: HashMap<String, String>) {
        self.shorthand_dict = ShorthandDict::new();
        for (k, v) in macros {
            self.shorthand_dict.add(&k, &v);
        }
    }

    /// Get current shorthand dictionary reference.
    pub fn shorthand_dict(&self) -> &ShorthandDict {
        &self.shorthand_dict
    }

    /// Load dictionary from a file
    pub fn load_dictionary(&mut self, path: &str) -> std::io::Result<()> {
        self.dictionary.load_from_file(path)
    }

    pub fn set_tone_placement(&mut self, style: bool) {
        self.config.modern_tone = style;
    }

    pub fn buffer(&self) -> &str {
        &self.buffer
    }

    pub fn case_map(&self) -> &Vec<bool> {
        &self.case_map
    }

    /// Reset the engine state. Call this on word boundary (space, punctuation).
    pub fn reset_soft(&mut self) {
        self.buffer.clear();
        self.current_syllable = Syllable::new();
        self.case_map.clear();
        self.z_level = 0;
        self.pre_z_syllable = None;
        self.pre_z_buffer = None;
        self.literal_mode = false;
        self.capitalize_next = false;
    }

    /// Hard reset: clear everything including committed history and capitalization flags.
    /// Call this on context changes (mouse click, navigation, focus change).
    pub fn reset(&mut self) {
        self.reset_soft();
        self.last_committed_word.clear();
        self.last_committed_buffer.clear();
        self.last_committed_case_map.clear();
        self.capitalize_next = false;
        self.literal_mode = false;
    }

    /// Signal that Enter was pressed.
    /// Returns the expanded shorthand if any, otherwise an empty string.
    pub fn on_enter(&mut self) -> String {
        let result = self.process_key('\n');
        
        if self.config.auto_capitalize_enter {
            self.capitalize_next = true;
        }
        
        // Remove the trailing newline since the hook handles the Enter key itself
        if result.ends_with('\n') {
            result[..result.len() - 1].to_string()
        } else {
            result
        }
    }

    /// Process a single key press and return the current word output.
    ///
    /// This is the primary API. Feed characters one by one and get the
    /// progressively transformed Vietnamese text.
    pub fn process_key(&mut self, key: char) -> String {
        // P1 FIX: Guard against buffer overflow (max 50 chars per word)
        // Longest valid Vietnamese word is ~10 chars; 50 is generous safety margin
        if self.buffer.len() >= 50 {
            self.reset();
            return key.to_string();
        }

        // Optimized: Avoid string creation if possible
        if key.is_whitespace() || (key.is_ascii_punctuation() && !self.is_special_punctuation(key)) {
            let mut transformed = if self.config.vietnamese_mode && !self.literal_mode {
                self.reconstruct()
            } else {
                self.apply_case(&self.buffer)
            };
            
            // P7 FIX: Push separator to buffer so shorthand logic sees the trigger
            self.buffer.push(key);
            self.case_map.push(key.is_uppercase());

            if self.is_shorthand_active() {
                let old_len = self.buffer.len();
                self.apply_shorthand_if_needed(&mut transformed);
                
                // If shorthand expanded, it cleared the buffer and updated `transformed` (including trigger)
                if self.buffer.is_empty() && old_len > 0 {
                    return transformed;
                }
            }
            
            // If no shorthand match, handle as normal word termination
            // Save for backspace restoration
            self.last_committed_word = transformed.clone();
            self.last_committed_word.push(key);
            self.last_committed_buffer = self.buffer.clone();
            self.last_committed_case_map = self.case_map.clone();

            // P8: Update capitalization state after sentence-ending punctuation
            if self.config.auto_capitalize_sentence && ".!?".contains(key) {
                self.capitalize_next = true;
            }

            let mut result = transformed;
            result.push(key);
            self.reset_soft();
            return result;
        }

        // Handle progressive Z (Telex/TelexEx only)
        if self.config.vietnamese_mode && (self.mode == InputMode::Telex || self.mode == InputMode::TelexEx)
            && key == 'z'
            && !self.buffer.is_empty()
            && self.current_syllable.has_vowel()
        {
            return self.handle_progressive_z();
        }

        self.reset_z_state();

        if self.literal_mode {
            // Smart break: If we are in literal mode (English/Non-VN) and type a CAPITAL letter,
            // or if the buffer is getting long, start a new word.
            if key.is_uppercase() && self.buffer.len() > 1 {
                self.reset_soft();
            } else {
                self.buffer.push(key);
                self.case_map.push(key.is_uppercase());
                return self.apply_case(&self.buffer);
            }
        }

        // P8: Auto-capitalize the first character of a new word if flagged
        let mut final_key = key;
        if self.capitalize_next && self.buffer.is_empty() && key.is_lowercase() {
            final_key = key.to_uppercase().next().unwrap_or(key);
            self.capitalize_next = false;
        } else if !self.buffer.is_empty() {
            // If we are already in a word, clear the flag (it only applies to the start)
            self.capitalize_next = false;
        }

        self.case_map.push(final_key.is_uppercase());
        self.buffer.push(final_key);

        let result = if self.config.vietnamese_mode {
            self.parse_buffer();
            self.check_literal_mode_transition();
            
            if self.literal_mode {
                return self.apply_case(&self.buffer);
            }

            let phonetic_score = phonology::validate_syllable(&self.current_syllable, self.config.allow_foreign_consonants);

            if self.config.spell_check {
                self.apply_smart_spelling_corrections();
            }
            
            let mut transformed = self.reconstruct();

            // P5 FIX: Apply shorthand (Gõ tắt)
            if self.is_shorthand_active() {
                self.apply_shorthand_if_needed(&mut transformed);
            }

            // UniKey Auto Restore: Restore if word is not in dictionary AND is phonetically invalid
            if self.config.spell_check && self.config.auto_restore {
                let is_in_dict = self.dictionary.contains(&transformed);
                
                // P13 FIX: Score < 10 marks anything linguistically suspicious (invalid onset/coda/vowel)
                if !is_in_dict && phonetic_score < 10 && self.buffer.len() > 1 {
                    transformed = self.apply_case(&self.buffer);
                }
            }
            transformed
        } else {
            let mut transformed = self.buffer.clone();
            // Shorthand while E mode is off (if enabled)
            if self.is_shorthand_active() {
                self.apply_shorthand_if_needed(&mut transformed);
            }
            transformed
        };

        result
    }

    /// Smart Check: If the current syllable has an invalid phonetic structure,
    /// and it's not a known Vietnamese exception, switch to literal mode.
    fn check_literal_mode_transition(&mut self) {
        if !self.config.smart_literal_mode || self.buffer.is_empty() || self.literal_mode { return; }

        let phonetic_score = validate_syllable(&self.current_syllable, self.config.allow_foreign_consonants);
        
        let onset_len = self.current_syllable.onset.chars().count();
        let coda_len = self.current_syllable.coda.chars().count();

        // Structural checks
        if (onset_len > 5 || coda_len > 5) || self.buffer.len() > 30 {
            self.literal_mode = true;
            return;
        }

        // Garbage scoring checks
        if phonetic_score <= 2 && self.buffer.len() > 10 {
            self.literal_mode = true;
            return;
        }

        if phonetic_score <= 5 && self.buffer.len() > 10 {
            self.literal_mode = true;
            return;
        }

        if phonetic_score <= 8 && self.buffer.len() > 10 {
            self.literal_mode = true;
            return;
        }

        // 2. Dictionary-aware short word check (Special focus on English collisions like of, is, as)
        if self.config.spell_check && self.buffer.len() == 2 && self.current_syllable.tone != 0 {
            let core = format!("{}{}", self.current_syllable.onset, self.current_syllable.vowel);
            let toned = if self.current_syllable.tone > 0 {
                tone::place_tone_with_style(
                    &core,
                    self.current_syllable.tone,
                    if self.config.modern_tone { config::TonePlacement::Modern } else { config::TonePlacement::Traditional },
                ).unwrap_or(core)
            } else {
                core
            };
            let transformed = format!("{}{}", toned, self.current_syllable.coda).to_lowercase();

            if !self.dictionary.contains(&transformed) {
                let last_char = self.buffer.chars().last().unwrap().to_ascii_lowercase();
                if matches!(last_char, 's' | 'f' | 'r' | 'x' | 'j') {
                    self.literal_mode = true;
                }
            }
        }
    }

    /// Process a Backspace key press.
    /// Returns true if the engine handled the backspace (internal buffer changed),
    /// false if the caller should handle it (e.g. by deleting a character in the app).
    pub fn process_backspace(&mut self) -> bool {
        if !self.buffer.is_empty() {
            // Normal backspace within a word
            self.buffer.pop();
            self.case_map.pop();
            self.reset_z_state();
            if self.buffer.is_empty() {
                self.last_committed_word.clear(); // Clear history if we manually deleted the word
            }
            if self.config.vietnamese_mode && !self.literal_mode {
                self.parse_buffer();
            }
            return true;
        } else if self.config.backspace_restore && !self.last_committed_word.is_empty() {
            // Specialized backspace: restore previous word context if we just pressed space
            let last = self.last_committed_word.clone();
            self.last_committed_word.clear();

            // Only restore if the last committed was [word] + [single space/punct]
            if last.ends_with(' ') || last.chars().last().map_or(false, |c| c.is_ascii_punctuation()) {
                // We're "undoing" the reset() that happened on space.
                // Restore the RAW buffer so the user can continue typing/modifying the word.
                if !self.last_committed_buffer.is_empty() {
                    self.buffer = self.last_committed_buffer.clone();
                    self.case_map = self.last_committed_case_map.clone();
                    self.last_committed_buffer.clear();
                    self.last_committed_case_map.clear();
                    if self.config.vietnamese_mode {
                        self.parse_buffer();
                    }
                    return true;
                }
            }
        }
        
        false
    }

    fn is_special_punctuation(&self, key: char) -> bool {
        match self.mode {
            InputMode::Telex | InputMode::TelexEx => key == '[' || key == ']',
            InputMode::Vni => key.is_ascii_digit(),
            InputMode::Viqr => "'^`?~.+(".contains(key),
        }
    }

    fn is_shorthand_active(&self) -> bool {
        self.config.macro_enabled && (self.config.vietnamese_mode || self.config.shorthand_while_off)
    }

    fn apply_shorthand_if_needed(&mut self, transformed: &mut String) {
        if self.buffer.is_empty() { return; }

        // Determine if the last character is a trigger (space or punctuation)
        let last_char = self.buffer.chars().last().unwrap();
        let has_trigger = last_char == ' ' || last_char.is_ascii_punctuation();
        
        // P6 FIX: ONLY expand shorthand if a trigger character is pressed (Space/Punctuation)
        if !has_trigger { return; }

        // The potential macro is the buffer minus the trigger
        let macro_part = &self.buffer[..self.buffer.len() - last_char.len_utf8()];

        if macro_part.is_empty() { return; }

        let mut expansion_found = None;

        // 1. Try exact match first
        if let Some(expansion) = self.shorthand_dict.lookup(macro_part) {
            expansion_found = Some(expansion.to_string());
        } 
        // 2. Fallback to smart casing if enabled
        else if self.config.macro_auto_case {
            let macro_lower = macro_part.to_lowercase();
            if let Some(expansion) = self.shorthand_dict.lookup(&macro_lower) {
                // Determine casing mode based on the original macro_part
                let is_all_caps = macro_part.chars().all(|c| !c.is_lowercase());
                let is_title_case = macro_part.chars().next().map_or(false, |c| c.is_uppercase()) && !is_all_caps;

                if is_all_caps {
                    expansion_found = Some(expansion.to_uppercase());
                } else if is_title_case {
                    let mut chars = expansion.chars();
                    if let Some(first) = chars.next() {
                        let rest: String = chars.collect();
                        expansion_found = Some(first.to_uppercase().collect::<String>() + &rest);
                    } else {
                        expansion_found = Some(expansion.to_string());
                    }
                } else {
                    expansion_found = Some(expansion.to_string());
                }
            }
        }

        if let Some(mut expansion) = expansion_found {
            if has_trigger {
                expansion.push(last_char);
            }
            *transformed = expansion;
            // Clear buffer and state after shorthand replacement to start fresh
            self.buffer.clear();
            self.case_map.clear();
            self.reset_z_state();
        }
    }

    fn reset_z_state(&mut self) {
        self.z_level = 0;
        self.pre_z_syllable = None;
        self.pre_z_buffer = None;
    }

    /// Automatically correct spelling mistakes based on Vietnamese phonotactics.
    /// 
    /// P3 FIX: Refined rules with proper gi/ghi handling
    /// Rules: 
    /// - c → k before i, e, ê, y (e.g., "ci" → "ki" is correct)
    /// - g → gh before e, ê (NOT before i — "gi" is its own onset)
    /// - ng → ngh before i, e, ê
    /// - Reverse: k → c, gh → g, ngh → ng before back vowels (a, o, u, ô, ơ, ă, â, ư)
    fn apply_smart_spelling_corrections(&mut self) {
        if self.current_syllable.vowel.is_empty() { return; }
        
        let first_v = self.current_syllable.vowel.chars().next().unwrap().to_lowercase().next().unwrap();
        let is_front = matches!(first_v, 'i' | 'e' | 'ê' | 'y');
        
        let onset = self.current_syllable.onset.to_lowercase();
        
        match onset.as_str() {
            "c" if is_front => self.current_syllable.onset = "k".to_string(),
            // P3 FIX: "g" only becomes "gh" before e/ê (NOT i/y)
            // Because "gi" is a valid onset in Vietnamese (giá, giờ, giêng)
            "g" if matches!(first_v, 'e' | 'ê') => self.current_syllable.onset = "gh".to_string(),
            "ng" if is_front => self.current_syllable.onset = "ngh".to_string(),
            
            "k" if !is_front => self.current_syllable.onset = "c".to_string(),
            // P3 FIX: "gh" stays "gh" before i (ghi là hợp lệ: ghi chép, ghi nhớ)
            // Only revert to "g" before back vowels
            "gh" if !is_front => self.current_syllable.onset = "g".to_string(),
            "ngh" if !is_front => self.current_syllable.onset = "ng".to_string(),
            
            _ => {}
        }
    }

    /// Process an entire string at once (convenience method).
    pub fn feed_str(&mut self, input: &str) -> String {
        self.reset();
        let mut result = String::new();
        for c in input.chars() {
            result = self.process_key(c);
        }
        result
    }

    /// Handle Progressive Z: multi-level undo
    /// Level 0→1: Remove tone (á → a)
    /// Level 1→2: Remove modifiers (â→a, ơ→o, ư→u, đ→d)
    /// Level 2→3: Output literal 'z'
    fn handle_progressive_z(&mut self) -> String {
        match self.z_level {
            0 => {
                if self.current_syllable.tone == 0 {
                    self.z_level = 1;
                    return self.handle_progressive_z();
                }
                
                // Save state before first Z
                self.pre_z_syllable = Some(self.current_syllable.clone());
                self.pre_z_buffer = Some(self.buffer.clone());
                
                self.current_syllable.tone = 0;
                self.z_level = 1;
                self.sync_buffer_from_reconstruction()
            }
            1 => {
                // Remove all modifiers
                let demod = |s: &str| -> String {
                    s.chars().map(|c| match c {
                        'â' | 'ă' => 'a', 'ê' => 'e', 'ô' | 'ơ' => 'o',
                        'ư' => 'u', 'đ' => 'd',
                        _ => c,
                    }).collect()
                };
                
                let demod_onset = demod(&self.current_syllable.onset);
                let demod_vowel = demod(&self.current_syllable.vowel);
                let demod_coda = demod(&self.current_syllable.coda);
                
                if demod_onset == self.current_syllable.onset &&
                   demod_vowel == self.current_syllable.vowel &&
                   demod_coda == self.current_syllable.coda {
                    // No modifiers found, skip to next level
                    self.z_level = 2;
                    return self.handle_progressive_z();
                }
                
                self.current_syllable.onset = demod_onset;
                self.current_syllable.vowel = demod_vowel;
                self.current_syllable.coda = demod_coda;
                self.z_level = 2;
                self.sync_buffer_from_reconstruction()
            }
            _ => {
                // Level 3+: append literal 'z' to the completely demodified buffer
                self.buffer.push('z');
                self.case_map.push(false);
                self.z_level = 0;
                self.current_syllable.coda.push('z');
                self.reconstruct()
            }
        }
    }

    /// P4 FIX DRY: Reconstruct output and sync buffer/case_map from it.
    /// Replaces duplicated pattern in handle_progressive_z levels 0 and 1.
    fn sync_buffer_from_reconstruction(&mut self) -> String {
        let result = self.reconstruct();
        self.buffer = result.to_lowercase();
        self.case_map = result.chars().map(|c| c.is_uppercase()).collect();
        result
    }

    /// Internal: parse the raw buffer into a Syllable.
    fn parse_buffer(&mut self) {
        let input = self.buffer.to_lowercase();

        // Step 1: Extract tone from raw input
        let (core, tone_idx) = match self.mode {
            InputMode::Telex | InputMode::TelexEx => telex::extract_tone(&input),
            InputMode::Vni => vni::extract_tone(&input),
            InputMode::Viqr => viqr::extract_tone(&input),
        };

        // Step 2: Apply character modifiers (aa→â, dd→đ, etc.)
        let transformed = match self.mode {
            InputMode::Telex => telex::apply_modifiers(&core),
            InputMode::TelexEx => telex::apply_modifiers_no_smart_w(&core),
            InputMode::Vni => vni::apply_modifiers(&core),
            InputMode::Viqr => viqr::apply_modifiers(&core),
        };

        // Step 3: Parse into onset/vowel/coda structure
        self.current_syllable = syllable::parse(&transformed, tone_idx);
    }

    /// Internal: reconstruct the output string from current syllable + apply case.
    fn reconstruct(&self) -> String {
        if self.literal_mode {
            return self.apply_case(&self.buffer);
        }
        // CRITICAL: Place tone ONLY on onset+vowel, then append coda.
        let core = format!("{}{}", self.current_syllable.onset, self.current_syllable.vowel);
        let toned_core = if self.current_syllable.tone > 0 {
            tone::place_tone_with_style(
                &core,
                self.current_syllable.tone,
                if self.config.modern_tone { config::TonePlacement::Modern } else { config::TonePlacement::Traditional },
            ).unwrap_or(core)
        } else {
            core
        };
        let toned = format!("{}{}", toned_core, self.current_syllable.coda);
        self.apply_case(&toned)
    }

    /// Apply casing from case_map to a target string.
    ///
    /// Uses semantic case mapping: detects ALLCAPS / TitleCase / lowercase
    /// from the input case_map, then applies consistently to output regardless
    /// of length differences caused by modifiers (aa→â, dd→đ, etc.).
    fn apply_case(&self, target: &str) -> String {
        if self.case_map.is_empty() {
            return target.to_string();
        }

        // Determine semantic case pattern from input
        let is_all_upper = self.case_map.iter().all(|&c| c);
        let is_first_upper = self.case_map.first().copied().unwrap_or(false);
        let is_title_case = is_first_upper && !is_all_upper;

        let target_char_count = target.chars().count();

        if target_char_count == self.case_map.len() {
            // Perfect length match: apply case mapping character by character
            let mut result = String::new();
            for (c, &is_upper) in target.chars().zip(self.case_map.iter()) {
                if is_upper {
                    result.extend(c.to_uppercase());
                } else {
                    result.extend(c.to_lowercase());
                }
            }
            result
        } else {
            // Lengths differ (due to modifiers): use semantic case pattern
            if is_all_upper {
                target.to_uppercase()
            } else if is_title_case {
                // Title Case: capitalize first char, lowercase the rest
                let mut chars = target.chars();
                match chars.next() {
                    Some(first) => {
                        let upper: String = first.to_uppercase().collect();
                        let rest: String = chars.as_str().to_lowercase();
                        upper + &rest
                    }
                    None => target.to_string(),
                }
            } else {
                // All lowercase
                target.to_lowercase()
            }
        }
    }
}

// ============================================================
// TESTS — the whole point of separating the engine!
// Run with: cargo test -p vietnamese-ime-engine
// ============================================================
#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_engine_no_panic(s in "\\PC*") {
            let mut engine = Engine::new(InputMode::Telex);
            for c in s.chars() {
                engine.process_key(c);
            }
        }

        #[test]
        fn test_buffer_sync_integrity(s in "[a-zA-Z0-9 ]*") {
            let mut engine = Engine::new(InputMode::Telex);
            for c in s.chars() {
                let _ = engine.process_key(c);
                if c == ' ' {
                    prop_assert!(engine.buffer.is_empty());
                } else {
                    prop_assert_eq!(engine.buffer.len(), engine.case_map().len());
                }
            }
        }
    }

    // ---- Basic Telex ----

    #[test]
    fn test_telex_basic_tone() {
        let mut e = Engine::new(InputMode::Telex);
        let mut cfg = e.config().clone();
        cfg.spell_check = false;
        cfg.auto_restore = false;
        e.set_config(cfg);
        assert_eq!(e.feed_str("as"), "\u{00E1}");
        e.reset();
        assert_eq!(e.feed_str("af"), "\u{00E0}");
        e.reset();
        assert_eq!(e.feed_str("ar"), "\u{1EA3}");
        e.reset();
        assert_eq!(e.feed_str("ax"), "\u{00E3}");
        e.reset();
        assert_eq!(e.feed_str("aj"), "\u{1EA1}");
    }

    #[test]
    fn test_telex_word_viet() {
        let mut e = Engine::new(InputMode::Telex);
        assert_eq!(e.feed_str("Vieejt"), "Vi\u{1EC7}t");
    }

    #[test]
    fn test_telex_word_tieng() {
        let mut e = Engine::new(InputMode::Telex);
        assert_eq!(e.feed_str("tieesng"), "ti\u{1EBF}ng");
    }

    // ---- QU / GI special cases ----

    #[test]
    fn test_telex_qu() {
        let mut e = Engine::new(InputMode::Telex);
        let mut cfg = e.config().clone();
        cfg.spell_check = false;
        cfg.auto_restore = false;
        e.set_config(cfg);
        assert_eq!(e.process_key('q'), "q");
        assert_eq!(e.process_key('u'), "qu");
        assert_eq!(e.process_key('a'), "qua");
        assert_eq!(e.process_key('s'), "qu\u{00E1}");
    }

    #[test]
    fn test_telex_gi() {
        let mut e = Engine::new(InputMode::Telex);
        assert_eq!(e.process_key('g'), "g");
        assert_eq!(e.process_key('i'), "gi");
        assert_eq!(e.process_key('f'), "g\u{00EC}");
    }

    // ---- Smart W & Bracket keys ----

    #[test]
    fn test_telex_bracket_u() {
        let mut e = Engine::new(InputMode::Telex);
        let mut cfg = e.config().clone();
        cfg.spell_check = false;
        cfg.auto_restore = false;
        e.set_config(cfg);
        assert_eq!(e.process_key('h'), "h");
        assert_eq!(e.process_key('['), "h\u{01B0}");
    }

    #[test]
    fn test_telex_bracket_o() {
        let mut e = Engine::new(InputMode::Telex);
        let mut cfg = e.config().clone();
        cfg.spell_check = false;
        cfg.auto_restore = false;
        e.set_config(cfg);
        assert_eq!(e.process_key('h'), "h");
        assert_eq!(e.process_key(']'), "h\u{01A1}");
    }

    // ---- Z and X at word start (no vowel yet) ----

    #[test]
    fn test_telex_x_no_vowel() {
        let mut e = Engine::new(InputMode::Telex);
        assert_eq!(e.process_key('x'), "x");
    }

    #[test]
    fn test_telex_z_no_vowel() {
        let mut e = Engine::new(InputMode::Telex);
        assert_eq!(e.process_key('z'), "z");
    }

    // ---- Double-tap cancellation ----

    #[test]
    fn test_telex_double_s() {
        let mut e = Engine::new(InputMode::Telex);
        let mut cfg = e.config().clone();
        cfg.spell_check = false;
        cfg.auto_restore = false;
        e.set_config(cfg);
        assert_eq!(e.feed_str("ass"), "as");
    }

    // ---- Casing Preservation ----
    
    #[test]
    fn test_casing_preservation() {
        let mut e = Engine::new(InputMode::Telex);
        let mut cfg = e.config().clone();
        cfg.spell_check = false;
        cfg.auto_restore = false;
        e.set_config(cfg);

        assert_eq!(e.feed_str("VNkey"), "VNkey");
        e.reset();
        assert_eq!(e.feed_str("Vnkey"), "Vnkey");
        e.reset();
        // viEejt -> lengths differ, falls back to first character capitalization check.
        // First char 'v' is lowercase, so the string falls back to lowercase -> việt
        let vi_et = "vi\u{1EC7}t"; 
        assert_eq!(e.feed_str("viEejt"), vi_et);
        e.reset();
        assert_eq!(e.feed_str("VIEEJT"), "VI\u{1EC6}T");
    }

    // ---- VNI basics ----

    #[test]
    fn test_vni_tone() {
        let mut e = Engine::new(InputMode::Vni);
        let mut cfg = e.config().clone();
        cfg.spell_check = false;
        cfg.auto_restore = false;
        e.set_config(cfg);
        assert_eq!(e.feed_str("a1"), "á");
        e.reset();
        assert_eq!(e.feed_str("a2"), "à");
    }

    // ---- Word boundary ----

    #[test]
    fn test_word_boundary_space() {
        let mut e = Engine::new(InputMode::Telex);
        let mut cfg = e.config().clone();
        cfg.spell_check = false;
        cfg.auto_restore = false;
        e.set_config(cfg);
        let r = e.feed_str("as ");
        assert_eq!(r, "\u{00E1} ");
        // After space, buffer should be reset
        assert_eq!(e.process_key('b'), "b");
    }

    // ---- Mode switching ----

    #[test]
    fn test_mode_switch() {
        let mut e = Engine::new(InputMode::Telex);
        assert_eq!(e.feed_str("as"), "á");
        e.set_mode(InputMode::Vni);
        assert_eq!(e.feed_str("a1"), "á");
    }

    // ---- Phase 9: UniKey Parity Tests ----

    #[test]
    fn test_shorthand_basic() {
        let mut e = Engine::new(InputMode::Telex);
        let mut cfg = e.config().clone();
        cfg.macro_enabled = true;
        e.set_config(cfg);

        // Add a clean test case to avoid ambiguity
        e.shorthand_dict.add("test", "th\u{00E0}nh c\u{00F4}ng");

        // "test" -> "thành công"
        assert_eq!(e.feed_str("test "), "th\u{00E0}nh c\u{00F4}ng ");
        
        // Casing: "Test" -> "Thành công"
        e.reset();
        assert_eq!(e.feed_str("Test "), "Th\u{00E0}nh c\u{00F4}ng ");

        // Built-in check
        e.reset();
        assert_eq!(e.feed_str("vn "), "Vi\u{1EC7}t Nam ");
    }

    #[test]
    fn test_shorthand_while_off() {
        let mut e = Engine::new(InputMode::Telex);
        let mut cfg = e.config().clone();
        cfg.vietnamese_mode = false; // E mode
        cfg.macro_enabled = true;
        cfg.shorthand_while_off = true;
        e.set_config(cfg);

        e.shorthand_dict.add("vn", "Việt Nam");

        // "vn" -> "Việt Nam" even in E mode
        assert_eq!(e.feed_str("vn "), "Việt Nam ");
        
        // Normal typing in E mode
        assert_eq!(e.feed_str("as"), "as");
    }

    #[test]
    fn test_modern_tone_placement() {
        let mut e = Engine::new(InputMode::Telex);
        let mut cfg = e.config().clone();
        
        // Traditional (default): hóa, túy
        cfg.modern_tone = false;
        e.set_config(cfg.clone());
        assert_eq!(e.feed_str("hoas"), "h\u{00F3}a"); // hóa
        e.reset();
        assert_eq!(e.feed_str("tuys"), "t\u{00FA}y"); // túy (Traditional)
        
        // Modern: hoá, tuý
        e.reset();
        cfg.modern_tone = true;
        e.set_config(cfg);
        assert_eq!(e.feed_str("hoas"), "ho\u{00E1}"); // hoá
        e.reset();
        assert_eq!(e.feed_str("tuys"), "tu\u{00FD}"); // tuý (Modern)
    }

    #[test]
    fn test_auto_restore_invalid_syllable() {
        let mut e = Engine::new(InputMode::Telex);
        let mut cfg = e.config().clone();
        cfg.spell_check = true;
        cfg.auto_restore = true;
        e.set_config(cfg);

        // "bcdf" is completely unparseable (no vowel at all, buffer > 2)
        // The engine should restore the raw buffer
        assert_eq!(e.feed_str("bcdf"), "bcdf");
        
        // Valid syllable should still transform
        e.reset();
        assert_eq!(e.feed_str("nguowif"), "người");

        // "ddax" SHOULD produce "đã" not "ddax" (has valid onset+vowel+tone)
        e.reset();
        assert_eq!(e.feed_str("ddax"), "\u{0111}\u{00E3}");
    }

    // ---- Double-tap and Triple-tap W ----
    #[test]
    fn test_telex_w_variants() {
        let mut e = Engine::new(InputMode::Telex);
        assert_eq!(e.feed_str("w"), "ư");
        e.reset();
        assert_eq!(e.feed_str("ww"), "w");
        e.reset();
        assert_eq!(e.feed_str("www"), "ww");
    }
}

