//! Example 07: Pixel Sampling
//! Demonstrates reading the RGB color value of any pixel on screen.

use computeruse::{ComputerUse, Point};

fn main() -> computeruse::Result<()> {
    println!("=== 07: PIXEL INSPECTION ===");

    // 1. Get pixel at current cursor position
    let cursor = ComputerUse::cursor_position()?;
    let (r, g, b) = ComputerUse::get_pixel(cursor)?;
    println!(
        "Pixel at cursor ({}, {}): RGB({}, {}, {}) | Hex: #{:02x}{:02x}{:02x}",
        cursor.x, cursor.y, r, g, b, r, g, b
    );

    // 2. Sample pixel at screen corners / centers
    let sample_points = [
        Point { x: 10, y: 10 },
        Point { x: 500, y: 300 },
        Point { x: 800, y: 600 },
    ];

    println!("\nSampling several coordinates:");
    for pt in sample_points {
        match ComputerUse::get_pixel(pt) {
            Ok((r, g, b)) => {
                println!(
                    "  At ({:4}, {:4}) -> RGB({:3}, {:3}, {:3}) | #{:02x}{:02x}{:02x}",
                    pt.x, pt.y, r, g, b, r, g, b
                );
            }
            Err(e) => println!("  At ({:4}, {:4}) -> Failed: {}", pt.x, pt.y, e),
        }
    }

    Ok(())
}
