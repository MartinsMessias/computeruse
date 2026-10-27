use crate::error::{CuError, Result};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, KEYEVENTF_UNICODE,
    VIRTUAL_KEY,
};

fn send_key(inp: INPUT) -> Result<()> {
    let _ = crate::desktop::attach_thread_to_input_desktop();
    unsafe {
        let sent = SendInput(&[inp], std::mem::size_of::<INPUT>() as i32);
        if sent == 0 {
            let last_err = windows::Win32::Foundation::GetLastError();
            return Err(CuError::Windows(format!("SendInput failed: {:?}", last_err)));
        }
    }
    Ok(())
}

pub fn make_keyboard_input(vk: u16, scan: u16, flags: u32) -> INPUT {
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        MapVirtualKeyW, KEYEVENTF_EXTENDEDKEY, MAPVK_VK_TO_VSC,
    };
    let mut final_scan = scan;
    let mut final_flags = flags;
    if vk != 0 && final_scan == 0 {
        final_scan = unsafe { MapVirtualKeyW(vk as u32, MAPVK_VK_TO_VSC) as u16 };
        // Navigation, Insert/Delete, Divide, and Win keys require KEYEVENTF_EXTENDEDKEY for DirectInput / RawInput apps
        if matches!(
            vk,
            0x21..=0x28 | 0x2D | 0x2E | 0x5B | 0x5C | 0x5D | 0x6F | 0x90 | 0xA3 | 0xA5
        ) {
            final_flags |= KEYEVENTF_EXTENDEDKEY.0;
        }
    }
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(vk),
                wScan: final_scan,
                dwFlags: windows::Win32::UI::Input::KeyboardAndMouse::KEYBD_EVENT_FLAGS(final_flags),
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

/// Resolve a key name to a virtual key code. Returns 0 for unicode fallback (handled separately).
pub fn resolve_vk(name: &str) -> Option<u16> {
    let n = name.trim().to_lowercase();
    // single char letters/digits
    if n.len() == 1 {
        let c = n.chars().next().unwrap();
        if c.is_ascii_alphabetic() {
            return Some(c.to_ascii_uppercase() as u16);
        }
        if c.is_ascii_digit() {
            return Some(c as u16);
        }
    }
    match n.as_str() {
        "enter" | "return" => Some(0x0D),
        "tab" => Some(0x09),
        "escape" | "esc" => Some(0x1B),
        "space" | "spacebar" => Some(0x20),
        "backspace" | "bs" => Some(0x08),
        "delete" | "del" => Some(0x2E),
        "insert" | "ins" => Some(0x2D),
        "home" => Some(0x24),
        "end" => Some(0x23),
        "pageup" | "pgup" => Some(0x21),
        "pagedown" | "pgdn" | "pgdown" => Some(0x22),
        "up" | "uparrow" => Some(0x26),
        "down" | "downarrow" => Some(0x28),
        "left" | "leftarrow" => Some(0x25),
        "right" | "rightarrow" => Some(0x27),
        "ctrl" | "control" => Some(0x11),
        "alt" => Some(0x12),
        "shift" => Some(0x10),
        "win" | "windows" | "super" | "meta" => Some(0x5B),
        "capslock" => Some(0x14),
        "numlock" => Some(0x90),
        "scrolllock" => Some(0x91),
        "printscreen" | "prtsc" => Some(0x2C),
        "pause" => Some(0x13),
        "f1" => Some(0x70),
        "f2" => Some(0x71),
        "f3" => Some(0x72),
        "f4" => Some(0x73),
        "f5" => Some(0x74),
        "f6" => Some(0x75),
        "f7" => Some(0x76),
        "f8" => Some(0x77),
        "f9" => Some(0x78),
        "f10" => Some(0x79),
        "f11" => Some(0x7A),
        "f12" => Some(0x7B),
        "multiply" | "numpad*" => Some(0x6A),
        "add" | "numpad+" => Some(0x6B),
        "subtract" | "numpad-" => Some(0x6D),
        "divide" | "numpad/" => Some(0x6F),
        "volumeup" | "vol_up" | "volume_up" => Some(0xAF),
        "volumedown" | "vol_down" | "volume_down" => Some(0xAE),
        "volumemute" | "vol_mute" | "volume_mute" | "mute" => Some(0xAD),
        "mediaplaypause" | "playpause" | "play_pause" => Some(0xB3),
        "medianext" | "nexttrack" | "next_track" => Some(0xB0),
        "mediaprev" | "prevtrack" | "prev_track" => Some(0xB1),
        "mediastop" | "stoptrack" => Some(0xB2),
        "+" | "plus" | "=" | "equals" => Some(0xBB),
        "-" | "minus" | "hyphen" | "_" => Some(0xBD),
        "," | "comma" | "<" => Some(0xBC),
        "." | "period" | "dot" | ">" => Some(0xBE),
        "/" | "slash" | "?" => Some(0xBF),
        ";" | "semicolon" | ":" => Some(0xBA),
        "'" | "quote" | "\"" => Some(0xDE),
        "[" | "lbracket" | "{" => Some(0xDB),
        "]" | "rbracket" | "}" => Some(0xDD),
        "\\" | "backslash" | "|" => Some(0xDC),
        "`" | "backtick" | "tilde" | "~" => Some(0xC0),
        "apps" | "contextmenu" | "menu" => Some(0x5D),
        _ => None,
    }
}

pub fn is_modifier(name: &str) -> bool {
    matches!(
        name.to_lowercase().as_str(),
        "ctrl" | "control" | "alt" | "shift" | "win" | "windows" | "super" | "meta"
    )
}

pub fn type_text(text: &str) -> Result<()> {
    type_text_with_delay(text, 6)
}

pub fn type_text_with_delay(text: &str, delay_ms: u64) -> Result<()> {
    let mut utf16_buf = [0u16; 2];
    let wait = delay_ms.max(2);
    for ch in text.chars() {
        match ch {
            '\r' => continue,
            '\n' => {
                key_press("enter")?;
                std::thread::sleep(std::time::Duration::from_millis(wait));
            }
            '\t' => {
                key_press("tab")?;
                std::thread::sleep(std::time::Duration::from_millis(wait));
            }
            _ => {
                for &mut scan in ch.encode_utf16(&mut utf16_buf) {
                    let down = make_keyboard_input(0, scan, KEYEVENTF_UNICODE.0);
                    let up = make_keyboard_input(0, scan, KEYEVENTF_UNICODE.0 | KEYEVENTF_KEYUP.0);
                    send_key(down)?;
                    std::thread::sleep(std::time::Duration::from_millis(wait));
                    send_key(up)?;
                    std::thread::sleep(std::time::Duration::from_millis(wait));
                }
            }
        }
    }
    Ok(())
}

/// Pastes text instantaneously via the Windows clipboard while preserving the user's previous clipboard contents.
pub fn paste_text(text: &str) -> Result<()> {
    let prev = crate::clipboard::get_text().ok();
    crate::clipboard::set_text(text)?;
    std::thread::sleep(std::time::Duration::from_millis(15));
    combo_str("ctrl+v")?;
    std::thread::sleep(std::time::Duration::from_millis(45));
    if let Some(old) = prev {
        let _ = crate::clipboard::set_text(&old);
    }
    Ok(())
}

pub fn key_press(name: &str) -> Result<()> {
    let vk = resolve_vk(name).ok_or_else(|| CuError::InvalidArg(format!("unknown key: {}", name)))?;
    let down = make_keyboard_input(vk, 0, 0);
    let up = make_keyboard_input(vk, 0, KEYEVENTF_KEYUP.0);
    send_key(down)?;
    std::thread::sleep(std::time::Duration::from_millis(30));
    send_key(up)?;
    Ok(())
}

pub fn key_down(name: &str) -> Result<()> {
    let vk = resolve_vk(name).ok_or_else(|| CuError::InvalidArg(format!("unknown key: {}", name)))?;
    send_key(make_keyboard_input(vk, 0, 0))
}

pub fn key_up(name: &str) -> Result<()> {
    let vk = resolve_vk(name).ok_or_else(|| CuError::InvalidArg(format!("unknown key: {}", name)))?;
    send_key(make_keyboard_input(vk, 0, KEYEVENTF_KEYUP.0))
}

pub fn hotkey(keys: &[String]) -> Result<()> {
    // press modifiers first
    let mut resolved: Vec<u16> = Vec::new();
    for k in keys {
        let vk = resolve_vk(k).ok_or_else(|| CuError::InvalidArg(format!("unknown key: {}", k)))?;
        resolved.push(vk);
    }
    // down all
    for &vk in &resolved {
        send_key(make_keyboard_input(vk, 0, 0))?;
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    std::thread::sleep(std::time::Duration::from_millis(40));
    // up in reverse
    for &vk in resolved.iter().rev() {
        send_key(make_keyboard_input(vk, 0, KEYEVENTF_KEYUP.0))?;
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    Ok(())
}

pub fn combo_str(s: &str) -> Result<()> {
    let parts: Vec<String> = s.split('+').map(|x| x.trim().to_string()).collect();
    hotkey(&parts)
}

/// Hold modifier keys (`ctrl`, `shift`, `alt`, `win`) while executing an arbitrary closure (e.g. modified click, drag, or scroll),
/// guaranteeing modifier release even if the closure returns an error.
pub fn with_modifiers<F, R>(modifiers: &[String], f: F) -> Result<R>
where
    F: FnOnce() -> Result<R>,
{
    let mut resolved: Vec<u16> = Vec::new();
    for k in modifiers {
        let vk = resolve_vk(k).ok_or_else(|| CuError::InvalidArg(format!("unknown modifier key: {}", k)))?;
        resolved.push(vk);
    }
    for &vk in &resolved {
        send_key(make_keyboard_input(vk, 0, 0))?;
        std::thread::sleep(std::time::Duration::from_millis(12));
    }
    let res = f();
    for &vk in resolved.iter().rev() {
        let _ = send_key(make_keyboard_input(vk, 0, KEYEVENTF_KEYUP.0));
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    res
}

/// Copies the current selection (`Ctrl+C`) while preserving the user's previous clipboard contents, returning the newly selected text.
pub fn copy_selection() -> Result<String> {
    let prev = crate::clipboard::get_text().ok();
    let _ = crate::clipboard::set_text("");
    std::thread::sleep(std::time::Duration::from_millis(15));
    combo_str("ctrl+c")?;
    let mut copied = String::new();
    for _ in 0..8 {
        std::thread::sleep(std::time::Duration::from_millis(20));
        if let Ok(t) = crate::clipboard::get_text() {
            if !t.is_empty() {
                copied = t;
                break;
            }
        }
    }
    if let Some(old) = prev {
        let _ = crate::clipboard::set_text(&old);
    }
    Ok(copied)
}
