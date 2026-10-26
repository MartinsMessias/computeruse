use std::thread;
use std::time::Duration;
use computeruse::{ComputerUse, Result};

fn main() -> Result<()> {
    println!("============================================================");
    println!("        BROWSER AUTOMATION DEMO WITH COMPUTERUSE            ");
    println!("============================================================");

    // 1. Locate Chrome window
    let windows = ComputerUse::list_windows(false)?;
    let chrome_win = windows
        .into_iter()
        .find(|w| w.title.contains("Chrome") || w.title.contains("Edge"))
        .expect("No Chrome or Edge browser window found!");

    println!("Found Browser Window: HWND={} \"{}\"", chrome_win.hwnd, chrome_win.title);
    println!("Bringing browser to focus...");
    ComputerUse::focus_window(chrome_win.hwnd)?;
    thread::sleep(Duration::from_millis(500));

    // 2. Generate Set-of-Marks snapshot of the browser
    println!("\n[1/5] Generating Set-of-Marks (SoM) perception map of the browser...");
    let som = ComputerUse::som_snapshot(chrome_win.hwnd)?;
    println!("Discovered {} interactive elements in browser UI.", som.elements.len());

    let preview: String = som.to_compact_prompt().lines().take(15).collect::<Vec<_>>().join("\n");
    println!("--- Browser SoM Preview ---\n{}\n... [truncated]\n---------------------------", preview);

    if let Ok(annotated_bytes) = som.generate_annotated_image() {
        std::fs::write("som_browser.png", &annotated_bytes)?;
        println!("Saved browser Set-of-Marks map to 'som_browser.png'!");
    }

    // 3. Open a new tab and navigate to a URL
    println!("\n[2/5] Opening a new tab via hotkey (Ctrl + T)...");
    ComputerUse::hotkey_str("ctrl+t")?;
    thread::sleep(Duration::from_millis(800));

    let target_url = "https://en.wikipedia.org/wiki/Rust_(programming_language)";
    println!("[3/5] Navigating to Wikipedia: '{}'...", target_url);
    ComputerUse::type_text(target_url)?;
    thread::sleep(Duration::from_millis(150));
    ComputerUse::key_press("enter")?;

    println!("Waiting for page to load (3.5 seconds)...");
    thread::sleep(Duration::from_millis(3500));

    // 4. Smooth scrolling down to explore page content
    println!("\n[4/5] Simulating natural human reading: scrolling down the page...");
    for i in 1..=4 {
        ComputerUse::scroll(-350, false)?;
        println!("  Scroll step {}/4 executed.", i);
        thread::sleep(Duration::from_millis(600));
    }

    // 5. Capture high-resolution screenshot of the navigated web page
    println!("\n[5/5] Capturing high-resolution verification screenshot of the web page...");
    let shot = ComputerUse::screenshot_window(chrome_win.hwnd)?;
    let shot_bytes = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        &shot.png_base64,
    )
    .map_err(|e| computeruse::CuError::Image(e.to_string()))?;

    std::fs::write("browser_wiki_rust.png", &shot_bytes)?;
    println!("Saved browser screenshot to 'browser_wiki_rust.png'!");

    println!("\n============================================================");
    println!(" [SUCCESS] Browser Automation Completed!");
    println!("   1. 'som_browser.png'        (Set-of-Marks on Browser)");
    println!("   2. 'browser_wiki_rust.png'  (Navigated & Scrolled Page)");
    println!("============================================================");

    Ok(())
}
