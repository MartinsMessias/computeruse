# computeruse — OS-Native Symbiosis & MCP Server for Autonomous Agents

A high-performance Rust library, CLI (`cu`), and **Model Context Protocol (MCP) Server** (`cu mcp`) that exposes Windows desktop automation primitives engineered around the **4 Pillars of OS-Native Symbiosis**.

Instead of relying on slow, lossy 4K screenshot-to-pixel guessing loops, `computeruse` gives LLM agents direct structural perception, atomic macro execution, hardware-exact input injection, and event-driven UI synchronization.

---

## Architecture: The 4 Pillars of OS-Native Symbiosis

### 1. Pillar I — Hybrid Sub-Token Perception & Targeted Visual Crop
- **Categorized Set-of-Marks (`cu_observe`)**: Traverses the Windows UI Automation (UIA) tree in milliseconds, filters out off-screen artifacts, extracts live values from inputs/sliders (`ValueValue` / `LegacyIAccessibleValue`), and compresses hundreds of elements into a `< 1.8 KB` categorized semantic digest (`Tools/Buttons`, `Palette/Items`, `Inputs/Controls`, `Menus/Other`). Pass `verbose: true` only when raw bounding boxes are needed.
- **Direct Window Text Extraction (`cu_read_window_text`)**: Extracts all visible text strings, labels, and control values from any window in milliseconds without OCR or screenshots.
- **Targeted Visual Crops & Annotated Badges (`cu_screenshot`)**: Crop directly to a specific UI element (`mark_id`), only the drawing canvas (`canvas_only: true`), or a custom rectangle (`region: {x, y, width, height}`), overlay numbered Set-of-Marks `[ID]` badges (`annotate_marks: true`), and save directly to disk (`save_path`, `omit_base64: true`).

### 2. Pillar II — Atomic Transactions & Vector Macro-JIT
- **Zero-Roundtrip Action Pipelines (`cu_batch` & `cu_fill_mark`)**: Execute multi-step workflows (`click_mark`, `right_click_mark`, `hover_mark`, `fill_mark`, `click_name`, `type`, `paste`, `key`, `hotkey`, `scroll`, `focus_window`, `wait`, `wait_for_change`, `wait_for_text`) in a single atomic call and receive the post-transaction `prompt_summary` and `ui_delta` immediately. `cu_fill_mark` clicks an input field by `mark_id`, optionally clears existing text (`Ctrl+A -> Backspace`), types/pastes text, and optionally submits (`Enter`) in a single call.
- **Parametric Vector & Pointillist Canvas Engine (`cu_canvas_draw`)**: Automatically detects the exact drawing surface (`DynamicCanvas`) and renders multi-layered geometric primitives (`gear`, `star`, `polygon`, `circle`, `spiral`, `line`, `rect`, `bezier`, `text`, `dots`/`points`) with aspect-ratio compensation, WinUI3 Ribbon focus-recovery, and automatic closed-shape first-edge sealing.
- **UTF-16 Surrogate & Instant Clipboard Typing (`cu_type`)**: Full UTF-16 surrogate pair support, automatic `\n`/`\t` virtual key translation, full OEM punctuation hotkey resolution (`+`, `-`, `=`, `/`, `[`, `]`, etc.), and non-destructive instant clipboard paste (`paste: true`).

### 3. Pillar III — Per-Monitor DPI v2 & Virtual Desktop Input Fidelity
- **Per-Monitor DPI v2**: Automatically activates `DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2` (`enable_per_monitor_dpi_v2`) and attaches threads to the active input desktop (`OpenInputDesktop` / `SetThreadDesktop`).
- **Virtual Desktop Hardware Injection**: Normalizes `SendInput` coordinates across the full virtual desktop (`SM_XVIRTUALSCREEN`, `SM_YVIRTUALSCREEN`, `SM_CXVIRTUALSCREEN`, `SM_CYVIRTUALSCREEN`) with `MOUSEEVENTF_VIRTUALDESK | MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_MOVE` for 1:1 physical accuracy across multi-monitor and negative-coordinate layouts.
- **O(1) Win32 Window Inspection & Visible-First Matching**: `find_window_by_hwnd` queries `IsWindow` + `GetWindowRect` in O(1) microseconds without enumerating the desktop, while `find_window` prioritizes visible non-zero windows over background message HWNDs.

### 4. Pillar IV — Event-Driven Reactive Synchronization & State Delta
- **Reactive UI & Visual Settling (`cu_wait_for_change` & `cu_wait_for_text`)**: Polls the foreground HWND, window title, UIA element structure, and 64-bit FNV-1a visual pixel checksum (`region_visual_checksum`) in `25ms` ticks, returning the instant a menu, popup, canvas stroke, or target text substring (`cu_wait_for_text`) appears instead of sleeping blindly.
- **Structural Delta Verification (`UiDelta`)**: Every interactive step reports a `ui_delta` summary (e.g., `"UI tree updated (+12 interactive elements, total 64)"` or `"Window switched -> ..."`), allowing the agent to detect immediately whether a click succeeded.

---

## MCP Server (`cu mcp`)

Run as a native stdio MCP server or synchronize schemas directly to Antigravity's MCP configuration directory:

```sh
# Start the stdio JSON-RPC 2.0 MCP server (auto-syncs schemas on startup)
cu mcp

# Force-sync all 44 tool schemas + instructions.md to ~/.gemini/antigravity-cli/mcp/computeruse
cu mcp --sync
```

### Available MCP Tools (44 Tools)

| Category | Tools |
|---|---|
| **Perception & Visual** | `cu_observe`, `cu_read_window_text`, `cu_screenshot`, `cu_find_color`, `cu_inspect_element_at` |
| **Semantic & Mouse Input** | `cu_click_mark`, `cu_right_click_mark`, `cu_hover_mark`, `cu_fill_mark`, `cu_drag_mark_to_mark`, `cu_click_name`, `cu_click_rel`, `cu_click_coords`, `cu_double_click`, `cu_triple_click`, `cu_move`, `cu_drag`, `cu_drag_rel`, `cu_mouse_down`, `cu_mouse_up`, `cu_scroll` |
| **Keyboard & Clipboard** | `cu_type`, `cu_key`, `cu_hotkey`, `cu_copy_selection`, `cu_clipboard_get`, `cu_clipboard_set`, `cu_clipboard_set_image` |
| **Atomic Batch & Reactive Sync** | `cu_batch`, `cu_wait_for_change`, `cu_wait_for_text` |
| **Vector Canvas Engine** | `cu_canvas_draw`, `cu_canvas_clear` |
| **Window Management** | `cu_focus_window`, `cu_list_windows`, `cu_close_window`, `cu_maximize_window`, `cu_minimize_window`, `cu_snap_window` |
| **System, Process & Audio** | `cu_launch_app`, `cu_list_processes`, `cu_kill_process`, `cu_resolve_association`, `cu_system_sound`, `cu_special_folder`, `cu_audio_beep`, `cu_audio_volume`, `cu_sysinfo`, `cu_sync_schemas` |

---

## Build & Test

```sh
cd computeruse
cargo test --test integration
cargo build --release
# binary: target/release/cu.exe
```

## CLI usage

Every subcommand prints a single JSON object to stdout.

```sh
# Display & Screen
cu screen-size
cu monitors
cu screenshot [--monitor N] [--region x,y,w,h]
cu screenshot-active
cu screenshot-window <HWND>
cu screenshot-all
cu pixel X Y

# Mouse
cu cursor
cu move X Y
cu move-rel DX DY
cu click [left|right|middle] [--x X --y Y]
cu dbl-click [button] [--x X --y Y]
cu mouse-down BUTTON
cu mouse-up BUTTON
cu drag X1 Y1 X2 Y2 [button]
cu scroll AMOUNT [--horizontal]

# Keyboard
cu type "text" [--delay MS]   # or: cu type --stdin
cu key NAME
cu key-down NAME
cu key-up NAME
cu hotkey "ctrl+c"

# Windows
cu windows [--all]
cu find-window "title substring"
cu foreground
cu focus --title "..." | --hwnd N
cu minimize HWND
cu maximize HWND
cu restore HWND
cu close HWND
cu move-window HWND X Y W H
cu window-topmost HWND [--disable]
cu window-opacity HWND ALPHA (0-255)
cu wait-for-window "title" [--timeout MS]

# Audio
cu volume                     # get volume 0-100
cu volume --set 75            # set volume to 75%
cu mute                       # get mute status
cu mute --set true            # mute
cu mute --toggle              # toggle mute

# System & Drives
cu sysinfo                    # RAM, CPU, uptime, battery
cu drives                     # disk drives, free and total bytes
cu lock                       # lock Windows workstation
cu special-folder desktop     # resolve path (downloads, documents, temp, appdata)

# Process & Launch
cu processes
cu find-process "name"
cu kill-process PID
cu kill-by-name "name.exe"
cu launch "app.exe" [--params "..."]

# UI Automation
cu uia-at X Y
cu uia-focused
cu uia-find "Button Name"
cu uia-click "Button Name"
cu uia-children [--hwnd N]

# Batch Execution
cu exec '[{"action":"click","x":100,"y":200}, {"action":"type","text":"hello"}]'
```

### Batch `exec`

Pass a JSON array (or single object) of commands. Each item has an `action` field
plus action-specific fields. Returns `{"results":[...]}`.

## Library usage

```rust
use computeruse::*;

// Screen & Screenshot
let (w, h) = ComputerUse::screen_size()?;
let shot = ComputerUse::screenshot_full(None)?; // base64 PNG

// Mouse & Keyboard
ComputerUse::click(MouseButton::Left, Some(Point { x: 100, y: 100 }))?;
ComputerUse::type_text("hello from agent")?;
ComputerUse::hotkey_str("ctrl+s")?;

// Audio & System
let vol = ComputerUse::audio_get_volume()?;
let sys = ComputerUse::system_info()?;
let drives = ComputerUse::system_drives()?;
```

## Supported keys

- Letters, digits, F1–F12
- Enter, Tab, Esc, Space, Backspace, Delete, Insert
- Home, End, PageUp, PageDown, arrows
- Ctrl, Alt, Shift, Win, CapsLock
- Media keys: `volumeup`, `volumedown`, `volumemute`, `mediaplaypause`, `medianext`, `mediaprev`, `mediastop`

## License

MIT
