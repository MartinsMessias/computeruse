use std::f64::consts::PI;
use std::thread;
use std::time::Duration;
use computeruse::{ComputerUse, Result};

fn main() -> Result<()> {
    println!("============================================================");
    println!("    POC: NEURO-SYMBOLIC COMPUTER-USE (SET-OF-MARKS + CANVAS)");
    println!("============================================================");

    // 1. Locate and Focus Paint
    let windows = ComputerUse::list_windows(false)?;
    let paint_win = windows
        .iter()
        .find(|w| w.title.contains("Paint"))
        .cloned()
        .or_else(|| {
            // If not found, try to launch it via Win+R
            let _ = ComputerUse::launch("mspaint", None);
            thread::sleep(Duration::from_millis(1500));
            ComputerUse::wait_for_window("Paint", 5000).ok()
        })
        .expect("Paint window not found!");

    println!("Target Window: HWND={} \"{}\"", paint_win.hwnd, paint_win.title);
    ComputerUse::focus_window(paint_win.hwnd)?;
    thread::sleep(Duration::from_millis(300));

    // Clear canvas: Ctrl + A -> Delete
    println!("\n[Reflex] Clearing Paint canvas with Ctrl+A -> Delete...");
    let _ = ComputerUse::hotkey_str("ctrl+a");
    thread::sleep(Duration::from_millis(150));
    let _ = ComputerUse::key_press("delete");
    thread::sleep(Duration::from_millis(250));

    // 2. Set-of-Marks (SoM) Snapshot
    println!("\n[Perception] Building Set-of-Marks (SoM) interactive catalog...");
    let som = ComputerUse::som_snapshot(paint_win.hwnd)?;
    println!("  Total interactive elements discovered: {}", som.elements.len());

    // Print top 20 interactive marks to show LLM token efficiency
    println!("\n--- LLM COMPACT PROMPT PREVIEW ---");
    let preview: String = som.to_compact_prompt().lines().take(22).collect::<Vec<_>>().join("\n");
    println!("{}\n... [truncated for brevity] ...\n---------------------------------", preview);

    // Generate annotated screenshot with visual badges for the LLM
    println!("\n[Vision] Generating Set-of-Marks annotated image with high-contrast badges...");
    if let Ok(annotated_bytes) = som.generate_annotated_image() {
        std::fs::write("som_annotated_paint.png", &annotated_bytes)?;
        println!("  Saved annotated perception map to 'som_annotated_paint.png'!");
    }

    // 3. Dynamic Canvas Detection
    println!("\n[Canvas] Dynamically resolving drawing surface inside window...");
    let canvas = ComputerUse::canvas_find_in_window(paint_win.hwnd)?;
    println!(
        "  Canvas resolved: Origin=({}, {}) Dimensions={}x{} AspectRatio={:.2}",
        canvas.origin_x, canvas.origin_y, canvas.width, canvas.height, canvas.aspect_ratio
    );

    // 4. Neuro-Symbolic Reflex: Select drawing tool (Pencil) via SoM
    println!("\n[Reflex] Selecting Pencil drawing tool via Set-of-Marks...");
    if let Some(pencil) = som.find_by_name("Pencil") {
        println!("  Clicking Mark [{}] for tool \"{}\"", pencil.mark_id, pencil.name);
        som.click_mark(pencil.mark_id)?;
    } else {
        let _ = ComputerUse::uia_click_by_name("Pencil");
    }
    thread::sleep(Duration::from_millis(250));

    // Helper: Select color using Set-of-Marks or fallback
    let select_color = |name: &str| -> Result<()> {
        println!("  [SoM Action] Selecting color '{}'...", name);
        if let Some(el) = som.find_by_name(name) {
            ComputerUse::click(computeruse::MouseButton::Left, Some(el.center))?;
        } else {
            // Fallback to direct UIA click
            let _ = ComputerUse::uia_click_by_name(name);
        }
        thread::sleep(Duration::from_millis(150));
        Ok(())
    };

    // ============================================================
    // THE POC MASTERPIECE: CYBERNETIC CHRONOMETER & ASTRONOMICAL GEARS
    // ============================================================
    println!("\n>>> EXECUTING COMPLEX MULTI-LAYERED CYBERNETIC CHRONOMETER <<<");

    // 1. Outer Tech-Grid Frame (Black)
    select_color("Black")?;
    println!("[Layer 1/6] Technical Frame and Corner Brackets...");
    canvas.stroke_line(0.02, 0.03, 0.98, 0.03)?;
    canvas.stroke_line(0.98, 0.03, 0.98, 0.97)?;
    canvas.stroke_line(0.98, 0.97, 0.02, 0.97)?;
    canvas.stroke_line(0.02, 0.97, 0.02, 0.03)?;

    // Corner targeting brackets
    let b_len = 0.04;
    // Top-left
    canvas.stroke_line(0.04, 0.06, 0.04 + b_len, 0.06)?;
    canvas.stroke_line(0.04, 0.06, 0.04, 0.06 + b_len)?;
    // Top-right
    canvas.stroke_line(0.96, 0.06, 0.96 - b_len, 0.06)?;
    canvas.stroke_line(0.96, 0.06, 0.96, 0.06 + b_len)?;
    // Bottom-left
    canvas.stroke_line(0.04, 0.94, 0.04 + b_len, 0.94)?;
    canvas.stroke_line(0.04, 0.94, 0.04, 0.94 - b_len)?;
    // Bottom-right
    canvas.stroke_line(0.96, 0.94, 0.96 - b_len, 0.94)?;
    canvas.stroke_line(0.96, 0.94, 0.96, 0.94 - b_len)?;

    // 2. Main Central Gear Train (Orange & Gold)
    let cx = 0.50;
    let cy = 0.48;

    select_color("Orange")?;
    println!("[Layer 2/6] Precision Mechanical Gears & Intermeshing Teeth...");
    // Main 18-tooth drive gear
    canvas.stroke_gear(cx, cy, 0.42, 0.38, 18, 0.0)?;
    canvas.stroke_polygon(cx, cy, 0.36, 36, 0.0)?;

    // Secondary 12-tooth planetary gear (Lavender / Purple)
    select_color("Lavender")?;
    let p_cx = 0.26;
    let p_cy = 0.48;
    canvas.stroke_gear(p_cx, p_cy, 0.22, 0.19, 12, PI / 12.0)?;
    canvas.stroke_polygon(p_cx, p_cy, 0.17, 24, 0.0)?;
    canvas.stroke_star(p_cx, p_cy, 0.15, 0.06, 6, 0.0)?;

    // Right gear (Turquoise)
    select_color("Turquoise")?;
    let r_cx = 0.74;
    let r_cy = 0.48;
    canvas.stroke_gear(r_cx, r_cy, 0.22, 0.19, 12, 0.0)?;
    canvas.stroke_polygon(r_cx, r_cy, 0.17, 24, 0.0)?;
    canvas.stroke_star(r_cx, r_cy, 0.15, 0.06, 6, PI / 6.0)?;

    // 3. Central Archimedean Vortex & Star Core (Yellow & Red)
    select_color("Yellow")?;
    println!("[Layer 3/6] Archimedean Energy Vortex & Concentric Dials...");
    canvas.stroke_spiral(cx, cy, 0.04, 0.33, 3.5, 90)?;
    canvas.stroke_polygon(cx, cy, 0.28, 24, 0.0)?;

    select_color("Red")?;
    println!("[Layer 4/6] Sacred Radial Spokes & Metatron Core...");
    // 12 Radial Dial Spokes
    for i in 0..12 {
        let angle = (i as f64 / 12.0) * 2.0 * PI;
        let p_in = canvas.polar_pt(cx, cy, 0.18, angle);
        let p_out = canvas.polar_pt(cx, cy, 0.35, angle);
        ComputerUse::drag(p_in, p_out, computeruse::MouseButton::Left)?;
    }
    // Hexagram in red
    canvas.stroke_polygon(cx, cy, 0.16, 3, 0.0)?;
    canvas.stroke_polygon(cx, cy, 0.16, 3, PI)?;
    // Core 8-point jewel star
    select_color("Black")?;
    canvas.stroke_star(cx, cy, 0.08, 0.03, 8, 0.0)?;

    // 4. Smooth Bézier Energy Arches & Plasma Waves (Turquoise & Rose)
    select_color("Turquoise")?;
    println!("[Layer 5/6] Smooth Cubic Bézier Energy Arches...");
    // Upper plasma bridge
    canvas.stroke_bezier(
        (0.12, 0.22),
        (0.30, 0.06),
        (0.70, 0.06),
        (0.88, 0.22),
        32,
    )?;
    canvas.stroke_bezier(
        (0.12, 0.24),
        (0.30, 0.09),
        (0.70, 0.09),
        (0.88, 0.24),
        32,
    )?;
    // Lower plasma bridge
    select_color("Rose")?;
    canvas.stroke_bezier(
        (0.12, 0.74),
        (0.30, 0.90),
        (0.70, 0.90),
        (0.88, 0.74),
        32,
    )?;
    canvas.stroke_bezier(
        (0.12, 0.72),
        (0.30, 0.87),
        (0.70, 0.87),
        (0.88, 0.72),
        32,
    )?;

    // 5. Digital Vector Typography (Red & Black)
    println!("[Layer 6/6] Digital Vector Typography & Data Readouts...");
    select_color("Red")?;
    // Brand header
    canvas.stroke_text(0.33, 0.12, "NEURO-SYMBOLIC CU", 0.024)?;
    // Secondary status telemetry
    select_color("Black")?;
    canvas.stroke_text(0.38, 0.85, "VERIFIED GROUNDING", 0.018)?;

    // Telemetry tick marks
    canvas.stroke_line(0.30, 0.18, 0.70, 0.18)?;
    canvas.stroke_line(0.30, 0.82, 0.70, 0.82)?;

    // Final Screenshot Capture
    println!("\n[Verification] Capturing final high-res verification screenshot...");
    thread::sleep(Duration::from_millis(300));
    let shot = ComputerUse::screenshot_window(paint_win.hwnd)?;
    let shot_bytes = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        &shot.png_base64,
    )
    .map_err(|e| computeruse::CuError::Image(e.to_string()))?;

    std::fs::write("poc_masterpiece.png", &shot_bytes)?;
    println!("\n============================================================");
    println!(" [SUCCESS] PoC Complete! Generated:");
    println!("   1. 'som_annotated_paint.png' (Set-of-Marks Perception Map)");
    println!("   2. 'poc_masterpiece.png'     (Final Cybernetic Chronometer)");
    println!("============================================================");

    Ok(())
}
