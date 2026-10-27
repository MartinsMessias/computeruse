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
    /// Reactively wait until foreground window or UI Automation tree changes (up to timeout_ms)
    WaitForChange { timeout_ms: u64 },
    /// Mission completed successfully
    Done { summary: String },
    /// Mission failed
    Fail { reason: String },
}

/// Captures the structural UI delta produced by an action.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UiDelta {
    pub hwnd_changed: bool,
    pub title_changed: bool,
    pub marks_delta: i32,
    pub summary: String,
}

/// Result returned to the LLM after executing a step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    pub success: bool,
    pub message: String,
    pub is_terminal: bool,
    pub observation: Observation,
    #[serde(default)]
    pub delta: UiDelta,
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

    /// Returns the most recent Set-of-Marks snapshot if available.
    pub fn last_som(&self) -> Option<&SomSnapshot> {
        self.last_som.as_ref()
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
            let char_count = s.chars().count();
            if char_count > 100 {
                let prefix: String = s.chars().take(100).collect();
                format!("{}... ({} chars)", prefix, char_count)
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

    /// Computes the structural UI delta between a baseline (hwnd, title, marks_count) and a new Observation.
    fn compute_delta(prev_hwnd: isize, prev_title: &str, prev_marks: usize, obs: &Observation) -> UiDelta {
        let hwnd_changed = prev_hwnd != 0 && prev_hwnd != obs.window_hwnd;
        let title_changed = !prev_title.is_empty() && prev_title != obs.window_title;
        let marks_delta = (obs.marks.len() as i32) - (prev_marks as i32);

        let summary = if hwnd_changed {
            format!("Window switched -> \"{}\" (HWND={}, {} marks)", obs.window_title, obs.window_hwnd, obs.marks.len())
        } else if title_changed {
            format!("Title changed -> \"{}\" (marks {:+})", obs.window_title, marks_delta)
        } else if marks_delta != 0 {
            format!("UI tree updated ({:+} interactive elements, total {})", marks_delta, obs.marks.len())
        } else {
            "UI structure unchanged".to_string()
        };

        UiDelta {
            hwnd_changed,
            title_changed,
            marks_delta,
            summary,
        }
    }

    /// Reactively polls the OS foreground window, UIA tree, and visual pixel checksum until a change occurs or `timeout_ms` elapses.
    /// Eliminates blind `sleep(2000)` waits by returning immediately as soon as a window/menu/popup or visual update appears.
    pub fn wait_for_change(&mut self, timeout_ms: u64) -> Result<(UiDelta, Observation, u64)> {
        if self.last_som.is_none() {
            let _ = self.observe();
        }
        let (base_hwnd, base_title, base_marks, base_rect) = self.last_som.as_ref()
            .map(|s| (s.hwnd, s.title.clone(), s.elements.len(), s.window_rect))
            .unwrap_or((0, String::new(), 0, Rect { x: 0, y: 0, width: 256, height: 256 }));

        let probe_rect = Rect {
            x: base_rect.x + (base_rect.width as i32 / 4),
            y: base_rect.y + (base_rect.height as i32 / 4),
            width: (base_rect.width / 2).clamp(64, 512),
            height: (base_rect.height / 2).clamp(64, 512),
        };
        let base_visual_hash = crate::pixel::region_visual_checksum(probe_rect).unwrap_or(0);

        let start = std::time::Instant::now();
        let timeout = Duration::from_millis(timeout_ms.max(25));

        loop {
            thread::sleep(Duration::from_millis(25));
            let obs = self.observe()?;
            let mut delta = Self::compute_delta(base_hwnd, &base_title, base_marks, &obs);
            let cur_visual_hash = crate::pixel::region_visual_checksum(probe_rect).unwrap_or(base_visual_hash);
            let visual_changed = base_visual_hash != 0 && cur_visual_hash != base_visual_hash;
            if visual_changed && !delta.hwnd_changed && !delta.title_changed && delta.marks_delta == 0 {
                delta.summary = "Visual canvas/pixel state updated".to_string();
            }
            let elapsed = start.elapsed().as_millis() as u64;

            if delta.hwnd_changed || delta.title_changed || delta.marks_delta != 0 || visual_changed || start.elapsed() >= timeout {
                return Ok((delta, obs, elapsed));
            }
        }
    }

    /// Executes a single atomic action decided by the LLM and returns the new observation.
    pub fn step(&mut self, action: AgentAction) -> Result<StepResult> {
        let (prev_hwnd, prev_title, prev_marks) = self.last_som.as_ref()
            .map(|s| (s.hwnd, s.title.clone(), s.elements.len()))
            .unwrap_or((0, String::new(), 0));

        let message;
        let mut is_terminal = false;
        let mut skip_settle = false;

        match action {
            AgentAction::ClickMark { mark_id } => {
                if self.last_som.is_none() {
                    let _ = self.observe();
                }
                if let Some(ref som) = self.last_som {
                    if let Some(el) = som.find_by_mark(mark_id) {
                        mouse::click(MouseButton::Left, Some(el.center))?;
                        message = format!("Clicked Mark [{:02}] \"{}\" at ({}, {})", mark_id, el.name, el.center.x, el.center.y);
                    } else {
                        return Err(CuError::NotFound(format!("Mark ID [{}] not found in current perception snapshot", mark_id)));
                    }
                } else {
                    return Err(CuError::Internal("No perception snapshot available.".into()));
                }
            }

            AgentAction::ClickName { name } => {
                if self.last_som.is_none() {
                    let _ = self.observe();
                }
                if let Some(ref som) = self.last_som {
                    if let Some(el) = som.find_by_name(&name) {
                        mouse::click(MouseButton::Left, Some(el.center))?;
                        message = format!("Clicked Element \"{}\" (Mark [{:02}]) at ({}, {})", el.name, el.mark_id, el.center.x, el.center.y);
                    } else {
                        return Err(CuError::NotFound(format!("Element with name matching \"{}\" not found", name)));
                    }
                } else {
                    return Err(CuError::Internal("No perception snapshot available.".into()));
                }
            }

            AgentAction::ClickRel { x, y, button } => {
                if self.last_som.is_none() {
                    let _ = self.observe();
                }
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
                if self.last_som.is_none() {
                    let _ = self.observe();
                }
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
                skip_settle = true;
            }

            AgentAction::WaitForChange { timeout_ms } => {
                let (delta, new_obs, elapsed) = self.wait_for_change(timeout_ms)?;
                return Ok(StepResult {
                    success: true,
                    message: format!("Reactively settled in {} ms: {}", elapsed, delta.summary),
                    is_terminal: false,
                    observation: new_obs,
                    delta,
                });
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
        if self.settle_delay_ms > 0 && !is_terminal && !skip_settle {
            thread::sleep(Duration::from_millis(self.settle_delay_ms));
        }

        // Return new observation with structural UI delta
        let new_obs = self.observe()?;
        let delta = Self::compute_delta(prev_hwnd, &prev_title, prev_marks, &new_obs);
        Ok(StepResult {
            success: true,
            message,
            is_terminal,
            observation: new_obs,
            delta,
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

impl AgentAction {
    /// Parses an `AgentAction` from flexible JSON representations used by LLMs in `cu_batch`:
    /// 1. Standard tagged: `{"action": "click_name", "name": "File"}`
    /// 2. Type-tagged: `{"type": "click_name", "name": "File"}` or `{"type": "type", "text": "hello"}`
    /// 3. Outer-keyed: `{"ClickName": {"name": "File"}}`
    pub fn from_flexible_json(val: &serde_json::Value) -> Result<Self> {
        if let Ok(act) = serde_json::from_value::<AgentAction>(val.clone()) {
            return Ok(act);
        }

        if let Some(obj) = val.as_object() {
            // Handle {"type": "..."} or {"action": "..."} with shorthand names
            if let Some(tag) = obj.get("action").or_else(|| obj.get("type")).and_then(|v| v.as_str()) {
                let norm = tag.to_lowercase().replace('-', "_");
                match norm.as_str() {
                    "click_mark" | "clickmark" => {
                        let mark_id = obj.get("mark_id").or_else(|| obj.get("id")).and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                        return Ok(AgentAction::ClickMark { mark_id });
                    }
                    "click_name" | "clickname" => {
                        let name = obj.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        return Ok(AgentAction::ClickName { name });
                    }
                    "click_rel" | "clickrel" => {
                        let x = obj.get("x").and_then(|v| v.as_f64()).unwrap_or(0.5);
                        let y = obj.get("y").and_then(|v| v.as_f64()).unwrap_or(0.5);
                        return Ok(AgentAction::ClickRel { x, y, button: None });
                    }
                    "click_coords" | "clickcoords" | "click" => {
                        let x = obj.get("x").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                        let y = obj.get("y").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                        return Ok(AgentAction::ClickCoords { x, y, button: None });
                    }
                    "type_text" | "typetext" | "type" => {
                        let text = obj.get("text").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        return Ok(AgentAction::TypeText { text });
                    }
                    "key_press" | "keypress" | "key" => {
                        let key = obj.get("key").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        return Ok(AgentAction::KeyPress { key });
                    }
                    "hotkey_str" | "hotkeystr" | "hotkey" => {
                        if let Some(combo) = obj.get("combo").and_then(|v| v.as_str()) {
                            return Ok(AgentAction::HotkeyStr { combo: combo.to_string() });
                        }
                        if let Some(arr) = obj.get("keys").and_then(|v| v.as_array()) {
                            let keys = arr.iter().filter_map(|k| k.as_str().map(|s| s.to_string())).collect();
                            return Ok(AgentAction::Hotkey { keys });
                        }
                    }
                    "scroll" => {
                        let amount = obj.get("amount").and_then(|v| v.as_i64()).unwrap_or(-350) as i32;
                        let horizontal = obj.get("horizontal").and_then(|v| v.as_bool());
                        return Ok(AgentAction::Scroll { amount, horizontal });
                    }
                    "focus_window" | "focuswindow" | "focus" => {
                        let title = obj.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        return Ok(AgentAction::FocusWindow { title });
                    }
                    "clipboard_set" | "clipboardset" => {
                        let text = obj.get("text").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        return Ok(AgentAction::ClipboardSet { text });
                    }
                    "wait" | "sleep" => {
                        let ms = obj.get("ms").and_then(|v| v.as_u64()).unwrap_or(250);
                        return Ok(AgentAction::Wait { ms });
                    }
                    "wait_for_change" | "waitforchange" => {
                        let timeout_ms = obj.get("timeout_ms").or_else(|| obj.get("ms")).and_then(|v| v.as_u64()).unwrap_or(2000);
                        return Ok(AgentAction::WaitForChange { timeout_ms });
                    }
                    _ => {}
                }
            }

            // Handle outer-keyed object: {"ClickName": {"name": "File"}}
            if obj.len() == 1 {
                if let Some((outer_key, inner_val)) = obj.iter().next() {
                    if let Some(inner_obj) = inner_val.as_object() {
                        let mut merged = inner_obj.clone();
                        merged.insert("action".to_string(), serde_json::Value::String(outer_key.clone()));
                        if let Ok(act) = Self::from_flexible_json(&serde_json::Value::Object(merged)) {
                            return Ok(act);
                        }
                    }
                }
            }
        }

        Err(CuError::InvalidArg(format!("Invalid action format: {}", val)))
    }
}
