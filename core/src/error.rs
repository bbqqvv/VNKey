use thiserror::Error;

/// Danh sách các lỗi có thể xảy ra trong Engine
/// P5 FIX: Uses thiserror derive macro per rust-pro skill recommendation
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum EngineError {
    /// Lỗi khi cấu hình không hợp lệ
    #[error("Cấu hình không hợp lệ: {0}")]
    InvalidConfig(String),
    /// Lỗi khi âm tiết không thể phân tích (thường do đầu vào rác)
    #[error("Âm tiết không hợp lệ: {0}")]
    InvalidSyllable(String),
    /// Lỗi khi tra cứu gõ tắt thất bại
    #[error("Lỗi gõ tắt: {0}")]
    ShorthandLookupError(String),
    /// Lỗi giới hạn bộ nhớ (buffer quá dài)
    #[error("Tràn bộ nhớ đệm Engine")]
    BufferOverflow,
}

/// Result type tùy chỉnh cho Engine
pub type EngineResult<T> = Result<T, EngineError>;

