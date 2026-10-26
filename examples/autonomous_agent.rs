use std::thread;
use std::time::Duration;
use computeruse::{AgentAction, ComputerUse, Observation, Result};

fn main() -> Result<()> {
    println!("============================================================");
    println!("    DYNAMIC AUTONOMOUS COMPUTER-USE LOOP (STATE MACHINE)    ");
    println!("============================================================");

    let goal = "Pesquisar 'Artificial Intelligence' na Wikipedia e extrair o resumo para a área de transferência";
    let mut env = ComputerUse::agent_env().with_settle_delay(350);

    // Dynamic State Machine Memory for the Autonomous Agent
    #[derive(Debug, PartialEq)]
    enum AgentStage {
        LocateAndFocusBrowser,
        OpenNewTab,
        NavigateToTarget,
        WaitForPageAndScroll,
        SelectAndCopyLeadText,
        VerifyExtractedData,
        Finished,
    }

    let mut current_stage = AgentStage::LocateAndFocusBrowser;
    let mut step_count = 0;

    let summary = env.run_loop(goal, 12, |_goal, obs: &Observation| -> Result<AgentAction> {
        step_count += 1;
        println!("\n  [Agent Cognitive Step {}] Current Stage: {:?}", step_count, current_stage);
        println!("  Active Window Detected: \"{}\" [HWND={}]", obs.window_title, obs.window_hwnd);

        match current_stage {
            AgentStage::LocateAndFocusBrowser => {
                if obs.window_title.contains("Chrome") || obs.window_title.contains("Edge") {
                    println!("  Observation: Browser is already in foreground! Transitioning to OpenNewTab.");
                    current_stage = AgentStage::OpenNewTab;
                    Ok(AgentAction::HotkeyStr { combo: "ctrl+t".into() })
                } else {
                    println!("  Observation: Browser is NOT in foreground. Emitting FocusWindow action.");
                    current_stage = AgentStage::OpenNewTab;
                    Ok(AgentAction::FocusWindow { title: "Chrome".into() })
                }
            }

            AgentStage::OpenNewTab => {
                println!("  Observation: Fresh browser tab requested. Ready to navigate.");
                current_stage = AgentStage::NavigateToTarget;
                Ok(AgentAction::TypeText {
                    text: "https://en.wikipedia.org/wiki/Artificial_intelligence".into(),
                })
            }

            AgentStage::NavigateToTarget => {
                // If still on New Tab, send Enter to commit navigation
                if obs.window_title.contains("Novo separador") || obs.window_title.contains("New Tab") {
                    println!("  Observation: URL typed in address bar. Committing navigation with Enter key...");
                    current_stage = AgentStage::WaitForPageAndScroll;
                    Ok(AgentAction::KeyPress { key: "enter".into() })
                } else {
                    current_stage = AgentStage::WaitForPageAndScroll;
                    Ok(AgentAction::Wait { ms: 2500 })
                }
            }

            AgentStage::WaitForPageAndScroll => {
                let title_lower = obs.window_title.to_lowercase();
                if title_lower.contains("artificial intelligence") || title_lower.contains("wikipedia") {
                    println!("  [VERIFIED] Article confirmed loaded: \"{}\"!", obs.window_title);
                    current_stage = AgentStage::SelectAndCopyLeadText;
                    Ok(AgentAction::Scroll { amount: -350, horizontal: Some(false) })
                } else {
                    println!("  Waiting for page rendering to complete...");
                    Ok(AgentAction::Wait { ms: 1500 })
                }
            }

            AgentStage::SelectAndCopyLeadText => {
                println!("  Observation: Selecting article lead text with mouse drag...");
                current_stage = AgentStage::VerifyExtractedData;
                // Drag across central paragraph: from (0.28, 0.45) to (0.60, 0.52)
                Ok(AgentAction::DragRel {
                    from_x: 0.28,
                    from_y: 0.45,
                    to_x: 0.60,
                    to_y: 0.52,
                })
            }

            AgentStage::VerifyExtractedData => {
                println!("  Observation: Copying selected text to Windows clipboard (Ctrl + C)...");
                current_stage = AgentStage::Finished;
                Ok(AgentAction::HotkeyStr { combo: "ctrl+c".into() })
            }

            AgentStage::Finished => {
                let clipboard = obs.clipboard_preview.clone().unwrap_or_default();
                println!("  Observation: Clipboard verification payload: \"{}\"", clipboard);
                Ok(AgentAction::Done {
                    summary: format!("Lead intelligence successfully extracted into clipboard (Preview: {})", clipboard),
                })
            }
        }
    })?;

    println!("\n============================================================");
    println!(" [AUTONOMOUS LOOP RESULT]");
    println!(" {}", summary);
    println!("============================================================");

    // Capture screenshot of final state
    thread::sleep(Duration::from_millis(500));
    if let Ok(shot) = ComputerUse::screenshot_active_window() {
        if let Ok(bytes) = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &shot.png_base64) {
            let _ = std::fs::write("autonomous_loop_verified.png", bytes);
            println!("Saved final verification capture to 'autonomous_loop_verified.png'!");
        }
    }

    Ok(())
}
