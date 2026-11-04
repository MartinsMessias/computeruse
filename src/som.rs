use std::io::Cursor;
use image::{ImageFormat, Rgba, RgbaImage};
use serde::{Deserialize, Serialize};
use uiautomation::core::UIAutomation;
use uiautomation::types::Handle;
use crate::error::{CuError, Result};
use crate::mouse;
use crate::screen;
use crate::types::{MouseButton, Point, Rect};
use crate::window;

/// A cataloged interactive UI element detected in a window.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractiveElement {
    pub mark_id: usize,
    pub name: String,
    pub control_type: String,
    pub automation_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    pub rect: Rect,
    pub center: Point,
    pub rel_center: (f64, f64),
    pub is_enabled: bool,
}

/// A complete Set-of-Marks snapshot of a window for LLM perception.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SomSnapshot {
    pub hwnd: isize,
    pub title: String,
    pub window_rect: Rect,
    pub elements: Vec<InteractiveElement>,
}

/// Sanitizes UIA strings by stripping invisible Unicode bidi marks and collapsing newlines/tabs into single spaces.
pub fn clean_ui_text(s: &str) -> String {
    let filtered: String = s
        .chars()
        .filter(|&c| !matches!(c, '\u{200e}' | '\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}' | '\u{feff}'))
        .map(|c| if matches!(c, '\r' | '\n' | '\t') { ' ' } else { c })
        .collect();
    filtered.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn truncate_str(s: &str, max_chars: usize) -> String {
    if s.chars().count() > max_chars {
        let prefix: String = s.chars().take(max_chars.saturating_sub(3)).collect();
        format!("{}...", prefix)
    } else {
        s.to_string()
    }
}

impl SomSnapshot {
    /// Formats the elements into a high-density, categorized representation for ultra-fast LLM comprehension.
    pub fn to_compact_prompt(&self) -> String {
        let clean_title = clean_ui_text(&self.title);
        let mut out = format!(
            "Window: \"{}\" [HWND={}] Bounds: {}x{} at ({}, {})\n",
            clean_title, self.hwnd, self.window_rect.width, self.window_rect.height, self.window_rect.x, self.window_rect.y
        );

        let mut tools_and_buttons = Vec::new();
        let mut palette_items = Vec::new();
        let mut inputs = Vec::new();
        let mut menus_and_other = Vec::new();

        for el in &self.elements {
            if matches!(el.control_type.as_str(), "Group" | "Window" | "Pane" | "TitleBar") {
                continue;
            }
            let raw_label = if !el.name.trim().is_empty() {
                clean_ui_text(&el.name)
            } else if !el.automation_id.trim().is_empty() {
                clean_ui_text(&el.automation_id)
            } else {
                String::new()
            };

            if raw_label.is_empty() && el.value.is_none() {
                continue;
            }

            let val_suffix = el.value.as_ref().map(|v| {
                let cv = clean_ui_text(v);
                format!("=\"{}\"", truncate_str(&cv, 28))
            }).unwrap_or_default();

            let label = truncate_str(&raw_label, 42);
            let entry = format!("[{:02}] \"{}\"{} ({},{})", el.mark_id, label, val_suffix, el.center.x, el.center.y);
            match el.control_type.as_str() {
                "ListItem" => palette_items.push(entry),
                "Button" | "RadioButton" | "SplitButton" | "CheckBox" => tools_and_buttons.push(entry),
                "Edit" | "Document" | "ComboBox" | "Spinner" | "Slider" | "ProgressBar" => inputs.push(entry),
                _ => {
                    let short_name = truncate_str(&raw_label, 35);
                    menus_and_other.push(format!("[{:02}] \"{}\"{} ({},{})", el.mark_id, short_name, val_suffix, el.center.x, el.center.y));
                }
            }
        }

        if !tools_and_buttons.is_empty() {
            out.push_str(&format!("Tools/Buttons ({}): {}\n", tools_and_buttons.len(), tools_and_buttons.join(" | ")));
        }
        if !palette_items.is_empty() {
            out.push_str(&format!("Palette/Items ({}): {}\n", palette_items.len(), palette_items.join(" | ")));
        }
        if !inputs.is_empty() {
            out.push_str(&format!("Inputs/Controls ({}): {}\n", inputs.len(), inputs.join(" | ")));
        }
        if !menus_and_other.is_empty() {
            out.push_str(&format!("Menus/Other ({}): {}\n", menus_and_other.len(), menus_and_other.join(" | ")));
        }
        out
    }

    /// Finds the best matching element by name or automation_id (prefers exact match and non-Group controls first).
    pub fn find_by_name(&self, name_substr: &str) -> Option<&InteractiveElement> {
        let q = name_substr.to_lowercase();
        // 1. Exact name or automation_id match on non-Group controls (Button, RadioButton, ListItem, etc.)
        if let Some(el) = self.elements.iter().find(|el| {
            (el.name.to_lowercase() == q || el.automation_id.to_lowercase() == q) && el.control_type != "Group"
        }) {
            return Some(el);
        }
        // 2. Exact name match
        if let Some(el) = self.elements.iter().find(|el| el.name.to_lowercase() == q) {
            return Some(el);
        }
        // 3. Substring match on non-Group controls
        if let Some(el) = self.elements.iter().find(|el| {
            (el.name.to_lowercase().contains(&q) || el.automation_id.to_lowercase().contains(&q)) && el.control_type != "Group"
        }) {
            return Some(el);
        }
        // 4. Fallback substring match
        self.elements.iter().find(|el| el.name.to_lowercase().contains(&q))
    }

    /// Finds an element by its numeric mark identifier.
    pub fn find_by_mark(&self, mark_id: usize) -> Option<&InteractiveElement> {
        self.elements.iter().find(|el| el.mark_id == mark_id)
    }

    /// Clicks on an element by its mark identifier.
    pub fn click_mark(&self, mark_id: usize) -> Result<()> {
        let el = self
            .find_by_mark(mark_id)
            .ok_or_else(|| CuError::NotFound(format!("Mark ID [{}] not found", mark_id)))?;
        mouse::click(MouseButton::Left, Some(el.center))
    }

    /// Clicks on an element matching a name substring.
    pub fn click_element_by_name(&self, name_substr: &str) -> Result<()> {
        let el = self
            .find_by_name(name_substr)
            .ok_or_else(|| CuError::NotFound(format!("Element matching \"{}\" not found", name_substr)))?;
        mouse::click(MouseButton::Left, Some(el.center))
    }

    /// Overlays numbered high-contrast visual badges onto an existing PNG image buffer
    /// whose top-left screen origin is `(region.x, region.y)`.
    pub fn annotate_png_bytes(&self, raw_png_bytes: &[u8], region: Rect) -> Result<Vec<u8>> {
        let img = image::load_from_memory(raw_png_bytes).map_err(|e| CuError::Image(e.to_string()))?;
        let mut rgba = img.to_rgba8();

        for el in &self.elements {
            let local_x = el.center.x - region.x;
            let local_y = el.center.y - region.y;

            if local_x >= 0
                && local_y >= 0
                && (local_x as u32) < rgba.width()
                && (local_y as u32) < rgba.height()
            {
                draw_mark_badge(&mut rgba, local_x as u32, local_y as u32, el.mark_id);
            }
        }

        let mut out = Vec::new();
        let mut cursor = Cursor::new(&mut out);
        rgba.write_to(&mut cursor, ImageFormat::Png)
            .map_err(|e| CuError::Image(e.to_string()))?;

        Ok(out)
    }

    /// Captures the window screenshot and overlays numbered high-contrast visual badges
    /// corresponding to each mark ID. Returns the encoded PNG bytes.
    pub fn generate_annotated_image(&self) -> Result<Vec<u8>> {
        let raw_shot = screen::screenshot_window(self.hwnd)?;
        let raw_bytes = base64::Engine::decode(
            &base64::engine::general_purpose::STANDARD,
            &raw_shot.png_base64,
        )
        .map_err(|e| CuError::Image(e.to_string()))?;

        self.annotate_png_bytes(&raw_bytes, self.window_rect)
    }
}

/// Takes a Set-of-Marks snapshot for a given window HWND.
pub fn capture_som(hwnd: isize) -> Result<SomSnapshot> {
    let win = window::find_window_by_hwnd(hwnd)?;
    let automation = UIAutomation::new().map_err(|e| CuError::Windows(e.to_string()))?;

    let handle: Handle = unsafe { std::mem::transmute(hwnd as *mut std::ffi::c_void) };
    let root = match automation.element_from_handle(handle) {
        Ok(r) => r,
        Err(_) => {
            return Ok(SomSnapshot {
                hwnd,
                title: win.title,
                window_rect: win.rect,
                elements: Vec::new(),
            });
        }
    };

    let walker = automation
        .get_control_view_walker()
        .map_err(|e| CuError::Windows(e.to_string()))?;

    let mut raw_elements = Vec::new();
    let mut visited = 0usize;
    traverse_interactive(&root, &walker, &mut raw_elements, 0, &mut visited);

    let mut elements = Vec::new();
    let mut mark_id = 1;

    for el in raw_elements {
        let name = clean_ui_text(&el.get_name().unwrap_or_default());
        let automation_id = clean_ui_text(&el.get_automation_id().unwrap_or_default());
        let control_type = format!("{:?}", el.get_control_type().unwrap_or(uiautomation::controls::ControlType::Custom));
        let r = el.get_bounding_rectangle().unwrap_or_default();
        let width = r.get_width().max(0) as u32;
        let height = r.get_height().max(0) as u32;
        let x = r.get_left();
        let y = r.get_top();

        // Filter out zero-size or out-of-window artifacts using all-4-bounds zero-cost arithmetic
        let win_right = win.rect.x + (win.rect.width as i32);
        let win_bottom = win.rect.y + (win.rect.height as i32);
        if width < 5
            || height < 5
            || x + (width as i32) <= win.rect.x
            || y + (height as i32) <= win.rect.y
            || x >= win_right
            || y >= win_bottom
        {
            continue;
        }

        // Extract live value for Edit, ComboBox, Slider, Spinner, Document, ProgressBar controls safely
        let value = match control_type.as_str() {
            "Edit" | "ComboBox" | "Slider" | "Spinner" | "Document" | "ProgressBar" => crate::uia::get_element_value(&el),
            _ => None,
        };

        // Skip completely unnamed, ID-less, value-less buttons/items that carry zero semantic utility
        if name.is_empty() && automation_id.is_empty() && value.is_none() {
            continue;
        }

        // Element center point
        let cx = x + (width as i32 / 2);
        let cy = y + (height as i32 / 2);

        // Deduplicate overlapping marks or inner Text labels of Buttons/TabItems within 8px with the same name
        if elements.iter().any(|prev: &InteractiveElement| {
            prev.name == name
                && (prev.center.x - cx).abs() <= 8
                && (prev.center.y - cy).abs() <= 8
        }) {
            continue;
        }

        // Normalize relative to window
        let win_w = win.rect.width.max(1) as f64;
        let win_h = win.rect.height.max(1) as f64;
        let rel_x = ((cx - win.rect.x) as f64 / win_w).clamp(0.0, 1.0);
        let rel_y = ((cy - win.rect.y) as f64 / win_h).clamp(0.0, 1.0);

        elements.push(InteractiveElement {
            mark_id,
            name,
            control_type,
            automation_id,
            value,
            rect: Rect { x, y, width, height },
            center: Point { x: cx, y: cy },
            rel_center: (rel_x, rel_y),
            is_enabled: el.is_enabled().unwrap_or(true),
        });

        mark_id += 1;
    }

    let display_title = if win.title.trim().is_empty() {
        format!("[{}]", win.class_name)
    } else {
        win.title
    };

    Ok(SomSnapshot {
        hwnd,
        title: display_title,
        window_rect: win.rect,
        elements,
    })
}

fn traverse_interactive(
    el: &uiautomation::core::UIElement,
    walker: &uiautomation::core::UITreeWalker,
    out: &mut Vec<uiautomation::core::UIElement>,
    depth: usize,
    visited: &mut usize,
) {
    if depth > 12 || out.len() >= 160 || *visited >= 400 {
        return;
    }
    *visited += 1;

    if !el.is_offscreen().unwrap_or(false) {
        if let Ok(ct) = el.get_control_type() {
            let ct_name = format!("{:?}", ct);
            let is_container = matches!(
                ct_name.as_str(),
                "Window" | "Group" | "Pane" | "TitleBar" | "ToolBar" | "MenuBar" | "StatusBar" | "ScrollBar"
            );
            let is_actionable = matches!(
                ct_name.as_str(),
                "Button"
                    | "SplitButton"
                    | "CheckBox"
                    | "ComboBox"
                    | "Edit"
                    | "Document"
                    | "Hyperlink"
                    | "ListItem"
                    | "TreeItem"
                    | "DataItem"
                    | "MenuItem"
                    | "RadioButton"
                    | "TabItem"
                    | "Slider"
                    | "Spinner"
            );

            let has_name = el.get_name().map(|n| !n.trim().is_empty()).unwrap_or(false);

            if !is_container && (is_actionable || has_name) {
                out.push(el.clone());
            }
        }
    }

    if let Ok(mut child) = walker.get_first_child(el) {
        loop {
            if *visited >= 400 || out.len() >= 160 {
                break;
            }
            traverse_interactive(&child, walker, out, depth + 1, visited);
            match walker.get_next_sibling(&child) {
                Ok(n) => child = n,
                Err(_) => break,
            }
        }
    }
}

// 5x7 bitmap font patterns for digits '0'..'9'
const DIGITS_5X7: [&[u8; 7]; 10] = [
    &[0x1F, 0x11, 0x11, 0x11, 0x11, 0x11, 0x1F], // 0
    &[0x04, 0x0C, 0x04, 0x04, 0x04, 0x04, 0x0E], // 1
    &[0x1F, 0x01, 0x01, 0x1F, 0x10, 0x10, 0x1F], // 2
    &[0x1F, 0x01, 0x01, 0x0F, 0x01, 0x01, 0x1F], // 3
    &[0x11, 0x11, 0x11, 0x1F, 0x01, 0x01, 0x01], // 4
    &[0x1F, 0x10, 0x10, 0x1F, 0x01, 0x01, 0x1F], // 5
    &[0x1F, 0x10, 0x10, 0x1F, 0x11, 0x11, 0x1F], // 6
    &[0x1F, 0x01, 0x02, 0x04, 0x08, 0x08, 0x08], // 7
    &[0x1F, 0x11, 0x11, 0x1F, 0x11, 0x11, 0x1F], // 8
    &[0x1F, 0x11, 0x11, 0x1F, 0x01, 0x01, 0x1F], // 9
];

fn draw_mark_badge(img: &mut RgbaImage, cx: u32, cy: u32, id: usize) {
    let id_str = id.to_string();
    let num_digits = id_str.len();
    let badge_w = (num_digits * 6 + 6) as u32;
    let badge_h = 13u32;

    let x0 = cx.saturating_sub(badge_w / 2);
    let y0 = cy.saturating_sub(badge_h / 2);

    let img_w = img.width();
    let img_h = img.height();

    // Solid red background with yellow border
    let bg_color = Rgba([220, 20, 60, 240]);
    let border_color = Rgba([255, 255, 0, 255]);
    let text_color = Rgba([255, 255, 255, 255]);

    for dy in 0..badge_h {
        for dx in 0..badge_w {
            let px = x0 + dx;
            let py = y0 + dy;
            if px < img_w && py < img_h {
                let is_border = dx == 0 || dx == badge_w - 1 || dy == 0 || dy == badge_h - 1;
                img.put_pixel(px, py, if is_border { border_color } else { bg_color });
            }
        }
    }

    // Render digits in white
    let start_text_x = x0 + 3;
    let start_text_y = y0 + 3;

    for (d_idx, ch) in id_str.chars().enumerate() {
        if let Some(digit) = ch.to_digit(10) {
            let pattern = DIGITS_5X7[digit as usize];
            let digit_x = start_text_x + (d_idx as u32 * 6);

            for (row, byte) in pattern.iter().enumerate() {
                for col in 0..5 {
                    if (byte & (0x10 >> col)) != 0 {
                        let px = digit_x + col;
                        let py = start_text_y + row as u32;
                        if px < img_w && py < img_h {
                            img.put_pixel(px, py, text_color);
                        }
                    }
                }
            }
        }
    }
}
