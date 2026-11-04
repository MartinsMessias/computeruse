use crate::error::{CuError, Result};
use crate::types::{Rect, UiElementInfo};
use uiautomation::core::UIAutomation;
use uiautomation::controls::ControlType;

fn to_info(el: &uiautomation::core::UIElement) -> UiElementInfo {
    let name = el.get_name().unwrap_or_default();
    let automation_id = el.get_automation_id().unwrap_or_default();
    let control_type = format!("{:?}", el.get_control_type().unwrap_or(ControlType::Custom));
    let class_name = el.get_classname().unwrap_or_default();
    let r = el.get_bounding_rectangle().unwrap_or_default();
    let hwnd = el.get_native_window_handle().ok().map(|h| {
        // parse "Handle(0x...)" debug format
        let s = format!("{:?}", h);
        let hex = s.trim_start_matches("Handle(0x").trim_end_matches(')');
        isize::from_str_radix(hex, 16).unwrap_or(0)
    }).unwrap_or(0);
    let is_enabled = el.is_enabled().unwrap_or(true);
    let is_offscreen = el.is_offscreen().unwrap_or(false);
    UiElementInfo {
        name,
        automation_id,
        control_type,
        class_name,
        rect: Rect {
            x: r.get_left(),
            y: r.get_top(),
            width: r.get_width().max(0) as u32,
            height: r.get_height().max(0) as u32,
        },
        hwnd,
        is_enabled,
        is_offscreen,
    }
}

pub fn element_at_point(x: i32, y: i32) -> Result<UiElementInfo> {
    let automation = UIAutomation::new().map_err(|e| CuError::Windows(e.to_string()))?;
    for _ in 0..3 {
        if let Ok(el) = automation.element_from_point(uiautomation::types::Point::new(x, y)) {
            return Ok(to_info(&el));
        }
        std::thread::sleep(std::time::Duration::from_millis(15));
    }
    let root = foreground_or_root(&automation)?;
    Ok(to_info(&root))
}

pub fn focused_element() -> Result<UiElementInfo> {
    let automation = UIAutomation::new().map_err(|e| CuError::Windows(e.to_string()))?;
    if let Ok(el) = automation.get_focused_element() {
        return Ok(to_info(&el));
    }
    let root = foreground_or_root(&automation)?;
    Ok(to_info(&root))
}

pub fn get_element_value(el: &uiautomation::core::UIElement) -> Option<String> {
    for prop in [
        uiautomation::types::UIProperty::ValueValue,
        uiautomation::types::UIProperty::LegacyIAccessibleValue,
    ] {
        if let Ok(val) = el.get_property_value(prop) {
            if val.is_string() {
                if let Ok(s) = val.get_string() {
                    let cleaned = crate::som::clean_ui_text(&s);
                    if !cleaned.is_empty() {
                        return Some(cleaned);
                    }
                }
            }
        }
    }
    if let Ok(val) = el.get_property_value(uiautomation::types::UIProperty::RangeValueValue) {
        let s = val.to_string();
        let cleaned = crate::som::clean_ui_text(&s);
        if !cleaned.is_empty() && cleaned != "EMPTY" && cleaned != "NULL" {
            return Some(cleaned);
        }
    }
    None
}

fn new_uia() -> Result<UIAutomation> {
    for _ in 0..3 {
        if let Ok(a) = UIAutomation::new() {
            return Ok(a);
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    UIAutomation::new().map_err(|e| CuError::Windows(e.to_string()))
}

fn foreground_or_root(automation: &UIAutomation) -> Result<uiautomation::core::UIElement> {
    for _ in 0..3 {
        if let Ok(fg) = crate::window::foreground_window() {
            if fg.hwnd != 0 {
                let handle: uiautomation::types::Handle = unsafe { std::mem::transmute(fg.hwnd as *mut std::ffi::c_void) };
                if let Ok(el) = automation.element_from_handle(handle) {
                    return Ok(el);
                }
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    automation.get_root_element().map_err(|e| CuError::Windows(e.to_string()))
}

pub fn find_by_name(name: &str) -> Result<Vec<UiElementInfo>> {
    let automation = new_uia()?;
    let cond = automation
        .create_property_condition(
            uiautomation::types::UIProperty::Name,
            uiautomation::variants::Variant::from(name.to_string()),
            None,
        )
        .map_err(|e| CuError::Windows(e.to_string()))?;
    let fg_root = foreground_or_root(&automation)?;
    if let Ok(els) = fg_root.find_all(uiautomation::types::TreeScope::Descendants, &cond) {
        if !els.is_empty() {
            return Ok(els.iter().map(to_info).collect());
        }
    }
    // Fuzzy & Substring fallback walk if exact match yields nothing
    if let Ok(walker) = automation.get_control_view_walker() {
        let mut fuzzy_matches = Vec::new();
        let mut visited = 0usize;
        find_element_fuzzy_recursive(&fg_root, &walker, name, &mut fuzzy_matches, 0, &mut visited);
        if !fuzzy_matches.is_empty() {
            return Ok(fuzzy_matches.iter().map(to_info).collect());
        }
    }

    if let Ok(root) = automation.get_root_element() {
        let els = root
            .find_all(uiautomation::types::TreeScope::Children, &cond)
            .unwrap_or_default();
        return Ok(els.iter().map(to_info).collect());
    }
    Ok(Vec::new())
}

fn find_element_fuzzy_recursive(
    el: &uiautomation::core::UIElement,
    walker: &uiautomation::core::UITreeWalker,
    query: &str,
    out: &mut Vec<uiautomation::core::UIElement>,
    depth: usize,
    visited: &mut usize,
) {
    if depth > 10 || out.len() >= 25 || *visited >= 200 {
        return;
    }
    *visited += 1;
    let name = el.get_name().unwrap_or_default().to_lowercase();
    let auto_id = el.get_automation_id().unwrap_or_default().to_lowercase();
    let q = query.to_lowercase();
    if (name.contains(&q) || auto_id.contains(&q)) && (!name.is_empty() || !auto_id.is_empty()) {
        out.push(el.clone());
    }
    if let Ok(mut child) = walker.get_first_child(el) {
        loop {
            if *visited >= 200 || out.len() >= 25 {
                break;
            }
            find_element_fuzzy_recursive(&child, walker, query, out, depth + 1, visited);
            match walker.get_next_sibling(&child) {
                Ok(next) => child = next,
                Err(_) => break,
            }
        }
    }
}

pub fn find_by_automation_id(id: &str) -> Result<Vec<UiElementInfo>> {
    let automation = UIAutomation::new().map_err(|e| CuError::Windows(e.to_string()))?;
    let cond = automation
        .create_property_condition(
            uiautomation::types::UIProperty::AutomationId,
            uiautomation::variants::Variant::from(id.to_string()),
            None,
        )
        .map_err(|e| CuError::Windows(e.to_string()))?;
    let fg_root = foreground_or_root(&automation)?;
    if let Ok(els) = fg_root.find_all(uiautomation::types::TreeScope::Descendants, &cond) {
        if !els.is_empty() {
            return Ok(els.iter().map(to_info).collect());
        }
    }
    let root = automation.get_root_element().map_err(|e| CuError::Windows(e.to_string()))?;
    let els = root
        .find_all(uiautomation::types::TreeScope::Children, &cond)
        .or_else(|_| fg_root.find_all(uiautomation::types::TreeScope::Descendants, &cond))
        .unwrap_or_default();
    Ok(els.iter().map(to_info).collect())
}

pub fn click_by_name(name: &str) -> Result<UiElementInfo> {
    let automation = UIAutomation::new().map_err(|e| CuError::Windows(e.to_string()))?;
    let cond = automation
        .create_property_condition(
            uiautomation::types::UIProperty::Name,
            uiautomation::variants::Variant::from(name.to_string()),
            None,
        )
        .map_err(|e| CuError::Windows(e.to_string()))?;
    let fg_root = foreground_or_root(&automation)?;
    let el = if let Ok(found) = fg_root.find_first(uiautomation::types::TreeScope::Descendants, &cond) {
        found
    } else {
        // Try fuzzy / substring match in foreground window
        if let Ok(walker) = automation.get_control_view_walker() {
            let mut matches = Vec::new();
            let mut visited = 0usize;
            find_element_fuzzy_recursive(&fg_root, &walker, name, &mut matches, 0, &mut visited);
            if let Some(first) = matches.into_iter().next() {
                first
            } else {
                let root = automation.get_root_element().map_err(|e| CuError::Windows(e.to_string()))?;
                root.find_first(uiautomation::types::TreeScope::Descendants, &cond)
                    .map_err(|e| CuError::Windows(e.to_string()))?
            }
        } else {
            let root = automation.get_root_element().map_err(|e| CuError::Windows(e.to_string()))?;
            root.find_first(uiautomation::types::TreeScope::Descendants, &cond)
                .map_err(|e| CuError::Windows(e.to_string()))?
        }
    };
    let click_res = el.click();
    let is_ok = click_res.is_ok()
        || click_res
            .as_ref()
            .err()
            .map(|e| e.to_string().contains("completed successfully"))
            .unwrap_or(false);
    if !is_ok {
        // Fallback to physical coordinate click on element center if programmatic InvokePattern fails
        if let Ok(r) = el.get_bounding_rectangle() {
            let cx = r.get_left() + (r.get_width() / 2);
            let cy = r.get_top() + (r.get_height() / 2);
            if cx > 0 && cy > 0 {
                let _ = crate::mouse::click(
                    crate::types::MouseButton::Left,
                    Some(crate::types::Point { x: cx, y: cy }),
                );
            }
        }
    }
    Ok(to_info(&el))
}

/// Extracts all visible text strings, labels, and control values from a window (or foreground window if hwnd == 0).
pub fn read_window_text(hwnd: isize) -> Result<Vec<String>> {
    let automation = new_uia()?;
    let root = if hwnd == 0 {
        foreground_or_root(&automation)?
    } else {
        let handle: uiautomation::types::Handle = unsafe { std::mem::transmute(hwnd as *mut std::ffi::c_void) };
        automation.element_from_handle(handle).map_err(|e| CuError::Windows(e.to_string()))?
    };
    let walker = automation.get_control_view_walker().map_err(|e| CuError::Windows(e.to_string()))?;
    let mut lines = Vec::new();
    let mut visited = 0usize;
    collect_text_recursive(&root, &walker, &mut lines, 0, &mut visited);
    Ok(lines)
}

fn collect_text_recursive(
    el: &uiautomation::core::UIElement,
    walker: &uiautomation::core::UITreeWalker,
    out: &mut Vec<String>,
    depth: usize,
    visited: &mut usize,
) {
    if depth > 10 || out.len() >= 200 || *visited >= 350 {
        return;
    }
    *visited += 1;
    if el.is_offscreen().unwrap_or(false) {
        return;
    }
    let name = crate::som::clean_ui_text(&el.get_name().unwrap_or_default());
    if !name.is_empty() {
        if out.last() != Some(&name) {
            out.push(name.clone());
        }
    }
    if let Ok(ct) = el.get_control_type() {
        if matches!(
            ct,
            uiautomation::controls::ControlType::Edit
                | uiautomation::controls::ControlType::ComboBox
                | uiautomation::controls::ControlType::Document
                | uiautomation::controls::ControlType::Slider
                | uiautomation::controls::ControlType::Spinner
        ) {
            if let Some(val) = get_element_value(el) {
                if out.last() != Some(&val) && val != name {
                    out.push(val);
                }
            }
        }
    }

    if let Ok(mut child) = walker.get_first_child(el) {
        loop {
            if *visited >= 350 || out.len() >= 200 {
                break;
            }
            collect_text_recursive(&child, walker, out, depth + 1, visited);
            match walker.get_next_sibling(&child) {
                Ok(n) => child = n,
                Err(_) => break,
            }
        }
    }
}

pub fn get_children(hwnd: isize) -> Result<Vec<UiElementInfo>> {
    let automation = UIAutomation::new().map_err(|e| CuError::Windows(e.to_string()))?;
    let el = if hwnd == 0 {
        automation.get_root_element().map_err(|e| CuError::Windows(e.to_string()))?
    } else {
        let handle: uiautomation::types::Handle = unsafe { std::mem::transmute(hwnd as *mut std::ffi::c_void) };
        automation
            .element_from_handle(handle)
            .map_err(|e| CuError::Windows(e.to_string()))?
    };
    let walker = automation.get_control_view_walker().map_err(|e| CuError::Windows(e.to_string()))?;
    let mut out = Vec::new();
    if let Ok(mut child) = walker.get_first_child(&el) {
        loop {
            out.push(to_info(&child));
            match walker.get_next_sibling(&child) {
                Ok(n) => child = n,
                Err(_) => break,
            }
        }
    }
    Ok(out)
}
