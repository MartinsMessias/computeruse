//! Example 02: Screenshots
//! Demonstrates full screen capture, region capture, active window capture, and multi-monitor capture.

use computeruse::{ComputerUse, Rect};

fn main() -> computeruse::Result<()> {
    println!("=== 02: SCREENSHOTS ===");

    // 1. Full screenshot of primary monitor
    println!("Capturing full screenshot...");
    let full = ComputerUse::screenshot_full(None)?;
    println!(
        "  Full capture: {}x{} px (PNG base64 length: {} chars)",
        full.width,
        full.height,
        full.png_base64.len()
    );

    // 2. Specific region screenshot (e.g. 300x200 at x=100, y=100)
    let region = Rect { x: 100, y: 100, width: 300, height: 200 };
    println!("\nCapturing region ({}, {}) {}x{}...", region.x, region.y, region.width, region.height);
    let crop = ComputerUse::screenshot_region(region, None)?;
    println!("  Crop capture: {}x{} px", crop.width, crop.height);

    // 3. Active (foreground) window screenshot
    println!("\nCapturing active foreground window screenshot...");
    match ComputerUse::screenshot_active_window() {
        Ok(win_shot) => println!("  Active window: {}x{} px", win_shot.width, win_shot.height),
        Err(e) => println!("  Could not capture active window: {}", e),
    }

    // 4. Combined screenshot of all monitors
    println!("\nCapturing combined desktop (all monitors)...");
    let all = ComputerUse::screenshot_all_monitors()?;
    println!("  Combined desktop: {}x{} px", all.width, all.height);

    println!("\nTip: Decode 'png_base64' using general base64 decoder to save to file.");
    Ok(())
}
