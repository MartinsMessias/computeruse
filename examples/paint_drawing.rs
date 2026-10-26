use base64::Engine;
use computeruse::*;
use std::f64::consts::PI;
use std::thread::sleep;
use std::time::Duration;

fn line(x1: i32, y1: i32, x2: i32, y2: i32) -> Result<()> {
    ComputerUse::drag(Point { x: x1, y: y1 }, Point { x: x2, y: y2 }, MouseButton::Left)?;
    sleep(Duration::from_millis(25));
    Ok(())
}

fn circle(cx: i32, cy: i32, r: f64, segments: usize) -> Result<()> {
    let mut prev_x = cx + r.round() as i32;
    let mut prev_y = cy;
    for i in 1..=segments {
        let angle = 2.0 * PI * (i as f64) / (segments as f64);
        let cur_x = cx + (r * angle.cos()).round() as i32;
        let cur_y = cy + (r * angle.sin()).round() as i32;
        line(prev_x, prev_y, cur_x, cur_y)?;
        prev_x = cur_x;
        prev_y = cur_y;
    }
    Ok(())
}

fn main() -> Result<()> {
    println!("=== DRAWING A MASTERPIECE IN PAINT WITH COMPUTERUSE ===");

    // 1. Locate and focus the Paint window
    let paint_win = ComputerUse::find_window("Paint")?;
    println!("Found Paint window: HWND={} ({:?})", paint_win.hwnd, paint_win.rect);
    ComputerUse::focus_window(paint_win.hwnd)?;
    sleep(Duration::from_millis(400));

    // Ensure it's on foreground
    let _ = ComputerUse::set_window_topmost(paint_win.hwnd, true);
    sleep(Duration::from_millis(200));

    // Clear canvas
    ComputerUse::click(MouseButton::Left, Some(Point { x: 800, y: 500 }))?;
    sleep(Duration::from_millis(150));
    let _ = ComputerUse::hotkey_str("ctrl+a");
    sleep(Duration::from_millis(150));
    let _ = ComputerUse::key_press("delete");
    sleep(Duration::from_millis(250));

    // Select Pencil tool via UIA
    println!("Selecting Pencil tool...");
    let _ = ComputerUse::uia_click_by_name("Pencil");
    sleep(Duration::from_millis(300));

    println!("Drawing house scene...\n");

    // ------------------------------------------------------------
    // 1. GROUND
    // ------------------------------------------------------------
    println!("1. Ground line");
    line(620, 640, 1450, 640)?;

    // ------------------------------------------------------------
    // 2. HOUSE WALLS
    // ------------------------------------------------------------
    println!("2. House walls");
    line(800, 480, 1100, 480)?; // Top wall
    line(1100, 480, 1100, 640)?; // Right wall
    line(1100, 640, 800, 640)?;  // Bottom wall
    line(800, 640, 800, 480)?;   // Left wall

    // ------------------------------------------------------------
    // 3. ROOF
    // ------------------------------------------------------------
    println!("3. Triangular roof");
    line(770, 480, 950, 340)?;  // Left roof slope
    line(950, 340, 1130, 480)?; // Right roof slope
    line(770, 480, 1130, 480)?; // Roof base overhang

    // Chimney
    println!("4. Chimney & smoke");
    line(1030, 400, 1030, 310)?;
    line(1030, 310, 1060, 310)?;
    line(1060, 310, 1060, 420)?;
    // Smoke
    line(1040, 295, 1045, 280)?;
    line(1045, 280, 1035, 265)?;
    line(1035, 265, 1048, 250)?;

    // ------------------------------------------------------------
    // 4. DOOR & KNOB
    // ------------------------------------------------------------
    println!("5. Door");
    line(920, 640, 920, 540)?; // Left door frame
    line(920, 540, 980, 540)?; // Top door frame
    line(980, 540, 980, 640)?; // Right door frame
    // Door knob
    line(965, 590, 968, 590)?;

    // ------------------------------------------------------------
    // 5. WINDOWS
    // ------------------------------------------------------------
    println!("6. Windows");
    // Left window
    line(830, 510, 875, 510)?;
    line(875, 510, 875, 565)?;
    line(875, 565, 830, 565)?;
    line(830, 565, 830, 510)?;
    line(852, 510, 852, 565)?; // Vertical sash
    line(830, 537, 875, 537)?; // Horizontal sash

    // Right window
    line(1005, 510, 1050, 510)?;
    line(1050, 510, 1050, 565)?;
    line(1050, 565, 1005, 565)?;
    line(1005, 565, 1005, 510)?;
    line(1027, 510, 1027, 565)?; // Vertical sash
    line(1005, 537, 1050, 537)?; // Horizontal sash

    // Attic round window
    circle(950, 420, 18.0, 12)?;
    line(950, 402, 950, 438)?;
    line(932, 420, 968, 420)?;

    // ------------------------------------------------------------
    // 6. SMILING SUN & RAYS
    // ------------------------------------------------------------
    println!("7. Smiling sun & rays");
    let sun_x = 1250;
    let sun_y = 350;
    circle(sun_x, sun_y, 35.0, 16)?;

    // Eyes
    line(sun_x - 12, sun_y - 8, sun_x - 12, sun_y - 4)?;
    line(sun_x + 12, sun_y - 8, sun_x + 12, sun_y - 4)?;
    // Smile
    line(sun_x - 12, sun_y + 10, sun_x - 5, sun_y + 16)?;
    line(sun_x - 5, sun_y + 16, sun_x + 5, sun_y + 16)?;
    line(sun_x + 5, sun_y + 16, sun_x + 12, sun_y + 10)?;

    // 8 Rays
    for i in 0..8 {
        let angle = 2.0 * PI * (i as f64) / 8.0;
        let r1 = 45.0;
        let r2 = 62.0;
        let p1x = sun_x + (r1 * angle.cos()).round() as i32;
        let p1y = sun_y + (r1 * angle.sin()).round() as i32;
        let p2x = sun_x + (r2 * angle.cos()).round() as i32;
        let p2y = sun_y + (r2 * angle.sin()).round() as i32;
        line(p1x, p1y, p2x, p2y)?;
    }

    // ------------------------------------------------------------
    // 7. TREE
    // ------------------------------------------------------------
    println!("8. Tree");
    // Trunk
    line(680, 640, 680, 530)?;
    line(705, 640, 705, 530)?;
    // Canopy
    circle(692, 490, 45.0, 14)?;
    circle(665, 460, 30.0, 12)?;
    circle(720, 460, 30.0, 12)?;

    // ------------------------------------------------------------
    // 8. 5-POINTED STAR
    // ------------------------------------------------------------
    println!("9. Five-pointed star");
    let star_cx = 1250.0;
    let star_cy = 580.0;
    let r = 26.0;
    // Calculate the 5 star tips
    let mut tips = Vec::new();
    for i in 0..5 {
        let angle = -PI / 2.0 + (i as f64) * 2.0 * PI / 5.0;
        tips.push(Point {
            x: (star_cx + r * angle.cos()).round() as i32,
            y: (star_cy + r * angle.sin()).round() as i32,
        });
    }
    // Connect tips: 0 -> 2 -> 4 -> 1 -> 3 -> 0
    line(tips[0].x, tips[0].y, tips[2].x, tips[2].y)?;
    line(tips[2].x, tips[2].y, tips[4].x, tips[4].y)?;
    line(tips[4].x, tips[4].y, tips[1].x, tips[1].y)?;
    line(tips[1].x, tips[1].y, tips[3].x, tips[3].y)?;
    line(tips[3].x, tips[3].y, tips[0].x, tips[0].y)?;

    // ------------------------------------------------------------
    // 9. SIGNATURE: "CU"
    // ------------------------------------------------------------
    println!("10. Signature 'CU'");
    // 'C'
    line(1160, 560, 1130, 560)?;
    line(1130, 560, 1130, 605)?;
    line(1130, 605, 1160, 605)?;

    // 'U'
    line(1175, 560, 1175, 605)?;
    line(1175, 605, 1205, 605)?;
    line(1205, 605, 1205, 560)?;

    // Unpin topmost
    let _ = ComputerUse::set_window_topmost(paint_win.hwnd, false);

    println!("\nDrawing complete! Taking a screenshot of the masterpiece in Paint...");
    let shot = ComputerUse::screenshot_window(paint_win.hwnd)?;
    println!("Screenshot captured: {}x{} px", shot.width, shot.height);

    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&shot.png_base64)
        .map_err(|e| CuError::Internal(format!("Base64 decode error: {}", e)))?;
    std::fs::write("desenho_paint.png", bytes)?;
    println!("Successfully saved result to 'desenho_paint.png'!");

    Ok(())
}
