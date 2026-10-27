use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use crate::agent::AgentEnvironment;
use crate::clipboard;
use crate::error::Result;
use crate::mouse;
use crate::keyboard;
use crate::screen;
use crate::system;
use crate::types::{MouseButton, Point, Rect};
use crate::window;
use crate::launch;
use crate::process;
use crate::uia;
use crate::audio;
use crate::canvas::DynamicCanvas;
use crate::ComputerUse;

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    #[allow(dead_code)]
    jsonrpc: Option<String>,
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: &'static str,
    id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<Value>,
}

/// Resolves the Antigravity MCP directory for computeruse
pub fn get_mcp_config_dir() -> PathBuf {
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_else(|_| ".".into());
    PathBuf::from(home)
        .join(".gemini")
        .join("antigravity-cli")
        .join("mcp")
        .join("computeruse")
}

/// Synchronizes all tool schemas and instructions directly into Antigravity's MCP directory,
/// pruning any stale `.json` files that are no longer part of `get_all_tools()`.
pub fn sync_all_mcp_schemas() -> Result<usize> {
    let dir = get_mcp_config_dir();
    let _ = std::fs::create_dir_all(&dir);

    let tools = get_all_tools();
    let count = tools.len();
    let mut active_files = std::collections::HashSet::new();

    for t in &tools {
        let name = t.get("name").and_then(|v| v.as_str()).unwrap_or("");
        if name.is_empty() {
            continue;
        }
        let filename = format!("{}.json", name);
        active_files.insert(filename.clone());

        let schema_file = dir.join(filename);
        let content = json!({
            "name": name,
            "description": t.get("description").unwrap_or(&Value::Null),
            "parameters": t.get("inputSchema").unwrap_or(&Value::Null)
        });

        let _ = std::fs::write(schema_file, serde_json::to_string(&content).unwrap_or_default());
    }

    // Prune stale .json schema files from previous versions
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let fname = entry.file_name().to_string_lossy().to_string();
            if fname.ends_with(".json") && !active_files.contains(&fname) {
                let _ = std::fs::remove_file(entry.path());
            }
        }
    }

    // Write updated instructions.md
    let instructions = r#"# Computer Use MCP Server Instructions (v0.3.0 — 4 Pillars of OS-Native Symbiosis)

Native Windows Computer Use MCP Server for Autonomous Agents.

## The 4 Pillars
1. **Pillar I — Hybrid Sub-Token Perception & Visual Crop**:
   - `cu_observe` extracts a categorized, ultra-compact Set-of-Marks (SoM) digest (`Interactive`, `Inputs`, `Passive`) in <1.8 KB. Pass `verbose: true` only when raw bounding boxes are needed.
   - `cu_screenshot` supports targeted low-token crops via `mark_id`, `canvas_only: true`, or `region: {x, y, width, height}`, plus direct `save_path` and `omit_base64: true`.
2. **Pillar II — Atomic Transactions & Macro-JIT**:
   - `cu_batch` executes multi-step action pipelines (e.g., focus -> click -> type -> key -> wait_for_change) in a single zero-roundtrip call and returns the final `prompt_summary` and `ui_delta`.
   - `cu_canvas_draw` renders multi-layered vector artwork (gears, stars, spirals, polygons, lines, text, bezier curves) with automatic closed-shape edge sealing.
3. **Pillar III — Per-Monitor DPI v2 & Virtual Desktop Input Fidelity**:
   - Hardware-exact coordinates across multi-monitor and negative-coordinate virtual desktops (`DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2` + `MOUSEEVENTF_VIRTUALDESK`).
4. **Pillar IV — Event-Driven Reactive Synchronization**:
   - `cu_wait_for_change` polls the OS foreground window and UIA tree in 25ms ticks, returning as soon as a popup, menu, or window transition occurs instead of sleeping blindly.
   - All interactive actions return a `ui_delta` summary indicating structural changes (`+N interactive elements`, title change, or window switch).
"#;
    let _ = std::fs::write(dir.join("instructions.md"), instructions);

    Ok(count)
}

pub fn run_mcp_server() -> Result<()> {
    eprintln!("[computeruse-mcp] Starting MCP Server over stdio (PID: {})...", std::process::id());
    
    // Auto-sync tool schemas on startup
    if let Ok(n) = sync_all_mcp_schemas() {
        eprintln!("[computeruse-mcp] Auto-synced {} tool schemas to disk.", n);
    }

    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut env = ComputerUse::agent_env();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        let trim = line.trim();
        if trim.is_empty() {
            continue;
        }

        let req: JsonRpcRequest = match serde_json::from_str(trim) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("[computeruse-mcp] Failed to parse request: {}", e);
                let err_resp = JsonRpcResponse {
                    jsonrpc: "2.0",
                    id: Value::Null,
                    result: None,
                    error: Some(json!({
                        "code": -32700,
                        "message": format!("Parse error: {}", e)
                    })),
                };
                let _ = writeln!(stdout, "{}", serde_json::to_string(&err_resp).unwrap());
                let _ = stdout.flush();
                continue;
            }
        };

        let id = req.id.unwrap_or(Value::Null);
        let resp = match req.method.as_str() {
            "initialize" => handle_initialize(id),
            "notifications/initialized" => {
                eprintln!("[computeruse-mcp] Client confirmed initialization.");
                continue;
            }
            "ping" => JsonRpcResponse {
                jsonrpc: "2.0",
                id,
                result: Some(json!({})),
                error: None,
            },
            "tools/list" => handle_tools_list(id),
            "tools/call" => handle_tools_call(id, req.params.unwrap_or(Value::Null), &mut env),
            other => {
                eprintln!("[computeruse-mcp] Unhandled method: {}", other);
                JsonRpcResponse {
                    jsonrpc: "2.0",
                    id,
                    result: None,
                    error: Some(json!({
                        "code": -32601,
                        "message": format!("Method not found: {}", other)
                    })),
                }
            }
        };

        let json_str = serde_json::to_string(&resp).unwrap();
        let _ = writeln!(stdout, "{}", json_str);
        let _ = stdout.flush();
    }

    eprintln!("[computeruse-mcp] Server terminated.");
    Ok(())
}

fn handle_initialize(id: Value) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0",
        id,
        result: Some(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {
                    "listChanged": true
                }
            },
            "serverInfo": {
                "name": "computeruse",
                "version": "0.2.0"
            }
        })),
        error: None,
    }
}

pub fn get_all_tools() -> Vec<Value> {
    vec![
        json!({
            "name": "cu_observe",
            "description": "Inspect the active foreground window and extract a categorized, ultra-compact Set-of-Marks (SoM) digest with numeric IDs for zero-waste navigation.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "verbose": { "type": "boolean", "description": "Optional: set true to include full raw marks array with bounding boxes", "default": false }
                }
            }
        }),
        json!({
            "name": "cu_click_mark",
            "description": "Click on an interactive UI element by its Set-of-Marks mark_id, optionally holding modifiers (e.g. ['ctrl'], ['shift']).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "mark_id": { "type": "integer", "description": "The mark_id number from cu_observe" },
                    "modifiers": { "type": "array", "items": { "type": "string" }, "description": "Optional held modifiers e.g. ['ctrl', 'shift']" }
                },
                "required": ["mark_id"]
            }
        }),
        json!({
            "name": "cu_click_name",
            "description": "Click on an interactive UI element matching a name/label substring.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "name": { "type": "string", "description": "Name or text of the button/control" }
                },
                "required": ["name"]
            }
        }),
        json!({
            "name": "cu_click_rel",
            "description": "Click at normalized (0.0..1.0) coordinates relative to the active window, optionally with held modifiers.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "x": { "type": "number", "description": "Normalized X (0.0=left, 1.0=right)" },
                    "y": { "type": "number", "description": "Normalized Y (0.0=top, 1.0=bottom)" },
                    "button": { "type": "string", "enum": ["left", "right", "middle"], "default": "left" },
                    "modifiers": { "type": "array", "items": { "type": "string" }, "description": "Optional held modifiers e.g. ['ctrl', 'shift']" }
                },
                "required": ["x", "y"]
            }
        }),
        json!({
            "name": "cu_click_coords",
            "description": "Click at absolute physical screen coordinates (x, y), optionally with held modifiers.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "x": { "type": "integer" },
                    "y": { "type": "integer" },
                    "button": { "type": "string", "enum": ["left", "right", "middle"], "default": "left" },
                    "modifiers": { "type": "array", "items": { "type": "string" }, "description": "Optional held modifiers e.g. ['ctrl', 'shift']" }
                },
                "required": ["x", "y"]
            }
        }),
        json!({
            "name": "cu_double_click",
            "description": "Double-click on an interactive element by mark_id or coordinates.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "mark_id": { "type": "integer", "description": "Optional mark_id to double-click" },
                    "x": { "type": "integer", "description": "Optional X coordinate" },
                    "y": { "type": "integer", "description": "Optional Y coordinate" }
                }
            }
        }),
        json!({
            "name": "cu_triple_click",
            "description": "Triple-click on an interactive element by mark_id or coordinates to select an entire line or paragraph.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "mark_id": { "type": "integer", "description": "Optional mark_id to triple-click" },
                    "x": { "type": "integer", "description": "Optional X coordinate" },
                    "y": { "type": "integer", "description": "Optional Y coordinate" }
                }
            }
        }),
        json!({
            "name": "cu_drag_mark_to_mark",
            "description": "Drag directly from one Set-of-Marks element (from_mark) onto another (to_mark), optionally holding modifiers (e.g. ['ctrl'] or ['alt']).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "from_mark": { "type": "integer", "description": "Source mark_id to drag from" },
                    "to_mark": { "type": "integer", "description": "Destination mark_id to drop onto" },
                    "modifiers": { "type": "array", "items": { "type": "string" }, "description": "Optional held modifiers e.g. ['ctrl', 'shift', 'alt']" }
                },
                "required": ["from_mark", "to_mark"]
            }
        }),
        json!({
            "name": "cu_copy_selection",
            "description": "Copy the currently highlighted text/selection (Ctrl+C) while preserving the user's previous clipboard contents, returning the selected text.",
            "inputSchema": {
                "type": "object",
                "properties": {}
            }
        }),
        json!({
            "name": "cu_clipboard_set_image",
            "description": "Copy a screen region directly into the Windows Clipboard as a native CF_BITMAP image so it can be pasted (Ctrl+V) into any application.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "x": { "type": "integer" },
                    "y": { "type": "integer" },
                    "width": { "type": "integer" },
                    "height": { "type": "integer" }
                },
                "required": ["x", "y", "width", "height"]
            }
        }),
        json!({
            "name": "cu_right_click_mark",
            "description": "Right-click on an interactive UI element by its Set-of-Marks mark_id to open context menus.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "mark_id": { "type": "integer", "description": "The mark_id number from cu_observe" }
                },
                "required": ["mark_id"]
            }
        }),
        json!({
            "name": "cu_hover_mark",
            "description": "Move the mouse cursor to hover over an interactive UI element by its Set-of-Marks mark_id.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "mark_id": { "type": "integer", "description": "The mark_id number from cu_observe" }
                },
                "required": ["mark_id"]
            }
        }),
        json!({
            "name": "cu_fill_mark",
            "description": "Click an input field by mark_id, optionally clear its existing text (Ctrl+A -> Backspace), type/paste text, and optionally press Enter in a single atomic call.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "mark_id": { "type": "integer", "description": "The mark_id of the target input field" },
                    "text": { "type": "string", "description": "Text to enter into the field" },
                    "clear": { "type": "boolean", "description": "Whether to clear existing text first (default: true)", "default": true },
                    "submit": { "type": "boolean", "description": "Whether to press Enter after typing (default: false)", "default": false }
                },
                "required": ["mark_id", "text"]
            }
        }),
        json!({
            "name": "cu_wait_for_text",
            "description": "Reactively poll the active window until a target text substring appears in the UI tree or window text (up to timeout_ms).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "text": { "type": "string", "description": "Case-insensitive text substring to wait for" },
                    "timeout_ms": { "type": "integer", "description": "Maximum wait time in milliseconds (default: 3000)", "default": 3000 }
                },
                "required": ["text"]
            }
        }),
        json!({
            "name": "cu_type",
            "description": "Type or paste text into the currently focused window or control.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "text": { "type": "string", "description": "Text string to type" },
                    "paste": { "type": "boolean", "description": "Optional: set true to paste instantaneously via clipboard (ideal for long strings)", "default": false }
                },
                "required": ["text"]
            }
        }),
        json!({
            "name": "cu_key",
            "description": "Press a keyboard key (e.g. 'enter', 'tab', 'escape', 'delete', 'backspace', 'down', 'up').",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "key": { "type": "string", "description": "Key name to press" }
                },
                "required": ["key"]
            }
        }),
        json!({
            "name": "cu_hotkey",
            "description": "Dispatch a hotkey combination (e.g. 'ctrl+c', 'ctrl+v', 'ctrl+t', 'ctrl+w', 'alt+tab', 'win+r').",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "combo": { "type": "string", "description": "Hotkey string e.g. 'ctrl+t'" }
                },
                "required": ["combo"]
            }
        }),
        json!({
            "name": "cu_scroll",
            "description": "Scroll the mouse wheel, optionally at a specific mark_id or (x,y) container and with optional modifiers (e.g. ['ctrl'] to zoom).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "amount": { "type": "integer", "description": "Scroll delta in clicks (e.g. -5 for down, 5 for up)" },
                    "horizontal": { "type": "boolean", "default": false },
                    "mark_id": { "type": "integer", "description": "Optional mark_id of the scrollable container" },
                    "x": { "type": "integer", "description": "Optional X coordinate to hover before scrolling" },
                    "y": { "type": "integer", "description": "Optional Y coordinate to hover before scrolling" },
                    "modifiers": { "type": "array", "items": { "type": "string" }, "description": "Optional held modifiers e.g. ['ctrl'] for zoom" }
                },
                "required": ["amount"]
            }
        }),
        json!({
            "name": "cu_move",
            "description": "Move the mouse cursor to absolute or relative coordinates, optionally using a smooth human ease-in-out glide.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "x": { "type": "integer", "description": "Absolute X screen position" },
                    "y": { "type": "integer", "description": "Absolute Y screen position" },
                    "dx": { "type": "integer", "description": "Relative X delta" },
                    "dy": { "type": "integer", "description": "Relative Y delta" },
                    "smooth_ms": { "type": "integer", "description": "Optional duration in ms for smooth human glide trajectory (e.g. 120)" }
                }
            }
        }),
        json!({
            "name": "cu_drag",
            "description": "Drag the mouse between absolute screen coordinates (from_x, from_y) -> (to_x, to_y), optionally holding modifiers (e.g. ['shift'], ['alt']).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "from_x": { "type": "integer" },
                    "from_y": { "type": "integer" },
                    "to_x": { "type": "integer" },
                    "to_y": { "type": "integer" },
                    "modifiers": { "type": "array", "items": { "type": "string" }, "description": "Optional held modifiers e.g. ['shift', 'alt']" }
                },
                "required": ["from_x", "from_y", "to_x", "to_y"]
            }
        }),
        json!({
            "name": "cu_drag_rel",
            "description": "Drag the mouse across normalized window coordinates (0.0..1.0), optionally holding modifiers.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "from_x": { "type": "number" },
                    "from_y": { "type": "number" },
                    "to_x": { "type": "number" },
                    "to_y": { "type": "number" },
                    "modifiers": { "type": "array", "items": { "type": "string" }, "description": "Optional held modifiers e.g. ['shift', 'alt']" }
                },
                "required": ["from_x", "from_y", "to_x", "to_y"]
            }
        }),
        json!({
            "name": "cu_mouse_down",
            "description": "Hold down a mouse button ('left', 'right', 'middle').",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "button": { "type": "string", "enum": ["left", "right", "middle"], "default": "left" }
                }
            }
        }),
        json!({
            "name": "cu_mouse_up",
            "description": "Release a held mouse button ('left', 'right', 'middle').",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "button": { "type": "string", "enum": ["left", "right", "middle"], "default": "left" }
                }
            }
        }),
        json!({
            "name": "cu_canvas_draw",
            "description": "HIGH-PERFORMANCE VECTOR DRAWING: Execute multi-layered geometric artwork on any window's drawing surface (gears, stars, spirals, polygons, lines, text, bezier curves) in a single atomic tool call.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "hwnd": { "type": "integer", "description": "Optional HWND of target window" },
                    "tool": { "type": "string", "description": "Optional tool name to select (e.g. 'Pencil', 'Brushes')" },
                    "color": { "type": "string", "description": "Optional initial color to select (e.g. 'Red', 'Gold', 'Turquoise', 'Purple', 'Orange', 'Green')" },
                    "commands": {
                        "type": "array",
                        "description": "List of geometric actions to execute. Types: 'gear', 'star', 'polygon', 'circle', 'spiral', 'line', 'rect', 'bezier', 'text'",
                        "items": { "type": "object" }
                    }
                },
                "required": ["commands"]
            }
        }),
        json!({
            "name": "cu_canvas_clear",
            "description": "Clear the active drawing canvas with Ctrl+A -> Delete.",
            "inputSchema": {
                "type": "object",
                "properties": {}
            }
        }),
        json!({
            "name": "cu_batch",
            "description": "Execute a pipeline of actions atomically in one call (supports click_mark, click_name, type, key, hotkey, scroll, focus_window, wait, wait_for_change) and return the final UI state.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "actions": {
                        "type": "array",
                        "items": { "type": "object" },
                        "description": "Array of action objects e.g. [{\"action\": \"click_name\", \"name\": \"File\"}, {\"action\": \"wait_for_change\", \"timeout_ms\": 1500}]"
                    }
                },
                "required": ["actions"]
            }
        }),
        json!({
            "name": "cu_wait_for_change",
            "description": "Reactively poll the foreground window and UI Automation tree in 25ms ticks until a structural UI transition occurs (menu/popup opens, title changes, or window switches) or timeout_ms elapses.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "timeout_ms": { "type": "integer", "description": "Max milliseconds to wait for a UI transition (default: 2000)", "default": 2000 }
                }
            }
        }),
        json!({
            "name": "cu_focus_window",
            "description": "Bring an application window to focus by title substring.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "title": { "type": "string", "description": "Title substring of the target window" }
                },
                "required": ["title"]
            }
        }),
        json!({
            "name": "cu_list_windows",
            "description": "List all visible top-level windows on the desktop.",
            "inputSchema": {
                "type": "object",
                "properties": {}
            }
        }),
        json!({
            "name": "cu_close_window",
            "description": "Close an application window by title or hwnd.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "title": { "type": "string", "description": "Title substring of window to close" },
                    "hwnd": { "type": "integer", "description": "Optional HWND of window to close" }
                }
            }
        }),
        json!({
            "name": "cu_maximize_window",
            "description": "Maximize a window by HWND or title substring.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "hwnd": { "type": "integer" },
                    "title": { "type": "string" }
                }
            }
        }),
        json!({
            "name": "cu_minimize_window",
            "description": "Minimize a window by HWND or title substring.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "hwnd": { "type": "integer" },
                    "title": { "type": "string" }
                }
            }
        }),
        json!({
            "name": "cu_snap_window",
            "description": "Snap a window to a human workspace layout ('left', 'right', 'top', 'bottom', 'top_left', 'top_right', 'bottom_left', 'bottom_right', 'center', 'maximize') on its current monitor.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "position": { "type": "string", "description": "Snap position: 'left', 'right', 'top', 'bottom', 'top_left', 'top_right', 'bottom_left', 'bottom_right', 'center', 'maximize'" },
                    "hwnd": { "type": "integer", "description": "Optional HWND (defaults to foreground window if title also omitted)" },
                    "title": { "type": "string", "description": "Optional window title substring" }
                },
                "required": ["position"]
            }
        }),
        json!({
            "name": "cu_screenshot",
            "description": "Capture a screenshot of the active window, specific HWND, a specific SoM mark_id bounding box, canvas_only, or a custom screen region. Optionally saves directly to disk.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "hwnd": { "type": "integer", "description": "Optional HWND. If omitted, captures active window." },
                    "mark_id": { "type": "integer", "description": "Optional Set-of-Marks ID to crop directly around that UI element." },
                    "canvas_only": { "type": "boolean", "description": "Optional: crop exclusively to the detected drawing canvas surface." },
                    "region": {
                        "type": "object",
                        "description": "Optional custom screen region {x, y, width, height} to crop.",
                        "properties": {
                            "x": { "type": "integer" },
                            "y": { "type": "integer" },
                            "width": { "type": "integer" },
                            "height": { "type": "integer" }
                        }
                    },
                    "save_path": { "type": "string", "description": "Optional file path to save the PNG directly to disk." },
                    "omit_base64": { "type": "boolean", "description": "Optional: omit png_base64 from response when save_path is used to save tokens.", "default": false },
                    "annotate_marks": { "type": "boolean", "description": "Optional: overlay numbered Set-of-Marks [ID] badges onto all interactive elements in the screenshot.", "default": false }
                }
            }
        }),
        json!({
            "name": "cu_read_window_text",
            "description": "Extract all visible text strings, labels, and live control values from a window (or active foreground window) in milliseconds without OCR.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "hwnd": { "type": "integer", "description": "Optional HWND. If omitted, reads active foreground window." }
                }
            }
        }),
        json!({
            "name": "cu_find_color",
            "description": "Scan a screen region (or active window) in <3ms for the first pixel matching target RGB [r, g, b] within tolerance.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "r": { "type": "integer", "description": "Red channel (0..255)" },
                    "g": { "type": "integer", "description": "Green channel (0..255)" },
                    "b": { "type": "integer", "description": "Blue channel (0..255)" },
                    "tolerance": { "type": "integer", "description": "Per-channel tolerance (default: 10)", "default": 10 },
                    "region": {
                        "type": "object",
                        "description": "Optional region {x, y, width, height}. Defaults to active window.",
                        "properties": {
                            "x": { "type": "integer" },
                            "y": { "type": "integer" },
                            "width": { "type": "integer" },
                            "height": { "type": "integer" }
                        }
                    }
                },
                "required": ["r", "g", "b"]
            }
        }),
        json!({
            "name": "cu_count_color_ratio",
            "description": "Calculate the proportion (0.0 to 1.0) and pixel count of target RGB [r, g, b] in a screen region (useful for visual verification, loading spinners, state badges, canvas fills).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "r": { "type": "integer", "description": "Red channel (0..255)" },
                    "g": { "type": "integer", "description": "Green channel (0..255)" },
                    "b": { "type": "integer", "description": "Blue channel (0..255)" },
                    "tolerance": { "type": "integer", "description": "Per-channel tolerance (default: 10)", "default": 10 },
                    "region": {
                        "type": "object",
                        "description": "Optional region {x, y, width, height}. Defaults to active window.",
                        "properties": {
                            "x": { "type": "integer" },
                            "y": { "type": "integer" },
                            "width": { "type": "integer" },
                            "height": { "type": "integer" }
                        }
                    }
                },
                "required": ["r", "g", "b"]
            }
        }),
        json!({
            "name": "cu_clipboard_get",
            "description": "Read text from the Windows clipboard.",
            "inputSchema": {
                "type": "object",
                "properties": {}
            }
        }),
        json!({
            "name": "cu_clipboard_set",
            "description": "Write text into the Windows clipboard.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "text": { "type": "string", "description": "Text to set" }
                },
                "required": ["text"]
            }
        }),
        json!({
            "name": "cu_launch_app",
            "description": "Launch an application or URL (e.g. 'mspaint', 'notepad', 'chrome https://google.com').",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "app": { "type": "string", "description": "Application executable or path" },
                    "params": { "type": "string", "description": "Optional command line parameters" },
                    "working_dir": { "type": "string", "description": "Optional working directory path" }
                },
                "required": ["app"]
            }
        }),
        json!({
            "name": "cu_list_processes",
            "description": "List running Windows processes with PID and executable names.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "filter": { "type": "string", "description": "Optional name substring filter" }
                }
            }
        }),
        json!({
            "name": "cu_kill_process",
            "description": "Terminate a process by its PID.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "pid": { "type": "integer", "description": "Process ID (PID) to terminate" }
                },
                "required": ["pid"]
            }
        }),
        json!({
            "name": "cu_inspect_element_at",
            "description": "Inspect UI Automation properties of the element directly under screen coordinates (x, y).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "x": { "type": "integer" },
                    "y": { "type": "integer" }
                },
                "required": ["x", "y"]
            }
        }),
        json!({
            "name": "cu_audio_beep",
            "description": "Emit an audible notification tone using the Windows system speaker.",
            "inputSchema": {
                "type": "object",
                "properties": {}
            }
        }),
        json!({
            "name": "cu_audio_volume",
            "description": "Inspect or adjust Windows master audio volume.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "set_percent": { "type": "integer", "description": "Optional volume percentage 0..100" }
                }
            }
        }),
        json!({
            "name": "cu_sysinfo",
            "description": "Get system RAM, CPU cores, uptime, battery status, and drives.",
            "inputSchema": {
                "type": "object",
                "properties": {}
            }
        }),
        json!({
            "name": "cu_resolve_association",
            "description": "Resolve a file extension or URI protocol scheme (e.g. '.pdf', '.png', 'https', 'mailto') to the registered default executable on Windows.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "target": { "type": "string", "description": "Extension or protocol (e.g. '.pdf', 'https')" }
                },
                "required": ["target"]
            }
        }),
        json!({
            "name": "cu_system_sound",
            "description": "Play a standard Windows audio notification cue ('default', 'error', 'warning', 'question', 'asterisk').",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "sound": { "type": "string", "enum": ["default", "error", "warning", "question", "asterisk"], "default": "default" }
                }
            }
        }),
        json!({
            "name": "cu_special_folder",
            "description": "Get the resolved absolute path of a Windows known folder (e.g. 'desktop', 'downloads', 'documents', 'appdata', 'temp', 'pictures').",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "name": { "type": "string", "description": "Folder name (e.g. 'desktop', 'downloads', 'documents', 'appdata', 'temp')" }
                },
                "required": ["name"]
            }
        }),
        json!({
            "name": "cu_sync_schemas",
            "description": "Force refresh and update all MCP schema files on disk in Antigravity's directory.",
            "inputSchema": {
                "type": "object",
                "properties": {}
            }
        }),
    ]
}

fn handle_tools_list(id: Value) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0",
        id,
        result: Some(json!({ "tools": get_all_tools() })),
        error: None,
    }
}

fn handle_tools_call(id: Value, params: Value, env: &mut AgentEnvironment) -> JsonRpcResponse {
    let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let args = params.get("arguments").cloned().unwrap_or(json!({}));

    let result: std::result::Result<Value, String> = (|| -> std::result::Result<Value, String> {
        match name {
        "cu_observe" => {
            let verbose = args.get("verbose").and_then(|v| v.as_bool()).unwrap_or(false);
            match env.observe() {
                Ok(obs) => {
                    let canvas_info = DynamicCanvas::find_in_window(obs.window_hwnd).ok().map(|c| json!({
                        "origin": [c.origin_x, c.origin_y],
                        "width": c.width,
                        "height": c.height,
                        "aspect_ratio": c.aspect_ratio
                    }));
                    let mut resp = json!({
                        "window": {
                            "title": obs.window_title,
                            "hwnd": obs.window_hwnd,
                            "bounds": obs.window_rect,
                            "is_maximized": obs.is_maximized
                        },
                        "canvas": canvas_info,
                        "cursor": obs.cursor_pos,
                        "clipboard_preview": obs.clipboard_preview,
                        "interactive_targets_count": obs.marks.len(),
                        "prompt_summary": obs.compact_prompt
                    });
                    if verbose {
                        resp["marks"] = json!(obs.marks);
                    }
                    Ok(resp)
                },
                Err(e) => Err(e.to_string()),
            }
        },

        "cu_click_mark" => {
            let mark_id = args.get("mark_id").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
            let modifiers = args.get("modifiers").and_then(|v| v.as_array()).map(|arr| {
                arr.iter().filter_map(|k| k.as_str().map(|s| s.to_string())).collect()
            });
            match env.step(crate::AgentAction::ClickMark { mark_id, modifiers }) {
                Ok(res) => Ok(json!({
                    "success": res.success,
                    "message": res.message,
                    "active_window": res.observation.window_title,
                    "ui_delta": res.delta.summary
                })),
                Err(e) => Err(e.to_string()),
            }
        }

        "cu_click_name" => {
            let el_name = args.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
            match env.step(crate::AgentAction::ClickName { name: el_name }) {
                Ok(res) => Ok(json!({
                    "success": res.success,
                    "message": res.message,
                    "active_window": res.observation.window_title,
                    "ui_delta": res.delta.summary
                })),
                Err(e) => Err(e.to_string()),
            }
        }

        "cu_click_rel" => {
            let x = args.get("x").and_then(|v| v.as_f64()).unwrap_or(0.5);
            let y = args.get("y").and_then(|v| v.as_f64()).unwrap_or(0.5);
            let btn_str = args.get("button").and_then(|v| v.as_str()).unwrap_or("left");
            let button = match btn_str {
                "right" => Some(MouseButton::Right),
                "middle" => Some(MouseButton::Middle),
                _ => Some(MouseButton::Left),
            };
            let modifiers = args.get("modifiers").and_then(|v| v.as_array()).map(|arr| {
                arr.iter().filter_map(|k| k.as_str().map(|s| s.to_string())).collect()
            });
            match env.step(crate::AgentAction::ClickRel { x, y, button, modifiers }) {
                Ok(res) => Ok(json!({ "success": res.success, "message": res.message })),
                Err(e) => Err(e.to_string()),
            }
        }

        "cu_click_coords" => {
            let x = args.get("x").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            let y = args.get("y").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            let btn_str = args.get("button").and_then(|v| v.as_str()).unwrap_or("left");
            let button = match btn_str {
                "right" => MouseButton::Right,
                "middle" => MouseButton::Middle,
                _ => MouseButton::Left,
            };
            let modifiers: Vec<String> = args.get("modifiers").and_then(|v| v.as_array()).map(|arr| {
                arr.iter().filter_map(|k| k.as_str().map(|s| s.to_string())).collect()
            }).unwrap_or_default();
            match mouse::click_modified(button, Some(Point { x, y }), &modifiers) {
                Ok(_) => Ok(json!({ "success": true, "x": x, "y": y })),
                Err(e) => Err(e.to_string()),
            }
        }

        "cu_double_click" => {
            if let Some(mark_id) = args.get("mark_id").and_then(|v| v.as_u64()) {
                match env.step(crate::AgentAction::DoubleClickMark { mark_id: mark_id as usize }) {
                    Ok(res) => Ok(json!({ "success": res.success, "message": res.message })),
                    Err(e) => Err(e.to_string()),
                }
            } else {
                let x = args.get("x").and_then(|v| v.as_i64()).map(|v| v as i32);
                let y = args.get("y").and_then(|v| v.as_i64()).map(|v| v as i32);
                let pt = if let (Some(x), Some(y)) = (x, y) { Some(Point { x, y }) } else { None };
                match mouse::double_click(MouseButton::Left, pt) {
                    Ok(_) => Ok(json!({ "success": true })),
                    Err(e) => Err(e.to_string()),
                }
            }
        }

        "cu_triple_click" => {
            let mark_id = args.get("mark_id").and_then(|v| v.as_u64()).map(|v| v as usize);
            let x = args.get("x").and_then(|v| v.as_i64()).map(|v| v as i32);
            let y = args.get("y").and_then(|v| v.as_i64()).map(|v| v as i32);
            match env.step(crate::AgentAction::TripleClick { mark_id, x, y }) {
                Ok(res) => Ok(json!({ "success": res.success, "message": res.message })),
                Err(e) => Err(e.to_string()),
            }
        }

        "cu_drag_mark_to_mark" => {
            let from_mark = args.get("from_mark").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
            let to_mark = args.get("to_mark").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
            let modifiers = args.get("modifiers").and_then(|v| v.as_array()).map(|arr| {
                arr.iter().filter_map(|k| k.as_str().map(|s| s.to_string())).collect()
            });
            match env.step(crate::AgentAction::DragMarkToMark { from_mark, to_mark, modifiers }) {
                Ok(res) => Ok(json!({
                    "success": res.success,
                    "message": res.message,
                    "active_window": res.observation.window_title,
                    "ui_delta": res.delta.summary
                })),
                Err(e) => Err(e.to_string()),
            }
        }

        "cu_copy_selection" => {
            let selected = keyboard::copy_selection().map_err(|e| e.to_string())?;
            Ok(json!({ "success": true, "selected_text": selected, "length": selected.len() }))
        }

        "cu_clipboard_set_image" => {
            let x = args.get("x").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            let y = args.get("y").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            let width = args.get("width").and_then(|v| v.as_u64()).unwrap_or(200).max(1) as u32;
            let height = args.get("height").and_then(|v| v.as_u64()).unwrap_or(200).max(1) as u32;
            clipboard::set_image_from_region(Rect { x, y, width, height }).map_err(|e| e.to_string())?;
            Ok(json!({ "success": true, "copied_region": { "x": x, "y": y, "width": width, "height": height } }))
        }

        "cu_right_click_mark" => {
            let mark_id = args.get("mark_id").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
            match env.step(crate::AgentAction::RightClickMark { mark_id }) {
                Ok(res) => Ok(json!({
                    "success": res.success,
                    "message": res.message,
                    "active_window": res.observation.window_title,
                    "ui_delta": res.delta.summary
                })),
                Err(e) => Err(e.to_string()),
            }
        }

        "cu_hover_mark" => {
            let mark_id = args.get("mark_id").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
            match env.step(crate::AgentAction::HoverMark { mark_id }) {
                Ok(res) => Ok(json!({
                    "success": res.success,
                    "message": res.message,
                    "active_window": res.observation.window_title,
                    "ui_delta": res.delta.summary
                })),
                Err(e) => Err(e.to_string()),
            }
        }

        "cu_fill_mark" => {
            let mark_id = args.get("mark_id").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
            let text = args.get("text").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let clear = args.get("clear").and_then(|v| v.as_bool());
            let submit = args.get("submit").and_then(|v| v.as_bool());
            match env.step(crate::AgentAction::FillMark { mark_id, text, clear, submit }) {
                Ok(res) => Ok(json!({
                    "success": res.success,
                    "message": res.message,
                    "active_window": res.observation.window_title,
                    "ui_delta": res.delta.summary,
                    "prompt_summary": res.observation.compact_prompt
                })),
                Err(e) => Err(e.to_string()),
            }
        }

        "cu_wait_for_text" => {
            let text = args.get("text").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let timeout_ms = args.get("timeout_ms").and_then(|v| v.as_u64()).unwrap_or(3000);
            let (obs, elapsed) = env.wait_for_text(&text, timeout_ms).map_err(|e| e.to_string())?;
            Ok(json!({
                "found": true,
                "text": text,
                "elapsed_ms": elapsed,
                "active_window": obs.window_title,
                "prompt_summary": obs.compact_prompt
            }))
        }

        "cu_type" => {
            let text = args.get("text").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let use_paste = args.get("paste").and_then(|v| v.as_bool()).unwrap_or(false);
            if use_paste {
                keyboard::paste_text(&text).map_err(|e| e.to_string())?;
                Ok(json!({ "success": true, "message": format!("Pasted {} chars via clipboard", text.len()) }))
            } else {
                match env.step(crate::AgentAction::TypeText { text }) {
                    Ok(res) => Ok(json!({ "success": res.success, "message": res.message })),
                    Err(e) => Err(e.to_string()),
                }
            }
        }

        "cu_key" => {
            let key = args.get("key").and_then(|v| v.as_str()).unwrap_or("").to_string();
            match env.step(crate::AgentAction::KeyPress { key }) {
                Ok(res) => Ok(json!({ "success": res.success, "message": res.message })),
                Err(e) => Err(e.to_string()),
            }
        }

        "cu_hotkey" => {
            let combo = args.get("combo").and_then(|v| v.as_str()).unwrap_or("").to_string();
            match env.step(crate::AgentAction::HotkeyStr { combo }) {
                Ok(res) => Ok(json!({ "success": res.success, "message": res.message })),
                Err(e) => Err(e.to_string()),
            }
        }

        "cu_scroll" => {
            let amount = args.get("amount").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            let horizontal = args.get("horizontal").and_then(|v| v.as_bool());
            let mark_id = args.get("mark_id").and_then(|v| v.as_u64()).map(|v| v as usize);
            let x = args.get("x").and_then(|v| v.as_i64()).map(|v| v as i32);
            let y = args.get("y").and_then(|v| v.as_i64()).map(|v| v as i32);
            let modifiers = args.get("modifiers").and_then(|v| v.as_array()).map(|arr| {
                arr.iter().filter_map(|k| k.as_str().map(|s| s.to_string())).collect()
            });
            match env.step(crate::AgentAction::Scroll { amount, horizontal, mark_id, x, y, modifiers }) {
                Ok(res) => Ok(json!({ "success": res.success, "message": res.message })),
                Err(e) => Err(e.to_string()),
            }
        }

        "cu_move" => {
            if let (Some(dx), Some(dy)) = (args.get("dx").and_then(|v| v.as_i64()), args.get("dy").and_then(|v| v.as_i64())) {
                match mouse::move_relative(dx as i32, dy as i32) {
                    Ok(pt) => Ok(json!({ "success": true, "x": pt.x, "y": pt.y })),
                    Err(e) => Err(e.to_string()),
                }
            } else {
                let x = args.get("x").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                let y = args.get("y").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                if let Some(smooth_ms) = args.get("smooth_ms").and_then(|v| v.as_u64()) {
                    match mouse::move_smooth(Point { x, y }, smooth_ms) {
                        Ok(_) => Ok(json!({ "success": true, "x": x, "y": y, "smooth_ms": smooth_ms })),
                        Err(e) => Err(e.to_string()),
                    }
                } else {
                    match mouse::move_to(Point { x, y }) {
                        Ok(_) => Ok(json!({ "success": true, "x": x, "y": y })),
                        Err(e) => Err(e.to_string()),
                    }
                }
            }
        }

        "cu_drag" => {
            let from_x = args.get("from_x").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            let from_y = args.get("from_y").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            let to_x = args.get("to_x").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            let to_y = args.get("to_y").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            let modifiers: Vec<String> = args.get("modifiers").and_then(|v| v.as_array()).map(|arr| {
                arr.iter().filter_map(|k| k.as_str().map(|s| s.to_string())).collect()
            }).unwrap_or_default();
            match mouse::drag_modified(Point { x: from_x, y: from_y }, Point { x: to_x, y: to_y }, MouseButton::Left, &modifiers) {
                Ok(_) => Ok(json!({ "success": true, "from": [from_x, from_y], "to": [to_x, to_y] })),
                Err(e) => Err(e.to_string()),
            }
        }

        "cu_drag_rel" => {
            let from_x = args.get("from_x").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let from_y = args.get("from_y").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let to_x = args.get("to_x").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let to_y = args.get("to_y").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let modifiers = args.get("modifiers").and_then(|v| v.as_array()).map(|arr| {
                arr.iter().filter_map(|k| k.as_str().map(|s| s.to_string())).collect()
            });
            match env.step(crate::AgentAction::DragRel { from_x, from_y, to_x, to_y, modifiers }) {
                Ok(res) => Ok(json!({ "success": res.success, "message": res.message })),
                Err(e) => Err(e.to_string()),
            }
        }

        "cu_mouse_down" => {
            let btn_str = args.get("button").and_then(|v| v.as_str()).unwrap_or("left");
            let button = match btn_str {
                "right" => MouseButton::Right,
                "middle" => MouseButton::Middle,
                _ => MouseButton::Left,
            };
            match mouse::mouse_down(button) {
                Ok(_) => Ok(json!({ "success": true, "button": btn_str })),
                Err(e) => Err(e.to_string()),
            }
        }

        "cu_mouse_up" => {
            let btn_str = args.get("button").and_then(|v| v.as_str()).unwrap_or("left");
            let button = match btn_str {
                "right" => MouseButton::Right,
                "middle" => MouseButton::Middle,
                _ => MouseButton::Left,
            };
            match mouse::mouse_up(button) {
                Ok(_) => Ok(json!({ "success": true, "button": btn_str })),
                Err(e) => Err(e.to_string()),
            }
        }

        "cu_canvas_draw" => {
            let hwnd = args.get("hwnd").and_then(|v| v.as_i64()).map(|h| h as isize)
                .unwrap_or_else(|| {
                    if let Some(ref som) = env.last_som() {
                        som.hwnd
                    } else if let Ok(win) = window::foreground_window() {
                        win.hwnd
                    } else {
                        0
                    }
                });

            if hwnd == 0 {
                Err("No active window to draw on".to_string())
            } else {
                // Ensure window is in focus
                let _ = window::focus_window(hwnd);
                std::thread::sleep(std::time::Duration::from_millis(120));

                // Auto-observe if last_som is missing or belongs to another window
                if env.last_som().map(|s| s.hwnd) != Some(hwnd) {
                    let _ = env.observe();
                }

                // Select tool if requested, or auto-select Pencil in Paint if no tool specified
                if let Some(tool_name) = args.get("tool").and_then(|v| v.as_str()) {
                    if let Some(ref som) = env.last_som() {
                        if let Some(el) = som.find_by_name(tool_name) {
                            let _ = mouse::click(MouseButton::Left, Some(el.center));
                            std::thread::sleep(std::time::Duration::from_millis(80));
                        }
                    }
                } else if let Some(ref som) = env.last_som() {
                    if let Some(el) = som.elements.iter().find(|e| e.automation_id == "PencilTool" || (e.name == "Pencil" && e.control_type == "Button")) {
                        let _ = mouse::click(MouseButton::Left, Some(el.center));
                        std::thread::sleep(std::time::Duration::from_millis(80));
                    }
                }

                // Select color if requested
                if let Some(color_name) = args.get("color").or_else(|| args.get("color_name")).and_then(|v| v.as_str()) {
                    if let Some(ref som) = env.last_som() {
                        if let Some(el) = som.find_by_name(color_name) {
                            let _ = mouse::click(MouseButton::Left, Some(el.center));
                            std::thread::sleep(std::time::Duration::from_millis(80));
                        }
                    }
                }

                // Detect Canvas surface
                match DynamicCanvas::find_in_window(hwnd) {
                    Err(e) => Err(format!("Could not detect canvas in window HWND {}: {}", hwnd, e)),
                    Ok(canvas) => {
                        let mut drawn_count = 0;
                        if let Some(cmds) = args.get("commands").and_then(|v| v.as_array()) {
                            for cmd in cmds {
                                let cmd_type = cmd.get("type").or_else(|| cmd.get("action")).and_then(|v| v.as_str()).unwrap_or("");
                                
                                // On-the-fly color change (accepts "color" or "color_name")
                                if let Some(col) = cmd.get("color").or_else(|| cmd.get("color_name")).and_then(|v| v.as_str()) {
                                    if let Some(ref som) = env.last_som() {
                                        if let Some(el) = som.find_by_name(col) {
                                            let _ = mouse::click(MouseButton::Left, Some(el.center));
                                            std::thread::sleep(std::time::Duration::from_millis(90));
                                        }
                                    }
                                }

                                match cmd_type {
                                    "gear" => {
                                        let cx = cmd.get("cx").and_then(|v| v.as_f64()).unwrap_or(0.5);
                                        let cy = cmd.get("cy").and_then(|v| v.as_f64()).unwrap_or(0.5);
                                        let outer_r = cmd.get("outer_r").or_else(|| cmd.get("r_outer")).or_else(|| cmd.get("radius")).and_then(|v| v.as_f64()).unwrap_or(0.3);
                                        let inner_r = cmd.get("inner_r").or_else(|| cmd.get("r_inner")).and_then(|v| v.as_f64()).unwrap_or(outer_r * 0.88);
                                        let teeth = cmd.get("teeth").and_then(|v| v.as_u64()).unwrap_or(16) as usize;
                                        let rot = cmd.get("rot").or_else(|| cmd.get("rot_deg")).or_else(|| cmd.get("rotation")).and_then(|v| v.as_f64()).unwrap_or(0.0);
                                        let _ = canvas.stroke_gear(cx, cy, outer_r, inner_r, teeth, rot);
                                        drawn_count += 1;
                                    }
                                    "star" => {
                                        let cx = cmd.get("cx").and_then(|v| v.as_f64()).unwrap_or(0.5);
                                        let cy = cmd.get("cy").and_then(|v| v.as_f64()).unwrap_or(0.5);
                                        let outer_r = cmd.get("outer_r").or_else(|| cmd.get("r_outer")).or_else(|| cmd.get("radius")).and_then(|v| v.as_f64()).unwrap_or(0.2);
                                        let inner_r = cmd.get("inner_r").or_else(|| cmd.get("r_inner")).and_then(|v| v.as_f64()).unwrap_or(outer_r * 0.4);
                                        let points = cmd.get("points").and_then(|v| v.as_u64()).unwrap_or(5) as usize;
                                        let rot = cmd.get("rot").or_else(|| cmd.get("rot_deg")).or_else(|| cmd.get("rotation")).and_then(|v| v.as_f64()).unwrap_or(0.0);
                                        let _ = canvas.stroke_star(cx, cy, outer_r, inner_r, points, rot);
                                        drawn_count += 1;
                                    }
                                    "polygon" => {
                                        let cx = cmd.get("cx").and_then(|v| v.as_f64()).unwrap_or(0.5);
                                        let cy = cmd.get("cy").and_then(|v| v.as_f64()).unwrap_or(0.5);
                                        let radius = cmd.get("radius").or_else(|| cmd.get("r")).and_then(|v| v.as_f64()).unwrap_or(0.25);
                                        let sides = cmd.get("sides").and_then(|v| v.as_u64()).unwrap_or(6) as usize;
                                        let rot = cmd.get("rot").or_else(|| cmd.get("rot_deg")).and_then(|v| v.as_f64()).unwrap_or(0.0);
                                        let _ = canvas.stroke_polygon(cx, cy, radius, sides, rot);
                                        drawn_count += 1;
                                    }
                                    "circle" => {
                                        let cx = cmd.get("cx").and_then(|v| v.as_f64()).unwrap_or(0.5);
                                        let cy = cmd.get("cy").and_then(|v| v.as_f64()).unwrap_or(0.5);
                                        let radius = cmd.get("radius").or_else(|| cmd.get("r")).and_then(|v| v.as_f64()).unwrap_or(0.2);
                                        let _ = canvas.stroke_polygon(cx, cy, radius, 48, 0.0);
                                        drawn_count += 1;
                                    }
                                    "spiral" => {
                                        let cx = cmd.get("cx").and_then(|v| v.as_f64()).unwrap_or(0.5);
                                        let cy = cmd.get("cy").and_then(|v| v.as_f64()).unwrap_or(0.5);
                                        let start_r = cmd.get("start_r").or_else(|| cmd.get("r_start")).and_then(|v| v.as_f64()).unwrap_or(0.02);
                                        let end_r = cmd.get("end_r").or_else(|| cmd.get("r_end")).or_else(|| cmd.get("radius")).and_then(|v| v.as_f64()).unwrap_or(0.3);
                                        let turns = cmd.get("turns").and_then(|v| v.as_f64()).unwrap_or(3.0);
                                        let steps = cmd.get("steps").or_else(|| cmd.get("points")).and_then(|v| v.as_u64()).unwrap_or(90) as usize;
                                        let _ = canvas.stroke_spiral(cx, cy, start_r, end_r, turns, steps);
                                        drawn_count += 1;
                                    }
                                    "line" => {
                                        let x1 = cmd.get("x1").and_then(|v| v.as_f64()).unwrap_or(0.0);
                                        let y1 = cmd.get("y1").and_then(|v| v.as_f64()).unwrap_or(0.0);
                                        let x2 = cmd.get("x2").and_then(|v| v.as_f64()).unwrap_or(1.0);
                                        let y2 = cmd.get("y2").and_then(|v| v.as_f64()).unwrap_or(1.0);
                                        let _ = canvas.stroke_line(x1, y1, x2, y2);
                                        drawn_count += 1;
                                    }
                                    "rect" => {
                                        let x1 = cmd.get("x1").and_then(|v| v.as_f64()).unwrap_or(0.1);
                                        let y1 = cmd.get("y1").and_then(|v| v.as_f64()).unwrap_or(0.1);
                                        let x2 = cmd.get("x2").and_then(|v| v.as_f64()).unwrap_or(0.9);
                                        let y2 = cmd.get("y2").and_then(|v| v.as_f64()).unwrap_or(0.9);
                                        let _ = canvas.stroke_polyline(&[(x1, y1), (x2, y1), (x2, y2), (x1, y2), (x1, y1)]);
                                        drawn_count += 1;
                                    }
                                    "bezier" => {
                                        let p0 = (cmd["p0"][0].as_f64().unwrap_or(0.0), cmd["p0"][1].as_f64().unwrap_or(0.0));
                                        let p1 = (cmd["p1"][0].as_f64().unwrap_or(0.3), cmd["p1"][1].as_f64().unwrap_or(0.2));
                                        let p2 = (cmd["p2"][0].as_f64().unwrap_or(0.7), cmd["p2"][1].as_f64().unwrap_or(0.8));
                                        let p3 = (cmd["p3"][0].as_f64().unwrap_or(1.0), cmd["p3"][1].as_f64().unwrap_or(1.0));
                                        let steps = cmd.get("steps").and_then(|v| v.as_u64()).unwrap_or(30) as usize;
                                        let _ = canvas.stroke_bezier(p0, p1, p2, p3, steps);
                                        drawn_count += 1;
                                    }
                                    "text" => {
                                        let text = cmd.get("text").and_then(|v| v.as_str()).unwrap_or("");
                                        let x = cmd.get("x").and_then(|v| v.as_f64()).unwrap_or(0.1);
                                        let y = cmd.get("y").and_then(|v| v.as_f64()).unwrap_or(0.8);
                                        let scale = cmd.get("scale").or_else(|| cmd.get("size")).and_then(|v| v.as_f64()).unwrap_or(0.04);
                                        let _ = canvas.stroke_text(x, y, text, scale);
                                        drawn_count += 1;
                                    }
                                    "dots" | "points" => {
                                        let radius_px = cmd.get("dot_radius").or_else(|| cmd.get("radius_px")).and_then(|v| v.as_i64()).unwrap_or(2) as i32;
                                        if let Some(pts_arr) = cmd.get("points").or_else(|| cmd.get("dots")).and_then(|v| v.as_array()) {
                                            let pts: Vec<(f64, f64)> = pts_arr.iter().filter_map(|p| {
                                                if let Some(a) = p.as_array() {
                                                    Some((a.first()?.as_f64()?, a.get(1)?.as_f64()?))
                                                } else if let Some(o) = p.as_object() {
                                                    Some((o.get("x")?.as_f64()?, o.get("y")?.as_f64()?))
                                                } else {
                                                    None
                                                }
                                            }).collect();
                                            let _ = canvas.stroke_dots(&pts, radius_px);
                                            drawn_count += 1;
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }

                        Ok(json!({
                            "success": true,
                            "commands_executed": drawn_count,
                            "canvas": {
                                "origin": [canvas.origin_x, canvas.origin_y],
                                "width": canvas.width,
                                "height": canvas.height,
                                "aspect_ratio": canvas.aspect_ratio
                            }
                        }))
                    }
                }
            }
        }

        "cu_canvas_clear" => {
            let _ = keyboard::combo_str("ctrl+a");
            std::thread::sleep(std::time::Duration::from_millis(150));
            let _ = keyboard::key_press("delete");
            std::thread::sleep(std::time::Duration::from_millis(150));
            // Automatically restore Pencil tool if present in last_som so Paint doesn't stay in Selection mode
            if let Some(ref som) = env.last_som() {
                if let Some(el) = som.elements.iter().find(|e| e.automation_id == "PencilTool" || (e.name == "Pencil" && e.control_type == "Button")) {
                    let _ = mouse::click(MouseButton::Left, Some(el.center));
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            }
            Ok(json!({ "success": true, "message": "Canvas cleared with Ctrl+A -> Delete and drawing tool restored" }))
        }

        "cu_batch" => {
            let actions = args.get("actions").and_then(|v| v.as_array()).cloned().unwrap_or_default();
            let mut results = Vec::new();
            let mut final_prompt = String::new();
            let mut final_window = String::new();
            for act_val in actions {
                let act = crate::AgentAction::from_flexible_json(&act_val).map_err(|e| e.to_string())?;
                let res = env.step(act).map_err(|e| e.to_string())?;
                final_prompt = res.observation.compact_prompt.clone();
                final_window = res.observation.window_title.clone();
                results.push(json!({
                    "success": res.success,
                    "message": res.message,
                    "ui_delta": res.delta.summary
                }));
            }
            Ok(json!({
                "success": true,
                "count": results.len(),
                "active_window": final_window,
                "results": results,
                "prompt_summary": final_prompt
            }))
        }

        "cu_wait_for_change" => {
            let timeout_ms = args.get("timeout_ms").and_then(|v| v.as_u64()).unwrap_or(2000);
            let (delta, obs, elapsed) = env.wait_for_change(timeout_ms).map_err(|e| e.to_string())?;
            Ok(json!({
                "changed": delta.hwnd_changed || delta.title_changed || delta.marks_delta != 0,
                "elapsed_ms": elapsed,
                "ui_delta": delta.summary,
                "active_window": obs.window_title,
                "interactive_targets_count": obs.marks.len(),
                "prompt_summary": obs.compact_prompt
            }))
        }

        "cu_focus_window" => {
            let title = args.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string();
            match env.step(crate::AgentAction::FocusWindow { title }) {
                Ok(res) => Ok(json!({ "success": res.success, "message": res.message, "active_window": res.observation.window_title })),
                Err(e) => Err(e.to_string()),
            }
        }

        "cu_list_windows" => {
            let include_all = args.get("all").and_then(|v| v.as_bool()).unwrap_or(false);
            match window::list_windows(!include_all) {
                Ok(wins) => {
                    let filtered: Vec<_> = if include_all {
                        wins
                    } else {
                        wins.into_iter().filter(|w| !w.title.trim().is_empty() && w.rect.width > 10 && w.rect.height > 10).collect()
                    };
                    Ok(json!({ "count": filtered.len(), "windows": filtered }))
                }
                Err(e) => Err(e.to_string()),
            }
        },

        "cu_close_window" => {
            let hwnd = args.get("hwnd").and_then(|v| v.as_i64()).map(|h| h as isize);
            let target_hwnd = if let Some(h) = hwnd {
                h
            } else if let Some(title) = args.get("title").and_then(|v| v.as_str()) {
                window::find_window(title).map(|w| w.hwnd).map_err(|e| e.to_string())?
            } else {
                return Err("Missing 'title' or 'hwnd'".into());
            };
            window::close_window(target_hwnd).map_err(|e| e.to_string())?;
            Ok(json!({ "success": true, "closed_hwnd": target_hwnd }))
        }

        "cu_maximize_window" => {
            let hwnd = args.get("hwnd").and_then(|v| v.as_i64()).map(|h| h as isize);
            let target_hwnd = if let Some(h) = hwnd {
                h
            } else if let Some(title) = args.get("title").and_then(|v| v.as_str()) {
                window::find_window(title).map(|w| w.hwnd).map_err(|e| e.to_string())?
            } else {
                window::foreground_window().map(|w| w.hwnd).map_err(|e| e.to_string())?
            };
            window::maximize_window(target_hwnd).map_err(|e| e.to_string())?;
            Ok(json!({ "success": true, "maximized_hwnd": target_hwnd }))
        }

        "cu_minimize_window" => {
            let hwnd = args.get("hwnd").and_then(|v| v.as_i64()).map(|h| h as isize);
            let target_hwnd = if let Some(h) = hwnd {
                h
            } else if let Some(title) = args.get("title").and_then(|v| v.as_str()) {
                window::find_window(title).map(|w| w.hwnd).map_err(|e| e.to_string())?
            } else {
                window::foreground_window().map(|w| w.hwnd).map_err(|e| e.to_string())?
            };
            window::minimize_window(target_hwnd).map_err(|e| e.to_string())?;
            Ok(json!({ "success": true, "minimized_hwnd": target_hwnd }))
        }

        "cu_snap_window" => {
            let position = args.get("position").and_then(|v| v.as_str()).unwrap_or("center");
            let hwnd = args.get("hwnd").and_then(|v| v.as_i64()).map(|h| h as isize);
            let target_hwnd = if let Some(h) = hwnd {
                h
            } else if let Some(title) = args.get("title").and_then(|v| v.as_str()) {
                window::find_window(title).map(|w| w.hwnd).map_err(|e| e.to_string())?
            } else {
                window::foreground_window().map(|w| w.hwnd).map_err(|e| e.to_string())?
            };
            let new_rect = window::snap_window(target_hwnd, position).map_err(|e| e.to_string())?;
            Ok(json!({ "success": true, "hwnd": target_hwnd, "position": position, "rect": new_rect }))
        }

        "cu_screenshot" => {
            let hwnd = args.get("hwnd").and_then(|v| v.as_i64()).map(|h| h as isize);
            let shot = if let Some(reg_obj) = args.get("region").and_then(|v| v.as_object()) {
                let x = reg_obj.get("x").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                let y = reg_obj.get("y").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                let width = reg_obj.get("width").and_then(|v| v.as_u64()).unwrap_or(200).max(1) as u32;
                let height = reg_obj.get("height").and_then(|v| v.as_u64()).unwrap_or(200).max(1) as u32;
                screen::screenshot_region(Rect { x, y, width, height }, None)
            } else if let Some(mark_id) = args.get("mark_id").and_then(|v| v.as_u64()).map(|m| m as usize) {
                if env.last_som().is_none() {
                    let _ = env.observe();
                }
                if let Some(el) = env.last_som().and_then(|s| s.find_by_mark(mark_id)) {
                    let pad = 8;
                    let r = Rect {
                        x: (el.rect.x - pad).max(0),
                        y: (el.rect.y - pad).max(0),
                        width: (el.rect.width + (pad as u32) * 2).max(16),
                        height: (el.rect.height + (pad as u32) * 2).max(16),
                    };
                    screen::screenshot_region(r, None)
                } else {
                    return Err(format!("Mark ID [{}] not found for screenshot crop", mark_id));
                }
            } else if args.get("canvas_only").and_then(|v| v.as_bool()).unwrap_or(false) {
                let target_hwnd = hwnd.or_else(|| window::foreground_window().ok().map(|w| w.hwnd)).unwrap_or(0);
                let c = DynamicCanvas::find_in_window(target_hwnd).map_err(|e| e.to_string())?;
                screen::screenshot_region(Rect {
                    x: c.origin_x,
                    y: c.origin_y,
                    width: c.width.max(16),
                    height: c.height.max(16),
                }, None)
            } else if let Some(h) = hwnd {
                screen::screenshot_window(h)
            } else {
                screen::screenshot_active_window()
            };

            match shot {
                Ok(mut s) => {
                    let annotate = args.get("annotate_marks").and_then(|v| v.as_bool()).unwrap_or(false);
                    if annotate {
                        if env.last_som().is_none() {
                            let _ = env.observe();
                        }
                        if let Some(som) = env.last_som() {
                            if let Ok(annotated_bytes) = som.generate_annotated_image() {
                                use base64::Engine;
                                s.png_base64 = base64::engine::general_purpose::STANDARD.encode(&annotated_bytes);
                            }
                        }
                    }
                    let save_path = args.get("save_path").and_then(|v| v.as_str());
                    let omit_b64 = args.get("omit_base64").and_then(|v| v.as_bool()).unwrap_or(false);
                    if let Some(p) = save_path {
                        use base64::Engine;
                        let bytes = base64::engine::general_purpose::STANDARD.decode(&s.png_base64).map_err(|e| e.to_string())?;
                        std::fs::write(p, &bytes).map_err(|e| e.to_string())?;
                    }
                    let mut resp = json!({
                        "width": s.width,
                        "height": s.height,
                        "region": s.region,
                        "annotated_marks": annotate
                    });
                    if let Some(p) = save_path {
                        resp["saved_to"] = json!(p);
                    }
                    if !omit_b64 {
                        resp["png_base64"] = json!(s.png_base64);
                    }
                    Ok(resp)
                },
                Err(e) => Err(e.to_string()),
            }
        }

        "cu_read_window_text" => {
            let hwnd = args.get("hwnd").and_then(|v| v.as_i64()).unwrap_or(0) as isize;
            let lines = uia::read_window_text(hwnd).map_err(|e| e.to_string())?;
            Ok(json!({ "count": lines.len(), "lines": lines }))
        }

        "cu_find_color" => {
            let r = args.get("r").and_then(|v| v.as_u64()).unwrap_or(0).min(255) as u8;
            let g = args.get("g").and_then(|v| v.as_u64()).unwrap_or(0).min(255) as u8;
            let b = args.get("b").and_then(|v| v.as_u64()).unwrap_or(0).min(255) as u8;
            let tol = args.get("tolerance").and_then(|v| v.as_u64()).unwrap_or(10).min(255) as u8;
            let region = if let Some(reg_obj) = args.get("region").and_then(|v| v.as_object()) {
                Rect {
                    x: reg_obj.get("x").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                    y: reg_obj.get("y").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                    width: reg_obj.get("width").and_then(|v| v.as_u64()).unwrap_or(400).max(1) as u32,
                    height: reg_obj.get("height").and_then(|v| v.as_u64()).unwrap_or(400).max(1) as u32,
                }
            } else {
                window::foreground_window().map(|w| w.rect).unwrap_or(Rect { x: 0, y: 0, width: 800, height: 600 })
            };
            let found = crate::pixel::find_color_in_region(region, (r, g, b), tol).map_err(|e| e.to_string())?;
            Ok(json!({ "found": found.is_some(), "point": found, "region": region }))
        }

        "cu_count_color_ratio" => {
            let r = args.get("r").and_then(|v| v.as_u64()).unwrap_or(0).min(255) as u8;
            let g = args.get("g").and_then(|v| v.as_u64()).unwrap_or(0).min(255) as u8;
            let b = args.get("b").and_then(|v| v.as_u64()).unwrap_or(0).min(255) as u8;
            let tol = args.get("tolerance").and_then(|v| v.as_u64()).unwrap_or(10).min(255) as u8;
            let region = if let Some(reg_obj) = args.get("region").and_then(|v| v.as_object()) {
                Rect {
                    x: reg_obj.get("x").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                    y: reg_obj.get("y").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                    width: reg_obj.get("width").and_then(|v| v.as_u64()).unwrap_or(400).max(1) as u32,
                    height: reg_obj.get("height").and_then(|v| v.as_u64()).unwrap_or(400).max(1) as u32,
                }
            } else {
                window::foreground_window().map(|w| w.rect).unwrap_or(Rect { x: 0, y: 0, width: 800, height: 600 })
            };
            let ratio = crate::pixel::count_color_ratio(region, (r, g, b), tol).map_err(|e| e.to_string())?;
            let total_pixels = region.width * region.height;
            let matching_pixels = (ratio * (total_pixels as f64)).round() as u64;
            Ok(json!({ "ratio": ratio, "matching_pixels": matching_pixels, "total_pixels": total_pixels, "region": region }))
        }

        "cu_clipboard_get" => match clipboard::get_text() {
            Ok(t) => Ok(json!({ "clipboard": t })),
            Err(e) => Err(e.to_string()),
        },

        "cu_clipboard_set" => {
            let text = args.get("text").and_then(|v| v.as_str()).unwrap_or("");
            match clipboard::set_text(text) {
                Ok(_) => Ok(json!({ "success": true, "bytes": text.len() })),
                Err(e) => Err(e.to_string()),
            }
        }

        "cu_launch_app" => {
            let app = args.get("app").and_then(|v| v.as_str()).unwrap_or("");
            let params = args.get("params").and_then(|v| v.as_str());
            let working_dir = args.get("working_dir").and_then(|v| v.as_str());
            launch::launch_with_dir(app, params, working_dir).map_err(|e| e.to_string())?;
            Ok(json!({ "success": true, "launched": app }))
        }

        "cu_list_processes" => {
            let filter = args.get("filter").and_then(|v| v.as_str()).map(|f| f.to_lowercase());
            let procs = process::list_processes().map_err(|e| e.to_string())?;
            let filtered: Vec<_> = if let Some(ref f) = filter {
                procs.into_iter().filter(|p| p.exe_name.to_lowercase().contains(f)).collect()
            } else {
                procs
            };
            Ok(json!({ "processes": filtered }))
        }

        "cu_kill_process" => {
            let pid = args.get("pid").and_then(|v| v.as_u64()).map(|p| p as u32);
            if let Some(p) = pid {
                process::kill_process(p).map_err(|e| e.to_string())?;
                Ok(json!({ "success": true, "terminated_pid": p }))
            } else {
                Err("Missing required 'pid' parameter".into())
            }
        }

        "cu_inspect_element_at" => {
            let x = args.get("x").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            let y = args.get("y").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            let el = uia::element_at_point(x, y).map_err(|e| e.to_string())?;
            Ok(json!({ "element": el }))
        }

        "cu_audio_beep" => {
            let freq = args.get("frequency").and_then(|v| v.as_u64()).unwrap_or(750) as u32;
            let duration = args.get("duration_ms").and_then(|v| v.as_u64()).unwrap_or(200) as u32;
            audio::beep(freq, duration).map_err(|e| e.to_string())?;
            Ok(json!({ "success": true, "beep": "dispatched", "frequency": freq, "duration_ms": duration }))
        }

        "cu_audio_volume" => {
            if let Some(set_val) = args.get("set_percent").and_then(|v| v.as_u64()) {
                let _ = audio::set_volume(set_val as u32);
            }
            let vol = audio::get_volume().map_err(|e| e.to_string())?;
            let mute = audio::get_mute().unwrap_or(false);
            Ok(json!({ "volume_percent": vol, "is_muted": mute }))
        }

        "cu_sysinfo" => match system::get_system_info() {
            Ok(info) => Ok(json!({ "sysinfo": info })),
            Err(e) => Err(e.to_string()),
        },

        "cu_resolve_association" => {
            let target = args.get("target").and_then(|v| v.as_str()).unwrap_or("");
            let exe = system::resolve_file_association(target).map_err(|e| e.to_string())?;
            Ok(json!({ "success": true, "target": target, "executable": exe }))
        }

        "cu_system_sound" => {
            let sound = args.get("sound").and_then(|v| v.as_str()).unwrap_or("default");
            system::play_system_sound(sound).map_err(|e| e.to_string())?;
            Ok(json!({ "success": true, "played": sound }))
        }

        "cu_special_folder" => {
            let name = args.get("name").and_then(|v| v.as_str()).unwrap_or("desktop");
            let path = system::get_special_folder(name).map_err(|e| e.to_string())?;
            Ok(json!({ "success": true, "name": name, "path": path }))
        }

        "cu_sync_schemas" => {
            let count = sync_all_mcp_schemas().map_err(|e| e.to_string())?;
            Ok(json!({ "success": true, "schemas_synced": count }))
        }

        unknown => Err(format!("Unknown tool: {}", unknown)),
        }
    })();

    match result {
        Ok(data) => JsonRpcResponse {
            jsonrpc: "2.0",
            id,
            result: Some(json!({
                "content": [{
                    "type": "text",
                    "text": serde_json::to_string_pretty(&data).unwrap()
                }],
                "isError": false
            })),
            error: None,
        },
        Err(err_msg) => JsonRpcResponse {
            jsonrpc: "2.0",
            id,
            result: Some(json!({
                "content": [{
                    "type": "text",
                    "text": format!("Error: {}", err_msg)
                }],
                "isError": true
            })),
            error: None,
        },
    }
}
