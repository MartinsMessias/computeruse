# AGENTS.md — AI Agent & Developer Engineering Playbook for `computeruse`

This document is the definitive engineering guide for AI coding agents and human contributors working on **`computeruse`** (`cu`). It formalizes architectural invariants, the **4 Pillars of OS-Native Symbiosis**, testing gates, and the self-updating documentation contract.

---

## 1. Project Architecture & Philosophy

`computeruse` is a pure-Rust Windows desktop automation engine, CLI (`cu`), and Model Context Protocol (`MCP 2024-11-05`) server designed to give autonomous agents sub-millisecond structural perception and hardware-exact input control.

```
                  ┌──────────────────────────────────────────────┐
                  │         computeruse Core (Pure Rust)         │
                  │  Win32 GDI/DWM • UI Automation • SendInput   │
                  │  Per-Monitor DPI v2 • Virtual Desktop Space  │
                  └──────────────────────┬───────────────────────┘
                                         │
        ┌────────────────────────────────┼────────────────────────────────┐
        ▼                                ▼                                ▼
  MCP Stdio Server                 Standalone CLI                   Rust Library API
  (`cu mcp` / `--sync`)            (`cu <subcommand>`)              (`ComputerUse`,
  54 Tools • 4 Resources           JSON I/O & Compact DSL           `AgentEnvironment`,
  Multi-Content PNG Blocks         (`cu exec "C x,y; T 'hi'"`)      `DynamicCanvas`, `SomSnapshot`)
```

### Core Architectural Invariants (The 4 Pillars):
1. **Pillar I — Sub-Token Structural Perception First:**
   - Never force an LLM agent to take a 4K screenshot just to find a button. `cu_observe` (`src/som.rs`) must always produce a `< 1.8 KB` categorized Set-of-Marks (`[ID]`) digest (`Tools/Buttons`, `Palette/Items`, `Inputs/Controls`, `Menus/Other`) with 4-edge viewport bounds filtering.
   - `cu_screenshot` must support low-token targeted crops (`mark_id`, `canvas_only`, `hwnd`, `region`) and emit native MCP `image/png` content blocks.
2. **Pillar II — Zero-Roundtrip Atomic Execution:**
   - Multi-step interactions must be executable in a single atomic call via `cu_batch`, `cu_fill_mark`, `cu_save_file_as`, or the CLI Compact Action DSL (`cu exec`).
3. **Pillar III — Hardware-Exact Input & Focus Fidelity:**
   - Always enable `DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2` (`src/desktop.rs`).
   - Normalize mouse coordinates across the full virtual desktop (`MOUSEEVENTF_VIRTUALDESK | MOUSEEVENTF_ABSOLUTE`).
   - Populate hardware scancodes (`MapVirtualKeyW`) and `KEYEVENTF_EXTENDEDKEY` for all keyboard events (`src/keyboard.rs`).
   - Resolve `ApplicationFrameWindow` child HWNDs and attach input threads (`AttachThreadInput`) when focusing UWP / WinUI 3 windows (`src/window.rs`).
4. **Pillar IV — Reactive Event-Driven Synchronization:**
   - Replace blind `sleep()` delays with 25ms–50ms reactive polling (`wait_for_change`, `wait_for_screen_change`, `wait_for_text`) using UIA structural deltas and 64-bit FNV-1a visual pixel checksums (`src/pixel.rs`).
   - Always fall back to physical coordinate click injection when UIA `InvokePattern::Invoke()` fails or returns spurious success HRESULTs (`src/uia.rs`).

---

## 2. Repository Layout

```
computeruse/
├── src/
│   ├── lib.rs          # Crate root & high-level `ComputerUse` facade
│   ├── mcp.rs          # MCP 2024-11-05 stdio server (54 Tools, 4 Resources, Prompts)
│   ├── agent.rs        # `AgentEnvironment`, `AgentAction`, `UiDelta`, reactive sync
│   ├── som.rs          # Set-of-Marks (`SomSnapshot`) UIA traversal & badge renderer
│   ├── uia.rs          # Windows UI Automation COM queries, 4-tier fuzzy search, text reader
│   ├── canvas.rs       # `DynamicCanvas` detector & parametric vector/pointillist engine
│   ├── window.rs       # Win32/UWP window management, snap, Save As macro, modal check
│   ├── screen.rs       # GDI/PrintWindow capture, multi-monitor work_area, visual change poll
│   ├── mouse.rs        # Virtual-desktop `SendInput`, smooth glide, modifiers, long/triple click
│   ├── keyboard.rs     # Hardware scancodes, UTF-16 surrogates, OEM/browser/media keys
│   ├── clipboard.rs    # RAII retry clipboard text & `CF_BITMAP` region copy
│   ├── pixel.rs        # Fast RGB tolerance search, color ratio, 64-bit FNV-1a checksum
│   ├── system.rs       # RAM/CPU/battery telemetry, drives, known folders, file associations
│   ├── audio.rs        # CoreAudio WASAPI endpoint volume/mute & COM apartment guard
│   ├── process.rs      # ToolHelp32 process enumeration & termination
│   ├── launch.rs       # `ShellExecuteW` app/URI launcher with working directory support
│   ├── desktop.rs      # Per-Monitor DPI v2 & `OpenInputDesktop` thread attachment
│   ├── types.rs        # Shared Serde data structures (`Point`, `Rect`, `WindowInfo`, etc.)
│   ├── error.rs        # `CuError` and `Result<T>`
│   └── bin/
│       └── cu.rs       # Clap CLI binary (`cu`) + Compact Action DSL parser
├── tests/
│   └── integration.rs  # Comprehensive Windows OS integration test suite (26 tests)
├── examples/           # 13 standalone Rust examples (`01_screen.rs` .. `autonomous_agent.rs`)
├── README.md           # User & MCP documentation
├── AGENTS.md           # Engineering playbook for AI agents & contributors
├── llms.txt            # Concise LLM discovery index
└── llms-full.txt       # Complete LLM technical reference
```

---

## 3. Local Development & Quality Gates

Before committing or pushing any changes, run all quality gates locally:

```sh
# 1. Fast type & borrow check (must produce 0 warnings / 0 errors)
cargo check

# 2. Full integration test battery (26 tests covering all 4 Pillars)
cargo test --test integration

# 3. Release binary build
cargo build --release

# 4. Sync MCP schemas if tools in src/mcp.rs were added or modified
./target/release/cu.exe mcp --sync
```

---

## 4. Autonomous Self-Updating Documentation Directive

Whenever any tool, CLI subcommand, `AgentAction` variant, or `ComputerUse` method is added or modified:
1. Update `src/mcp.rs` (`get_all_tools` + `handle_tools_call`) and ensure tool names are unique.
2. Update `README.md` (tool count, category table, CLI usage, or Rust examples).
3. Update `llms.txt` and `llms-full.txt` so external LLMs and indexers always have accurate tool schemas and counts.
4. Run `cargo test --test integration` to verify zero regressions.
