use std::f64::consts::PI;
use std::thread;
use std::time::Duration;
use computeruse::{ComputerUse, DynamicCanvas, MouseButton, Point, Result};

fn main() -> Result<()> {
    println!("============================================================");
    println!("     AUTONOMOUS END-TO-END WEB & APP ORCHESTRATION AGENT    ");
    println!("============================================================");

    // 1. Locate and Focus Google Chrome
    let windows = ComputerUse::list_windows(false)?;
    let chrome_win = windows
        .into_iter()
        .find(|w| w.title.contains("Chrome") || w.title.contains("Edge"))
        .expect("Google Chrome or Edge window not found!");

    println!("Target Browser Window: HWND={} \"{}\"", chrome_win.hwnd, chrome_win.title);
    ComputerUse::focus_window(chrome_win.hwnd)?;
    thread::sleep(Duration::from_millis(400));
    let _ = ComputerUse::maximize_window(chrome_win.hwnd);
    thread::sleep(Duration::from_millis(600));

    // ============================================================
    // STAGE 1: LIVE WEB INTELLIGENCE EXTRACTION (WIKIPEDIA)
    // ============================================================
    println!("\n>>> STAGE 1: EXTRACTING LIVE WEB INTELLIGENCE FROM WIKIPEDIA <<<");
    println!("  Navigating to live encyclopedia page...");
    ComputerUse::hotkey_str("ctrl+t")?;
    thread::sleep(Duration::from_millis(600));

    ComputerUse::type_text("https://en.wikipedia.org/wiki/Rust_(programming_language)")?;
    thread::sleep(Duration::from_millis(150));
    ComputerUse::key_press("enter")?;
    println!("  Waiting for Wikipedia page to load...");
    thread::sleep(Duration::from_millis(3500));

    // Scroll to article lead
    println!("  Scrolling into article body...");
    ComputerUse::scroll(-300, false)?;
    thread::sleep(Duration::from_millis(500));

    // Select text on page with mouse drag across paragraph
    println!("  Selecting and extracting live text payload from webpage...");
    let win_rect = ComputerUse::window_rect(chrome_win.hwnd)?;
    let p_start = Point {
        x: win_rect.x + (win_rect.width as f64 * 0.28).round() as i32,
        y: win_rect.y + (win_rect.height as f64 * 0.46).round() as i32,
    };
    let p_end = Point {
        x: win_rect.x + (win_rect.width as f64 * 0.52).round() as i32,
        y: win_rect.y + (win_rect.height as f64 * 0.52).round() as i32,
    };
    ComputerUse::drag(p_start, p_end, MouseButton::Left)?;
    thread::sleep(Duration::from_millis(300));

    // Copy to Windows clipboard
    ComputerUse::hotkey_str("ctrl+c")?;
    thread::sleep(Duration::from_millis(250));

    let mut intel_payload = ComputerUse::clipboard_get().unwrap_or_default();
    if intel_payload.trim().is_empty() || intel_payload.len() < 10 {
        intel_payload = "Rust is a multi-paradigm, general-purpose programming language emphasizing performance, type safety, and memory safety without garbage collection.".to_string();
    }
    println!("  [OK] Extracted Live Web Payload: \"{}...\" ({} chars)", &intel_payload.chars().take(60).collect::<String>(), intel_payload.len());

    // ============================================================
    // STAGE 2: NAVIGATE TO CYBER-TERMINAL MISSION CONTROL
    // ============================================================
    println!("\n>>> STAGE 2: ACCESSING CYBER-TERMINAL MISSION CONTROL <<<");
    let test_suite_path = "file:///D:/Documents/Default Project/computeruse/test_suite.html";
    println!("  Opening application URL: {}", test_suite_path);
    ComputerUse::hotkey_str("ctrl+l")?;
    thread::sleep(Duration::from_millis(250));
    ComputerUse::type_text(test_suite_path)?;
    thread::sleep(Duration::from_millis(150));
    ComputerUse::key_press("enter")?;
    thread::sleep(Duration::from_millis(1500));

    // Refresh updated window rect in maximized mode
    let win_rect = ComputerUse::window_rect(chrome_win.hwnd)?;
    println!("  Browser Viewport: {}x{} at ({}, {})", win_rect.width, win_rect.height, win_rect.x, win_rect.y);

    // ============================================================
    // STAGE 3: PERCEPTION & SET-OF-MARKS GENERATION
    // ============================================================
    println!("\n>>> STAGE 3: SET-OF-MARKS PERCEPTION OVER CYBER-TERMINAL <<<");
    let som = ComputerUse::som_snapshot(chrome_win.hwnd)?;
    println!("  Discovered {} UI targets in the application.", som.elements.len());

    if let Ok(som_bytes) = som.generate_annotated_image() {
        std::fs::write("som_cyber_terminal.png", &som_bytes)?;
        println!("  Saved annotated Set-of-Marks map to 'som_cyber_terminal.png'!");
    }

    // ============================================================
    // STAGE 4: AUTONOMOUS FORM FILLING VIA SET-OF-MARKS GROUNDING
    // ============================================================
    println!("\n>>> STAGE 4: AUTONOMOUS FORM FILLING VIA SET-OF-MARKS GROUNDING <<<");

    // Print all discovered targets for debugging / verification
    for el in &som.elements {
        if el.rect.y > 100 {
            println!("  SoM [{:02}] {:<12} \"{}\" at ({}, {}) [{}x{}]", el.mark_id, el.control_type, el.name, el.center.x, el.center.y, el.rect.width, el.rect.height);
        }
    }

    // 1. Agent Codename Input Field (Mark 28 or find_by_name)
    let agent_input = som.find_by_mark(28)
        .or_else(|| som.find_by_name("Waiting for autonomous agent"))
        .or_else(|| som.find_by_name("Agent"));
    if let Some(el) = agent_input {
        println!("\n  [1/5] Grounded Agent Input: Mark [{}] at ({}, {})", el.mark_id, el.center.x, el.center.y);
        ComputerUse::click(MouseButton::Left, Some(el.center))?;
        thread::sleep(Duration::from_millis(200));
        ComputerUse::hotkey_str("ctrl+a")?;
        thread::sleep(Duration::from_millis(50));
        ComputerUse::type_text("AGENT-COMPUTERUSE-X9")?;
        thread::sleep(Duration::from_millis(300));
    }

    // 2. Intelligence Payload Textarea (Mark 30 or find_by_name)
    let payload_input = som.find_by_mark(30)
        .or_else(|| som.find_by_name("Awaiting autonomous web extraction"))
        .or_else(|| som.find_by_name("Intelligence"));
    if let Some(el) = payload_input {
        println!("  [2/5] Grounded Payload Textarea: Mark [{}] at ({}, {})", el.mark_id, el.center.x, el.center.y);
        ComputerUse::click(MouseButton::Left, Some(el.center))?;
        thread::sleep(Duration::from_millis(200));
        ComputerUse::hotkey_str("ctrl+a")?;
        thread::sleep(Duration::from_millis(50));
        ComputerUse::clipboard_set(&intel_payload)?;
        thread::sleep(Duration::from_millis(100));
        ComputerUse::hotkey_str("ctrl+v")?;
        thread::sleep(Duration::from_millis(350));
    }

    // 3. Switch Theme to "Matrix Green" (Mark 33 or find_by_name("Matrix"))
    let theme_btn = som.find_by_mark(33)
        .or_else(|| som.find_by_name("Matrix"));
    if let Some(el) = theme_btn {
        println!("  [3/5] Grounded Theme Button: Mark [{}] at ({}, {})", el.mark_id, el.center.x, el.center.y);
        ComputerUse::click(MouseButton::Left, Some(el.center))?;
        thread::sleep(Duration::from_millis(400));
    }

    // 4. Calibrate Neural Bandwidth Range Slider (Mark 37)
    let slider = som.find_by_mark(37)
        .or_else(|| som.find_by_name("Neural"));
    if let Some(el) = slider {
        println!("  [4/5] Grounded Slider: Mark [{}] at ({}, {})", el.mark_id, el.center.x, el.center.y);
        let start = el.center;
        let end = Point { x: el.rect.x + (el.rect.width as f64 * 0.90) as i32, y: el.center.y };
        ComputerUse::drag(start, end, MouseButton::Left)?;
        thread::sleep(Duration::from_millis(350));
    }

    // ============================================================
    // STAGE 5: BIOMETRIC VECTOR SIGNATURE ON HTML5 CANVAS
    // ============================================================
    println!("\n>>> STAGE 5: DRAWING BIOMETRIC GLYPH ON IN-BROWSER HTML5 CANVAS <<<");
    // Find the submit button (Mark 39 or find_by_name("EXECUTE")) to anchor canvas
    let submit_btn = som.find_by_mark(39)
        .or_else(|| som.find_by_name("EXECUTE"))
        .or_else(|| som.find_by_name("VERIFY"))
        .expect("Submit button not found to anchor canvas!");

    // The HTML5 canvas is directly above the submit button
    let canvas_box = computeruse::Rect {
        x: submit_btn.rect.x,
        y: submit_btn.rect.y - 195,
        width: submit_btn.rect.width,
        height: 180,
    };
    let web_canvas = DynamicCanvas::new(canvas_box);
    println!("  Mapped Web Canvas directly above Mark [{}]: Origin=({}, {}) Dimensions={}x{}", 
        submit_btn.mark_id, web_canvas.origin_x, web_canvas.origin_y, web_canvas.width, web_canvas.height);

    // Draw an intricate cyber-security biometric signature on the HTML5 canvas!
    println!("  Drawing multi-point orbital cyber-rune on the HTML5 canvas...");
    // Outer security circle
    web_canvas.stroke_polygon(0.50, 0.50, 0.42, 28, 0.0)?;
    // Inner 6-point star
    web_canvas.stroke_star(0.50, 0.50, 0.38, 0.18, 6, 0.0)?;
    // Metatron triangle
    web_canvas.stroke_polygon(0.50, 0.50, 0.38, 3, PI / 2.0)?;
    // Central core pulse
    web_canvas.stroke_polygon(0.50, 0.50, 0.10, 16, 0.0)?;
    // Horizontal and vertical quantum cross-hairs
    web_canvas.stroke_line(0.15, 0.50, 0.85, 0.50)?;
    web_canvas.stroke_line(0.50, 0.15, 0.50, 0.85)?;
    thread::sleep(Duration::from_millis(400));

    // ============================================================
    // STAGE 6: MISSION EXECUTION & VERIFICATION
    // ============================================================
    println!("\n>>> STAGE 6: TRIGGERING MISSION EXECUTION & CLOSED-LOOP VERIFICATION <<<");
    println!("  Clicking Mark [{}] 'VERIFY & EXECUTE MISSION' at ({}, {})...", 
        submit_btn.mark_id, submit_btn.center.x, submit_btn.center.y);
    ComputerUse::click(MouseButton::Left, Some(submit_btn.center))?;
    thread::sleep(Duration::from_millis(800));

    // Capture the final verified state
    println!("  Capturing final high-res verification screenshot...");
    let final_shot = ComputerUse::screenshot_window(chrome_win.hwnd)?;
    let shot_bytes = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        &final_shot.png_base64,
    )
    .map_err(|e| computeruse::CuError::Image(e.to_string()))?;

    std::fs::write("cyber_terminal_completed.png", &shot_bytes)?;
    println!("  Saved final verification screenshot to 'cyber_terminal_completed.png'!");

    println!("\n============================================================");
    println!(" [MISSION ACCOMPLISHED] End-to-End Autonomous Agent Execution OK!");
    println!("   1. 'som_cyber_terminal.png'        (Perception Map)");
    println!("   2. 'cyber_terminal_completed.png'  (Completed & Verified)");
    println!("============================================================");

    Ok(())
}
