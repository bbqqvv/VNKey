use crate::{Engine, InputMode};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

/// Helper to convert C string to rust string
unsafe fn cstr_to_string(c_str: *const c_char) -> String {
    if c_str.is_null() {
        return String::new();
    }
    CStr::from_ptr(c_str).to_string_lossy().into_owned()
}

/// Helper to convert rust string to C string pointer
/// The caller (C#) MUST free this pointer using `vnkey_free_string`
fn string_to_cstr(s: String) -> *mut c_char {
    CString::new(s)
        .unwrap_or_else(|_| CString::new("").unwrap())
        .into_raw()
}

#[no_mangle]
pub extern "C" fn vnkey_engine_new(mode: u8) -> *mut Engine {
    let input_mode = match mode {
        0 => InputMode::Telex,
        1 => InputMode::Vni,
        2 => InputMode::Viqr,
        3 => InputMode::TelexEx,
        _ => InputMode::Telex,
    };

    let engine = Box::new(Engine::new(input_mode));
    Box::into_raw(engine)
}

/// Frees the Engine instance.
///
/// # Safety
///
/// The pointer `ptr` must be a valid pointer to an `Engine` allocated by `vnkey_engine_new`.
/// Calling this multiple times on the same pointer or with an invalid pointer is undefined behavior.
#[no_mangle]
pub unsafe extern "C" fn vnkey_engine_free(ptr: *mut Engine) {
    if !ptr.is_null() {
        drop(Box::from_raw(ptr));
    }
}

/// Processes a single character and returns the newly transformed word.
/// String returned must be freed by C#.
///
/// # Safety
///
/// `ptr` must be a valid pointer to an `Engine`.
#[no_mangle]
pub unsafe extern "C" fn vnkey_process_key(ptr: *mut Engine, c: u32) -> *mut c_char {
    if ptr.is_null() {
        return string_to_cstr(String::new());
    }

    let engine = &mut *ptr;
    let ch = match std::char::from_u32(c) {
        Some(char_val) => char_val,
        None => return string_to_cstr(String::new()),
    };

    let result = engine.process_key(ch);
    string_to_cstr(result)
}

/// Feed an entire string (e.g., from clipboard or fast typing lag repair)
///
/// # Safety
///
/// `ptr` must be a valid pointer to an `Engine`.
/// `input` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn vnkey_feed_str(ptr: *mut Engine, input: *const c_char) -> *mut c_char {
    if ptr.is_null() || input.is_null() {
        return string_to_cstr(String::new());
    }

    let engine = &mut *ptr;
    let s = cstr_to_string(input);
    let result = engine.feed_str(&s);
    string_to_cstr(result)
}

/// Resets the engine state.
///
/// # Safety
///
/// `ptr` must be a valid pointer to an `Engine`.
#[no_mangle]
pub unsafe extern "C" fn vnkey_reset(ptr: *mut Engine) {
    if !ptr.is_null() {
        let engine = &mut *ptr;
        engine.reset();
    }
}

/// Processes a Backspace key.
/// Returns true if the engine handled it internally.
///
/// # Safety
///
/// `ptr` must be a valid pointer to an `Engine`.
#[no_mangle]
pub unsafe extern "C" fn vnkey_process_backspace(ptr: *mut Engine) -> bool {
    if ptr.is_null() {
        return false;
    }
    let engine = &mut *ptr;
    engine.process_backspace()
}

/// Sets the input mode.
///
/// # Safety
///
/// `ptr` must be a valid pointer to an `Engine`.
#[no_mangle]
pub unsafe extern "C" fn vnkey_set_mode(ptr: *mut Engine, mode: u8) {
    if !ptr.is_null() {
        let input_mode = match mode {
            0 => InputMode::Telex,
            1 => InputMode::Vni,
            2 => InputMode::Viqr,
            3 => InputMode::TelexEx,
            _ => InputMode::Telex,
        };
        (*ptr).set_mode(input_mode);
    }
}

/// Sets the Vietnamese mode.
///
/// # Safety
///
/// `ptr` must be a valid pointer to an `Engine`.
#[no_mangle]
pub unsafe extern "C" fn vnkey_set_vietnamese_mode(ptr: *mut Engine, enabled: bool) {
    if !ptr.is_null() {
        let mut cfg = (*ptr).config().clone();
        cfg.vietnamese_mode = enabled;
        (*ptr).set_config(cfg);
    }
}

/// Gets the diagnostic info as a JSON string.
///
/// # Safety
///
/// `ptr` must be a valid pointer to an `Engine`.
#[no_mangle]
pub unsafe extern "C" fn vnkey_get_diagnostic_info(ptr: *mut Engine) -> *mut c_char {
    if ptr.is_null() {
        return string_to_cstr(String::new());
    }
    let engine = &*ptr;
    let info = engine.get_diagnostic_info();
    let json = serde_json::to_string(&info).unwrap_or_else(|_| String::new());
    string_to_cstr(json)
}

/// Free strings allocated by Rust
///
/// # Safety
///
/// `s` must be a valid pointer to a C string allocated by `string_to_cstr`.
#[no_mangle]
pub unsafe extern "C" fn vnkey_free_string(s: *mut c_char) {
    if !s.is_null() {
        drop(CString::from_raw(s));
    }
}

// --- Native Hook API (Windows only) ---
#[no_mangle]
#[cfg(windows)]
pub extern "C" fn vnkey_hook_start() {
    crate::hook::start_hook();
}

#[no_mangle]
#[cfg(windows)]
pub extern "C" fn vnkey_hook_stop() {
    crate::hook::stop_hook();
}

/// Sets the global engine mode.
///
/// # Safety
///
/// This function interacts with global state.
#[no_mangle]
#[cfg(windows)]
pub unsafe extern "C" fn vnkey_global_set_mode(mode: u8) {
    let input_mode = match mode {
        0 => InputMode::Telex,
        1 => InputMode::Vni,
        2 => InputMode::Viqr,
        3 => InputMode::TelexEx,
        _ => InputMode::Telex,
    };
    crate::hook::update_global_engine(|e| e.set_mode(input_mode));
}

/// Sets the global Vietnamese mode.
///
/// # Safety
///
/// This function interacts with global state.
#[no_mangle]
#[cfg(windows)]
pub unsafe extern "C" fn vnkey_global_set_vietnamese_mode(enabled: bool) {
    crate::hook::update_global_engine(|e| {
        let mut cfg = e.config().clone();
        cfg.vietnamese_mode = enabled;
        e.set_config(cfg);
    });
}

#[no_mangle]
#[cfg(windows)]
pub extern "C" fn vnkey_set_toggle_callback(cb: extern "C" fn(bool)) {
    crate::hook::set_toggle_callback(cb);
}

/// Sets the global engine configuration.
///
/// # Safety
///
/// `json` must be a valid null-terminated C string.
#[no_mangle]
#[cfg(windows)]
pub unsafe extern "C" fn vnkey_global_set_config_json(json: *const c_char) {
    let s = cstr_to_string(json);
    if let Ok(config) = serde_json::from_str::<crate::config::EngineConfig>(&s) {
        crate::hook::update_global_engine(|e| e.set_config(config));
    }
}

/// Sets the global shorthand configuration.
///
/// # Safety
///
/// `json` must be a valid null-terminated C string.
#[no_mangle]
#[cfg(windows)]
pub unsafe extern "C" fn vnkey_global_set_shorthand_json(json: *const c_char) {
    let s = cstr_to_string(json);
    if let Ok(macros) = serde_json::from_str::<std::collections::HashMap<String, String>>(&s) {
        crate::hook::update_global_engine(|e| e.set_macros(macros));
    }
}

/// Loads a dictionary for the global engine.
///
/// # Safety
///
/// `path` must be a valid null-terminated C string.
#[no_mangle]
#[cfg(windows)]
pub unsafe extern "C" fn vnkey_global_load_dictionary(path: *const c_char) {
    let s = cstr_to_string(path);
    crate::hook::update_global_engine(|e| {
        let _ = e.load_dictionary(&s);
    });
}

/// Processes a backspace for the global engine.
///
/// # Safety
///
/// This function interacts with global state.
#[no_mangle]
#[cfg(windows)]
pub unsafe extern "C" fn vnkey_global_process_backspace() -> bool {
    let mut handled = false;
    crate::hook::update_global_engine(|e| {
        handled = e.process_backspace();
    });
    handled
}

/// Gets diagnostic info for the global engine.
///
/// # Safety
///
/// This function interacts with global state.
#[no_mangle]
#[cfg(windows)]
pub unsafe extern "C" fn vnkey_global_get_diagnostic_info() -> *mut c_char {
    let mut json = String::new();
    crate::hook::update_global_engine(|e| {
        let info = e.get_diagnostic_info();
        json = serde_json::to_string(&info).unwrap_or_else(|_| String::new());
    });
    string_to_cstr(json)
}
