# computeruse — Windows Computer-Use Library for LLMs

A high-performance Rust library + CLI (`cu`) that exposes Windows desktop automation primitives
in a form optimized for LLM agents: screenshots, mouse, keyboard, windows, UI Automation,
audio/volume, system metrics, clipboard, and pixel inspection. All output is clean JSON.

## Features

- **Robust Desktop Attachment**: Automatically connects background threads and agent sessions to the active input desktop (`OpenInputDesktop` / `SetThreadDesktop`), preventing `BitBlt` and cursor access errors.
- **Screen & Display**: Multi-monitor support, primary monitor detection, DPI scaling awareness, full/region/window screenshots (base64 PNG), pixel color sampling.
- **Mouse**: Move, relative move, left/right/middle click, double click, mouse down/up, drag-and-drop, vertical/horizontal scroll.
- **Keyboard**: Unicode text typing (with configurable delay), key press/down/up, key combinations (`ctrl+shift+esc`), and media playback / volume keys.
- **Window Management**: Enumerate visible/all windows, title search, focus/foreground, minimize, maximize, restore, close, move/resize, pin topmost, adjust opacity.
- **UI Automation (UIA)**: Inspect accessibility element at point, find by name, get focused element, traverse window children, invoke clicks on UI elements directly.
- **Audio Control**: Get/set master volume (0–100%), get/set mute, toggle mute.
- **System & Power**: RAM stats (total, available, load %), CPU cores, system uptime, battery/power status, logical drives & disk space, workstation lock.
- **Special Folders**: Resolve standard Windows paths (Desktop, Downloads, Documents, AppData, Temp, etc.).
- **Process Management**: List processes, find by name, kill by PID or process name.
- **Batch Execution**: Run sequences of actions in a single atomic invocation (`cu exec`).

## Build

```sh
cd computeruse
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
