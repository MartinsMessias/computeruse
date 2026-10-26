use std::io::{self, BufRead, Write};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use crate::agent::AgentEnvironment;
use crate::clipboard;
use crate::error::Result;
use crate::screen;
use crate::system;
use crate::types::MouseButton;
use crate::window;
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

pub fn run_mcp_server() -> Result<()> {
    eprintln!("[computeruse-mcp] Starting MCP Server over stdio (PID: {})...", std::process::id());
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
                    "listChanged": false
                }
            },
            "serverInfo": {
                "name": "computeruse",
                "version": "0.1.0"
            }
        })),
        error: None,
    }
}

fn handle_tools_list(id: Value) -> JsonRpcResponse {
    let tools = vec![
        json!({
            "name": "cu_observe",
            "description": "Inspect the active foreground window and extract all interactive UI elements using Set-of-Marks with numeric IDs for zero-waste navigation.",
            "inputSchema": {
                "type": "object",
                "properties": {}
            }
        }),
        json!({
            "name": "cu_click_mark",
            "description": "Click on an interactive UI element by its Set-of-Marks mark_id.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "mark_id": { "type": "integer", "description": "The mark_id number from cu_observe" }
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
            "description": "Click at normalized (0.0..1.0) coordinates relative to the active window.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "x": { "type": "number", "description": "Normalized X (0.0=left, 1.0=right)" },
                    "y": { "type": "number", "description": "Normalized Y (0.0=top, 1.0=bottom)" },
                    "button": { "type": "string", "enum": ["left", "right", "middle"], "default": "left" }
                },
                "required": ["x", "y"]
            }
        }),
        json!({
            "name": "cu_type",
            "description": "Type text into the currently focused window or control.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "text": { "type": "string", "description": "Text string to type" }
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
            "description": "Dispatch a hotkey combination (e.g. 'ctrl+c', 'ctrl+v', 'ctrl+t', 'ctrl+w', 'alt+tab').",
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
            "description": "Scroll the mouse wheel. Negative values scroll down, positive scroll up.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "amount": { "type": "integer", "description": "Scroll delta (e.g. -350 for down, 350 for up)" },
                    "horizontal": { "type": "boolean", "default": false }
                },
                "required": ["amount"]
            }
        }),
        json!({
            "name": "cu_drag_rel",
            "description": "Drag the mouse across normalized window coordinates.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "from_x": { "type": "number" },
                    "from_y": { "type": "number" },
                    "to_x": { "type": "number" },
                    "to_y": { "type": "number" }
                },
                "required": ["from_x", "from_y", "to_x", "to_y"]
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
            "name": "cu_screenshot",
            "description": "Capture a screenshot of the active window or specific HWND. Returns base64 PNG.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "hwnd": { "type": "integer", "description": "Optional HWND. If omitted, captures active window." }
                }
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
            "name": "cu_sysinfo",
            "description": "Get system RAM, CPU cores, uptime, battery status, and drives.",
            "inputSchema": {
                "type": "object",
                "properties": {}
            }
        }),
    ];

    JsonRpcResponse {
        jsonrpc: "2.0",
        id,
        result: Some(json!({ "tools": tools })),
        error: None,
    }
}

fn handle_tools_call(id: Value, params: Value, env: &mut AgentEnvironment) -> JsonRpcResponse {
    let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let args = params.get("arguments").cloned().unwrap_or(json!({}));

    let result = match name {
        "cu_observe" => match env.observe() {
            Ok(obs) => Ok(json!({
                "window": {
                    "title": obs.window_title,
                    "hwnd": obs.window_hwnd,
                    "bounds": obs.window_rect,
                    "is_maximized": obs.is_maximized
                },
                "cursor": obs.cursor_pos,
                "clipboard_preview": obs.clipboard_preview,
                "interactive_targets_count": obs.marks.len(),
                "prompt_summary": obs.compact_prompt,
                "marks": obs.marks
            })),
            Err(e) => Err(e.to_string()),
        },

        "cu_click_mark" => {
            let mark_id = args.get("mark_id").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
            match env.step(crate::AgentAction::ClickMark { mark_id }) {
                Ok(res) => Ok(json!({ "success": res.success, "message": res.message, "active_window": res.observation.window_title })),
                Err(e) => Err(e.to_string()),
            }
        }

        "cu_click_name" => {
            let el_name = args.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
            match env.step(crate::AgentAction::ClickName { name: el_name }) {
                Ok(res) => Ok(json!({ "success": res.success, "message": res.message, "active_window": res.observation.window_title })),
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
            match env.step(crate::AgentAction::ClickRel { x, y, button }) {
                Ok(res) => Ok(json!({ "success": res.success, "message": res.message, "active_window": res.observation.window_title })),
                Err(e) => Err(e.to_string()),
            }
        }

        "cu_type" => {
            let text = args.get("text").and_then(|v| v.as_str()).unwrap_or("").to_string();
            match env.step(crate::AgentAction::TypeText { text }) {
                Ok(res) => Ok(json!({ "success": res.success, "message": res.message })),
                Err(e) => Err(e.to_string()),
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
            match env.step(crate::AgentAction::Scroll { amount, horizontal }) {
                Ok(res) => Ok(json!({ "success": res.success, "message": res.message })),
                Err(e) => Err(e.to_string()),
            }
        }

        "cu_drag_rel" => {
            let from_x = args.get("from_x").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let from_y = args.get("from_y").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let to_x = args.get("to_x").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let to_y = args.get("to_y").and_then(|v| v.as_f64()).unwrap_or(0.0);
            match env.step(crate::AgentAction::DragRel { from_x, from_y, to_x, to_y }) {
                Ok(res) => Ok(json!({ "success": res.success, "message": res.message })),
                Err(e) => Err(e.to_string()),
            }
        }

        "cu_focus_window" => {
            let title = args.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string();
            match env.step(crate::AgentAction::FocusWindow { title }) {
                Ok(res) => Ok(json!({ "success": res.success, "message": res.message, "active_window": res.observation.window_title })),
                Err(e) => Err(e.to_string()),
            }
        }

        "cu_list_windows" => match window::list_windows(false) {
            Ok(wins) => Ok(json!({ "windows": wins })),
            Err(e) => Err(e.to_string()),
        },

        "cu_screenshot" => {
            let hwnd = args.get("hwnd").and_then(|v| v.as_i64()).map(|h| h as isize);
            let shot = if let Some(h) = hwnd {
                screen::screenshot_window(h)
            } else {
                screen::screenshot_active_window()
            };
            match shot {
                Ok(s) => Ok(json!({
                    "width": s.width,
                    "height": s.height,
                    "png_base64": s.png_base64
                })),
                Err(e) => Err(e.to_string()),
            }
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

        "cu_sysinfo" => match system::get_system_info() {
            Ok(info) => Ok(json!({ "sysinfo": info })),
            Err(e) => Err(e.to_string()),
        },

        unknown => Err(format!("Unknown tool: {}", unknown)),
    };

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
