//! Example 01: Screen & Displays
//! Demonstrates querying screen resolution, DPI scaling, and multi-monitor setup.

use computeruse::ComputerUse;

fn main() -> computeruse::Result<()> {
    println!("=== 01: SCREEN & MONITORS ===");

    // 1. Primary screen dimensions
    let (width, height) = ComputerUse::screen_size()?;
    println!("Screen Size: {}x{} pixels", width, height);

    // 2. Enumerate all connected monitors
    let monitors = ComputerUse::list_monitors()?;
    println!("\nDetected {} monitor(s):", monitors.len());
    for m in &monitors {
        println!(
            "  [{}] Name: {} | Primary: {} | Scale: {:.2}x | Bounds: ({}, {}) {}x{}",
            m.index,
            m.name,
            m.is_primary,
            m.scale_factor,
            m.rect.x,
            m.rect.y,
            m.rect.width,
            m.rect.height,
        );
    }

    Ok(())
}
