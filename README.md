# computeruse — Windows Computer-Use CLI, Library & MCP Server in Rust

[![CI](https://github.com/MartinsMessias/computeruse/actions/workflows/ci.yml/badge.svg)](https://github.com/MartinsMessias/computeruse/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![MCP 2024-11-05](https://img.shields.io/badge/MCP-2024--11--05%20(54%20Tools)-green.svg)](#how-to-use-the-mcp-server-cu-mcp)
[![Platform: Windows](https://img.shields.io/badge/Platform-Windows%2010%2F11-0078D6.svg)](https://github.com/MartinsMessias/computeruse)
[![LLM Context](https://img.shields.io/badge/LLMs-llms.txt%20%7C%20llms--full.txt-purple.svg)](llms.txt)

**`computeruse`** is a blazing-fast Windows desktop automation library, CLI (`cu`), and native **Model Context Protocol (MCP) Server** (`cu mcp`) written in pure Rust.

It gives AI agents (Claude Desktop, Claude Code, Cursor, Windsurf, Google Antigravity, OpenCode) direct control over the Windows desktop using sub-millisecond **UI Automation Set-of-Marks (`cu_observe`)**, **zero-roundtrip atomic action pipelines (`cu_batch`, `cu_fill_mark`)**, **targeted visual crops (`cu_screenshot`)**, and **reactive UI synchronization (`cu_wait_for_change`)** — without slow 4K screenshot-guessing loops.

> **Documentation & LLM Index:**
> - [Architecture & The 4 Pillars (`docs/ARCHITECTURE.md`)](docs/ARCHITECTURE.md)
> - [Concise LLM Context (`llms.txt`)](llms.txt) | [Full 54-Tool Technical Reference (`llms-full.txt`)](llms-full.txt)
> - [Contributor & Agent Playbook (`AGENTS.md`)](AGENTS.md)

---

## Installation & Build

```sh
git clone https://github.com/MartinsMessias/computeruse.git
cd computeruse

# Build optimized release binary (target/release/cu.exe)
cargo build --release

# Or install globally into ~/.cargo/bin/cu.exe
cargo install --path .
```

---

## How to Use the MCP Server (`cu mcp`)

The `cu` binary includes a native **MCP `2024-11-05` stdio server** exposing **54 tools**, **4 live desktop resources**, **prompts**, and **native Multi-Content image blocks**.

### 1. Connect Your MCP Client

#### Claude Desktop (`%APPDATA%\Claude\claude_desktop_config.json`)
```json
{
  "mcpServers": {
    "computeruse": {
      "command": "C:\\Users\\YOUR_USER\\.cargo\\bin\\cu.exe",
      "args": ["mcp"]
    }
  }
}
```

#### Cursor / Windsurf (`~/.cursor/mcp.json` or `.cursor/mcp.json`)
```json
{
  "mcpServers": {
    "computeruse": {
      "command": "C:\\Users\\YOUR_USER\\.cargo\\bin\\cu.exe",
      "args": ["mcp"]
    }
  }
}
```

#### Claude Code CLI
```sh
claude mcp add computeruse -- C:\Users\YOUR_USER\.cargo\bin\cu.exe mcp
```

#### Google Antigravity CLI
```sh
# Sync all 54 tool JSON schemas + instructions.md to ~/.gemini/antigravity-cli/mcp/computeruse
cu mcp --sync
```

---

### 2. Practical MCP Workflows (How Agents Use `computeruse`)

#### A. Observe the Active Window Without Screenshots (`cu_observe`)
Instead of taking a full screenshot, call `cu_observe` first. It scans the Windows UI Automation tree in milliseconds and returns numbered `[ID]` marks:

```json
// Tool Call: cu_observe
{}
```
**Response (`prompt_summary`):**
```text
Window: "Untitled - Paint" (HWND: 132948, 1920x1080)
Tools/Buttons (8): [01] "File" (28,48) | [02] "Pencil" (246,98) | [03] "Fill" (282,98) | [04] "Text" (318,98) | [05] "Brushes" (410,98)
Palette/Items (6): [06] "Black" (760,88) | [07] "Red" (804,88) | [08] "Blue" (870,88)
Inputs/Controls (2): [09] "Size"="4px" (640,98) | [10] "Zoom"="100%" (1820,1040)
```

#### B. Click & Fill Controls by `[ID]` (`cu_click_mark` & `cu_fill_mark`)
Use the numeric `mark_id` from `cu_observe` to interact with exact UI elements:

```json
// Tool Call: cu_fill_mark (clicks mark [09], clears existing text, types "12", and presses Enter)
{
  "mark_id": 9,
  "text": "12",
  "clear": true,
  "submit": true
}
```

#### C. Multi-Step Atomic Pipelines (`cu_batch`)
Execute an entire sequence of actions in a **single tool call** and get the updated `prompt_summary` and `ui_delta` at the end:

```json
// Tool Call: cu_batch
{
  "actions": [
    { "action": "focus_window", "title": "Notepad" },
    { "action": "hotkey", "combo": "ctrl+n" },
    { "action": "wait_for_change", "timeout_ms": 1500 },
    { "action": "type", "text": "Hello from computeruse MCP!", "paste": true },
    { "action": "hotkey", "combo": "ctrl+s" }
  ]
}
```

#### D. Targeted Visual Crops & Set-of-Marks Badges (`cu_screenshot`)
When visual inspection is needed, crop directly to a specific element (`mark_id`), only the drawing canvas (`canvas_only: true`), or overlay numbered `[ID]` badges (`annotate_marks: true`):

```json
// Tool Call: cu_screenshot (crops only the drawing canvas or overlays [ID] badges)
{
  "canvas_only": true,
  "annotate_marks": false,
  "save_path": "C:\\temp\\canvas.png"
}
```

#### E. Parametric Vector Canvas Drawing & File Saving (`cu_canvas_draw` & `cu_save_file_as`)
Draw complex multi-layered shapes on MS Paint or any canvas in one call, and save via the native `#32770` Save As dialog macro:

```json
// Tool Call: cu_canvas_draw
{
  "shapes": [
    { "type": "circle", "cx": 0.5, "cy": 0.5, "r": 0.35, "segments": 36 },
    { "type": "star", "cx": 0.5, "cy": 0.5, "r_outer": 0.25, "r_inner": 0.10, "points": 5 }
  ]
}
```
```json
// Tool Call: cu_save_file_as
{
  "path": "C:\\Users\\Public\\Pictures\\star_badge.png",
  "timeout_ms": 3000
}
```

---

### 3. Available MCP Tools (54 Tools)

| Category | Count | Tools |
|---|---|---|
| **Perception & Visual** | 7 | `cu_observe`, `cu_find_element`, `cu_read_window_text`, `cu_screenshot`, `cu_find_color`, `cu_count_color_ratio`, `cu_inspect_element_at` |
| **Semantic & Mouse Input** | 17 | `cu_click_mark`, `cu_right_click_mark`, `cu_hover_mark`, `cu_fill_mark`, `cu_drag_mark_to_mark`, `cu_click_name`, `cu_click_rel`, `cu_click_coords`, `cu_double_click`, `cu_triple_click`, `cu_long_click`, `cu_move`, `cu_drag`, `cu_drag_rel`, `cu_mouse_down`, `cu_mouse_up`, `cu_scroll` |
| **Keyboard & Clipboard** | 7 | `cu_type`, `cu_key`, `cu_hotkey`, `cu_copy_selection`, `cu_clipboard_get`, `cu_clipboard_set`, `cu_clipboard_set_image` |
| **Atomic Batch & Reactive Sync** | 3 | `cu_batch`, `cu_wait_for_change`, `cu_wait_for_text` |
| **Vector Canvas Engine** | 2 | `cu_canvas_draw`, `cu_canvas_clear` |
| **Window & Modal Management** | 8 | `cu_focus_window`, `cu_list_windows`, `cu_close_window`, `cu_maximize_window`, `cu_minimize_window`, `cu_snap_window`, `cu_save_file_as`, `cu_check_modal` |
| **System, Process & Audio** | 10 | `cu_launch_app`, `cu_list_processes`, `cu_kill_process`, `cu_resolve_association`, `cu_system_sound`, `cu_special_folder`, `cu_audio_beep`, `cu_audio_volume`, `cu_sysinfo`, `cu_sync_schemas` |

### 4. MCP Resources & Prompts

- **Live Desktop Resources (`resources/read`)**:
  - `desktop://windows` — Visible top-level windows and coordinates (`application/json`).
  - `desktop://sysinfo` — CPU cores, RAM load, uptime, and battery status (`application/json`).
  - `desktop://monitors` — Monitors, DPI scale factors, and taskbar-aware `work_area` bounds (`application/json`).
  - `desktop://clipboard` — Current text in the Windows clipboard (`text/plain`).
- **Operational Prompt (`prompts/get`)**:
  - `desktop_autonomous_operator` — System prompt instructing agents to combine `cu_observe`, `cu_batch`, and targeted crops for low-token desktop operation.

---

## CLI Usage (`cu`)

Every subcommand prints a structured JSON object to `stdout`.

```sh
# Agent Perception, Step, REPL & MCP Server
cu observe [--compact] [--hwnd N] [--title "..."]
cu step '{"action":"click_mark","mark_id":3}'
cu repl
cu mcp [--sync]

# Display, Screenshots & Pixel Telemetry
cu screen-size
cu monitors
cu screenshot [--monitor N] [--region x,y,w,h] [--canvas] [--hwnd N] [--save path.png]
cu screenshot-active
cu screenshot-window <HWND>
cu screenshot-all
cu pixel X Y
cu find-color R G B [--tolerance 10]

# Mouse
cu cursor
cu move X Y
cu move-rel DX DY
cu click [left|right|middle] [--x X --y Y]
cu dbl-click [button] [--x X --y Y]
cu mouse-down [button]
cu mouse-up [button]
cu drag X1 Y1 X2 Y2 [button]
cu scroll AMOUNT [--horizontal]

# Keyboard & Clipboard
cu type "text" [--delay MS]   # or: cu type --stdin
cu key NAME
cu key-down NAME
cu key-up NAME
cu hotkey "ctrl+c"
cu clipboard-get
cu clipboard-set "text"       # or: cu clipboard-set --stdin

# Windows & File Dialogs
cu windows [--all]
cu find-window "title substring"
cu foreground
cu focus --title "..." | --hwnd N
cu snap-window <left|right|top|bottom|top_left|top_right|bottom_left|bottom_right|center|maximize> [--hwnd N] [--title "..."]
cu save-as "C:\path\to\file.png" [--timeout 3000]
cu minimize HWND
cu maximize HWND
cu restore HWND
cu close HWND
cu move-window HWND X Y W H
cu window-topmost HWND [--disable]
cu window-opacity HWND ALPHA (0-255)
cu wait-for-window "title" [--timeout MS]

# UI Automation (UIA)
cu uia-at X Y
cu uia-focused
cu uia-find "Button Name"
cu uia-click "Button Name"
cu uia-children [--hwnd N]
cu read-window-text [--hwnd N]

# Audio, System & Processes
cu volume [--set 75]
cu mute [--set true] [--toggle]
cu sysinfo
cu drives
cu lock
cu special-folder desktop     # desktop, downloads, documents, appdata, localappdata, temp, startmenu, startup
cu resolve-assoc .pdf
cu system-sound [default|error|warning|question|asterisk]
cu processes
cu find-process "name"
cu kill-process PID
cu kill-by-name "name.exe"
cu launch "app.exe" [--params "..."]

# Batch Execution (JSON Array or Compact DSL)
cu exec '[{"action":"click","x":100,"y":200}, {"action":"type","text":"hello"}]'
cu exec "C 500,300; W 100; T 'hello world'; K enter; H ctrl+s; D 100,100,400,400"
```

### Compact Action DSL (`cu exec`)

| Command | Syntax | Description |
|---|---|---|
| `C` / `CLICK` | `C x,y` or `C` | Left-click at `(x, y)` or current cursor position |
| `DC` / `DBLCLICK` | `DC x,y` or `DC` | Double-click at `(x, y)` or current cursor position |
| `RC` / `RCLICK` | `RC x,y` or `RC` | Right-click at `(x, y)` or current cursor position |
| `M` / `MOVE` | `M x,y` | Move cursor to `(x, y)` |
| `D` / `DRAG` | `D x1,y1,x2,y2` | Drag from `(x1, y1)` to `(x2, y2)` |
| `S` / `SCROLL` | `S -5` | Scroll mouse wheel (`-N` down, `+N` up) |
| `T` / `TYPE` | `T 'text'` | Type text string |
| `P` / `PASTE` | `P 'text'` | Paste text instantaneously via non-destructive clipboard |
| `K` / `KEY` | `K enter` | Press a named key |
| `H` / `HOTKEY` | `H ctrl+s` | Dispatch a hotkey combination |
| `F` / `FOCUS` | `F 'Notepad'` | Focus a window by title substring |
| `W` / `WAIT` | `W 150` | Sleep for `N` milliseconds |

---

## Rust Library Usage

```rust
use computeruse::*;

fn main() -> Result<()> {
    // 1. Agent Perception & Set-of-Marks
    let mut env = ComputerUse::agent_env();
    let obs = env.observe()?;
    println!("Active Window: {}\n{}", obs.window_title, obs.compact_prompt);

    // 2. Atomic Action Execution
    let step = env.step(AgentAction::FillMark {
        mark_id: 1,
        text: "Hello from computeruse".into(),
        clear: Some(true),
        submit: Some(true),
    })?;
    println!("Delta: {}", step.delta.summary);

    // 3. Direct OS Primitives
    let (w, h) = ComputerUse::screen_size()?;
    ComputerUse::click(MouseButton::Left, Some(Point { x: w as i32 / 2, y: h as i32 / 2 }))?;
    ComputerUse::paste_text("Instant clipboard paste")?;
    ComputerUse::hotkey_str("ctrl+s")?;

    Ok(())
}
```

---

## Supported Keyboard Keys

- **Alphanumeric & Function**: `a`–`z`, `0`–`9`, `f1`–`f24`
- **Navigation & Editing**: `enter`, `tab`, `esc`, `space`, `backspace`, `delete`, `insert`, `home`, `end`, `pageup`, `pagedown`, `up`, `down`, `left`, `right`
- **Modifiers & Lock Keys**: `ctrl`, `lctrl`, `rctrl`, `alt`, `lalt`, `ralt`, `shift`, `lshift`, `rshift`, `win`, `lwin`, `rwin`, `capslock`, `numlock`, `scrolllock`, `apps` / `contextmenu`
- **OEM Punctuation**: `+` (`plus`), `-` (`minus`), `=`, `/`, `.`, `,`, `;`, `'`, `[`, `]`, `\`, `` ` ``
- **Browser Navigation**: `browser_back`, `browser_forward`, `browser_refresh`, `browser_stop`, `browser_search`, `browser_favorites`, `browser_home`
- **Media & Volume**: `volumeup`, `volumedown`, `volumemute`, `mediaplaypause`, `medianext`, `mediaprev`, `mediastop`

---

## License

[MIT](LICENSE)
