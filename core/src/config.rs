use serde::{Serialize, Deserialize};

/// Tone placement style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TonePlacement {
    /// Modern/New style: tone on last vowel in diphthong
    /// Example: "hoà", "loà" (tone on 'a')
    Modern,
    /// Traditional/Old style: tone on first vowel in diphthong
    /// Example: "hòa", "lòa" (tone on 'o')
    Traditional,
}

impl Default for TonePlacement {
    fn default() -> Self { TonePlacement::Traditional }
}

/// Engine configuration — aligned with UniKey 4.6 options
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct EngineConfig {
    /// Chế độ gõ: true = Tiếng Việt (V), false = Tiếng Anh (E)
    pub vietnamese_mode: bool,
    // --- Tùy chọn khác ---
    /// Cho phép gõ tự do (free-form late-binding modifiers)
    pub free_typing: bool,
    /// Đặt dấu oà, uý (thay vì òa, úy) — true = Modern, false = Traditional
    pub modern_tone: bool,
    /// Luôn sử dụng clipboard cho unicode
    pub clipboard_unicode: bool,
    /// Bật kiểm tra chính tả (Phonology validation)
    pub spell_check: bool,
    /// Tự động khôi phục phím với từ sai (Smart recovery)
    pub auto_restore: bool,
    /// Hiện thông báo phản hồi
    pub show_feedback: bool,
    /// Bật chuông khi chuyển ngôn ngữ
    pub beep_on_switch: bool,
    /// Cho phép 'z w j f' làm phụ âm (Foreign consonants support)
    pub allow_foreign_consonants: bool,

    // --- Tùy chọn gõ tắt ---
    /// Cho phép gõ tắt (Shorthand/Macros)
    pub macro_enabled: bool,
    /// Cho phép gõ tắt cả khi tắt tiếng Việt
    pub shorthand_while_off: bool,
    /// Tự động đổi chữ hoa theo phím tắt (Smart Casing fallback)
    pub macro_auto_case: bool,

    // --- Hệ thống ---
    /// Bật hội thoại này khi khởi động
    pub show_on_startup: bool,
    /// Khởi động cùng Windows
    pub start_with_windows: bool,
    /// Bật/Tắt theo từng ứng dụng
    pub per_app_state: bool,
    /// Giao diện Tiếng Việt
    pub vietnamese_interface: bool,

    // --- Bảng mã ---
    /// Bảng mã xuất (Output Encoding)
    pub output_charset: String,

    // --- Tính năng thông minh ---
    /// Tự động viết hoa sau dấu chấm (., ?, !)
    pub auto_capitalize_sentence: bool,
    /// Tự động viết hoa sau khi nhấn Enter
    pub auto_capitalize_enter: bool,

    // --- Tự động chuyển đổi & Phím tắt ---
    /// Danh sách ứng dụng tự động chuyển sang E (e.g. ["League of Legends.exe"])
    pub auto_switch_apps: Vec<String>,
    /// Phím tắt chuyển đổi E/V (e.g. "Ctrl+Shift", "Alt+Z")
    pub switch_shortcut: String,
    /// Độ trễ mô phỏng phím (ms) để tương thích môi trường chậm
    pub simulation_delay: u32,
    /// Cho phép khôi phục từ trước đó khi nhấn Backspace ở đầu từ mới
    pub backspace_restore: bool,
    /// Tự động chuyển Literal Mode khi từ dài hoặc nghi ngờ tiếng Anh
    pub smart_literal_mode: bool,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            vietnamese_mode: true,
            free_typing: false,
            modern_tone: false,
            clipboard_unicode: false,
            spell_check: false,
            auto_restore: false,
            show_feedback: true,
            beep_on_switch: false,
            allow_foreign_consonants: true,
            macro_enabled: false,
            shorthand_while_off: false,
            macro_auto_case: true,
            show_on_startup: false,
            start_with_windows: false,
            per_app_state: false,
            vietnamese_interface: true,
            output_charset: "Unicode".to_string(),
            auto_capitalize_sentence: false,
            auto_capitalize_enter: false,
            auto_switch_apps: vec![
                "League of Legends.exe".to_string(),
                "Valorant.exe".to_string(),
                "Counter-Strike 2.exe".to_string(),
                "Notepad.exe".to_string(), // For testing
            ],
            switch_shortcut: "Ctrl+Shift".to_string(),
            simulation_delay: 1,
            backspace_restore: true,
            smart_literal_mode: true,
        }
    }
}
