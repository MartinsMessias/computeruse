//! Example 11: UI Automation (Accessibility Inspection)
//! Demonstrates inspecting UI elements at point, getting focused element,
//! searching UI controls by name, and enumerating child elements of a window.

use computeruse::ComputerUse;

fn main() -> computeruse::Result<()> {
    println!("=== 11: UI AUTOMATION (UIA) ===");

    // 1. Inspect UI element under current cursor
    let cursor = ComputerUse::cursor_position()?;
    println!("Inspecting UI element at cursor position ({}, {})...", cursor.x, cursor.y);
    match ComputerUse::uia_element_at_point(cursor.x, cursor.y) {
        Ok(el) => {
            println!("  Element Name:    '{}'", el.name);
            println!("  Control Type:    {}", el.control_type);
            println!("  Automation ID:   {}", el.automation_id);
            println!("  Enabled:         {}", el.is_enabled);
            println!("  Bounds:          ({}, {}) {}x{}", el.rect.x, el.rect.y, el.rect.width, el.rect.height);
        }
        Err(e) => println!("  UIA element at point failed: {}", e),
    }

    // 2. Focused UI element
    println!("\nInspecting currently focused UI element...");
    match ComputerUse::uia_focused_element() {
        Ok(focused) => {
            println!("  Focused Element: '{}'", focused.name);
            println!("  Control Type:    {}", focused.control_type);
            println!("  Automation ID:   {}", focused.automation_id);
        }
        Err(e) => println!("  Could not query focused element: {}", e),
    }

    // 3. Root UI Automation children
    println!("\nEnumerating top-level desktop UI children...");
    match ComputerUse::uia_children(0) {
        Ok(children) => {
            println!("  Found {} top-level UI elements (showing first 5):", children.len());
            for (i, c) in children.iter().take(5).enumerate() {
                println!("    #{}: Name='{}' | Type={}", i + 1, c.name, c.control_type);
            }
        }
        Err(e) => println!("  Could not query UIA children: {}", e),
    }

    Ok(())
}
