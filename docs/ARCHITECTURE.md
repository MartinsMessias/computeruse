# Architecture — The 4 Pillars of OS-Native Symbiosis

Instead of relying on slow, lossy 4K screenshot-to-pixel guessing loops, `computeruse` gives LLM agents direct structural perception, atomic macro execution, hardware-exact input injection, and event-driven UI synchronization.

---

## 1. Pillar I — Hybrid Sub-Token Perception & Targeted Visual Crop

- **Categorized Set-of-Marks (`cu_observe`)**: Traverses the Windows UI Automation (UIA) tree in milliseconds, enforces 4-edge viewport bounds filtering, extracts live values from inputs/sliders (`ValueValue` / `LegacyIAccessibleValue`), and compresses hundreds of elements into a `< 1.8 KB` categorized semantic digest (`Tools/Buttons`, `Palette/Items`, `Inputs/Controls`, `Menus/Other`). Pass `verbose: true` only when raw bounding boxes are needed.
- **4-Tier Semantic & Fuzzy Element Search (`cu_find_element`)**: Locates interactive controls by:
  1. Exact or substring UIA `Name` match.
  2. UIA `AutomationId` substring match.
  3. Host-window tokenized fuzzy matching.
  4. Set-of-Marks (`SomSnapshot`) interactive element fallback.
- **Direct Window Text Extraction (`cu_read_window_text`)**: Extracts all visible text strings, labels, and control values from any window in milliseconds without OCR or screenshots.
- **Targeted Visual Crops & Annotated Badges (`cu_screenshot`)**: Crop directly to a specific UI element (`mark_id`), only the drawing canvas (`canvas_only: true`), a target window (`hwnd`), or a custom rectangle (`region: {x, y, width, height}`), overlay numbered Set-of-Marks `[ID]` badges (`annotate_marks: true`), and save directly to disk (`save_path`, `omit_base64: true`).
- **Fast Color Scanning & Pixel Ratio Telemetry (`cu_find_color` & `cu_count_color_ratio`)**: Single-GDI-capture RGB tolerance scanning and area coverage ratio calculation for instant visual verification.

---

## 2. Pillar II — Atomic Transactions, Compact DSL & Vector Macro-JIT

- **Zero-Roundtrip Action Pipelines (`cu_batch`, `cu_fill_mark`, `cu_save_file_as`)**:
  - `cu_batch`: Execute multi-step workflows (`click_mark`, `right_click_mark`, `hover_mark`, `fill_mark`, `drag_mark_to_mark`, `click_name`, `type`, `paste`, `key`, `hotkey`, `scroll`, `focus_window`, `wait`, `wait_for_change`, `wait_for_text`) in a single atomic call and receive the post-transaction `prompt_summary` and `ui_delta` immediately.
  - `cu_fill_mark`: Clicks an input field by `mark_id`, optionally clears existing text (`Ctrl+A -> Backspace`), types/pastes text, and optionally submits (`Enter`) in one call.
  - `cu_save_file_as`: Atomic macro that dispatches `Ctrl+S`, waits for the native Windows `#32770` Save As dialog, injects the full target path, and confirms overwrite modals automatically.
- **Compact Action DSL (`cu exec`)**: Execute ultra-low-token semicolon-separated command strings directly from the CLI (e.g. `cu exec "C 500,300; W 100; T 'hello'; K enter; H ctrl+s; D 100,100,400,400"`).
- **Parametric Vector & Pointillist Canvas Engine (`cu_canvas_draw` & `cu_canvas_clear`)**: Automatically detects the exact drawing surface (`DynamicCanvas`) and renders multi-layered geometric primitives (`gear`, `star`, `polygon`, `circle`, `spiral`, `line`, `rect`, `bezier`, `text`, `dots`/`points`) with aspect-ratio compensation, WinUI 3 Ribbon focus-recovery, and automatic closed-shape first-edge sealing.
- **Human-Parity Input & Clipboard Injection (`cu_type`, `cu_copy_selection`, `cu_clipboard_set_image`)**: Full hardware scancode (`MapVirtualKeyW`) + `KEYEVENTF_EXTENDEDKEY` injection, UTF-16 surrogate pair support, modifier-held clicks/drags/scrolls (`ctrl`, `shift`, `alt`), smooth human ease-in-out cursor trajectories (`smooth_ms`), and direct screen-region-to-`CF_BITMAP` clipboard copying.

---

## 3. Pillar III — Per-Monitor DPI v2, UWP/WinUI 3 Focus & Virtual Desktop Fidelity

- **Per-Monitor DPI v2**: Automatically activates `DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2` (`enable_per_monitor_dpi_v2`) and attaches threads to the active input desktop (`OpenInputDesktop` / `SetThreadDesktop`).
- **Virtual Desktop Hardware Injection & Work-Area Snapping**: Normalizes `SendInput` coordinates across the full virtual desktop (`SM_XVIRTUALSCREEN`, `SM_YVIRTUALSCREEN`, `SM_CXVIRTUALSCREEN`, `SM_CYVIRTUALSCREEN`) with `MOUSEEVENTF_VIRTUALDESK | MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_MOVE`, and snaps windows (`cu_snap_window`) respecting taskbar-aware monitor `work_area` bounds.
- **UWP / WinUI 3 `ApplicationFrameWindow` Resolution & UIPI Focus Bypass (`cu_focus_window`)**: Automatically drills down into packaged `ApplicationFrameWindow` containers to resolve the real visible child `HWND`, synchronizes input queues via `AttachThreadInput`, and performs legal `AllowSetForegroundWindow` + `VK_MENU` UIPI elevation.
- **O(1) Win32 Window Inspection & Modal Detection (`cu_check_modal`)**: `find_window_by_hwnd` queries `IsWindow` + `GetWindowRect` in O(1) microseconds, while `cu_check_modal` detects whether an application is blocked by a modal `#32770` dialog.

---

## 4. Pillar IV — Event-Driven Reactive Synchronization & State Delta

- **Reactive UI & Visual Settling (`cu_wait_for_change` & `cu_wait_for_text`)**: Polls the foreground HWND, window title, UIA element structure, 64-bit FNV-1a visual pixel checksum (`region_visual_checksum`), or a custom screen `region` in `25ms–50ms` ticks, returning the instant a menu, popup, canvas stroke, or target text substring (`cu_wait_for_text`) appears instead of sleeping blindly.
- **Resilient UIA Invocation & Structural Delta (`UiDelta`)**: Automatically falls back to physical coordinate click injection when programmatic `InvokePattern` fails or returns spurious success HRESULTs, and reports a `ui_delta` summary after every step (e.g., `"UI tree updated (+12 interactive elements, total 64)"`).
