use clap::{Parser, Subcommand};
use computeruse::*;
use base64::Engine;
use serde_json::json;
use std::io::{self, Read};

#[derive(Parser)]
#[command(name = "cu", version, about = "Windows computer-use CLI for LLMs (JSON I/O)")]
struct Cli {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Screen size
    ScreenSize,
    /// List monitors
    Monitors,
    /// Full screenshot or targeted canvas/region crop (base64 png or save to file)
    Screenshot {
        #[arg(long)] monitor: Option<u32>,
        #[arg(long)] region: Option<String>, // x,y,w,h
        #[arg(long)] canvas: bool,
        #[arg(long)] hwnd: Option<isize>,
        #[arg(long)] save: Option<String>,
    },
    /// Cursor position
    Cursor,
    /// Move cursor
    Move { x: i32, y: i32 },
    /// Move cursor relative
    MoveRel { dx: i32, dy: i32 },
    /// Click
    Click {
        #[arg(default_value = "left")] button: String,
        #[arg(long)] x: Option<i32>,
        #[arg(long)] y: Option<i32>,
    },
    /// Double click
    DblClick {
        #[arg(default_value = "left")] button: String,
        #[arg(long)] x: Option<i32>,
        #[arg(long)] y: Option<i32>,
    },
    /// Mouse down
    MouseDown { #[arg(default_value = "left")] button: String },
    /// Mouse up
    MouseUp { #[arg(default_value = "left")] button: String },
    /// Drag
    Drag { x1: i32, y1: i32, x2: i32, y2: i32, #[arg(default_value = "left")] button: String },
    /// Scroll
    Scroll { amount: i32, #[arg(long)] horizontal: bool },
    /// Type text
    Type { text: Option<String>, #[arg(long)] stdin: bool, #[arg(long, default_value = "25")] delay: u64 },
    /// Press a key
    Key { name: String },
    /// Key down
    KeyDown { name: String },
    /// Key up
    KeyUp { name: String },
    /// Hotkey combo e.g. "ctrl+c"
    Hotkey { combo: String },
    /// List windows
    Windows { #[arg(long)] all: bool },
    /// Find window by title substring
    FindWindow { title: String },
    /// Foreground window
    Foreground,
    /// Focus window by title or hwnd
    Focus { #[arg(long)] title: Option<String>, #[arg(long)] hwnd: Option<isize> },
    /// Minimize
    Minimize { hwnd: isize },
    /// Maximize
    Maximize { hwnd: isize },
    /// Restore
    Restore { hwnd: isize },
    /// Close window
    Close { hwnd: isize },
    /// Move/resize window
    MoveWindow { hwnd: isize, x: i32, y: i32, w: u32, h: u32 },
    /// Clipboard get
    ClipboardGet,
    /// Clipboard set
    ClipboardSet { text: Option<String>, #[arg(long)] stdin: bool },
    /// Wait for window to appear
    WaitForWindow { title: String, #[arg(long, default_value = "5000")] timeout: u64 },
    /// UI Automation: element at point
    UiaAt { x: i32, y: i32 },
    /// UI Automation: focused element
    UiaFocused,
    /// UI Automation: find elements by name
    UiaFind { name: String },
    /// UI Automation: click element by name
    UiaClick { name: String },
    /// UI Automation: children of window (hwnd 0 = root)
    UiaChildren { #[arg(long, default_value = "0")] hwnd: isize },
    /// UI Automation: extract all visible text strings and values from window (hwnd 0 = foreground)
    ReadWindowText { #[arg(long, default_value = "0")] hwnd: isize },
    /// Scan active window for target RGB color
    FindColor { r: u8, g: u8, b: u8, #[arg(long, default_value = "10")] tolerance: u8 },
    /// List processes
    Processes,
    /// Find process by name
    FindProcess { name: String },
    /// Kill process by pid
    KillProcess { pid: u32 },
    /// Kill process by name
    KillByName { name: String },
    /// Screenshot of a window by hwnd
    ScreenshotWindow { hwnd: isize },
    /// Screenshot of active (foreground) window
    ScreenshotActive,
    /// Screenshot of all monitors combined
    ScreenshotAll,
    /// Wait ms
    Wait { ms: u64 },
    /// Pixel color at point
    Pixel { x: i32, y: i32 },
    /// Launch app/file
    Launch { target: String, #[arg(long)] params: Option<String> },
    /// Master audio volume (get or set 0-100)
    Volume { #[arg(long)] set: Option<u32> },
    /// Audio mute (get, set, or toggle)
    Mute { #[arg(long)] set: Option<bool>, #[arg(long)] toggle: bool },
    /// System health, RAM, CPU, uptime, battery
    Sysinfo,
    /// Disk drives and free space
    Drives,
    /// Lock workstation
    Lock,
    /// Windows special folder path (desktop, downloads, documents, appdata, temp, etc.)
    SpecialFolder { name: String },
    /// Resolve default application for extension or URI protocol
    ResolveAssoc { target: String },
    /// Play system sound alert (default, error, warning, question, asterisk)
    SystemSound { #[arg(default_value = "default")] sound: String },
    /// Window snap to screen edge/corner (left, right, top, bottom, top_left, top_right, bottom_left, bottom_right, center, maximize)
    SnapWindow { position: String, #[arg(long)] hwnd: Option<isize>, #[arg(long)] title: Option<String> },
    /// Save As zero-roundtrip atomic macro: handles standard Windows file dialog
    SaveAs { path: String, #[arg(long, default_value = "3000")] timeout: u64 },
    /// Window topmost pin
    WindowTopmost { hwnd: isize, #[arg(long)] disable: bool },
    /// Window opacity (0-255)
    WindowOpacity { hwnd: isize, alpha: u8 },
    /// Execute a JSON command from stdin or arg
    Exec { json: Option<String> },
    /// Observe current desktop state & Set-of-Marks perception
    Observe {
        /// Output only the compact Set-of-Marks prompt summary
        #[arg(long)]
        compact: bool,
        /// Optional window HWND to inspect without changing focus
        #[arg(long)]
        hwnd: Option<isize>,
        /// Optional window title substring to inspect without changing focus
        #[arg(long)]
        title: Option<String>,
    },
    /// Execute an atomic AgentAction from JSON string or stdin
    Step { action: Option<String>, #[arg(long)] stdin: bool },
    /// Interactive Agent REPL loop (reads JSON action on stdin, emits JSON observation on stdout)
    Repl,
    /// Run as a standard Model Context Protocol (MCP) server over stdio
    Mcp {
        /// Synchronize all MCP tool schemas directly to disk and exit
        #[arg(long)]
        sync: bool,
    },
}

fn btn(s: &str) -> MouseButton {
    match s.to_lowercase().as_str() {
        "right" => MouseButton::Right,
        "middle" => MouseButton::Middle,
        _ => MouseButton::Left,
    }
}

fn out(v: serde_json::Value) {
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
}

fn err(e: CuError) {
    eprintln!("{}", serde_json::to_string(&json!({"error": e.to_string()})).unwrap());
    std::process::exit(1);
}

fn parse_compact_dsl(input: &str) -> Option<Vec<serde_json::Value>> {
    let trimmed = input.trim();
    if trimmed.starts_with('{') || trimmed.starts_with('[') {
        return None;
    }
    let mut actions = Vec::new();
    for stmt in trimmed.split([';', '\n']) {
        let s = stmt.trim();
        if s.is_empty() { continue; }
        let (cmd, rest) = if s.len() > 1 && s.chars().nth(1) == Some(' ') {
            (&s[..1], s[1..].trim())
        } else if s.len() > 1 && !s.chars().nth(1).unwrap().is_alphanumeric() {
            (&s[..1], s[1..].trim())
        } else {
            let split_pos = s.find(|c: char| c.is_whitespace() || c == ':').unwrap_or(s.len());
            (&s[..split_pos], s[split_pos..].trim().trim_start_matches(':').trim())
        };

        match cmd.to_uppercase().as_str() {
            "D" | "DRAG" => {
                let parts: Vec<&str> = rest.split([',', ' ']).filter(|p| !p.is_empty()).collect();
                if parts.len() >= 4 {
                    if let (Ok(x1), Ok(y1), Ok(x2), Ok(y2)) = (parts[0].parse::<i32>(), parts[1].parse::<i32>(), parts[2].parse::<i32>(), parts[3].parse::<i32>()) {
                        actions.push(json!({"action": "drag", "x1": x1, "y1": y1, "x2": x2, "y2": y2}));
                    }
                }
            }
            "C" | "CLICK" => {
                let parts: Vec<&str> = rest.split([',', ' ']).filter(|p| !p.is_empty()).collect();
                if parts.len() >= 2 {
                    if let (Ok(x), Ok(y)) = (parts[0].parse::<i32>(), parts[1].parse::<i32>()) {
                        actions.push(json!({"action": "click", "x": x, "y": y}));
                    }
                } else {
                    actions.push(json!({"action": "click"}));
                }
            }
            "M" | "MOVE" => {
                let parts: Vec<&str> = rest.split([',', ' ']).filter(|p| !p.is_empty()).collect();
                if parts.len() >= 2 {
                    if let (Ok(x), Ok(y)) = (parts[0].parse::<i32>(), parts[1].parse::<i32>()) {
                        actions.push(json!({"action": "move", "x": x, "y": y}));
                    }
                }
            }
            "DC" | "DBLCLICK" => {
                let parts: Vec<&str> = rest.split([',', ' ']).filter(|p| !p.is_empty()).collect();
                if parts.len() >= 2 {
                    if let (Ok(x), Ok(y)) = (parts[0].parse::<i32>(), parts[1].parse::<i32>()) {
                        actions.push(json!({"action": "double_click", "x": x, "y": y}));
                    }
                } else {
                    actions.push(json!({"action": "double_click"}));
                }
            }
            "RC" | "RCLICK" | "RIGHTCLICK" => {
                let parts: Vec<&str> = rest.split([',', ' ']).filter(|p| !p.is_empty()).collect();
                if parts.len() >= 2 {
                    if let (Ok(x), Ok(y)) = (parts[0].parse::<i32>(), parts[1].parse::<i32>()) {
                        actions.push(json!({"action": "click", "button": "right", "x": x, "y": y}));
                    }
                } else {
                    actions.push(json!({"action": "click", "button": "right"}));
                }
            }
            "P" | "PASTE" => {
                let text = rest.trim_matches('"').trim_matches('\'');
                actions.push(json!({"action": "paste", "text": text}));
            }
            "S" | "SCROLL" => {
                if let Ok(amount) = rest.parse::<i32>() {
                    actions.push(json!({"action": "scroll", "amount": amount}));
                }
            }
            "F" | "FOCUS" => {
                let title = rest.trim_matches('"').trim_matches('\'');
                actions.push(json!({"action": "focus", "title": title}));
            }
            "T" | "TYPE" => {
                let text = rest.trim_matches('"').trim_matches('\'');
                actions.push(json!({"action": "type", "text": text}));
            }
            "K" | "KEY" => {
                actions.push(json!({"action": "key", "name": rest}));
            }
            "H" | "HOTKEY" => {
                let combo = rest.trim_matches('"').trim_matches('\'');
                actions.push(json!({"action": "hotkey", "combo": combo}));
            }
            "W" | "WAIT" => {
                if let Ok(ms) = rest.parse::<u64>() {
                    actions.push(json!({"action": "wait", "ms": ms}));
                }
            }
            _ => {}
        }
    }
    if actions.is_empty() { None } else { Some(actions) }
}

fn run_action(cmd: &serde_json::Value) -> serde_json::Value {
    let action = cmd.get("action").and_then(|v| v.as_str()).unwrap_or("").to_lowercase();
    let g = |key: &str| cmd.get(key);
    let gi = |key: &str| g(key).and_then(|v| v.as_i64()).map(|x| x as i32);
    let gu = |key: &str| g(key).and_then(|v| v.as_u64()).map(|x| x as u32);
    match action.as_str() {
        "screen_size" => match ComputerUse::screen_size() {
            Ok((w, h)) => json!({"ok": true, "width": w, "height": h}),
            Err(e) => json!({"ok": false, "error": e.to_string()}),
        },
        "monitors" => match ComputerUse::list_monitors() {
            Ok(m) => json!({"ok": true, "monitors": m}),
            Err(e) => json!({"ok": false, "error": e.to_string()}),
        },
        "screenshot" => {
            let monitor = cmd.get("monitor").and_then(|v| v.as_u64()).map(|x| x as u32);
            let region = if let (Some(x), Some(y), Some(w), Some(h)) = (gi("x"), gi("y"), gu("width"), gu("height")) {
                Some(Rect { x, y, width: w, height: h })
            } else { None };
            let res = if let Some(r) = region { ComputerUse::screenshot_region(r, monitor) } else { ComputerUse::screenshot_full(monitor) };
            match res {
                Ok(s) => json!({"ok": true, "width": s.width, "height": s.height, "png_base64": s.png_base64}),
                Err(e) => json!({"ok": false, "error": e.to_string()}),
            }
        }
        "cursor" => match ComputerUse::cursor_position() {
            Ok(p) => json!({"ok": true, "x": p.x, "y": p.y}),
            Err(e) => json!({"ok": false, "error": e.to_string()}),
        },
        "move" => {
            let (x, y) = (gi("x").unwrap_or(0), gi("y").unwrap_or(0));
            match ComputerUse::mouse_move(Point { x, y }) {
                Ok(_) => json!({"ok": true}), Err(e) => json!({"ok": false, "error": e.to_string()}),
            }
        }
        "move_rel" => {
            let (dx, dy) = (gi("dx").unwrap_or(0), gi("dy").unwrap_or(0));
            match ComputerUse::mouse_move_relative(dx, dy) {
                Ok(p) => json!({"ok": true, "x": p.x, "y": p.y}), Err(e) => json!({"ok": false, "error": e.to_string()}),
            }
        }
        "click" => {
            let btn = btn(cmd.get("button").and_then(|v| v.as_str()).unwrap_or("left"));
            let p = if let (Some(x), Some(y)) = (gi("x"), gi("y")) { Some(Point { x, y }) } else { None };
            match ComputerUse::click(btn, p) {
                Ok(_) => json!({"ok": true}), Err(e) => json!({"ok": false, "error": e.to_string()}),
            }
        }
        "double_click" => {
            let btn = btn(cmd.get("button").and_then(|v| v.as_str()).unwrap_or("left"));
            let p = if let (Some(x), Some(y)) = (gi("x"), gi("y")) { Some(Point { x, y }) } else { None };
            match ComputerUse::double_click(btn, p) {
                Ok(_) => json!({"ok": true}), Err(e) => json!({"ok": false, "error": e.to_string()}),
            }
        }
        "mouse_down" => {
            let btn = btn(cmd.get("button").and_then(|v| v.as_str()).unwrap_or("left"));
            match ComputerUse::mouse_down(btn) { Ok(_) => json!({"ok": true}), Err(e) => json!({"ok": false, "error": e.to_string()}) }
        }
        "mouse_up" => {
            let btn = btn(cmd.get("button").and_then(|v| v.as_str()).unwrap_or("left"));
            match ComputerUse::mouse_up(btn) { Ok(_) => json!({"ok": true}), Err(e) => json!({"ok": false, "error": e.to_string()}) }
        }
        "drag" => {
            let (x1,y1,x2,y2) = (gi("x1").unwrap_or(0), gi("y1").unwrap_or(0), gi("x2").unwrap_or(0), gi("y2").unwrap_or(0));
            let btn = btn(cmd.get("button").and_then(|v| v.as_str()).unwrap_or("left"));
            match ComputerUse::drag(Point{x:x1,y:y1}, Point{x:x2,y:y2}, btn) {
                Ok(_) => json!({"ok": true}), Err(e) => json!({"ok": false, "error": e.to_string()}),
            }
        }
        "scroll" => {
            let amount = cmd.get("amount").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            let horizontal = cmd.get("horizontal").and_then(|v| v.as_bool()).unwrap_or(false);
            match ComputerUse::scroll(amount, horizontal) { Ok(_) => json!({"ok": true}), Err(e) => json!({"ok": false, "error": e.to_string()}) }
        }
        "type" => {
            let text = cmd.get("text").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let delay = cmd.get("delay").and_then(|v| v.as_u64()).unwrap_or(25);
            match ComputerUse::type_text_with_delay(&text, delay) { Ok(_) => json!({"ok": true, "chars": text.chars().count()}), Err(e) => json!({"ok": false, "error": e.to_string()}) }
        }
        "paste" => {
            let text = cmd.get("text").and_then(|v| v.as_str()).unwrap_or("").to_string();
            match ComputerUse::paste_text(&text) { Ok(_) => json!({"ok": true, "chars": text.chars().count()}), Err(e) => json!({"ok": false, "error": e.to_string()}) }
        }
        "key" => {
            let name = cmd.get("name").or_else(|| cmd.get("key")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            match ComputerUse::key_press(&name) { Ok(_) => json!({"ok": true}), Err(e) => json!({"ok": false, "error": e.to_string()}) }
        }
        "key_down" => {
            let name = cmd.get("name").or_else(|| cmd.get("key")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            match ComputerUse::key_down(&name) { Ok(_) => json!({"ok": true}), Err(e) => json!({"ok": false, "error": e.to_string()}) }
        }
        "key_up" => {
            let name = cmd.get("name").or_else(|| cmd.get("key")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            match ComputerUse::key_up(&name) { Ok(_) => json!({"ok": true}), Err(e) => json!({"ok": false, "error": e.to_string()}) }
        }
        "hotkey" => {
            let combo = cmd.get("combo").or_else(|| cmd.get("hotkey")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            match ComputerUse::hotkey_str(&combo) { Ok(_) => json!({"ok": true}), Err(e) => json!({"ok": false, "error": e.to_string()}) }
        }
        "windows" => {
            let all = cmd.get("all").and_then(|v| v.as_bool()).unwrap_or(false);
            match ComputerUse::list_windows(!all) { Ok(w) => json!({"ok": true, "windows": w}), Err(e) => json!({"ok": false, "error": e.to_string()}) }
        }
        "find_window" => {
            let title = cmd.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string();
            match ComputerUse::find_window(&title) { Ok(w) => json!({"ok": true, "window": w}), Err(e) => json!({"ok": false, "error": e.to_string()}) }
        }
        "foreground" => match ComputerUse::foreground_window() { Ok(w) => json!({"ok": true, "window": w}), Err(e) => json!({"ok": false, "error": e.to_string()}) },
        "focus" => {
            if let Some(title) = cmd.get("title").and_then(|v| v.as_str()) {
                match ComputerUse::focus_by_title(title) { Ok(w) => json!({"ok": true, "hwnd": w.hwnd}), Err(e) => json!({"ok": false, "error": e.to_string()}) }
            } else if let Some(hwnd) = cmd.get("hwnd").and_then(|v| v.as_i64()) {
                match ComputerUse::focus_window(hwnd as isize) { Ok(_) => json!({"ok": true, "hwnd": hwnd}), Err(e) => json!({"ok": false, "error": e.to_string()}) }
            } else { json!({"ok": false, "error": "title or hwnd required"}) }
        }
        "minimize" => {
            let hwnd = cmd.get("hwnd").and_then(|v| v.as_i64()).unwrap_or(0) as isize;
            match ComputerUse::minimize_window(hwnd) { Ok(_) => json!({"ok": true}), Err(e) => json!({"ok": false, "error": e.to_string()}) }
        }
        "maximize" => {
            let hwnd = cmd.get("hwnd").and_then(|v| v.as_i64()).unwrap_or(0) as isize;
            match ComputerUse::maximize_window(hwnd) { Ok(_) => json!({"ok": true}), Err(e) => json!({"ok": false, "error": e.to_string()}) }
        }
        "restore" => {
            let hwnd = cmd.get("hwnd").and_then(|v| v.as_i64()).unwrap_or(0) as isize;
            match ComputerUse::restore_window(hwnd) { Ok(_) => json!({"ok": true}), Err(e) => json!({"ok": false, "error": e.to_string()}) }
        }
        "close" => {
            let hwnd = cmd.get("hwnd").and_then(|v| v.as_i64()).unwrap_or(0) as isize;
            match ComputerUse::close_window(hwnd) { Ok(_) => json!({"ok": true}), Err(e) => json!({"ok": false, "error": e.to_string()}) }
        }
        "move_window" => {
            let hwnd = cmd.get("hwnd").and_then(|v| v.as_i64()).unwrap_or(0) as isize;
            let x = gi("x").unwrap_or(0); let y = gi("y").unwrap_or(0);
            let w = gu("width").unwrap_or(800); let h = gu("height").unwrap_or(600);
            match ComputerUse::move_resize_window(hwnd, Rect { x, y, width: w, height: h }) { Ok(_) => json!({"ok": true}), Err(e) => json!({"ok": false, "error": e.to_string()}) }
        }
        "clipboard_get" => match ComputerUse::clipboard_get() { Ok(t) => json!({"ok": true, "text": t}), Err(e) => json!({"ok": false, "error": e.to_string()}) },
        "clipboard_set" => {
            let text = cmd.get("text").and_then(|v| v.as_str()).unwrap_or("").to_string();
            match ComputerUse::clipboard_set(&text) { Ok(_) => json!({"ok": true}), Err(e) => json!({"ok": false, "error": e.to_string()}) }
        }
        "wait_for_window" => {
            let title = cmd.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let timeout = cmd.get("timeout").and_then(|v| v.as_u64()).unwrap_or(5000);
            match ComputerUse::wait_for_window(&title, timeout) { Ok(w) => json!({"ok": true, "window": w}), Err(e) => json!({"ok": false, "error": e.to_string()}) }
        }
        "uia_at" => {
            let (x, y) = (gi("x").unwrap_or(0), gi("y").unwrap_or(0));
            match ComputerUse::uia_element_at_point(x, y) { Ok(e) => json!(e), Err(e) => json!({"ok": false, "error": e.to_string()}) }
        }
        "uia_focused" => match ComputerUse::uia_focused_element() { Ok(e) => json!(e), Err(e) => json!({"ok": false, "error": e.to_string()}) },
        "uia_find" => {
            let name = cmd.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
            match ComputerUse::uia_find_by_name(&name) { Ok(v) => json!({"elements": v}), Err(e) => json!({"ok": false, "error": e.to_string()}) }
        }
        "uia_click" => {
            let name = cmd.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
            match ComputerUse::uia_click_by_name(&name) { Ok(e) => json!(e), Err(e) => json!({"ok": false, "error": e.to_string()}) }
        }
        "uia_children" => {
            let hwnd = cmd.get("hwnd").and_then(|v| v.as_i64()).unwrap_or(0) as isize;
            match ComputerUse::uia_children(hwnd) { Ok(v) => json!({"children": v}), Err(e) => json!({"ok": false, "error": e.to_string()}) }
        }
        "processes" => match ComputerUse::list_processes() { Ok(v) => json!({"processes": v}), Err(e) => json!({"ok": false, "error": e.to_string()}) },
        "find_process" => {
            let name = cmd.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
            match ComputerUse::find_process(&name) { Ok(v) => json!(v), Err(e) => json!({"ok": false, "error": e.to_string()}) }
        }
        "kill_process" => {
            let pid = cmd.get("pid").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            match ComputerUse::kill_process(pid) { Ok(_) => json!({"ok": true}), Err(e) => json!({"ok": false, "error": e.to_string()}) }
        }
        "kill_by_name" => {
            let name = cmd.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
            match ComputerUse::kill_by_name(&name) { Ok(pid) => json!({"ok": true, "pid": pid}), Err(e) => json!({"ok": false, "error": e.to_string()}) }
        }
        "wait" => {
            let ms = cmd.get("ms").and_then(|v| v.as_u64()).unwrap_or(0);
            std::thread::sleep(std::time::Duration::from_millis(ms));
            json!({"ok": true, "waited_ms": ms})
        }
        "pixel" => {
            let (x, y) = (gi("x").unwrap_or(0), gi("y").unwrap_or(0));
            match ComputerUse::get_pixel(Point { x, y }) {
                Ok((r,g,b)) => json!({"ok": true, "r": r, "g": g, "b": b, "hex": format!("#{:02x}{:02x}{:02x}", r, g, b)}),
                Err(e) => json!({"ok": false, "error": e.to_string()}),
            }
        }
        "launch" => {
            let target = cmd.get("target").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let params = cmd.get("params").and_then(|v| v.as_str());
            match ComputerUse::launch(&target, params) { Ok(_) => json!({"ok": true}), Err(e) => json!({"ok": false, "error": e.to_string()}) }
        }
        "volume" => {
            if let Some(set) = cmd.get("set").and_then(|v| v.as_u64()) {
                match ComputerUse::audio_set_volume(set as u32) { Ok(_) => json!({"ok": true, "volume": set}), Err(e) => json!({"ok": false, "error": e.to_string()}) }
            } else {
                match ComputerUse::audio_get_volume() { Ok(vol) => json!({"ok": true, "volume": vol}), Err(e) => json!({"ok": false, "error": e.to_string()}) }
            }
        }
        "mute" => {
            if cmd.get("toggle").and_then(|v| v.as_bool()).unwrap_or(false) {
                match ComputerUse::audio_toggle_mute() { Ok(m) => json!({"ok": true, "muted": m}), Err(e) => json!({"ok": false, "error": e.to_string()}) }
            } else if let Some(m) = cmd.get("set").and_then(|v| v.as_bool()) {
                match ComputerUse::audio_set_mute(m) { Ok(_) => json!({"ok": true, "muted": m}), Err(e) => json!({"ok": false, "error": e.to_string()}) }
            } else {
                match ComputerUse::audio_get_mute() { Ok(m) => json!({"ok": true, "muted": m}), Err(e) => json!({"ok": false, "error": e.to_string()}) }
            }
        }
        "sysinfo" | "system_info" => match ComputerUse::system_info() {
            Ok(s) => json!({"ok": true, "system": s}),
            Err(e) => json!({"ok": false, "error": e.to_string()}),
        },
        "drives" => match ComputerUse::system_drives() {
            Ok(d) => json!({"ok": true, "drives": d}),
            Err(e) => json!({"ok": false, "error": e.to_string()}),
        },
        "lock" => match ComputerUse::system_lock() {
            Ok(_) => json!({"ok": true}),
            Err(e) => json!({"ok": false, "error": e.to_string()}),
        },
        "special_folder" => {
            let name = cmd.get("name").and_then(|v| v.as_str()).unwrap_or("");
            match ComputerUse::system_special_folder(name) {
                Ok(path) => json!({"ok": true, "path": path}),
                Err(e) => json!({"ok": false, "error": e.to_string()}),
            }
        }
        "window_topmost" => {
            let hwnd = cmd.get("hwnd").and_then(|v| v.as_i64()).unwrap_or(0) as isize;
            let topmost = cmd.get("topmost").and_then(|v| v.as_bool()).unwrap_or(true);
            match ComputerUse::set_window_topmost(hwnd, topmost) {
                Ok(_) => json!({"ok": true, "hwnd": hwnd, "topmost": topmost}),
                Err(e) => json!({"ok": false, "error": e.to_string()}),
            }
        }
        "window_opacity" => {
            let hwnd = cmd.get("hwnd").and_then(|v| v.as_i64()).unwrap_or(0) as isize;
            let alpha = cmd.get("alpha").and_then(|v| v.as_u64()).unwrap_or(255) as u8;
            match ComputerUse::set_window_opacity(hwnd, alpha) {
                Ok(_) => json!({"ok": true, "hwnd": hwnd, "alpha": alpha}),
                Err(e) => json!({"ok": false, "error": e.to_string()}),
            }
        }
        _ => json!({"ok": false, "error": format!("unknown action: {}", action)}),
    }
}

fn main() {
    let _ = computeruse::desktop::attach_thread_to_input_desktop();
    let cli = Cli::parse();
    match cli.cmd {
        Commands::ScreenSize => match ComputerUse::screen_size() {
            Ok((w, h)) => out(json!({"width": w, "height": h})),
            Err(e) => err(e),
        },
        Commands::Monitors => match ComputerUse::list_monitors() {
            Ok(m) => out(json!({"monitors": m})),
            Err(e) => err(e),
        },
        Commands::Screenshot { monitor, region, canvas, hwnd, save } => {
            let res = if canvas {
                let target_hwnd = hwnd.or_else(|| ComputerUse::foreground_window().ok().map(|w| w.hwnd)).unwrap_or(0);
                match computeruse::DynamicCanvas::find_in_window(target_hwnd) {
                    Ok(c) => {
                        let rect = Rect {
                            x: c.origin_x,
                            y: c.origin_y,
                            width: c.width.max(16),
                            height: c.height.max(16),
                        };
                        ComputerUse::screenshot_region(rect, monitor)
                    }
                    Err(e) => Err(e),
                }
            } else if let Some(r) = region {
                let parts: Vec<&str> = r.split(',').collect();
                if parts.len() != 4 { err(CuError::InvalidArg("region x,y,w,h".into())); return; }
                let rect = Rect {
                    x: parts[0].parse().unwrap_or(0),
                    y: parts[1].parse().unwrap_or(0),
                    width: parts[2].parse().unwrap_or(100),
                    height: parts[3].parse().unwrap_or(100),
                };
                ComputerUse::screenshot_region(rect, monitor)
            } else {
                ComputerUse::screenshot_full(monitor)
            };
            match res {
                Ok(s) => {
                    if let Some(path) = save {
                        match base64::engine::general_purpose::STANDARD.decode(&s.png_base64) {
                            Ok(bytes) => match std::fs::write(&path, bytes) {
                                Ok(_) => out(json!({"ok": true, "saved_path": path, "width": s.width, "height": s.height, "region": s.region})),
                                Err(e) => err(CuError::Windows(format!("Failed to write {}: {}", path, e))),
                            },
                            Err(e) => err(CuError::Windows(format!("Failed to decode base64: {}", e))),
                        }
                    } else {
                        out(json!({"width": s.width, "height": s.height, "monitor": s.monitor, "region": s.region, "png_base64": s.png_base64}))
                    }
                }
                Err(e) => err(e),
            }
        }
        Commands::Cursor => match ComputerUse::cursor_position() {
            Ok(p) => out(json!({"x": p.x, "y": p.y})),
            Err(e) => err(e),
        },
        Commands::Move { x, y } => match ComputerUse::mouse_move(Point { x, y }) {
            Ok(_) => out(json!({"ok": true, "x": x, "y": y})),
            Err(e) => err(e),
        },
        Commands::MoveRel { dx, dy } => match ComputerUse::mouse_move_relative(dx, dy) {
            Ok(p) => out(json!({"ok": true, "x": p.x, "y": p.y})),
            Err(e) => err(e),
        },
        Commands::Click { button, x, y } => {
            let b = btn(&button);
            let p = match (x, y) { (Some(x), Some(y)) => Some(Point { x, y }), _ => None };
            match ComputerUse::click(b, p) {
                Ok(_) => out(json!({"ok": true})),
                Err(e) => err(e),
            }
        }
        Commands::DblClick { button, x, y } => {
            let b = btn(&button);
            let p = match (x, y) { (Some(x), Some(y)) => Some(Point { x, y }), _ => None };
            match ComputerUse::double_click(b, p) {
                Ok(_) => out(json!({"ok": true})),
                Err(e) => err(e),
            }
        }
        Commands::MouseDown { button } => match ComputerUse::mouse_down(btn(&button)) {
            Ok(_) => out(json!({"ok": true})),
            Err(e) => err(e),
        },
        Commands::MouseUp { button } => match ComputerUse::mouse_up(btn(&button)) {
            Ok(_) => out(json!({"ok": true})),
            Err(e) => err(e),
        },
        Commands::Drag { x1, y1, x2, y2, button } => {
            match ComputerUse::drag(Point { x: x1, y: y1 }, Point { x: x2, y: y2 }, btn(&button)) {
                Ok(_) => out(json!({"ok": true})),
                Err(e) => err(e),
            }
        }
        Commands::Scroll { amount, horizontal } => match ComputerUse::scroll(amount, horizontal) {
            Ok(_) => out(json!({"ok": true})),
            Err(e) => err(e),
        },
        Commands::Type { text, stdin, delay } => {
            let t = if stdin {
                let mut s = String::new();
                io::stdin().read_to_string(&mut s).ok();
                s
            } else {
                text.unwrap_or_default()
            };
            match ComputerUse::type_text_with_delay(&t, delay) {
                Ok(_) => out(json!({"ok": true, "chars": t.chars().count()})),
                Err(e) => err(e),
            }
        }
        Commands::Key { name } => match ComputerUse::key_press(&name) {
            Ok(_) => out(json!({"ok": true})),
            Err(e) => err(e),
        },
        Commands::KeyDown { name } => match ComputerUse::key_down(&name) {
            Ok(_) => out(json!({"ok": true})),
            Err(e) => err(e),
        },
        Commands::KeyUp { name } => match ComputerUse::key_up(&name) {
            Ok(_) => out(json!({"ok": true})),
            Err(e) => err(e),
        },
        Commands::Hotkey { combo } => match ComputerUse::hotkey_str(&combo) {
            Ok(_) => out(json!({"ok": true})),
            Err(e) => err(e),
        },
        Commands::Windows { all } => match ComputerUse::list_windows(!all) {
            Ok(w) => out(json!({"windows": w})),
            Err(e) => err(e),
        },
        Commands::FindWindow { title } => match ComputerUse::find_window(&title) {
            Ok(w) => out(json!(w)),
            Err(e) => err(e),
        },
        Commands::Foreground => match ComputerUse::foreground_window() {
            Ok(w) => out(json!(w)),
            Err(e) => err(e),
        },
        Commands::Focus { title, hwnd } => {
            let res = if let Some(t) = title {
                ComputerUse::focus_by_title(&t).map(|w| w.hwnd)
            } else if let Some(h) = hwnd {
                ComputerUse::focus_window(h).map(|_| h)
            } else {
                Err(CuError::InvalidArg("title or hwnd required".into()))
            };
            match res {
                Ok(h) => out(json!({"ok": true, "hwnd": h})),
                Err(e) => err(e),
            }
        }
        Commands::Minimize { hwnd } => match ComputerUse::minimize_window(hwnd) {
            Ok(_) => out(json!({"ok": true})),
            Err(e) => err(e),
        },
        Commands::Maximize { hwnd } => match ComputerUse::maximize_window(hwnd) {
            Ok(_) => out(json!({"ok": true})),
            Err(e) => err(e),
        },
        Commands::Restore { hwnd } => match ComputerUse::restore_window(hwnd) {
            Ok(_) => out(json!({"ok": true})),
            Err(e) => err(e),
        },
        Commands::Close { hwnd } => match ComputerUse::close_window(hwnd) {
            Ok(_) => out(json!({"ok": true})),
            Err(e) => err(e),
        },
        Commands::MoveWindow { hwnd, x, y, w, h } => {
            match ComputerUse::move_resize_window(hwnd, Rect { x, y, width: w, height: h }) {
                Ok(_) => out(json!({"ok": true})),
                Err(e) => err(e),
            }
        }
        Commands::ClipboardGet => match ComputerUse::clipboard_get() {
            Ok(t) => out(json!({"text": t})),
            Err(e) => err(e),
        },
        Commands::ClipboardSet { text, stdin } => {
            let t = if stdin {
                let mut s = String::new();
                io::stdin().read_to_string(&mut s).ok();
                s
            } else {
                text.unwrap_or_default()
            };
            match ComputerUse::clipboard_set(&t) {
                Ok(_) => out(json!({"ok": true})),
                Err(e) => err(e),
            }
        }
        Commands::Pixel { x, y } => match ComputerUse::get_pixel(Point { x, y }) {
            Ok((r, g, b)) => out(json!({"r": r, "g": g, "b": b, "hex": format!("#{:02x}{:02x}{:02x}", r, g, b)})),
            Err(e) => err(e),
        },
        Commands::Launch { target, params } => match ComputerUse::launch(&target, params.as_deref()) {
            Ok(_) => out(json!({"ok": true})),
            Err(e) => err(e),
        },
        Commands::WaitForWindow { title, timeout } => match ComputerUse::wait_for_window(&title, timeout) {
            Ok(w) => out(json!(w)),
            Err(e) => err(e),
        },
        Commands::UiaAt { x, y } => match ComputerUse::uia_element_at_point(x, y) {
            Ok(e) => out(json!(e)), Err(e) => err(e),
        },
        Commands::UiaFocused => match ComputerUse::uia_focused_element() {
            Ok(e) => out(json!(e)), Err(e) => err(e),
        },
        Commands::UiaFind { name } => match ComputerUse::uia_find_by_name(&name) {
            Ok(v) => out(json!({"elements": v})), Err(e) => err(e),
        },
        Commands::UiaClick { name } => match ComputerUse::uia_click_by_name(&name) {
            Ok(e) => out(json!(e)), Err(e) => err(e),
        },
        Commands::UiaChildren { hwnd } => match ComputerUse::uia_children(hwnd) {
            Ok(v) => out(json!({"children": v})), Err(e) => err(e),
        },
        Commands::ReadWindowText { hwnd } => match ComputerUse::uia_read_window_text(hwnd) {
            Ok(lines) => out(json!({"count": lines.len(), "lines": lines})), Err(e) => err(e),
        },
        Commands::FindColor { r, g, b, tolerance } => {
            let reg = ComputerUse::foreground_window().map(|w| w.rect).unwrap_or(Rect { x: 0, y: 0, width: 800, height: 600 });
            match ComputerUse::find_color_in_region(reg, (r, g, b), tolerance) {
                Ok(pt) => out(json!({"found": pt.is_some(), "point": pt})),
                Err(e) => err(e),
            }
        }
        Commands::Processes => match ComputerUse::list_processes() {
            Ok(v) => out(json!({"processes": v})), Err(e) => err(e),
        },
        Commands::FindProcess { name } => match ComputerUse::find_process(&name) {
            Ok(v) => out(json!(v)), Err(e) => err(e),
        },
        Commands::KillProcess { pid } => match ComputerUse::kill_process(pid) {
            Ok(_) => out(json!({"ok": true})), Err(e) => err(e),
        },
        Commands::KillByName { name } => match ComputerUse::kill_by_name(&name) {
            Ok(pid) => out(json!({"ok": true, "pid": pid})), Err(e) => err(e),
        },
        Commands::ScreenshotWindow { hwnd } => match ComputerUse::screenshot_window(hwnd) {
            Ok(s) => out(json!({"width": s.width, "height": s.height, "png_base64": s.png_base64})),
            Err(e) => err(e),
        },
        Commands::ScreenshotActive => match ComputerUse::screenshot_active_window() {
            Ok(s) => out(json!({"width": s.width, "height": s.height, "png_base64": s.png_base64})),
            Err(e) => err(e),
        },
        Commands::ScreenshotAll => match ComputerUse::screenshot_all_monitors() {
            Ok(s) => out(json!({"width": s.width, "height": s.height, "png_base64": s.png_base64})),
            Err(e) => err(e),
        },
        Commands::Wait { ms } => {
            std::thread::sleep(std::time::Duration::from_millis(ms));
            out(json!({"ok": true, "waited_ms": ms}));
        }
        Commands::Volume { set } => {
            if let Some(val) = set {
                match ComputerUse::audio_set_volume(val) {
                    Ok(_) => out(json!({"ok": true, "volume": val})),
                    Err(e) => err(e),
                }
            } else {
                match ComputerUse::audio_get_volume() {
                    Ok(val) => out(json!({"ok": true, "volume": val})),
                    Err(e) => err(e),
                }
            }
        }
        Commands::Mute { set, toggle } => {
            if toggle {
                match ComputerUse::audio_toggle_mute() {
                    Ok(m) => out(json!({"ok": true, "muted": m})),
                    Err(e) => err(e),
                }
            } else if let Some(m) = set {
                match ComputerUse::audio_set_mute(m) {
                    Ok(_) => out(json!({"ok": true, "muted": m})),
                    Err(e) => err(e),
                }
            } else {
                match ComputerUse::audio_get_mute() {
                    Ok(m) => out(json!({"ok": true, "muted": m})),
                    Err(e) => err(e),
                }
            }
        }
        Commands::Sysinfo => match ComputerUse::system_info() {
            Ok(s) => out(json!(s)),
            Err(e) => err(e),
        },
        Commands::Drives => match ComputerUse::system_drives() {
            Ok(d) => out(json!({"drives": d})),
            Err(e) => err(e),
        },
        Commands::Lock => match ComputerUse::system_lock() {
            Ok(_) => out(json!({"ok": true})),
            Err(e) => err(e),
        },
        Commands::SpecialFolder { name } => match ComputerUse::system_special_folder(&name) {
            Ok(path) => out(json!({"ok": true, "folder": name, "path": path})),
            Err(e) => err(e),
        },
        Commands::ResolveAssoc { target } => match ComputerUse::system_resolve_association(&target) {
            Ok(exe) => out(json!({"ok": true, "target": target, "executable": exe})),
            Err(e) => err(e),
        },
        Commands::SystemSound { sound } => match ComputerUse::system_play_sound(&sound) {
            Ok(_) => out(json!({"ok": true, "sound": sound})),
            Err(e) => err(e),
        },
        Commands::SnapWindow { position, hwnd, title } => {
            let target_hwnd = if let Some(h) = hwnd {
                Ok(h)
            } else if let Some(ref t) = title {
                ComputerUse::find_window(t).map(|w| w.hwnd)
            } else {
                ComputerUse::foreground_window().map(|w| w.hwnd)
            };
            match target_hwnd.and_then(|h| ComputerUse::snap_window(h, &position).map(|r| (h, r))) {
                Ok((h, r)) => out(json!({"ok": true, "hwnd": h, "position": position, "rect": r})),
                Err(e) => err(e),
            }
        }
        Commands::SaveAs { path, timeout } => match ComputerUse::save_file_as(&path, timeout) {
            Ok(_) => out(json!({"ok": true, "saved_path": path})),
            Err(e) => err(e),
        },
        Commands::WindowTopmost { hwnd, disable } => match ComputerUse::set_window_topmost(hwnd, !disable) {
            Ok(_) => out(json!({"ok": true, "hwnd": hwnd, "topmost": !disable})),
            Err(e) => err(e),
        },
        Commands::WindowOpacity { hwnd, alpha } => match ComputerUse::set_window_opacity(hwnd, alpha) {
            Ok(_) => out(json!({"ok": true, "hwnd": hwnd, "alpha": alpha})),
            Err(e) => err(e),
        },
        Commands::Exec { json: j } => {
            // read JSON or compact DSL from arg or stdin
            let input = if let Some(s) = j {
                s
            } else {
                let mut s = String::new();
                io::stdin().read_to_string(&mut s).ok();
                s
            };
            let cmds = if let Some(dsl_cmds) = parse_compact_dsl(&input) {
                dsl_cmds
            } else {
                let parsed: serde_json::Value = match serde_json::from_str(&input) {
                    Ok(v) => v,
                    Err(e) => { err(CuError::Json(e)); return; }
                };
                if parsed.is_array() { parsed.as_array().cloned().unwrap_or_default() } else { vec![parsed] }
            };
            let mut results = Vec::new();
            for c in cmds {
                results.push(run_action(&c));
            }
            out(json!({"results": results}));
        }
        Commands::Observe { compact, hwnd, title } => {
            let mut env = ComputerUse::agent_env();
            match env.observe_target(hwnd, title.as_deref()) {
                Ok(obs) => {
                    if compact {
                        out(json!({
                            "ok": true,
                            "window_title": obs.window_title,
                            "window_hwnd": obs.window_hwnd,
                            "marks_count": obs.marks.len(),
                            "compact_prompt": obs.compact_prompt
                        }));
                    } else {
                        out(json!({"ok": true, "observation": obs}));
                    }
                }
                Err(e) => err(e),
            }
        }
        Commands::Step { action: a, stdin } => {
            let input = if stdin || a.as_deref() == Some("-") {
                let mut s = String::new();
                io::stdin().read_to_string(&mut s).ok();
                s
            } else if let Some(s) = a {
                s
            } else {
                let mut s = String::new();
                io::stdin().read_to_string(&mut s).ok();
                s
            };
            let mut env = ComputerUse::agent_env();
            let _ = env.observe();
            match serde_json::from_str::<serde_json::Value>(&input) {
                Ok(val) => match AgentAction::from_flexible_json(&val) {
                    Ok(act) => match env.step(act) {
                        Ok(res) => out(json!({"ok": true, "step": res})),
                        Err(e) => err(e),
                    },
                    Err(e) => err(e),
                },
                Err(e) => err(CuError::Json(e)),
            }
        }
        Commands::Repl => {
            use std::io::BufRead;
            let stdin = io::stdin();
            let mut env = ComputerUse::agent_env();
            match env.observe() {
                Ok(obs) => println!("{}", json!({"event": "ready", "observation": obs})),
                Err(e) => println!("{}", json!({"event": "error", "error": e.to_string()})),
            }
            for line in stdin.lock().lines() {
                if let Ok(l) = line {
                    let trim = l.trim();
                    if trim.is_empty() { continue; }
                    if trim == "quit" || trim == "exit" { break; }
                    match serde_json::from_str::<serde_json::Value>(trim) {
                        Ok(val) => match AgentAction::from_flexible_json(&val) {
                            Ok(act) => match env.step(act) {
                                Ok(res) => println!("{}", json!({"event": "step", "result": res})),
                                Err(e) => println!("{}", json!({"event": "step_error", "error": e.to_string()})),
                            },
                            Err(e) => println!("{}", json!({"event": "parse_error", "error": e.to_string()})),
                        },
                        Err(e) => println!("{}", json!({"event": "parse_error", "error": e.to_string()})),
                    }
                }
            }
        }
        Commands::Mcp { sync } => {
            if sync {
                match computeruse::mcp::sync_all_mcp_schemas() {
                    Ok(n) => {
                        println!("Successfully synced {} MCP schemas to Antigravity directory.", n);
                    }
                    Err(e) => {
                        eprintln!("Failed to sync MCP schemas: {}", e);
                        std::process::exit(1);
                    }
                }
                return;
            }
            if let Err(e) = computeruse::mcp::run_mcp_server() {
                eprintln!("[computeruse-mcp] Error: {}", e);
                std::process::exit(1);
            }
        }
    }
}
