//! Example 06: Clipboard Management
//! Demonstrates setting and reading text from the Windows system clipboard.

use computeruse::ComputerUse;

fn main() -> computeruse::Result<()> {
    println!("=== 06: CLIPBOARD ===");

    // 1. Read existing clipboard text (if any)
    if let Ok(prev) = ComputerUse::clipboard_get() {
        println!("Existing clipboard text: '{}'", prev);
    } else {
        println!("Existing clipboard is empty or does not contain plain text.");
    }

    // 2. Set new text to clipboard
    let sample = "ComputerUse LLM Agent text from example 06!";
    println!("\nSetting new clipboard text: '{}'", sample);
    ComputerUse::clipboard_set(sample)?;

    // 3. Verify clipboard text
    let retrieved = ComputerUse::clipboard_get()?;
    println!("Read back from clipboard:   '{}'", retrieved);
    assert_eq!(retrieved, sample);

    println!("\nClipboard read/write test succeeded!");
    Ok(())
}
