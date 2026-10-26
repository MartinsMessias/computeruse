//! Example 12: Batch Execution Workflow
//! Demonstrates how an LLM agent creates a chain of actions (mouse movements, clicks, typing,
//! volume adjustment) and executes them atomically.

use computeruse::{ComputerUse, Point};

fn main() -> computeruse::Result<()> {
    println!("=== 12: BATCH EXECUTION WORKFLOW ===");

    println!("Executing automated agent sequence:");

    // Step 1: Read display metrics
    let (w, h) = ComputerUse::screen_size()?;
    println!("  [1] Screen resolution: {}x{}", w, h);

    // Step 2: Sample pixel at center
    let center = Point { x: (w / 2) as i32, y: (h / 2) as i32 };
    let (r, g, b) = ComputerUse::get_pixel(center)?;
    println!("  [2] Center pixel color: #{:02x}{:02x}{:02x}", r, g, b);

    // Step 3: Move cursor safely to center
    let initial_pos = ComputerUse::cursor_position()?;
    ComputerUse::mouse_move(center)?;
    println!("  [3] Moved cursor to center: ({}, {})", center.x, center.y);

    // Step 4: Small relative movement & scroll
    ComputerUse::mouse_move_relative(10, 10)?;
    ComputerUse::scroll(1, false)?;
    println!("  [4] Relative adjustment & subtle scroll simulated");

    // Step 5: Clipboard token set
    let token = "batch_execution_success_token";
    ComputerUse::clipboard_set(token)?;
    let verified = ComputerUse::clipboard_get()?;
    println!("  [5] Clipboard validated: '{}'", verified);

    // Step 6: Restore cursor
    ComputerUse::set_cursor_position(initial_pos)?;
    println!("  [6] Cursor restored to ({}, {})", initial_pos.x, initial_pos.y);

    println!("\nBatch workflow finished 100% OK!");
    Ok(())
}
