use std::sync::Mutex;
use std::thread;
use lazy_static::lazy_static;
use windows::Win32::Foundation::{HINSTANCE, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, GetKeyboardState, GetKeyState, MapVirtualKeyW, SendInput, ToUnicode, INPUT,
    INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, KEYEVENTF_UNICODE, VIRTUAL_KEY,
    VK_BACK, VK_ESCAPE, VK_LMENU, VK_RETURN, VK_RMENU, VK_Z,
    VK_LEFT, VK_UP, VK_RIGHT, VK_DOWN, VK_HOME, VK_END, VK_PRIOR, VK_NEXT, VK_DELETE,
    VK_LBUTTON, VK_RBUTTON, VK_MBUTTON, // Added mouse button imports
    VK_SHIFT, VK_CAPITAL, VK_CONTROL, VK_MENU, VK_LSHIFT, VK_RSHIFT, VK_LCONTROL, VK_RCONTROL,
    VK_LWIN, VK_RWIN
};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetMessageW, SetWindowsHookExW, UnhookWindowsHookEx, HHOOK, KBDLLHOOKSTRUCT,
    MSG, WH_KEYBOARD_LL, WH_MOUSE_LL, WM_KEYDOWN, WM_SYSKEYDOWN, WM_LBUTTONDOWN, WM_RBUTTONDOWN, WM_MBUTTONDOWN
};

use crate::{Engine, InputMode};

const LLKHF_INJECTED: u32 = 0x00000010;

lazy_static! {
    static ref GLOBAL_ENGINE: Mutex<Engine> = Mutex::new({
        let mut e = Engine::new(InputMode::Telex);
        let mut cfg = e.config().clone();
        cfg.backspace_restore = true; // RE-ENABLED for the "Dụ" feature request
        e.set_config(cfg);
        e
    });
    static ref CURRENT_WORD: Mutex<String> = Mutex::new(String::new());
    static ref HOOK_HANDLE: Mutex<Option<isize>> = Mutex::new(None);
    static ref MOUSE_HOOK_HANDLE: Mutex<Option<isize>> = Mutex::new(None);
    static ref TOGGLE_CALLBACK: Mutex<Option<extern "C" fn(bool)>> = Mutex::new(None);
}

pub fn set_toggle_callback(cb: extern "C" fn(bool)) {
    if let Ok(mut lock) = TOGGLE_CALLBACK.lock() {
        *lock = Some(cb);
    }
}

pub fn start_hook() {
    let handle = HOOK_HANDLE.lock().unwrap();
    if handle.is_some() { return; }

    thread::spawn(|| {
        unsafe {
            let h_instance = GetModuleHandleW(None).unwrap_or_default();
            let h_inst = HINSTANCE(h_instance.0);
            let kb_hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_hook_callback), Some(h_inst), 0);
            let ms_hook = SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_hook_callback), Some(h_inst), 0);
            if let Ok(h) = kb_hook {
                *HOOK_HANDLE.lock().unwrap() = Some(h.0 as isize);
            }
            if let Ok(m) = ms_hook {
                *MOUSE_HOOK_HANDLE.lock().unwrap() = Some(m.0 as isize);
            }
            let mut msg = MSG::default();
            while GetMessageW(&mut msg, None, 0, 0).into() {}
        }
    });
}

pub fn stop_hook() {
    let mut handle = HOOK_HANDLE.lock().unwrap();
    if let Some(h) = *handle {
        unsafe {
            let hook = HHOOK(h as *mut std::ffi::c_void);
            let _ = UnhookWindowsHookEx(hook);
        }
        *handle = None;
    }
    
    let mut mhandle = MOUSE_HOOK_HANDLE.lock().unwrap();
    if let Some(h) = *mhandle {
        unsafe {
            let hook = HHOOK(h as *mut std::ffi::c_void);
            let _ = UnhookWindowsHookEx(hook);
        }
        *mhandle = None;
    }
}

unsafe extern "system" fn mouse_hook_callback(n_code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    if n_code >= 0 && (w_param.0 as u32 == WM_LBUTTONDOWN || w_param.0 as u32 == WM_RBUTTONDOWN || w_param.0 as u32 == WM_MBUTTONDOWN) {
        if let Ok(mut engine) = GLOBAL_ENGINE.lock() { engine.reset(); }
        if let Ok(mut cw) = CURRENT_WORD.lock() { cw.clear(); }
    }
    CallNextHookEx(None, n_code, w_param, l_param)
}

pub fn update_global_engine<F>(updater: F) where F: FnOnce(&mut Engine) {
    if let Ok(mut engine) = GLOBAL_ENGINE.lock() { updater(&mut engine); }
}

unsafe extern "system" fn keyboard_hook_callback(n_code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    if n_code >= 0 && (w_param.0 as u32 == WM_KEYDOWN || w_param.0 as u32 == WM_SYSKEYDOWN) {
        let kbd_struct = *(l_param.0 as *const KBDLLHOOKSTRUCT);

        // Bỏ qua các phím do chính VNKey sinh ra để tránh vòng lặp vô hạn
        if (kbd_struct.flags.0 & LLKHF_INJECTED) != 0 {
            return CallNextHookEx(None, n_code, w_param, l_param);
        }

        let vk_code = kbd_struct.vkCode;

        // === EARLY MODIFIER BYPASS ===
        // Check Ctrl/Alt/Win state BEFORE any mutex locks or char conversion.
        // Modifier combos are NEVER Vietnamese input — safe to bypass 100%.
        let is_ctrl_down = is_key_down(VK_CONTROL.0 as i32);
        let is_alt_down = is_key_down(VK_LMENU.0 as i32) || is_key_down(VK_RMENU.0 as i32);
        let is_win_down = is_key_down(VK_LWIN.0 as i32) || is_key_down(VK_RWIN.0 as i32);
        let is_shift_down = is_key_down(VK_SHIFT.0 as i32);

        // Dynamic toggle shortcut: handle BEFORE bypass
        if is_shortcut_triggered(vk_code, is_ctrl_down, is_alt_down, is_shift_down, is_win_down) {
            let mut new_state = true;
            if let Ok(mut engine) = GLOBAL_ENGINE.lock() {
                let mut cfg = engine.config().clone();
                cfg.vietnamese_mode = !cfg.vietnamese_mode;
                new_state = cfg.vietnamese_mode;
                engine.set_config(cfg);
                engine.reset();
            }
            if let Ok(mut cw) = CURRENT_WORD.lock() { cw.clear(); }
            if let Ok(cb_lock) = TOGGLE_CALLBACK.lock() {
                if let Some(cb) = *cb_lock { cb(new_state); }
            }
            return LRESULT(1);
        }

        // Bypass ALL modifier combos (Ctrl+C, Ctrl+V, Alt+Tab, Win+D, etc.)
        // IMPORTANT: Reset engine because these actions likely change text/selection context!
        if is_ctrl_down || is_alt_down || is_win_down {
            if let Ok(mut engine) = GLOBAL_ENGINE.lock() { engine.reset(); }
            if let Ok(mut cw) = CURRENT_WORD.lock() { cw.clear(); }
            return CallNextHookEx(None, n_code, w_param, l_param);
        }

        // === END EARLY MODIFIER BYPASS ===

        let mut shorthand_active = false;
        let mut vietnamese_enabled = true;
        if let Ok(engine) = GLOBAL_ENGINE.lock() {
            vietnamese_enabled = engine.config().vietnamese_mode;
            shorthand_active = engine.config().macro_enabled && engine.config().shorthand_while_off;
        }

        if !vietnamese_enabled && !shorthand_active { return CallNextHookEx(None, n_code, w_param, l_param); }

        // Navigation keys, Enter, and Delete should just reset the engine state
        if vk_code == VK_LEFT.0 as u32 || vk_code == VK_RIGHT.0 as u32 || vk_code == VK_UP.0 as u32 || vk_code == VK_DOWN.0 as u32 ||
           vk_code == VK_HOME.0 as u32 || vk_code == VK_END.0 as u32 || vk_code == VK_PRIOR.0 as u32 || vk_code == VK_NEXT.0 as u32 ||
           vk_code == VK_DELETE.0 as u32 || vk_code == VK_ESCAPE.0 as u32 || vk_code == VK_RETURN.0 as u32 ||
           is_key_down(VK_LBUTTON.0 as i32) || is_key_down(VK_RBUTTON.0 as i32) || is_key_down(VK_MBUTTON.0 as i32) {
            
            if let Ok(mut engine) = GLOBAL_ENGINE.lock() { 
                if vk_code == VK_RETURN.0 as u32 {
                    let expansion = engine.on_enter();
                    if let Ok(mut cw) = CURRENT_WORD.lock() {
                        if !expansion.is_empty() && expansion != *cw {
                            // Shorthand expansion happened!
                            let backspaces = cw.chars().count();
                            unsafe { apply_changes_atomic(backspaces, &expansion); }
                        }
                        cw.clear();
                    }
                } else {
                    engine.reset(); 
                    if let Ok(mut cw) = CURRENT_WORD.lock() { cw.clear(); }
                }
            }
            return CallNextHookEx(None, n_code, w_param, l_param);
        }

        if vk_code == VK_BACK.0 as u32 {
            // Engine handles backspace internally
            let backspace_action = if let Ok(mut engine) = GLOBAL_ENGINE.lock() {
                let current_on_screen = if let Ok(cw) = CURRENT_WORD.lock() {
                    cw.clone()
                } else {
                    String::new()
                };

                let handled = engine.process_backspace();
                if handled {
                    let new_transformed = engine.get_state().transformed;
                    
                    // Logic to sync: compare current screen with new state
                    let common_prefix_len = current_on_screen.chars()
                        .zip(new_transformed.chars())
                        .take_while(|(a, b)| a == b)
                        .count();
                    
                    let backspaces_needed = current_on_screen.chars().count() - common_prefix_len;
                    let text_to_send: String = new_transformed.chars().skip(common_prefix_len).collect();
                    
                    Some((backspaces_needed, text_to_send, new_transformed))
                } else {
                    None
                }
            } else {
                None
            };

            if let Some((backspaces, text, new_cw)) = backspace_action {
                if let Ok(mut cw) = CURRENT_WORD.lock() {
                    *cw = new_cw;
                }
                unsafe { apply_changes_atomic(backspaces, &text); }
                return LRESULT(1); // Suppress original backspace
            } else {
                // If not handled by engine (e.g., buffer already empty), let Windows handle it normally
                if let Ok(mut cw) = CURRENT_WORD.lock() {
                    cw.clear();
                }
                // Also reset engine just in case it was in Literal Mode or had a dirty buffer
                if let Ok(mut engine) = GLOBAL_ENGINE.lock() {
                    engine.reset();
                }
                return CallNextHookEx(None, n_code, w_param, l_param);
            }
        }

        let ch = get_char_from_vk_code(vk_code);
        if ch != '\0' && !ch.is_control() {
            // Sequential lock: engine first — process key and get result
            let engine_result = if let Ok(mut engine) = GLOBAL_ENGINE.lock() {
                let is_separator = ch.is_whitespace() || (ch.is_ascii_punctuation() && !engine.is_special_punctuation(ch));
                let old_buffer_empty = engine.get_state().buffer.is_empty();
                
                let new_word = engine.process_key(ch);
                
                
                // NEW SENIOR LOGIC:
                // If buffer was empty and we type a NON-separator -> start of a new word -> reset CW sync
                let should_reset_sync = old_buffer_empty && !is_separator;
                
                // If we type a separator but the buffer was ALREADY empty -> outside any word -> reset CW sync
                let is_hard_reset = is_separator && old_buffer_empty;

                Some((new_word, should_reset_sync, is_hard_reset))
            } else {
                None
            };

            if let Some((new_word, should_reset_sync, is_hard_reset)) = engine_result {
                let send_action = if let Ok(mut cw) = CURRENT_WORD.lock() {
                    if should_reset_sync || is_hard_reset {
                        cw.clear();
                    }

                    let mut base_word = cw.clone();
                    base_word.push(ch);

                    let action = if new_word != base_word {
                        let common_prefix_len = cw.chars()
                            .zip(new_word.chars())
                            .take_while(|(a, b)| a == b)
                            .count();

                        let backspaces_needed = cw.chars().count() - common_prefix_len;
                        let text_to_send: String = new_word.chars().skip(common_prefix_len).collect();
                        Some((backspaces_needed, text_to_send))
                    } else {
                        None
                    };

                    // Update CURRENT_WORD with what's actually on screen now
                    if is_hard_reset {
                        cw.clear();
                    } else {
                        *cw = new_word;
                    }

                    action
                } else {
                    None
                };

                if let Some((backspaces, text)) = send_action {
                    unsafe { apply_changes_atomic(backspaces, &text); }
                    return LRESULT(1);
                }
            }
        }
    }
    CallNextHookEx(None, n_code, w_param, l_param)
}

fn is_shortcut_triggered(vk_code: u32, ctrl: bool, alt: bool, shift: bool, win: bool) -> bool {
    let shortcut = if let Ok(engine) = GLOBAL_ENGINE.lock() {
        engine.config().switch_shortcut.clone()
    } else {
        return false;
    };

    if shortcut.is_empty() { return false; }

    // Special case: Ctrl+Shift
    if shortcut == "Ctrl+Shift" {
        let is_shift_key = vk_code == VK_SHIFT.0 as u32 || vk_code == VK_LSHIFT.0 as u32 || vk_code == VK_RSHIFT.0 as u32;
        let is_ctrl_key = vk_code == VK_CONTROL.0 as u32 || vk_code == VK_LCONTROL.0 as u32 || vk_code == VK_RCONTROL.0 as u32;
        return (ctrl && is_shift_key) || (shift && is_ctrl_key);
    }

    // Generic case: Parsing "Ctrl+Alt+S"
    let parts: Vec<&str> = shortcut.split('+').collect();
    if parts.is_empty() { return false; }

    let mut req_ctrl = false;
    let mut req_alt = false;
    let mut req_shift = false;
    let mut req_win = false;
    let mut target_key = "";

    for part in parts {
        match part {
            "Ctrl" => req_ctrl = true,
            "Alt" => req_alt = true,
            "Shift" => req_shift = true,
            "Win" => req_win = true,
            _ => target_key = part,
        }
    }

    // Check modifiers first
    if ctrl != req_ctrl || alt != req_alt || shift != req_shift || win != req_win {
        return false;
    }

    // Check main key
    if target_key.is_empty() { return false; }

    // Map common key names to VK codes
    let vk_target = match target_key {
        "A" => 0x41, "B" => 0x42, "C" => 0x43, "D" => 0x44, "E" => 0x45, "F" => 0x46, "G" => 0x47, "H" => 0x48, "I" => 0x49,
        "J" => 0x4A, "K" => 0x4B, "L" => 0x4C, "M" => 0x4D, "N" => 0x4E, "O" => 0x4F, "P" => 0x50, "Q" => 0x51, "R" => 0x52,
        "S" => 0x53, "T" => 0x54, "U" => 0x55, "V" => 0x56, "W" => 0x57, "X" => 0x58, "Y" => 0x59, "Z" => 0x5A,
        "D0" => 0x30, "D1" => 0x31, "D2" => 0x32, "D3" => 0x33, "D4" => 0x34, "D5" => 0x35, "D6" => 0x36, "D7" => 0x37, "D8" => 0x38, "D9" => 0x39,
        "F1" => 0x70, "F2" => 0x71, "F3" => 0x72, "F4" => 0x73, "F5" => 0x74, "F6" => 0x75, "F7" => 0x76, "F8" => 0x77, "F9" => 0x78, "F10" => 0x79, "F11" => 0x7A, "F12" => 0x7B,
        "Space" => 0x20, "Return" => 0x0D, "Tab" => 0x09, "Escape" => 0x1B, "Back" => 0x08, "Delete" => 0x2E,
        _ => 0,
    };

    vk_code == vk_target as u32
}

fn is_key_down(vk: i32) -> bool {
    unsafe { (GetAsyncKeyState(vk) as i16) < 0 }
}

fn get_char_from_vk_code(vk_code: u32) -> char {
    unsafe {
        let mut keyboard_state = [0u8; 256];
        let _ = GetKeyboardState(&mut keyboard_state);

        // Khắc phục lỗi Thread Background không lấy được trạng thái Modifier (Shift/Caps) của Thread Active
        let mut sync_key = |vk: u16| {
            let state = GetKeyState(vk as i32) as u16;
            keyboard_state[vk as usize] = ((state & 0x8000) >> 8) as u8 | (state & 0x0001) as u8;
        };
        sync_key(VK_SHIFT.0);
        sync_key(VK_CAPITAL.0);
        sync_key(VK_CONTROL.0);
        sync_key(VK_MENU.0);
        sync_key(VK_LSHIFT.0);
        sync_key(VK_RSHIFT.0);

        // Map vk_code to scan_code
        let scan_code = MapVirtualKeyW(vk_code, Default::default());

        let mut buffer = [0u16; 5];
        // flag 4 avoids destroying the dead-key state
        let result = ToUnicode(
            vk_code,
            scan_code,
            Some(&keyboard_state),
            &mut buffer,
            4,
        );

        if result > 0 {
            if let Some(ch) = std::char::from_u32(buffer[0] as u32) {
                return ch;
            }
        }
        '\0'
    }
}

unsafe fn apply_changes_atomic(backspaces: usize, text: &str) {
    let mut inputs = Vec::with_capacity(backspaces * 2 + text.encode_utf16().count() * 2);

    // 1. Queue all backspaces
    for _ in 0..backspaces {
        let mut down = INPUT::default();
        down.r#type = INPUT_KEYBOARD;
        down.Anonymous.ki = KEYBDINPUT { wVk: VIRTUAL_KEY(VK_BACK.0), wScan: 0, dwFlags: Default::default(), time: 0, dwExtraInfo: 0 };
        inputs.push(down);

        let mut up = INPUT::default();
        up.r#type = INPUT_KEYBOARD;
        up.Anonymous.ki = KEYBDINPUT { wVk: VIRTUAL_KEY(VK_BACK.0), wScan: 0, dwFlags: KEYEVENTF_KEYUP, time: 0, dwExtraInfo: 0 };
        inputs.push(up);
    }

    // 2. Queue all characters
    for c in text.encode_utf16() {
        let mut down = INPUT::default();
        down.r#type = INPUT_KEYBOARD;
        down.Anonymous.ki = KEYBDINPUT { wVk: VIRTUAL_KEY(0), wScan: c, dwFlags: KEYEVENTF_UNICODE, time: 0, dwExtraInfo: 0 };
        inputs.push(down);

        let mut up = INPUT::default();
        up.r#type = INPUT_KEYBOARD;
        up.Anonymous.ki = KEYBDINPUT { wVk: VIRTUAL_KEY(0), wScan: c, dwFlags: KEYEVENTF_UNICODE | KEYEVENTF_KEYUP, time: 0, dwExtraInfo: 0 };
        inputs.push(up);
    }

    // 3. Send all in one atomic batch!
    if !inputs.is_empty() {
        let _ = SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
    }
}
