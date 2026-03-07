use lazy_static::lazy_static;
use std::sync::Mutex;
use std::thread;
use windows::Win32::Foundation::{HINSTANCE, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::Diagnostics::Debug::Beep;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState,
    GetKeyState,
    GetKeyboardState,
    MapVirtualKeyW,
    SendInput,
    ToUnicode,
    INPUT,
    INPUT_KEYBOARD,
    KEYBDINPUT,
    KEYEVENTF_KEYUP,
    KEYEVENTF_UNICODE,
    VIRTUAL_KEY,
    VK_BACK,
    VK_CAPITAL,
    VK_CONTROL,
    VK_DELETE,
    VK_DOWN,
    VK_END,
    VK_ESCAPE,
    VK_HOME,
    VK_LBUTTON,
    VK_LCONTROL,
    VK_LEFT,
    VK_LMENU,
    VK_LSHIFT,
    VK_LWIN,
    VK_MBUTTON, // Added mouse button imports
    VK_MENU,
    VK_NEXT,
    VK_PRIOR,
    VK_RBUTTON,
    VK_RCONTROL,
    VK_RETURN,
    VK_RIGHT,
    VK_RMENU,
    VK_RSHIFT,
    VK_RWIN,
    VK_SHIFT,
    VK_UP,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetMessageW, SetWindowsHookExW, UnhookWindowsHookEx, HHOOK, KBDLLHOOKSTRUCT,
    MSG, WH_KEYBOARD_LL, WH_MOUSE_LL, WM_KEYDOWN, WM_LBUTTONDOWN, WM_MBUTTONDOWN, WM_RBUTTONDOWN,
    WM_SYSKEYDOWN,
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
    static ref RAW_HISTORY: Mutex<String> = Mutex::new(String::new());
    static ref RAW_MATCH_FOUND: Mutex<bool> = Mutex::new(false);
    static ref CURRENT_WORD: Mutex<String> = Mutex::new(String::new());
    static ref HOOK_HANDLE: Mutex<Option<isize>> = Mutex::new(None);
    static ref MOUSE_HOOK_HANDLE: Mutex<Option<isize>> = Mutex::new(None);
    static ref TOGGLE_CALLBACK: Mutex<Option<extern "C" fn(bool)>> = Mutex::new(None);
}

pub fn check_and_reset_raw_match() -> bool {
    if let Ok(mut lock) = RAW_MATCH_FOUND.lock() {
        let val = *lock;
        *lock = false;
        val
    } else {
        false
    }
}

pub fn get_raw_history() -> String {
    RAW_HISTORY.lock().map(|h| h.clone()).unwrap_or_default()
}

pub fn set_toggle_callback(cb: extern "C" fn(bool)) {
    if let Ok(mut lock) = TOGGLE_CALLBACK.lock() {
        *lock = Some(cb);
    }
}

pub fn start_hook() {
    let handle = HOOK_HANDLE.lock().unwrap();
    if handle.is_some() {
        return;
    }

    thread::spawn(|| unsafe {
        let h_instance = GetModuleHandleW(None).unwrap_or_default();
        let h_inst = HINSTANCE(h_instance.0);
        let kb_hook = SetWindowsHookExW(
            WH_KEYBOARD_LL,
            Some(keyboard_hook_callback),
            Some(h_inst),
            0,
        );
        let ms_hook = SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_hook_callback), Some(h_inst), 0);
        if let Ok(h) = kb_hook {
            *HOOK_HANDLE.lock().unwrap() = Some(h.0 as isize);
        }
        if let Ok(m) = ms_hook {
            *MOUSE_HOOK_HANDLE.lock().unwrap() = Some(m.0 as isize);
        }
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).into() {}
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

unsafe extern "system" fn mouse_hook_callback(
    n_code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if n_code >= 0
        && (w_param.0 as u32 == WM_LBUTTONDOWN
            || w_param.0 as u32 == WM_RBUTTONDOWN
            || w_param.0 as u32 == WM_MBUTTONDOWN)
    {
        if let Ok(mut engine) = GLOBAL_ENGINE.lock() {
            engine.reset();
        }
        if let Ok(mut cw) = CURRENT_WORD.lock() {
            cw.clear();
        }
    }
    CallNextHookEx(None, n_code, w_param, l_param)
}

pub fn update_global_engine<F>(updater: F)
where
    F: FnOnce(&mut Engine),
{
    if let Ok(mut engine) = GLOBAL_ENGINE.lock() {
        updater(&mut engine);
    }
}

unsafe extern "system" fn keyboard_hook_callback(
    n_code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if n_code >= 0 && (w_param.0 as u32 == WM_KEYDOWN || w_param.0 as u32 == WM_SYSKEYDOWN) {
        let kbd_struct = *(l_param.0 as *const KBDLLHOOKSTRUCT);

        // Bỏ qua các phím do chính VNKey sinh ra để tránh vòng lặp vô hạn
        if (kbd_struct.flags.0 & LLKHF_INJECTED) != 0 {
            return CallNextHookEx(None, n_code, w_param, l_param);
        }

        let vk_code = kbd_struct.vkCode;

        // === EARLY MODIFIER BYPASS ===
        let is_ctrl_down = is_key_down(VK_CONTROL.0 as i32);
        let is_alt_down = is_key_down(VK_LMENU.0 as i32) || is_key_down(VK_RMENU.0 as i32);
        let is_win_down = is_key_down(VK_LWIN.0 as i32) || is_key_down(VK_RWIN.0 as i32);
        let is_shift_down = is_key_down(VK_SHIFT.0 as i32);

        // Dynamic toggle shortcut
        if is_shortcut_triggered(
            vk_code,
            is_ctrl_down,
            is_alt_down,
            is_shift_down,
            is_win_down,
        ) {
            let mut new_state = true;
            if let Ok(mut engine) = GLOBAL_ENGINE.lock() {
                let mut cfg = engine.config().clone();
                cfg.vietnamese_mode = !cfg.vietnamese_mode;
                new_state = cfg.vietnamese_mode;
                engine.set_config(cfg);
                engine.reset();
                engine.last_raw_key = format!("VK_{:02X}", vk_code);
                engine.last_action = "Toggled Mode".to_string();
            }
            if let Ok(mut cw) = CURRENT_WORD.lock() {
                cw.clear();
            }
            if let Ok(cb_lock) = TOGGLE_CALLBACK.lock() {
                if let Some(cb) = *cb_lock {
                    cb(new_state);
                }
            }
            return LRESULT(1);
        }

        // Bypass logic: Only bypass if it's a real systemic shortcut (Ctrl+..., Alt+..., Win+...)
        // We exclude Shift-only because it's used for capitalization.
        // We handle AltGr (LControl + RMenu) carefully for international layouts.
        let is_modifier_key = vk_code == VK_CONTROL.0 as u32
            || vk_code == VK_LCONTROL.0 as u32
            || vk_code == VK_RCONTROL.0 as u32
            || vk_code == VK_MENU.0 as u32
            || vk_code == VK_LMENU.0 as u32
            || vk_code == VK_RMENU.0 as u32
            || vk_code == VK_LWIN.0 as u32
            || vk_code == VK_RWIN.0 as u32;

        if is_ctrl_down || is_alt_down || is_win_down {
            // If it's just the modifier key itself being pressed/released, we just reset state and let it pass through.
            if is_modifier_key {
                if let Ok(mut engine) = GLOBAL_ENGINE.lock() {
                    engine.last_raw_key = format!("Modifier (VK_{:02X})", vk_code);
                    engine.last_action = "Bypassed (System)".to_string();
                    engine.reset();
                }
                if let Ok(mut cw) = CURRENT_WORD.lock() {
                    cw.clear();
                }
                return CallNextHookEx(None, n_code, w_param, l_param);
            }

            // If it's a regular key while a modifier is down, it's a combo (e.g. Ctrl+C).
            // We bypass these but log them clearly.
            if let Ok(mut engine) = GLOBAL_ENGINE.lock() {
                engine.last_raw_key = format!("Combo (VK_{:02X})", vk_code);
                engine.last_action = "Bypassed (Combo)".to_string();
                engine.reset();
            }
            if let Ok(mut cw) = CURRENT_WORD.lock() {
                cw.clear();
            }
            return CallNextHookEx(None, n_code, w_param, l_param);
        }

        // Navigation keys, Enter, and Delete
        if vk_code == VK_LEFT.0 as u32
            || vk_code == VK_RIGHT.0 as u32
            || vk_code == VK_UP.0 as u32
            || vk_code == VK_DOWN.0 as u32
            || vk_code == VK_HOME.0 as u32
            || vk_code == VK_END.0 as u32
            || vk_code == VK_PRIOR.0 as u32
            || vk_code == VK_NEXT.0 as u32
            || vk_code == VK_DELETE.0 as u32
            || vk_code == VK_ESCAPE.0 as u32
            || vk_code == VK_RETURN.0 as u32
            || is_key_down(VK_LBUTTON.0 as i32)
            || is_key_down(VK_RBUTTON.0 as i32)
            || is_key_down(VK_MBUTTON.0 as i32)
        {
            if let Ok(mut engine) = GLOBAL_ENGINE.lock() {
                engine.last_raw_key = format!("VK_{:02X}", vk_code);
                if vk_code == VK_RETURN.0 as u32 {
                    let expansion = engine.on_enter();
                    if let Ok(mut cw) = CURRENT_WORD.lock() {
                        if !expansion.is_empty() && expansion != *cw {
                            let backspaces = cw.chars().count();
                            unsafe {
                                apply_changes_atomic(backspaces, &expansion);
                            }
                            engine.last_action = "Shorthand Expanded".to_string();
                        } else {
                            engine.last_action = "Reset (Enter)".to_string();
                        }
                        cw.clear();
                    } else {
                        engine.last_action = "Reset (Enter)".to_string();
                    }
                } else {
                    engine.last_action = "Reset (Nav/Mouse)".to_string();
                    engine.reset();
                    if let Ok(mut cw) = CURRENT_WORD.lock() {
                        cw.clear();
                    }
                }
            }
            return CallNextHookEx(None, n_code, w_param, l_param);
        }

        // Backspace handling
        if vk_code == VK_BACK.0 as u32 {
            if let Ok(mut engine) = GLOBAL_ENGINE.lock() {
                engine.last_raw_key = "VK_BACK".to_string();
                let current_on_screen = if let Ok(cw) = CURRENT_WORD.lock() {
                    cw.clone()
                } else {
                    String::new()
                };
                if engine.process_backspace() {
                    engine.last_action = "Absorbed (Engine)".to_string();
                    let new_transformed = engine.get_state().transformed;
                    let prefix = get_common_prefix(&current_on_screen, &new_transformed);
                    let backspaces = current_on_screen.chars().count() - prefix.chars().count();
                    let text = new_transformed
                        .chars()
                        .skip(prefix.chars().count())
                        .collect::<String>();
                    if let Ok(mut cw) = CURRENT_WORD.lock() {
                        *cw = new_transformed;
                    }
                    unsafe {
                        apply_changes_atomic(backspaces, &text);
                    }
                    return LRESULT(1);
                } else {
                    engine.last_action = "Passed (Normal)".to_string();
                    if let Ok(mut cw) = CURRENT_WORD.lock() {
                        cw.clear();
                    }
                }
            }
            return CallNextHookEx(None, n_code, w_param, l_param);
        }

        let ch = get_char_from_vk_code(vk_code);
        if ch != '\0' {
            // Raw history tracking for Dev Mode password
            if let Ok(mut history) = RAW_HISTORY.lock() {
                history.push(ch);
                if history.len() > 30 {
                    history.remove(0);
                }
                if history.to_lowercase().contains("vnkdev") {
                    if let Ok(mut match_lock) = RAW_MATCH_FOUND.lock() {
                        *match_lock = true;
                    }
                    history.clear();
                }
            }

            let mut absorbed = false;
            let mut engine_result = None;
            if let Ok(mut engine) = GLOBAL_ENGINE.lock() {
                engine.last_raw_key = format!("'{}' (VK_{:02X})", ch, vk_code);
                let is_separator = ch.is_whitespace()
                    || (ch.is_ascii_punctuation() && !engine.is_special_punctuation(ch));
                let old_buffer_empty = engine.get_state().buffer.is_empty();

                let new_word = engine.process_key(ch);
                if new_word != format!("{}{}", engine.get_state().buffer, ch) {
                    absorbed = true;
                    engine.last_action = "Absorbed (Processed)".to_string();
                } else {
                    engine.last_action = "Passed (Literal)".to_string();
                }

                let should_reset_sync = old_buffer_empty && !is_separator;
                let is_hard_reset = is_separator && old_buffer_empty;
                engine_result = Some((new_word, should_reset_sync, is_hard_reset, absorbed));
            }

            if let Some((new_word, should_reset_sync, is_hard_reset, absorbed)) = engine_result {
                let mut send_action = None;
                if let Ok(mut cw) = CURRENT_WORD.lock() {
                    if should_reset_sync || is_hard_reset {
                        cw.clear();
                    }
                    let mut base_word = cw.clone();
                    base_word.push(ch);
                    if new_word != base_word {
                        let prefix = get_common_prefix(&*cw, &new_word);
                        let backspaces = cw.chars().count() - prefix.chars().count();
                        let text = new_word
                            .chars()
                            .skip(prefix.chars().count())
                            .collect::<String>();
                        send_action = Some((backspaces, text));
                    }
                    if is_hard_reset {
                        cw.clear();
                    } else {
                        *cw = new_word;
                    }
                }

                if let Some((backspaces, text)) = send_action {
                    unsafe {
                        apply_changes_atomic(backspaces, &text);
                    }
                    return LRESULT(1);
                }
            }
        }
    }
    CallNextHookEx(None, n_code, w_param, l_param)
}

fn get_common_prefix(s1: &str, s2: &str) -> String {
    s1.chars()
        .zip(s2.chars())
        .take_while(|(a, b)| a == b)
        .map(|(a, _)| a)
        .collect()
}

fn is_shortcut_triggered(vk_code: u32, ctrl: bool, alt: bool, shift: bool, win: bool) -> bool {
    let shortcut = if let Ok(engine) = GLOBAL_ENGINE.lock() {
        engine.config().switch_shortcut.clone()
    } else {
        return false;
    };

    if shortcut.is_empty() {
        return false;
    }

    // Special case: Ctrl+Shift
    if shortcut == "Ctrl+Shift" {
        let is_shift_key = vk_code == VK_SHIFT.0 as u32
            || vk_code == VK_LSHIFT.0 as u32
            || vk_code == VK_RSHIFT.0 as u32;
        let is_ctrl_key = vk_code == VK_CONTROL.0 as u32
            || vk_code == VK_LCONTROL.0 as u32
            || vk_code == VK_RCONTROL.0 as u32;
        return (ctrl && is_shift_key) || (shift && is_ctrl_key);
    }

    // Generic case: Parsing "Ctrl+Alt+S"
    let parts: Vec<&str> = shortcut.split('+').collect();
    if parts.is_empty() {
        return false;
    }

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
    if target_key.is_empty() {
        return false;
    }

    // Map common key names to VK codes
    let vk_target = match target_key {
        "A" => 0x41,
        "B" => 0x42,
        "C" => 0x43,
        "D" => 0x44,
        "E" => 0x45,
        "F" => 0x46,
        "G" => 0x47,
        "H" => 0x48,
        "I" => 0x49,
        "J" => 0x4A,
        "K" => 0x4B,
        "L" => 0x4C,
        "M" => 0x4D,
        "N" => 0x4E,
        "O" => 0x4F,
        "P" => 0x50,
        "Q" => 0x51,
        "R" => 0x52,
        "S" => 0x53,
        "T" => 0x54,
        "U" => 0x55,
        "V" => 0x56,
        "W" => 0x57,
        "X" => 0x58,
        "Y" => 0x59,
        "Z" => 0x5A,
        "D0" => 0x30,
        "D1" => 0x31,
        "D2" => 0x32,
        "D3" => 0x33,
        "D4" => 0x34,
        "D5" => 0x35,
        "D6" => 0x36,
        "D7" => 0x37,
        "D8" => 0x38,
        "D9" => 0x39,
        "F1" => 0x70,
        "F2" => 0x71,
        "F3" => 0x72,
        "F4" => 0x73,
        "F5" => 0x74,
        "F6" => 0x75,
        "F7" => 0x76,
        "F8" => 0x77,
        "F9" => 0x78,
        "F10" => 0x79,
        "F11" => 0x7A,
        "F12" => 0x7B,
        "Space" => 0x20,
        "Return" => 0x0D,
        "Tab" => 0x09,
        "Escape" => 0x1B,
        "Back" => 0x08,
        "Delete" => 0x2E,
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
        let result = ToUnicode(vk_code, scan_code, Some(&keyboard_state), &mut buffer, 4);

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
        down.Anonymous.ki = KEYBDINPUT {
            wVk: VIRTUAL_KEY(VK_BACK.0),
            wScan: 0,
            dwFlags: Default::default(),
            time: 0,
            dwExtraInfo: 0,
        };
        inputs.push(down);

        let mut up = INPUT::default();
        up.r#type = INPUT_KEYBOARD;
        up.Anonymous.ki = KEYBDINPUT {
            wVk: VIRTUAL_KEY(VK_BACK.0),
            wScan: 0,
            dwFlags: KEYEVENTF_KEYUP,
            time: 0,
            dwExtraInfo: 0,
        };
        inputs.push(up);
    }

    // 2. Queue all characters
    for c in text.encode_utf16() {
        let mut down = INPUT::default();
        down.r#type = INPUT_KEYBOARD;
        down.Anonymous.ki = KEYBDINPUT {
            wVk: VIRTUAL_KEY(0),
            wScan: c,
            dwFlags: KEYEVENTF_UNICODE,
            time: 0,
            dwExtraInfo: 0,
        };
        inputs.push(down);

        let mut up = INPUT::default();
        up.r#type = INPUT_KEYBOARD;
        up.Anonymous.ki = KEYBDINPUT {
            wVk: VIRTUAL_KEY(0),
            wScan: c,
            dwFlags: KEYEVENTF_UNICODE | KEYEVENTF_KEYUP,
            time: 0,
            dwExtraInfo: 0,
        };
        inputs.push(up);
    }

    // 3. Send all in one atomic batch!
    if !inputs.is_empty() {
        let _ = SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
    }
}
