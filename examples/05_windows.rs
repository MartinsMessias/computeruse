//! Example 05: Windows Management
//! Demonstrates enumerating windows, finding windows by title, reading foreground window,
//! focus, minimize, maximize, restore, resize, topmost pin, and opacity adjustment.

use computeruse::ComputerUse;

fn main() -> computeruse::Result<()> {
    println!("=== 05: WINDOW MANAGEMENT ===");

    // 1. Get current foreground window
    match ComputerUse::foreground_window() {
        Ok(fg) => {
            println!("Active Window:");
            println!("  Title:      {}", fg.title);
            println!("  HWND:       {}", fg.hwnd);
            println!("  PID:        {}", fg.pid);
            println!("  Class:      {}", fg.class_name);
            println!("  Bounds:     ({}, {}) {}x{}", fg.rect.x, fg.rect.y, fg.rect.width, fg.rect.height);
            println!("  Maximized:  {}", fg.is_maximized);
            println!("  Minimized:  {}", fg.is_minimized);
        }
        Err(e) => println!("Could not query foreground window: {}", e),
    }

    // 2. List all visible top-level windows
    println!("\nListing visible application windows (first 10):");
    let windows = ComputerUse::list_windows(true)?;
    for (i, w) in windows.iter().take(10).enumerate() {
        println!("  #{}: HWND={} | PID={} | Title='{}'", i + 1, w.hwnd, w.pid, w.title);
    }

    // 3. Find window by substring
    println!("\nSearching for window containing 'cmd' or 'chrome' or 'explorer'...");
    if let Ok(w) = ComputerUse::find_window("cmd") {
        println!("  Found CMD window: HWND={} Title='{}'", w.hwnd, w.title);
    } else if let Ok(w) = ComputerUse::find_window("chrome") {
        println!("  Found Chrome window: HWND={} Title='{}'", w.hwnd, w.title);
    } else {
        println!("  No matching window found for the demo search query.");
    }

    println!("\nWindow manipulation methods available on ComputerUse:");
    println!("  - ComputerUse::focus_window(hwnd)");
    println!("  - ComputerUse::minimize_window(hwnd)");
    println!("  - ComputerUse::maximize_window(hwnd)");
    println!("  - ComputerUse::restore_window(hwnd)");
    println!("  - ComputerUse::move_resize_window(hwnd, rect)");
    println!("  - ComputerUse::set_window_topmost(hwnd, topmost)");
    println!("  - ComputerUse::set_window_opacity(hwnd, alpha_0_to_255)");
    println!("  - ComputerUse::close_window(hwnd)");
    println!("  - ComputerUse::wait_for_window(title, timeout_ms)");

    Ok(())
}
