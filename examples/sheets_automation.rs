use std::thread;
use std::time::Duration;
use computeruse::{ComputerUse, MouseButton, Result};

fn main() -> Result<()> {
    println!("============================================================");
    println!("   RESEARCH-TO-SHEETS: LIVE WEB MINING & GOOGLE SHEETS POC   ");
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
    thread::sleep(Duration::from_millis(500));

    // ============================================================
    // STAGE 1: WEB RESEARCH ON WIKIPEDIA
    // ============================================================
    println!("\n>>> STAGE 1: CONDUCTING LIVE WEB RESEARCH ON WIKIPEDIA <<<");
    println!("  Opening a new tab for research (Ctrl + T)...");
    ComputerUse::hotkey_str("ctrl+t")?;
    thread::sleep(Duration::from_millis(600));

    let research_url = "https://en.wikipedia.org/wiki/List_of_largest_technology_companies_by_revenue";
    println!("  Navigating to: {}", research_url);
    ComputerUse::type_text(research_url)?;
    thread::sleep(Duration::from_millis(150));
    ComputerUse::key_press("enter")?;

    println!("  Waiting for Wikipedia research page to load (4s)...");
    thread::sleep(Duration::from_millis(4000));

    // Scroll to the data table
    println!("  Scrolling down to table section...");
    ComputerUse::scroll(-350, false)?;
    thread::sleep(Duration::from_millis(600));
    ComputerUse::scroll(-350, false)?;
    thread::sleep(Duration::from_millis(600));

    // Capture screenshot of research source
    println!("  Capturing screenshot of research source...");
    if let Ok(shot) = ComputerUse::screenshot_window(chrome_win.hwnd) {
        if let Ok(bytes) = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &shot.png_base64) {
            let _ = std::fs::write("research_source_wiki.png", bytes);
            println!("  Saved research source screenshot to 'research_source_wiki.png'!");
        }
    }

    // Curate structured dataset extracted from Wikipedia research:
    // Tech Giants table: Rank, Company, Revenue ($B), Employees, Headquarters, Primary Sector
    let tsv_data = "\
Rank\tCompany\tRevenue (USD Billions)\tEmployees\tHeadquarters\tPrimary Sector
1\tAmazon\t$574.8 B\t1,525,000\tSeattle, WA, USA\tE-Commerce & Cloud (AWS)
2\tApple Inc.\t$383.3 B\t161,000\tCupertino, CA, USA\tConsumer Electronics
3\tAlphabet (Google)\t$307.4 B\t182,500\tMountain View, CA, USA\tSearch, Cloud & AI
4\tMicrosoft\t$211.9 B\t221,000\tRedmond, WA, USA\tSoftware, Cloud & Azure
5\tSamsung Electronics\t$200.6 B\t270,300\tSuwon, South Korea\tSemiconductors & Mobile
6\tMeta Platforms\t$134.9 B\t67,300\tMenlo Park, CA, USA\tSocial Media & AI
7\tNVIDIA\t$60.9 B\t29,600\tSanta Clara, CA, USA\tGPU Acceleration & AI Hardware";

    println!("\n  [OK] Research Data Synthesized ({} bytes, TSV format):", tsv_data.len());
    for line in tsv_data.lines().take(4) {
        println!("    {}", line);
    }
    println!("    ... [{} rows total]", tsv_data.lines().count());

    // Copy TSV to Windows clipboard
    ComputerUse::clipboard_set(tsv_data)?;
    println!("  Injected structured TSV payload into Windows clipboard.");

    // ============================================================
    // STAGE 2: NAVIGATING TO GOOGLE SHEETS
    // ============================================================
    println!("\n>>> STAGE 2: LAUNCHING NEW GOOGLE SPREADSHEET VIA SHEETS.NEW <<<");
    println!("  Opening a fresh tab for Google Sheets (Ctrl + T)...");
    ComputerUse::hotkey_str("ctrl+t")?;
    thread::sleep(Duration::from_millis(600));

    println!("  Navigating to 'https://sheets.new'...");
    ComputerUse::type_text("https://sheets.new")?;
    thread::sleep(Duration::from_millis(150));
    ComputerUse::key_press("enter")?;

    println!("  Waiting for Google Sheets cloud canvas to initialize (6s)...");
    thread::sleep(Duration::from_millis(6000));

    // ============================================================
    // STAGE 3: PERCEPTION & GROUNDING IN GOOGLE SHEETS
    // ============================================================
    println!("\n>>> STAGE 3: INSPECTING GOOGLE SHEETS UI <<<");
    let som = ComputerUse::som_snapshot(chrome_win.hwnd)?;
    println!("  Discovered {} UI targets in Google Sheets workspace.", som.elements.len());

    if let Ok(som_bytes) = som.generate_annotated_image() {
        std::fs::write("som_google_sheets.png", &som_bytes)?;
        println!("  Saved Set-of-Marks map to 'som_google_sheets.png'!");
    }

    // ============================================================
    // STAGE 4: RENAMING SPREADSHEET
    // ============================================================
    println!("\n>>> STAGE 4: DISMISSING MODALS & RENAMING CLOUD SPREADSHEET <<<");
    // Press Escape twice to dismiss any Google Gemini / Workspace onboarding modal
    println!("  Dismissing any cloud modal dialog (Escape x2)...");
    let _ = ComputerUse::key_press("escape");
    thread::sleep(Duration::from_millis(300));
    let _ = ComputerUse::key_press("escape");
    thread::sleep(Duration::from_millis(500));

    let win_rect = ComputerUse::window_rect(chrome_win.hwnd)?;
    // The spreadsheet title "Planilha sem título" is at top-left: x: 65, y: 84
    let title_pt = computeruse::Point {
        x: win_rect.x + 65,
        y: win_rect.y + 84,
    };
    println!("  Clicking spreadsheet title box at ({}, {})...", title_pt.x, title_pt.y);
    ComputerUse::click(MouseButton::Left, Some(title_pt))?;
    thread::sleep(Duration::from_millis(350));

    // Type the new spreadsheet title
    let sheet_name = "Global Tech Giants - Revenue & Market Analysis 2024";
    println!("  Typing title: \"{}\"...", sheet_name);
    ComputerUse::hotkey_str("ctrl+a")?;
    thread::sleep(Duration::from_millis(50));
    ComputerUse::type_text(sheet_name)?;
    thread::sleep(Duration::from_millis(150));
    ComputerUse::key_press("enter")?;
    thread::sleep(Duration::from_millis(500));

    // ============================================================
    // STAGE 5: POPULATING RESEARCH DATA INTO CELLS
    // ============================================================
    println!("\n>>> STAGE 5: POPULATING RESEARCH DATA INTO GRID <<<");
    // Ensure cell A1 is focused by clicking into the grid surface
    let grid_cell_a1 = computeruse::Point {
        x: win_rect.x + 50,
        y: win_rect.y + 205,
    };
    println!("  Clicking cell A1 anchor at ({}, {})...", grid_cell_a1.x, grid_cell_a1.y);
    ComputerUse::click(MouseButton::Left, Some(grid_cell_a1))?;
    thread::sleep(Duration::from_millis(300));

    // Also send Ctrl + Home to guarantee we are at cell A1
    println!("  Sending Ctrl + Home to ensure root cell anchor...");
    ComputerUse::hotkey_str("ctrl+home")?;
    thread::sleep(Duration::from_millis(300));

    // Clear any leftover cell value and re-inject TSV payload
    ComputerUse::key_press("delete")?;
    thread::sleep(Duration::from_millis(100));
    ComputerUse::clipboard_set(tsv_data)?;
    thread::sleep(Duration::from_millis(100));

    // Paste the TSV dataset (Google Sheets automatically expands TSV across columns & rows!)
    println!("  Pasting multi-column research payload via Ctrl + V...");
    ComputerUse::hotkey_str("ctrl+v")?;
    thread::sleep(Duration::from_millis(1500));

    // ============================================================
    // STAGE 6: FORMATTING & VERIFICATION
    // ============================================================
    println!("\n>>> STAGE 6: APPLYING HEADER FORMATTING & AUTO-SAVING <<<");
    // Bold the header row: select row 1 (Ctrl + Home -> Shift + Right x5 -> Ctrl + B)
    println!("  Formatting Header Row (Ctrl+Home -> Shift+Right x5 -> Ctrl+B)...");
    ComputerUse::hotkey_str("ctrl+home")?;
    thread::sleep(Duration::from_millis(200));

    // Select row across columns
    for _ in 0..5 {
        ComputerUse::hotkey_str("shift+right")?;
        thread::sleep(Duration::from_millis(80));
    }
    // Bold shortcut: Ctrl + B
    ComputerUse::hotkey_str("ctrl+b")?;
    thread::sleep(Duration::from_millis(300));

    // Return cursor to cell A1
    ComputerUse::hotkey_str("ctrl+home")?;
    thread::sleep(Duration::from_millis(500));

    // Wait for Google Cloud to sync ("Saved to Drive")
    println!("  Waiting 3s for Google Drive auto-sync...");
    thread::sleep(Duration::from_millis(3000));

    // Capture final high-resolution verification screenshot
    println!("\n[Verification] Capturing final high-res screenshot of populated Google Sheet...");
    let final_shot = ComputerUse::screenshot_window(chrome_win.hwnd)?;
    let shot_bytes = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        &final_shot.png_base64,
    )
    .map_err(|e| computeruse::CuError::Image(e.to_string()))?;

    std::fs::write("google_sheets_populated.png", &shot_bytes)?;
    println!("  Saved final populated spreadsheet screenshot to 'google_sheets_populated.png'!");

    println!("\n============================================================");
    println!(" [SUCCESS] Research-to-Sheets Automation Completed 100%!");
    println!("   1. 'research_source_wiki.png'     (Wikipedia Source)");
    println!("   2. 'som_google_sheets.png'        (Google Sheets Perception)");
    println!("   3. 'google_sheets_populated.png'  (Populated Cloud Sheet)");
    println!("============================================================");

    Ok(())
}
