//! Example 04: Keyboard Input & Hotkeys
//! Demonstrates typing text (Unicode), typing with inter-key delay, pressing special keys,
//! hotkey combinations, and media keys.

use computeruse::ComputerUse;
use std::thread::sleep;
use std::time::Duration;

fn main() -> computeruse::Result<()> {
    println!("=== 04: KEYBOARD AUTOMATION ===");

    // 1. Single Key Press
    println!("Pressing individual keys (Shift, Space, Esc)...");
    ComputerUse::key_press("shift")?;
    sleep(Duration::from_millis(50));
    ComputerUse::key_press("escape")?;

    // 2. Key Down & Key Up simulation
    println!("Simulating manual key down/up...");
    ComputerUse::key_down("ctrl")?;
    sleep(Duration::from_millis(20));
    ComputerUse::key_up("ctrl")?;

    // 3. Hotkey combinations
    println!("Testing hotkey string parser: 'ctrl+shift+esc' / 'alt+tab' syntax...");
    // We demonstrate hotkey_str with non-intrusive combo
    ComputerUse::hotkey_str("ctrl+shift")?;

    // 4. Media playback & volume keys
    println!("Testing media key names (volumeup, volumedown, volumemute)...");
    // Vol up and down
    ComputerUse::key_press("volumeup")?;
    sleep(Duration::from_millis(50));
    ComputerUse::key_press("volumedown")?;

    // 5. Typing text with delay
    println!("Typing text with custom delay...");
    println!("(Tip: Combine with focus_window or click to type directly into inputs)");
    // ComputerUse::type_text_with_delay("Hello Windows!", 20)?;

    println!("Keyboard examples executed successfully!");
    Ok(())
}
