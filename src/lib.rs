//! # computeruse
//!
//! Windows computer-use library for LLM agents.
//!
//! Provides screenshots, mouse, keyboard, window management, clipboard,
//! UI Automation, process management, and pixel inspection — all with
//! JSON-friendly types and a CLI (`cu`).
//!
//! ## Quick start
//! ```no_run
//! use computeruse::*;
//! let (w, h) = ComputerUse::screen_size().unwrap();
//! let shot = ComputerUse::screenshot_full(None).unwrap();
//! ComputerUse::click(MouseButton::Left, Some(Point { x: 100, y: 100 })).unwrap();
//! ```
//!
//! See the README for CLI usage.

pub mod agent;
pub mod audio;
pub mod canvas;
pub mod clipboard;
pub mod desktop;
pub mod error;
pub mod keyboard;
pub mod launch;
pub mod mcp;
pub mod mouse;
pub mod pixel;
pub mod process;
pub mod screen;
pub mod som;
pub mod system;
pub mod types;
pub mod uia;
pub mod window;

pub use agent::{AgentAction, AgentEnvironment, Observation, StepResult};
pub use canvas::DynamicCanvas;
pub use error::{CuError, Result};
pub use som::{InteractiveElement, SomSnapshot};
pub use types::*;

/// High-level handle. All methods are free functions too; this struct groups them for ergonomics.
pub struct ComputerUse;

impl ComputerUse {
    // Screen
    pub fn screen_size() -> Result<(u32, u32)> { screen::screen_size() }
    pub fn list_monitors() -> Result<Vec<Monitor>> { screen::list_monitors() }
    pub fn screenshot_full(monitor: Option<u32>) -> Result<Screenshot> { screen::screenshot_full(monitor) }
    pub fn screenshot_region(region: Rect, monitor: Option<u32>) -> Result<Screenshot> { screen::screenshot_region(region, monitor) }
    pub fn screenshot_window(hwnd: isize) -> Result<Screenshot> { screen::screenshot_window(hwnd) }
    pub fn screenshot_active_window() -> Result<Screenshot> { screen::screenshot_active_window() }
    pub fn screenshot_all_monitors() -> Result<Screenshot> { screen::screenshot_all_monitors() }
    pub fn cursor_position() -> Result<Point> { screen::cursor_position() }
    pub fn set_cursor_position(p: Point) -> Result<()> { screen::set_cursor_position(p) }

    // Mouse
    pub fn mouse_move(p: Point) -> Result<()> { mouse::move_to(p) }
    pub fn mouse_move_smooth(p: Point, duration_ms: u64) -> Result<()> { mouse::move_smooth(p, duration_ms) }
    pub fn mouse_move_relative(dx: i32, dy: i32) -> Result<Point> { mouse::move_relative(dx, dy) }
    pub fn click(btn: MouseButton, p: Option<Point>) -> Result<()> { mouse::click(btn, p) }
    pub fn click_modified(btn: MouseButton, p: Option<Point>, modifiers: &[String]) -> Result<()> { mouse::click_modified(btn, p, modifiers) }
    pub fn double_click(btn: MouseButton, p: Option<Point>) -> Result<()> { mouse::double_click(btn, p) }
    pub fn triple_click(btn: MouseButton, p: Option<Point>) -> Result<()> { mouse::triple_click(btn, p) }
    pub fn mouse_down(btn: MouseButton) -> Result<()> { mouse::mouse_down(btn) }
    pub fn mouse_up(btn: MouseButton) -> Result<()> { mouse::mouse_up(btn) }
    pub fn drag(from: Point, to: Point, btn: MouseButton) -> Result<()> { mouse::drag(from, to, btn) }
    pub fn drag_modified(from: Point, to: Point, btn: MouseButton, modifiers: &[String]) -> Result<()> { mouse::drag_modified(from, to, btn, modifiers) }
    pub fn scroll(amount: i32, horizontal: bool) -> Result<()> { mouse::scroll(amount, horizontal) }
    pub fn scroll_at(amount: i32, horizontal: bool, p: Option<Point>, modifiers: &[String]) -> Result<()> { mouse::scroll_at(amount, horizontal, p, modifiers) }

    // Keyboard
    pub fn type_text(text: &str) -> Result<()> { keyboard::type_text(text) }
    pub fn type_text_with_delay(text: &str, delay_ms: u64) -> Result<()> { keyboard::type_text_with_delay(text, delay_ms) }
    pub fn paste_text(text: &str) -> Result<()> { keyboard::paste_text(text) }
    pub fn copy_selection() -> Result<String> { keyboard::copy_selection() }
    pub fn key_press(name: &str) -> Result<()> { keyboard::key_press(name) }
    pub fn key_down(name: &str) -> Result<()> { keyboard::key_down(name) }
    pub fn key_up(name: &str) -> Result<()> { keyboard::key_up(name) }
    pub fn hotkey(keys: &[String]) -> Result<()> { keyboard::hotkey(keys) }
    pub fn hotkey_str(s: &str) -> Result<()> { keyboard::combo_str(s) }

    // Window
    pub fn list_windows(visible_only: bool) -> Result<Vec<WindowInfo>> { window::list_windows(visible_only) }
    pub fn find_window(title: &str) -> Result<WindowInfo> { window::find_window(title) }
    pub fn foreground_window() -> Result<WindowInfo> { window::foreground_window() }
    pub fn focus_window(hwnd: isize) -> Result<()> { window::focus_window(hwnd) }
    pub fn focus_by_title(title: &str) -> Result<WindowInfo> { window::focus_by_title(title) }
    pub fn minimize_window(hwnd: isize) -> Result<()> { window::minimize_window(hwnd) }
    pub fn maximize_window(hwnd: isize) -> Result<()> { window::maximize_window(hwnd) }
    pub fn restore_window(hwnd: isize) -> Result<()> { window::restore_window(hwnd) }
    pub fn close_window(hwnd: isize) -> Result<()> { window::close_window(hwnd) }
    pub fn move_resize_window(hwnd: isize, rect: Rect) -> Result<()> { window::move_resize_window(hwnd, rect) }
    pub fn snap_window(hwnd: isize, position: &str) -> Result<Rect> { window::snap_window(hwnd, position) }
    pub fn wait_for_window(title: &str, timeout_ms: u64) -> Result<WindowInfo> { window::wait_for_window(title, timeout_ms) }
    pub fn set_window_topmost(hwnd: isize, topmost: bool) -> Result<()> { window::set_topmost(hwnd, topmost) }
    pub fn set_window_opacity(hwnd: isize, alpha: u8) -> Result<()> { window::set_opacity(hwnd, alpha) }
    pub fn window_rect(hwnd: isize) -> Result<Rect> {
        let win = window::find_window_by_hwnd(hwnd)?;
        Ok(win.rect)
    }
    pub fn click_window_rel(hwnd: isize, rel_x: f64, rel_y: f64, btn: MouseButton) -> Result<()> {
        let r = Self::window_rect(hwnd)?;
        let abs_x = r.x + ((r.width as f64) * rel_x).round() as i32;
        let abs_y = r.y + ((r.height as f64) * rel_y).round() as i32;
        mouse::click(btn, Some(Point { x: abs_x, y: abs_y }))
    }
    pub fn drag_window_rel(hwnd: isize, from: (f64, f64), to: (f64, f64), btn: MouseButton) -> Result<()> {
        let r = Self::window_rect(hwnd)?;
        let p1 = Point {
            x: r.x + ((r.width as f64) * from.0).round() as i32,
            y: r.y + ((r.height as f64) * from.1).round() as i32,
        };
        let p2 = Point {
            x: r.x + ((r.width as f64) * to.0).round() as i32,
            y: r.y + ((r.height as f64) * to.1).round() as i32,
        };
        mouse::drag(p1, p2, btn)
    }

    // Clipboard
    pub fn clipboard_get() -> Result<String> { clipboard::get_text() }
    pub fn clipboard_set(text: &str) -> Result<()> { clipboard::set_text(text) }
    pub fn clipboard_set_image_from_region(region: Rect) -> Result<()> { clipboard::set_image_from_region(region) }
    // Pixel
    pub fn get_pixel(p: Point) -> Result<(u8, u8, u8)> { pixel::get_pixel(p) }
    pub fn find_color_in_region(region: Rect, target_rgb: (u8, u8, u8), tolerance: u8) -> Result<Option<Point>> {
        pixel::find_color_in_region(region, target_rgb, tolerance)
    }
    pub fn region_visual_checksum(region: Rect) -> Result<u64> {
        pixel::region_visual_checksum(region)
    }
    // Launch
    pub fn launch(target: &str, params: Option<&str>) -> Result<()> { launch::launch(target, params) }
    // Process
    pub fn list_processes() -> Result<Vec<process::ProcessInfo>> { process::list_processes() }
    pub fn find_process(name: &str) -> Result<process::ProcessInfo> { process::find_process(name) }
    pub fn kill_process(pid: u32) -> Result<()> { process::kill_process(pid) }
    pub fn kill_by_name(name: &str) -> Result<u32> { process::kill_by_name(name) }
    // UI Automation
    pub fn uia_element_at_point(x: i32, y: i32) -> Result<UiElementInfo> { uia::element_at_point(x, y) }
    pub fn uia_focused_element() -> Result<UiElementInfo> { uia::focused_element() }
    pub fn uia_find_by_name(name: &str) -> Result<Vec<UiElementInfo>> { uia::find_by_name(name) }
    pub fn uia_find_by_id(id: &str) -> Result<Vec<UiElementInfo>> { uia::find_by_automation_id(id) }
    pub fn uia_click_by_name(name: &str) -> Result<UiElementInfo> { uia::click_by_name(name) }
    pub fn uia_children(hwnd: isize) -> Result<Vec<UiElementInfo>> { uia::get_children(hwnd) }
    pub fn uia_read_window_text(hwnd: isize) -> Result<Vec<String>> { uia::read_window_text(hwnd) }

    // Audio
    pub fn audio_get_volume() -> Result<u32> { audio::get_volume() }
    pub fn audio_set_volume(percent: u32) -> Result<()> { audio::set_volume(percent) }
    pub fn audio_get_mute() -> Result<bool> { audio::get_mute() }
    pub fn audio_set_mute(muted: bool) -> Result<()> { audio::set_mute(muted) }
    pub fn audio_toggle_mute() -> Result<bool> { audio::toggle_mute() }
    pub fn audio_beep(freq_hz: u32, duration_ms: u32) -> Result<()> { audio::beep(freq_hz, duration_ms) }

    // System
    pub fn system_info() -> Result<SystemInfo> { system::get_system_info() }
    pub fn system_drives() -> Result<Vec<DriveInfo>> { system::get_drives() }
    pub fn system_lock() -> Result<()> { system::lock() }
    pub fn system_special_folder(name: &str) -> Result<String> { system::get_special_folder(name) }

    // Dynamic Canvas & Adaptive Vector Drawing
    pub fn canvas_find_in_window(hwnd: isize) -> Result<DynamicCanvas> {
        DynamicCanvas::find_in_window(hwnd)
    }

    // Set-of-Marks & Neuro-Symbolic Perception
    pub fn som_snapshot(hwnd: isize) -> Result<SomSnapshot> {
        som::capture_som(hwnd)
    }

    // Dynamic Agent Perception-Action Environment
    pub fn agent_env() -> AgentEnvironment {
        AgentEnvironment::new()
    }
}
