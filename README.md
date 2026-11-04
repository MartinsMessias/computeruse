# computeruse — OS-Native Symbiosis & MCP Server for Autonomous Agents

[![CI](https://github.com/MartinsMessias/computeruse/actions/workflows/ci.yml/badge.svg)](https://github.com/MartinsMessias/computeruse/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![MCP 2024-11-05](https://img.shields.io/badge/MCP-2024--11--05%20(54%20Tools)-green.svg)](#mcp-server-cu-mcp)
[![Platform: Windows](https://img.shields.io/badge/Platform-Windows%2010%2F11-0078D6.svg)](https://github.com/MartinsMessias/computeruse)
[![LLM Context](https://img.shields.io/badge/LLMs-llms.txt%20%7C%20llms--full.txt-purple.svg)](llms.txt)

A high-performance Rust library, CLI (`cu`), and **Model Context Protocol (MCP) Server** (`cu mcp`) that exposes Windows desktop automation primitives engineered around the **4 Pillars of OS-Native Symbiosis**.

Instead of relying on slow, lossy 4K screenshot-to-pixel guessing loops, `computeruse` gives LLM agents direct structural perception, atomic macro execution, hardware-exact input injection, and event-driven UI synchronization.

> **For AI Agents & LLM Indexers:** Read [`llms.txt`](llms.txt) for a concise overview, [`llms-full.txt`](llms-full.txt) for the complete 54-tool & API reference, and [`AGENTS.md`](AGENTS.md) for the engineering playbook.

---

## Architecture: The 4 Pillars of OS-Native Symbiosis

### 1. Pillar I — Hybrid Sub-Token Perception & Targeted Visual Crop
- **Categorized Set-of-Marks (`cu_observe`)**: Traverses the Windows UI Automation (UIA) tree in milliseconds, enforces 4-edge viewport bounds filtering, extracts live values from inputs/sliders (`ValueValue` / `LegacyIAccessibleValue`), and compresses hundreds of elements into a `< 1.8 KB` categorized semantic digest (`Tools/Buttons`, `Palette/Items`, `Inputs/Controls`, `Menus/Other`). Pass `verbose: true` only when raw bounding boxes are needed.
- **4-Tier Semantic & Fuzzy Element Search (`cu_find_element`)**: Locates interactive controls by substring name, `AutomationId`, host-window token matching, or Set-of-Marks fuzzy fallback without scanning the full tree manually.
- **Direct Window Text Extraction (`cu_read_window_text`)**: Extracts all visible text strings, labels, and control values from any window in milliseconds without OCR or screenshots.
- **Targeted Visual Crops & Annotated Badges (`cu_screenshot`)**: Crop directly to a specific UI element (`mark_id`), only the drawing canvas (`canvas_only: true`), a target window (`hwnd`), or a custom rectangle (`region: {x, y, width, height}`), overlay numbered Set-of-Marks `[ID]` badges (`annotate_marks: true`), and save directly to disk (`save_path`, `omit_base64: true`).
- **Fast Color Scanning & Pixel Ratio Telemetry (`cu_find_color` & `cu_count_color_ratio`)**: Single-GDI-capture RGB tolerance scanning and area coverage ratio calculation for instant visual verification.

### 2. Pillar II — Atomic Transactions, Compact DSL & Vector Macro-JIT
- **Zero-Roundtrip Action Pipelines (`cu_batch`, `cu_fill_mark`, `cu_save_file_as`)**:
  - `cu_batch`: Execute multi-step workflows (`click_mark`, `right_click_mark`, `hover_mark`, `fill_mark`, `drag_mark_to_mark`, `click_name`, `type`, `paste`, `key`, `hotkey`, `scroll`, `focus_window`, `wait`, `wait_for_change`, `wait_for_text`) in a single atomic call and receive the post-transaction `prompt_summary` and `ui_delta` immediately.
  - `cu_fill_mark`: Clicks an input field by `mark_id`, optionally clears existing text (`Ctrl+A -> Backspace`), types/pastes text, and optionally submits (`Enter`) in one call.
  - `cu_save_file_as`: Atomic macro that dispatches `Ctrl+S`, waits for the native Windows `#32770` Save As dialog, injects the full target path, and confirms overwrite modals automatically.
- **Compact Action DSL (`cu exec`)**: Execute ultra-low-token semicolon-separated command strings directly from the CLI (e.g. `cu exec "C 500,300; W 100; T 'hello'; K enter; H ctrl+s; D 100,100,400,400"`).
- **Parametric Vector & Pointillist Canvas Engine (`cu_canvas_draw` & `cu_canvas_clear`)**: Automatically detects the exact drawing surface (`DynamicCanvas`) and renders multi-layered geometric primitives (`gear`, `star`, `polygon`, `circle`, `spiral`, `line`, `rect`, `bezier`, `text`, `dots`/`points`) with aspect-ratio compensation, WinUI 3 Ribbon focus-recovery, and automatic closed-shape first-edge sealing.
- **Human-Parity Input & Clipboard Injection (`cu_type`, `cu_copy_selection`, `cu_clipboard_set_image`)**: Full hardware scancode (`MapVirtualKeyW`) + `KEYEVENTF_EXTENDEDKEY` injection, UTF-16 surrogate pair support, modifier-held clicks/drags/scrolls (`ctrl`, `shift`, `alt`), smooth human ease-in-out cursor trajectories (`smooth_ms`), and direct screen-region-to-`CF_BITMAP` clipboard copying.

### 3. Pillar III — Per-Monitor DPI v2, UWP/WinUI 3 Focus & Virtual Desktop Fidelity
- **Per-Monitor DPI v2**: Automatically activates `DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2` (`enable_per_monitor_dpi_v2`) and attaches threads to the active input desktop (`OpenInputDesktop` / `SetThreadDesktop`).
- **Virtual Desktop Hardware Injection & Work-Area Snapping**: Normalizes `SendInput` coordinates across the full virtual desktop (`SM_XVIRTUALSCREEN`, `SM_YVIRTUALSCREEN`, `SM_CXVIRTUALSCREEN`, `SM_CYVIRTUALSCREEN`) with `MOUSEEVENTF_VIRTUALDESK | MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_MOVE`, and snaps windows (`cu_snap_window`) respecting taskbar-aware monitor `work_area` bounds.
- **UWP / WinUI 3 `ApplicationFrameWindow` Resolution & UIPI Focus Bypass (`cu_focus_window`)**: Automatically drills down into packaged `ApplicationFrameWindow` containers to resolve the real visible child `HWND`, synchronizes input queues via `AttachThreadInput`, and performs legal `AllowSetForegroundWindow` + `VK_MENU` UIPI elevation.
- **O(1) Win32 Window Inspection & Modal Detection (`cu_check_modal`)**: `find_window_by_hwnd` queries `IsWindow` + `GetWindowRect` in O(1) microseconds, while `cu_check_modal` detects whether an application is blocked by a modal `#32770` dialog.

### 4. Pillar IV — Event-Driven Reactive Synchronization & State Delta
- **Reactive UI & Visual Settling (`cu_wait_for_change` & `cu_wait_for_text`)**: Polls the foreground HWND, window title, UIA element structure, 64-bit FNV-1a visual pixel checksum (`region_visual_checksum`), or a custom screen `region` in `25ms–50ms` ticks, returning the instant a menu, popup, canvas stroke, or target text substring (`cu_wait_for_text`) appears instead of sleeping blindly.
- **Resilient UIA Invocation & Structural Delta (`UiDelta`)**: Automatically falls back to physical coordinate click injection when programmatic `InvokePattern` fails or returns spurious success HRESULTs, and reports a `ui_delta` summary after every step (e.g., `"UI tree updated (+12 interactive elements, total 64)"`).

---

## MCP Server (`cu mcp`)

Run as a native stdio MCP server or synchronize schemas directly to Antigravity's MCP configuration directory:

```sh
# Start the stdio JSON-RPC 2.0 MCP server (auto-syncs schemas on startup)
cu mcp

# Force-sync all 54 tool schemas + instructions.md to ~/.gemini/antigravity-cli/mcp/computeruse
cu mcp --sync
```

### Protocol Capabilities (MCP `2024-11-05` Spec)
- **Tools (54 Tools)**: Full support for atomic batch execution, Set-of-Marks perception, vector canvas drawing, and OS-level automation.
- **Resources (4 Live Desktop URI Endpoints)**: Read desktop state directly without tool roundtrips:
  - `desktop://windows` — Live list of visible top-level windows and coordinates (`application/json`).
  - `desktop://sysinfo` — CPU cores, RAM load, system uptime, and battery telemetry (`application/json`).
  - `desktop://monitors` — Connected monitors, DPI scale factors, and taskbar-aware `work_area` geometries (`application/json`).
  - `desktop://clipboard` — Current text content stored in the Windows clipboard (`text/plain`).
- **Prompts**: Built-in `desktop_autonomous_operator` operational system prompt template.
- **Multi-Content Image Rendering**: `cu_screenshot` emits native MCP `{"type": "image", "data": "...", "mimeType": "image/png"}` blocks alongside structured metadata JSON.

### Available MCP Tools (54 Tools)

| Category | Tools (Count) | Tool Names |
|---|---|---|
| **Perception & Visual** | 7 | `cu_observe`, `cu_find_element`, `cu_read_window_text`, `cu_screenshot`, `cu_find_color`, `cu_count_color_ratio`, `cu_inspect_element_at` |
| **Semantic & Mouse Input** | 17 | `cu_click_mark`, `cu_right_click_mark`, `cu_hover_mark`, `cu_fill_mark`, `cu_drag_mark_to_mark`, `cu_click_name`, `cu_click_rel`, `cu_click_coords`, `cu_double_click`, `cu_triple_click`, `cu_long_click`, `cu_move`, `cu_drag`, `cu_drag_rel`, `cu_mouse_down`, `cu_mouse_up`, `cu_scroll` |
| **Keyboard & Clipboard** | 7 | `cu_type`, `cu_key`, `cu_hotkey`, `cu_copy_selection`, `cu_clipboard_get`, `cu_clipboard_set`, `cu_clipboard_set_image` |
| **Atomic Batch & Reactive Sync** | 3 | `cu_batch`, `cu_wait_for_change`, `cu_wait_for_text` |
| **Vector Canvas Engine** | 2 | `cu_canvas_draw`, `cu_canvas_clear` |
| **Window & Modal Management** | 8 | `cu_focus_window`, `cu_list_windows`, `cu_close_window`, `cu_maximize_window`, `cu_minimize_window`, `cu_snap_window`, `cu_save_file_as`, `cu_check_modal` |
| **System, Process & Audio** | 10 | `cu_launch_app`, `cu_list_processes`, `cu_kill_process`, `cu_resolve_association`, `cu_system_sound`, `cu_special_folder`, `cu_audio_beep`, `cu_audio_volume`, `cu_sysinfo`, `cu_sync_schemas` |

---

## Build & Test

```sh
cd computeruse
cargo test --test integration
cargo build --release
# binary: target/release/cu.exe
```

---

## CLI Usage (`cu`)

Every subcommand prints a structured JSON object to `stdout`.

```sh
# Agent Perception, Step, REPL & MCP Server
cu observe
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

# Audio
cu volume                     # get volume 0-100
cu volume --set 75            # set volume to 75%
cu mute                       # get mute status
cu mute --set true            # mute
cu mute --toggle              # toggle mute

# System, Special Folders & File Associations
cu sysinfo                    # RAM, CPU cores, uptime, battery
cu drives                     # disk drives, free and total bytes
cu lock                       # lock Windows workstation
cu special-folder desktop     # desktop, downloads, documents, appdata, localappdata, temp, startmenu, startup
cu resolve-assoc .pdf         # resolve default executable for file extension or URI protocol
cu system-sound [default|error|warning|question|asterisk]

# Processes & Launch
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

In addition to JSON arrays, `cu exec` supports a compact semicolon- or newline-delimited DSL for minimal token overhead:

| Command | Syntax | Description |
|---|---|---|
| `C` / `CLICK` | `C x,y` or `C` | Left-click at `(x, y)` or current cursor position |
| `M` / `MOVE` | `M x,y` | Move cursor to `(x, y)` |
| `D` / `DRAG` | `D x1,y1,x2,y2` | Drag from `(x1, y1)` to `(x2, y2)` |
| `T` / `TYPE` | `T 'text'` | Type text string |
| `K` / `KEY` | `K enter` | Press a named key |
| `H` / `HOTKEY` | `H ctrl+s` | Dispatch a hotkey combination |
| `W` / `WAIT` | `W 150` | Sleep for `N` milliseconds |

---

## Rust Library Usage

```rust
use computeruse::*;

fn main() -> Result<()> {
    // 1. Agent Perception & Set-of-Marks (Pillar I)
    let mut env = ComputerUse::agent_env();
    let obs = env.observe()?;
    println!("Active Window: {}\n{}", obs.window_title, obs.compact_prompt);

    // 2. Atomic Action Execution & Reactive Sync (Pillar II & IV)
    let step = env.step(AgentAction::FillMark {
        mark_id: 1,
        text: "Hello from computeruse".into(),
        clear: Some(true),
        submit: Some(true),
    })?;
    println!("Delta: {}", step.delta.summary);

    // 3. Direct OS Primitives (Screen, Mouse, Keyboard, Window, System)
    let (w, h) = ComputerUse::screen_size()?;
    let shot = ComputerUse::screenshot_full(None)?; // base64 PNG
    ComputerUse::click(MouseButton::Left, Some(Point { x: w as i32 / 2, y: h as i32 / 2 }))?;
    ComputerUse::paste_text("Instant clipboard paste")?;
    ComputerUse::hotkey_str("ctrl+s")?;

    // Check for blocking modal dialogs (#32770) or wait for visual change
    if let Some(modal) = ComputerUse::get_blocking_modal(0)? {
        println!("Blocked by modal: {} (HWND {})", modal.title, modal.hwnd);
    }
    let changed = ComputerUse::wait_for_screen_change(None, 2000, 50)?;
    println!("Screen changed: {}", changed);

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

MIT
