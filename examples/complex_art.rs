use base64::Engine;
use computeruse::*;
use std::f64::consts::PI;
use std::thread::sleep;
use std::time::Duration;

struct Canvas {
    rect: Rect,
    aspect: f64,
}

impl Canvas {
    fn new(rect: Rect) -> Self {
        let aspect = (rect.width as f64) / (rect.height as f64).max(1.0);
        Canvas { rect, aspect }
    }

    /// Normalized canvas coordinates: rx (0.0 to 1.0), ry (0.0 to 1.0)
    fn pt(&self, rx: f64, ry: f64) -> Point {
        Point {
            x: self.rect.x + ((self.rect.width as f64) * rx).round() as i32,
            y: self.rect.y + ((self.rect.height as f64) * ry).round() as i32,
        }
    }

    /// Polar coordinates with circular aspect ratio correction
    fn polar(&self, cx: f64, cy: f64, radius_y: f64, angle: f64) -> Point {
        let rx = cx + (radius_y / self.aspect) * angle.cos();
        let ry = cy + radius_y * angle.sin();
        self.pt(rx, ry)
    }

    fn line_norm(&self, rx1: f64, ry1: f64, rx2: f64, ry2: f64) -> Result<()> {
        let p1 = self.pt(rx1, ry1);
        let p2 = self.pt(rx2, ry2);
        ComputerUse::drag(p1, p2, MouseButton::Left)?;
        sleep(Duration::from_millis(15));
        Ok(())
    }

    fn polygon_polar(&self, cx: f64, cy: f64, r: f64, sides: usize, rotation: f64) -> Result<()> {
        let mut pts = Vec::with_capacity(sides);
        for i in 0..sides {
            let theta = rotation + 2.0 * PI * (i as f64) / (sides as f64);
            pts.push(self.polar(cx, cy, r, theta));
        }
        for i in 0..sides {
            let next = (i + 1) % sides;
            ComputerUse::drag(pts[i], pts[next], MouseButton::Left)?;
            sleep(Duration::from_millis(15));
        }
        Ok(())
    }

    fn star_polar(&self, cx: f64, cy: f64, r_out: f64, r_in: f64, points: usize, rotation: f64) -> Result<()> {
        let count = points * 2;
        let mut pts = Vec::with_capacity(count);
        for i in 0..count {
            let r = if i % 2 == 0 { r_out } else { r_in };
            let theta = rotation + PI * (i as f64) / (points as f64);
            pts.push(self.polar(cx, cy, r, theta));
        }
        for i in 0..count {
            let next = (i + 1) % count;
            ComputerUse::drag(pts[i], pts[next], MouseButton::Left)?;
            sleep(Duration::from_millis(15));
        }
        Ok(())
    }

    fn classic_5star(&self, cx: f64, cy: f64, r: f64) -> Result<()> {
        let mut pts = Vec::with_capacity(5);
        for i in 0..5 {
            let theta = -PI / 2.0 + 2.0 * PI * (i as f64) / 5.0;
            pts.push(self.polar(cx, cy, r, theta));
        }
        // Connect 0 -> 2 -> 4 -> 1 -> 3 -> 0
        ComputerUse::drag(pts[0], pts[2], MouseButton::Left)?;
        sleep(Duration::from_millis(15));
        ComputerUse::drag(pts[2], pts[4], MouseButton::Left)?;
        sleep(Duration::from_millis(15));
        ComputerUse::drag(pts[4], pts[1], MouseButton::Left)?;
        sleep(Duration::from_millis(15));
        ComputerUse::drag(pts[1], pts[3], MouseButton::Left)?;
        sleep(Duration::from_millis(15));
        ComputerUse::drag(pts[3], pts[0], MouseButton::Left)?;
        sleep(Duration::from_millis(15));
        Ok(())
    }
}

fn set_color(color_name: &str) -> Result<()> {
    println!("  [Color] Selecting {}...", color_name);
    let _ = ComputerUse::uia_click_by_name(color_name);
    sleep(Duration::from_millis(150));
    Ok(())
}

fn main() -> Result<()> {
    println!("============================================================");
    println!("   DYNAMIC WINDOW-ADAPTIVE SACRED CELESTIAL ART IN PAINT    ");
    println!("============================================================");

    // 1. Locate and focus Paint window dynamically
    let paint_win = ComputerUse::find_window("Paint")?;
    println!("Found Paint window: HWND={} ({:?})", paint_win.hwnd, paint_win.rect);
    ComputerUse::focus_window(paint_win.hwnd)?;
    sleep(Duration::from_millis(400));
    let _ = ComputerUse::set_window_topmost(paint_win.hwnd, true);
    sleep(Duration::from_millis(200));

    // 2. Locate the Canvas element dynamically via UI Automation
    let win_center_x = paint_win.rect.x + (paint_win.rect.width / 2) as i32;
    let win_center_y = paint_win.rect.y + (paint_win.rect.height / 2) as i32;
    let canvas_el = match ComputerUse::uia_element_at_point(win_center_x, win_center_y) {
        Ok(el) if el.automation_id == "image" || el.name.contains("Canvas") => el,
        _ => {
            // Fallback: search window children for canvas
            let children = ComputerUse::uia_children(paint_win.hwnd)?;
            children.into_iter().find(|c| c.automation_id == "image" || c.name.contains("Canvas"))
                .unwrap_or_else(|| {
                    println!("Canvas element fallback to estimated window rect");
                    UiElementInfo {
                        name: "CanvasFallback".into(),
                        automation_id: "image".into(),
                        control_type: "Group".into(),
                        class_name: "".into(),
                        rect: Rect {
                            x: paint_win.rect.x + 50,
                            y: paint_win.rect.y + 200,
                            width: paint_win.rect.width.saturating_sub(100),
                            height: paint_win.rect.height.saturating_sub(250),
                        },
                        hwnd: 0,
                        is_enabled: true,
                        is_offscreen: false,
                    }
                })
        }
    };

    println!("\n[OK] DYNAMIC CANVAS DETECTED:");
    println!("     Origin: ({}, {}) | Dimensions: {}x{}",
        canvas_el.rect.x, canvas_el.rect.y, canvas_el.rect.width, canvas_el.rect.height);

    let canvas = Canvas::new(canvas_el.rect);

    // 3. Focus canvas and clear
    let center_pt = canvas.pt(0.5, 0.5);
    ComputerUse::click(MouseButton::Left, Some(center_pt))?;
    sleep(Duration::from_millis(150));
    let _ = ComputerUse::hotkey_str("ctrl+a");
    sleep(Duration::from_millis(150));
    let _ = ComputerUse::key_press("delete");
    sleep(Duration::from_millis(250));

    // 4. Select Pencil tool
    let _ = ComputerUse::uia_click_by_name("Pencil");
    sleep(Duration::from_millis(300));

    // ============================================================
    // LAYER 1: OUTER ORNATE FRAME & CORNER DIAMONDS (BLACK)
    // ============================================================
    println!("\n[Layer 1/7] Outer Ornate Frame & Corner Diamonds...");
    set_color("Black")?;

    // Outer frame
    canvas.line_norm(0.03, 0.04, 0.97, 0.04)?;
    canvas.line_norm(0.97, 0.04, 0.97, 0.96)?;
    canvas.line_norm(0.97, 0.96, 0.03, 0.96)?;
    canvas.line_norm(0.03, 0.96, 0.03, 0.04)?;

    // Inner frame
    canvas.line_norm(0.036, 0.05, 0.964, 0.05)?;
    canvas.line_norm(0.964, 0.05, 0.964, 0.95)?;
    canvas.line_norm(0.964, 0.95, 0.036, 0.95)?;
    canvas.line_norm(0.036, 0.95, 0.036, 0.05)?;

    // 4 Corner Diamond Rosettes
    canvas.star_polar(0.055, 0.08, 0.03, 0.012, 4, 0.0)?;
    canvas.star_polar(0.945, 0.08, 0.03, 0.012, 4, 0.0)?;
    canvas.star_polar(0.055, 0.92, 0.03, 0.012, 4, 0.0)?;
    canvas.star_polar(0.945, 0.92, 0.03, 0.012, 4, 0.0)?;

    // Center coordinates of mandala
    let mcx = 0.50;
    let mcy = 0.47;

    // ============================================================
    // LAYER 2: 24-POINTED CELESTIAL CORONA (ORANGE)
    // ============================================================
    println!("\n[Layer 2/7] 24-Pointed Solar Corona (Orange)...");
    set_color("Orange")?;

    // 24 sharp coronal flares radiating outward
    canvas.star_polar(mcx, mcy, 0.38, 0.32, 24, 0.0)?;
    canvas.polygon_polar(mcx, mcy, 0.32, 36, 0.0)?;

    // ============================================================
    // LAYER 3: SACRED 12-POINT DODECAGRAM (TURQUOISE)
    // ============================================================
    println!("\n[Layer 3/7] Sacred 12-Point Dodecagram Ring (Turquoise)...");
    set_color("Turquoise")?;

    canvas.polygon_polar(mcx, mcy, 0.28, 32, 0.0)?;
    canvas.star_polar(mcx, mcy, 0.28, 0.23, 12, 0.0)?;
    canvas.polygon_polar(mcx, mcy, 0.23, 24, 0.0)?;

    // ============================================================
    // LAYER 4: SACRED METATRON'S HEXAGRAMS (RED)
    // ============================================================
    println!("\n[Layer 4/7] Metatron's Interlocking Hexagrams (Red)...");
    set_color("Red")?;

    // Hexagram 1 (Star of David)
    canvas.polygon_polar(mcx, mcy, 0.22, 3, -PI / 2.0)?;
    canvas.polygon_polar(mcx, mcy, 0.22, 3, PI / 2.0)?;

    // Hexagram 2 (Rotated 30 degrees to form 12-pointed star)
    canvas.polygon_polar(mcx, mcy, 0.22, 3, -PI / 2.0 + PI / 6.0)?;
    canvas.polygon_polar(mcx, mcy, 0.22, 3, PI / 2.0 + PI / 6.0)?;

    // Radiating spokes connecting outer vertices to inner hexagon
    for i in 0..12 {
        let theta = 2.0 * PI * (i as f64) / 12.0;
        let p_out = canvas.polar(mcx, mcy, 0.22, theta);
        let p_in = canvas.polar(mcx, mcy, 0.11, theta);
        ComputerUse::drag(p_in, p_out, MouseButton::Left)?;
        sleep(Duration::from_millis(15));
    }

    // ============================================================
    // LAYER 5: GOLDEN 8-POINT STAR & CORE JEWEL (YELLOW & BLACK)
    // ============================================================
    println!("\n[Layer 5/7] Golden 8-Point Star & Core Jewel (Yellow & Black)...");
    set_color("Yellow")?;

    canvas.star_polar(mcx, mcy, 0.11, 0.055, 8, 0.0)?;
    canvas.polygon_polar(mcx, mcy, 0.11, 24, 0.0)?;

    // Central Core Jewel in Black
    set_color("Black")?;
    canvas.star_polar(mcx, mcy, 0.055, 0.022, 8, PI / 8.0)?;
    canvas.polygon_polar(mcx, mcy, 0.022, 16, 0.0)?;
    canvas.line_norm(mcx - 0.008, mcy, mcx + 0.008, mcy)?;
    canvas.line_norm(mcx, mcy - 0.015, mcx, mcy + 0.015)?;

    // ============================================================
    // LAYER 6: FLANKING CELESTIAL GUARDIANS: MOON & SUN
    // ============================================================
    println!("\n[Layer 6/7] Celestial Guardians: Moon (Left) & Sun (Right)...");

    // LEFT: Crescent Moon (Turquoise) + Stars (Yellow)
    set_color("Turquoise")?;
    let moon_cx = 0.20;
    let moon_cy = 0.47;
    let segs = 24;
    let mut outer_moon = Vec::with_capacity(segs + 1);
    for i in 0..=segs {
        let theta = -0.6 * PI + 1.2 * PI * (i as f64) / (segs as f64);
        outer_moon.push(canvas.polar(moon_cx, moon_cy, 0.12, theta));
    }
    for i in 0..segs {
        ComputerUse::drag(outer_moon[i], outer_moon[i + 1], MouseButton::Left)?;
        sleep(Duration::from_millis(15));
    }

    let mut inner_moon = Vec::with_capacity(segs + 1);
    for i in 0..=segs {
        let theta = -0.52 * PI + 1.04 * PI * (i as f64) / (segs as f64);
        inner_moon.push(canvas.polar(moon_cx + 0.025, moon_cy, 0.11, theta));
    }
    for i in 0..segs {
        ComputerUse::drag(inner_moon[i], inner_moon[i + 1], MouseButton::Left)?;
        sleep(Duration::from_millis(15));
    }
    ComputerUse::drag(outer_moon[0], inner_moon[0], MouseButton::Left)?;
    ComputerUse::drag(outer_moon[segs], inner_moon[segs], MouseButton::Left)?;

    // Stars around Moon
    set_color("Yellow")?;
    canvas.classic_5star(0.14, 0.28, 0.035)?;
    canvas.classic_5star(0.24, 0.25, 0.025)?;
    canvas.classic_5star(0.14, 0.68, 0.030)?;
    canvas.classic_5star(0.24, 0.69, 0.022)?;

    // RIGHT: Radiant Celestial Sun (Orange & Yellow)
    set_color("Orange")?;
    let sun_cx = 0.80;
    let sun_cy = 0.47;
    canvas.star_polar(sun_cx, sun_cy, 0.15, 0.09, 12, 0.0)?;
    canvas.polygon_polar(sun_cx, sun_cy, 0.09, 24, 0.0)?;

    set_color("Yellow")?;
    canvas.polygon_polar(sun_cx, sun_cy, 0.075, 24, 0.0)?;

    // Sun Face (Black)
    set_color("Black")?;
    canvas.polygon_polar(sun_cx - 0.015, sun_cy - 0.02, 0.008, 8, 0.0)?; // Left eye
    canvas.polygon_polar(sun_cx + 0.015, sun_cy - 0.02, 0.008, 8, 0.0)?; // Right eye
    // Smile
    canvas.line_norm(sun_cx - 0.018, sun_cy + 0.02, sun_cx - 0.006, sun_cy + 0.035)?;
    canvas.line_norm(sun_cx - 0.006, sun_cy + 0.035, sun_cx + 0.006, sun_cy + 0.035)?;
    canvas.line_norm(sun_cx + 0.006, sun_cy + 0.035, sun_cx + 0.018, sun_cy + 0.02)?;

    // ============================================================
    // LAYER 7: ORNATE BANNER & TYPOGRAPHY
    // ============================================================
    println!("\n[Layer 7/7] Ornate Title Banner & Typography...");
    set_color("Red")?;

    let by1 = 0.865;
    let by2 = 0.915;
    let bx1 = 0.35;
    let bx2 = 0.65;

    // Frame
    canvas.line_norm(bx1, by1, bx2, by1)?;
    canvas.line_norm(bx2, by1, bx2, by2)?;
    canvas.line_norm(bx2, by2, bx1, by2)?;
    canvas.line_norm(bx1, by2, bx1, by1)?;

    // Swallowtail ends
    canvas.line_norm(bx1, by1, bx1 - 0.02, (by1 + by2) / 2.0)?;
    canvas.line_norm(bx1 - 0.02, (by1 + by2) / 2.0, bx1, by2)?;
    canvas.line_norm(bx2, by1, bx2 + 0.02, (by1 + by2) / 2.0)?;
    canvas.line_norm(bx2 + 0.02, (by1 + by2) / 2.0, bx2, by2)?;

    // Text: "COMPUTERUSE  AI"
    set_color("Black")?;
    let ty = by1 + 0.008;
    let th = 0.034;
    let dw = 0.011;

    let letters: &[(&str, f64)] = &[
        ("C", 0.380),
        ("O", 0.398),
        ("M", 0.416),
        ("P", 0.434),
        ("U", 0.452),
        ("T", 0.470),
        ("E", 0.488),
        ("R", 0.506),
        ("U", 0.524),
        ("S", 0.542),
        ("E", 0.560),
        ("A", 0.588),
        ("I", 0.606),
    ];

    for &(ch, lx) in letters {
        match ch {
            "C" => {
                canvas.line_norm(lx + dw, ty, lx, ty)?;
                canvas.line_norm(lx, ty, lx, ty + th)?;
                canvas.line_norm(lx, ty + th, lx + dw, ty + th)?;
            }
            "O" => {
                canvas.line_norm(lx, ty, lx + dw, ty)?;
                canvas.line_norm(lx + dw, ty, lx + dw, ty + th)?;
                canvas.line_norm(lx + dw, ty + th, lx, ty + th)?;
                canvas.line_norm(lx, ty + th, lx, ty)?;
            }
            "M" => {
                canvas.line_norm(lx, ty + th, lx, ty)?;
                canvas.line_norm(lx, ty, lx + dw * 0.5, ty + th * 0.6)?;
                canvas.line_norm(lx + dw * 0.5, ty + th * 0.6, lx + dw, ty)?;
                canvas.line_norm(lx + dw, ty, lx + dw, ty + th)?;
            }
            "P" => {
                canvas.line_norm(lx, ty + th, lx, ty)?;
                canvas.line_norm(lx, ty, lx + dw, ty)?;
                canvas.line_norm(lx + dw, ty, lx + dw, ty + th * 0.55)?;
                canvas.line_norm(lx + dw, ty + th * 0.55, lx, ty + th * 0.55)?;
            }
            "U" => {
                canvas.line_norm(lx, ty, lx, ty + th)?;
                canvas.line_norm(lx, ty + th, lx + dw, ty + th)?;
                canvas.line_norm(lx + dw, ty + th, lx + dw, ty)?;
            }
            "T" => {
                canvas.line_norm(lx - 0.002, ty, lx + dw + 0.002, ty)?;
                canvas.line_norm(lx + dw * 0.5, ty, lx + dw * 0.5, ty + th)?;
            }
            "E" => {
                canvas.line_norm(lx + dw, ty, lx, ty)?;
                canvas.line_norm(lx, ty, lx, ty + th)?;
                canvas.line_norm(lx, ty + th * 0.5, lx + dw * 0.8, ty + th * 0.5)?;
                canvas.line_norm(lx, ty + th, lx + dw, ty + th)?;
            }
            "R" => {
                canvas.line_norm(lx, ty + th, lx, ty)?;
                canvas.line_norm(lx, ty, lx + dw, ty)?;
                canvas.line_norm(lx + dw, ty, lx + dw, ty + th * 0.55)?;
                canvas.line_norm(lx + dw, ty + th * 0.55, lx, ty + th * 0.55)?;
                canvas.line_norm(lx + dw * 0.5, ty + th * 0.55, lx + dw, ty + th)?;
            }
            "S" => {
                canvas.line_norm(lx + dw, ty, lx, ty)?;
                canvas.line_norm(lx, ty, lx, ty + th * 0.5)?;
                canvas.line_norm(lx, ty + th * 0.5, lx + dw, ty + th * 0.5)?;
                canvas.line_norm(lx + dw, ty + th * 0.5, lx + dw, ty + th)?;
                canvas.line_norm(lx + dw, ty + th, lx, ty + th)?;
            }
            "A" => {
                canvas.line_norm(lx, ty + th, lx + dw * 0.5, ty)?;
                canvas.line_norm(lx + dw * 0.5, ty, lx + dw, ty + th)?;
                canvas.line_norm(lx + dw * 0.25, ty + th * 0.65, lx + dw * 0.75, ty + th * 0.65)?;
            }
            "I" => {
                canvas.line_norm(lx, ty, lx + dw, ty)?;
                canvas.line_norm(lx + dw * 0.5, ty, lx + dw * 0.5, ty + th)?;
                canvas.line_norm(lx, ty + th, lx + dw, ty + th)?;
            }
            _ => {}
        }
    }

    // Unpin window
    let _ = ComputerUse::set_window_topmost(paint_win.hwnd, false);

    println!("\n============================================================");
    println!("     MASTERPIECE COMPLETE! CAPTURING HIGH-RES SCREENSHOT    ");
    println!("============================================================");
    let shot = ComputerUse::screenshot_window(paint_win.hwnd)?;
    println!("Screenshot captured: {}x{} px", shot.width, shot.height);

    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&shot.png_base64)
        .map_err(|e| CuError::Internal(format!("Base64 decode error: {}", e)))?;
    std::fs::write("arte_complexa_paint.png", bytes)?;
    println!("Saved final artwork to 'arte_complexa_paint.png'!");

    Ok(())
}
