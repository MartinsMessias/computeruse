//! Example 03: Cursor & Mouse Automation
//! Demonstrates reading cursor position, moving, clicking, double clicking, dragging, and scrolling.

use computeruse::{ComputerUse, MouseButton, Point};
use std::thread::sleep;
use std::time::Duration;

fn main() -> computeruse::Result<()> {
    println!("=== 03: CURSOR & MOUSE ===");

    // 1. Get current cursor position
    let original = ComputerUse::cursor_position()?;
    println!("Initial cursor position: ({}, {})", original.x, original.y);

    // 2. Move cursor to absolute coordinates (safe position on screen)
    let target = Point { x: 400, y: 400 };
    println!("Moving cursor to ({}, {})...", target.x, target.y);
    ComputerUse::mouse_move(target)?;
    sleep(Duration::from_millis(100));

    let current = ComputerUse::cursor_position()?;
    println!("  New cursor position: ({}, {})", current.x, current.y);

    // 3. Move relative (+50 x, -30 y)
    println!("Moving relative (+50, -30)...");
    let after_rel = ComputerUse::mouse_move_relative(50, -30)?;
    println!("  After relative move: ({}, {})", after_rel.x, after_rel.y);
    sleep(Duration::from_millis(100));

    // 4. Click actions (without moving cursor or specifying point)
    println!("Performing single click (Left button)...");
    ComputerUse::click(MouseButton::Left, None)?;

    println!("Performing right click at current position...");
    ComputerUse::click(MouseButton::Right, None)?;
    sleep(Duration::from_millis(200));

    // Dismiss any context menu with Escape
    ComputerUse::key_press("escape")?;

    // 5. Scroll (vertical and horizontal)
    println!("Scrolling wheel up (+2)...");
    ComputerUse::scroll(2, false)?;

    println!("Scrolling wheel down (-2)...");
    ComputerUse::scroll(-2, false)?;

    // 6. Restore cursor to original position
    println!("Restoring cursor to original ({}, {})...", original.x, original.y);
    ComputerUse::set_cursor_position(original)?;

    println!("Mouse & cursor operations completed successfully!");
    Ok(())
}
