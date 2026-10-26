use std::thread;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use crate::clipboard;
use crate::error::{CuError, Result};
use crate::keyboard;
use crate::mouse;
use crate::screen;
use crate::som::{capture_som, InteractiveElement, SomSnapshot};
use crate::types::{MouseButton, Point, Rect, WindowInfo};
use crate::window;

/// A perception snapshot delivered to an LLM at each turn of the agent loop.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
    pub window_title: String,
    pub window_hwnd: isize,
    pub window_rect: Rect,
    pub is_maximized: bool,
    pub cursor_pos: Point,
    pub clipboard_preview: Option<String>,
    pub marks: Vec<InteractiveElement>,
    pub compact_prompt: String,
}

/// An atomic action emitted by the LLM agent to interact with the OS.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum AgentAction {
    /// Click directly on a recognized UI element by its Set-of-Marks numeric ID
    ClickMark { mark_id: usize },
    /// Click directly on a recognized UI element by name substring
    ClickName { name: String },
    /// Click relative to the active window in normalized coordinates (0.0 .. 1.0)
    ClickRel {
        x: f64,
        y: f64,
        #[serde(default)]
        button: Option<MouseButton>,
    },
    /// Click on absolute screen coordinates (x, y)
    ClickCoords {
        x: i32,
        y: i32,
        #[serde(default)]
        button: Option<MouseButton>,
    },
    /// Double click on a recognized mark
    DoubleClickMark { mark_id: usize },
    /// Type text into the currently focused element
    TypeText { text: String },
    /// Press a single key (e.g., "enter", "escape", "tab", "delete", "f5")
    KeyPress { key: String },
    /// Send a hotkey combination (e.g., ["ctrl", "t"] or ["alt", "f4"])
    Hotkey { keys: Vec<String> },
    /// Send a hotkey combo string (e.g., "ctrl+t", "ctrl+v", "alt+tab")
    HotkeyStr { combo: String },
    /// Scroll the mouse wheel
    Scroll {
        amount: i32,
        #[serde(default)]
        horizontal: Option<bool>,
    },
    /// Drag the mouse across normalized window coordinates
    DragRel {
        from_x: f64,
        from_y: f64,
        to_x: f64,
        to_y: f64,
    },
    /// Bring a window to focus by title substring
    FocusWindow { title: String },
    /// Set text into the Windows clipboard
    ClipboardSet { text: String },
    /// Wait for a duration in milliseconds
    Wait { ms: u64 },
    /// Mission completed successfully
    Done { summary: String },
    /// Mission failed
    Fail { reason: String },
}

/// Result returned to the LLM after executing a step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    pub success: bool,
    pub message: String,
    pub is_terminal: bool,
    pub observation: Observation,
}

/// The stateful environment running the dynamic Agent perception-action loop.
pub struct AgentEnvironment {
    last_som: Option<SomSnapshot>,
    settle_delay_ms: u64,
}

impl Default for AgentEnvironment {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentEnvironment {
    /// Creates a new agent environment.
    pub fn new() -> Self {
        Self {
            last_som: None,
            settle_delay_ms: 250,
        }
    }

    /// Sets the settling delay after each action (default: 250ms).
    pub fn with_settle_delay(mut self, ms: u64) -> Self {
        self.settle_delay_ms = ms;
        self
    }

    /// Observes the current state of the active desktop and foreground window.
    pub fn observe(&mut self) -> Result<Observation> {
        let win = window::foreground_window().unwrap_or(WindowInfo {
            hwnd: 0,
            title: "Desktop".into(),
            class_name: "Desktop".into(),
            pid: 0,
            rect: Rect { x: 0, y: 0, width: 1920, height: 1080 },
            is_visible: true,
            is_minimized: false,
            is_maximized: false,
        });

        let cursor_pos = screen::cursor_position().unwrap_or(Point { x: 0, y: 0 });
        let clipboard_preview = clipboard::get_text().ok().map(|s| {
            if s.len() > 100 {
                format!("{}... ({} chars)", &s[..100], s.len())
            } else {
                s
            }
        });

        // Capture Set-of-Marks snapshot of foreground window
        let som = capture_som(win.hwnd).unwrap_or(SomSnapshot {
            hwnd: win.hwnd,
            title: win.title.clone(),
            window_rect: win.rect,
            elements: Vec::new(),
        });

        let compact_prompt = som.to_compact_prompt();
        let marks = som.elements.clone();
        self.last_som = Some(som);

        Ok(Observation {
            window_title: win.title,
            window_hwnd: win.hwnd,
            window_rect: win.rect,
            is_maximized: win.is_maximized,
            cursor_pos,
            clipboard_preview,
            marks,
            compact_prompt,
        })
    }

    /// Executes a single atomic action decided by the LLM and returns the new observation.
    pub fn step(&mut self, action: AgentAction) -> Result<StepResult> {
        let message;
        let mut is_terminal = false;

        match action {
            AgentAction::ClickMark { mark_id } => {
                if let Some(ref som) = self.last_som {
                    if let Some(el) = som.find_by_mark(mark_id) {
                        mouse::click(MouseButton::Left, Some(el.center))?;
                        message = format!("Clicked Mark [{:02}] \"{}\" at ({}, {})", mark_id, el.name, el.center.x, el.center.y);
                    } else {
                        return Err(CuError::NotFound(format!("Mark ID [{}] not found in current perception snapshot", mark_id)));
                    }
                } else {
                    return Err(CuError::Internal("No perception snapshot available. Call observe() first.".into()));
                }
            }

            AgentAction::ClickName { name } => {
                if let Some(ref som) = self.last_som {
                    if let Some(el) = som.find_by_name(&name) {
                        mouse::click(MouseButton::Left, Some(el.center))?;
                        message = format!("Clicked Element \"{}\" (Mark [{:02}]) at ({}, {})", el.name, el.mark_id, el.center.x, el.center.y);
                    } else {
                        return Err(CuError::NotFound(format!("Element with name matching \"{}\" not found", name)));
                    }
                } else {
                    return Err(CuError::Internal("No perception snapshot available. Call observe() first.".into()));
                }
            }

            AgentAction::ClickRel { x, y, button } => {
                let btn = button.unwrap_or(MouseButton::Left);
                if let Some(ref som) = self.last_som {
                    let r = som.window_rect;
                    let abs_x = r.x + ((r.width as f64) * x.clamp(0.0, 1.0)).round() as i32;
                    let abs_y = r.y + ((r.height as f64) * y.clamp(0.0, 1.0)).round() as i32;
                    mouse::click(btn, Some(Point { x: abs_x, y: abs_y }))?;
                    message = format!("Clicked window-relative ({:.2}, {:.2}) -> Screen ({}, {})", x, y, abs_x, abs_y);
                } else {
                    return Err(CuError::Internal("No active window to calculate relative coordinates".into()));
                }
            }

            AgentAction::ClickCoords { x, y, button } => {
                let btn = button.unwrap_or(MouseButton::Left);
                mouse::click(btn, Some(Point { x, y }))?;
                message = format!("Clicked absolute coords ({}, {})", x, y);
            }

            AgentAction::DoubleClickMark { mark_id } => {
                if let Some(ref som) = self.last_som {
                    if let Some(el) = som.find_by_mark(mark_id) {
                        mouse::double_click(MouseButton::Left, Some(el.center))?;
                        message = format!("Double-clicked Mark [{:02}] \"{}\"", mark_id, el.name);
                    } else {
                        return Err(CuError::NotFound(format!("Mark ID [{}] not found", mark_id)));
                    }
                } else {
                    return Err(CuError::Internal("No perception snapshot available.".into()));
                }
            }

            AgentAction::TypeText { text } => {
                keyboard::type_text(&text)?;
                message = format!("Typed {} characters into focused target", text.len());
            }

            AgentAction::KeyPress { key } => {
                keyboard::key_press(&key)?;
                message = format!("Pressed key '{}'", key);
            }

            AgentAction::Hotkey { keys } => {
                keyboard::hotkey(&keys)?;
                message = format!("Dispatched hotkey combo: {:?}", keys);
            }

            AgentAction::HotkeyStr { combo } => {
                keyboard::combo_str(&combo)?;
                message = format!("Dispatched hotkey: '{}'", combo);
            }

            AgentAction::Scroll { amount, horizontal } => {
                let is_horiz = horizontal.unwrap_or(false);
                mouse::scroll(amount, is_horiz)?;
                message = format!("Scrolled amount: {} (horizontal: {})", amount, is_horiz);
            }

            AgentAction::DragRel { from_x, from_y, to_x, to_y } => {
                if let Some(ref som) = self.last_som {
                    let r = som.window_rect;
                    let p1 = Point {
                        x: r.x + ((r.width as f64) * from_x.clamp(0.0, 1.0)).round() as i32,
                        y: r.y + ((r.height as f64) * from_y.clamp(0.0, 1.0)).round() as i32,
                    };
                    let p2 = Point {
                        x: r.x + ((r.width as f64) * to_x.clamp(0.0, 1.0)).round() as i32,
                        y: r.y + ((r.height as f64) * to_y.clamp(0.0, 1.0)).round() as i32,
                    };
                    mouse::drag(p1, p2, MouseButton::Left)?;
                    message = format!("Dragged from ({:.2}, {:.2}) to ({:.2}, {:.2})", from_x, from_y, to_x, to_y);
                } else {
                    return Err(CuError::Internal("No active window to calculate drag coordinates".into()));
                }
            }

            AgentAction::FocusWindow { title } => {
                let win = window::find_window(&title)?;
                window::focus_window(win.hwnd)?;
                message = format!("Focused window HWND={} \"{}\"", win.hwnd, win.title);
            }

            AgentAction::ClipboardSet { text } => {
                clipboard::set_text(&text)?;
                message = format!("Copied {} bytes into Windows clipboard", text.len());
            }

            AgentAction::Wait { ms } => {
                thread::sleep(Duration::from_millis(ms));
                message = format!("Waited {} ms", ms);
            }

            AgentAction::Done { summary } => {
                message = format!("MISSION ACCOMPLISHED: {}", summary);
                is_terminal = true;
            }

            AgentAction::Fail { reason } => {
                message = format!("MISSION FAILED: {}", reason);
                is_terminal = true;
            }
        }

        // Settling time for Windows UI animations and event queue
        if self.settle_delay_ms > 0 && !is_terminal {
            thread::sleep(Duration::from_millis(self.settle_delay_ms));
        }

        // Return new observation
        let new_obs = self.observe()?;
        Ok(StepResult {
            success: true,
            message,
            is_terminal,
            observation: new_obs,
        })
    }

    /// Autonomous loop: runs iteratively until the brain decides `Done` or `Fail`, or `max_steps` is reached.
    pub fn run_loop<F>(
        &mut self,
        goal: &str,
        max_steps: usize,
        mut brain: F,
    ) -> Result<String>
    where
        F: FnMut(&str, &Observation) -> Result<AgentAction>,
    {
        println!("============================================================");
        println!("         DYNAMIC COMPUTER-USE AUTONOMOUS AGENT LOOP         ");
        println!("============================================================");
        println!("Goal: \"{}\"", goal);
        println!("Max Allowed Steps: {}\n", max_steps);

        let mut obs = self.observe()?;

        for step_idx in 1..=max_steps {
            println!("\n>>> STEP {}/{} | ACTIVE WINDOW: \"{}\" <<<", step_idx, max_steps, obs.window_title);
            println!("Interactive Marks visible: {}", obs.marks.len());

            // Brain decides next atomic action based on current observation
            let action = brain(goal, &obs)?;
            let action_debug = format!("{:?}", action);
            println!("  Brain Decision: {}", action_debug);

            let res = self.step(action)?;
            println!("  Action Feedback: {}", res.message);

            if res.is_terminal {
                println!("\n[TERMINAL STATE REACHED]");
                return Ok(res.message);
            }

            obs = res.observation;
        }

        Err(CuError::Internal(format!("Exceeded maximum of {} steps without completing goal", max_steps)))
    }
}
