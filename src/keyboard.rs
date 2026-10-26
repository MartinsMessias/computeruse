use crate::error::{CuError, Result};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, KEYEVENTF_UNICODE,
    VIRTUAL_KEY,
};

fn send_key(inp: INPUT) -> Result<()> {
    unsafe {
        let sent = SendInput(&[inp], std::mem::size_of::<INPUT>() as i32);
        if sent == 0 {
            return Err(CuError::Windows("SendInput failed".into()));
        }
    }
    Ok(())
}

fn make_keyboard_input(vk: u16, scan: u16, flags: u32) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(vk),
                wScan: scan,
                dwFlags: windows::Win32::UI::Input::KeyboardAndMouse::KEYBD_EVENT_FLAGS(flags),
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
    type_text_with_delay(text, 25)
}

pub fn type_text_with_delay(text: &str, delay_ms: u64) -> Result<()> {
    for ch in text.chars() {
        let scan = ch as u16;
        let down = make_keyboard_input(0, scan, KEYEVENTF_UNICODE.0);
        let up = make_keyboard_input(0, scan, KEYEVENTF_UNICODE.0 | KEYEVENTF_KEYUP.0);
        send_key(down)?;
        std::thread::sleep(std::time::Duration::from_millis(delay_ms.max(5)));
        send_key(up)?;
        std::thread::sleep(std::time::Duration::from_millis(delay_ms.max(5)));
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
