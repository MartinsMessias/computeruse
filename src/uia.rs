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
    if let Ok(val) = el.get_property_value(uiautomation::types::UIProperty::ValueValue) {
        if let Ok(s) = val.get_string() {
            let trimmed = s.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

fn foreground_or_root(automation: &UIAutomation) -> Result<uiautomation::core::UIElement> {
    if let Ok(fg) = crate::window::foreground_window() {
        if fg.hwnd != 0 {
            let handle: uiautomation::types::Handle = unsafe { std::mem::transmute(fg.hwnd as *mut std::ffi::c_void) };
            if let Ok(el) = automation.element_from_handle(handle) {
                return Ok(el);
            }
        }
    }
    automation.get_root_element().map_err(|e| CuError::Windows(e.to_string()))
}

pub fn find_by_name(name: &str) -> Result<Vec<UiElementInfo>> {
    let automation = UIAutomation::new().map_err(|e| CuError::Windows(e.to_string()))?;
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
    let root = automation.get_root_element().map_err(|e| CuError::Windows(e.to_string()))?;
    let els = root
        .find_all(uiautomation::types::TreeScope::Descendants, &cond)
        .map_err(|e| CuError::Windows(e.to_string()))?;
    Ok(els.iter().map(to_info).collect())
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
        .find_all(uiautomation::types::TreeScope::Descendants, &cond)
        .map_err(|e| CuError::Windows(e.to_string()))?;
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
        let root = automation.get_root_element().map_err(|e| CuError::Windows(e.to_string()))?;
        root.find_first(uiautomation::types::TreeScope::Descendants, &cond)
            .map_err(|e| CuError::Windows(e.to_string()))?
    };
    el.click().map_err(|e| CuError::Windows(e.to_string()))?;
    Ok(to_info(&el))
}

/// Extracts all visible text strings, labels, and control values from a window (or foreground window if hwnd == 0).
pub fn read_window_text(hwnd: isize) -> Result<Vec<String>> {
    let automation = UIAutomation::new().map_err(|e| CuError::Windows(e.to_string()))?;
    let root = if hwnd == 0 {
        foreground_or_root(&automation)?
    } else {
        let handle: uiautomation::types::Handle = unsafe { std::mem::transmute(hwnd as *mut std::ffi::c_void) };
        automation.element_from_handle(handle).map_err(|e| CuError::Windows(e.to_string()))?
    };
    let walker = automation.get_control_view_walker().map_err(|e| CuError::Windows(e.to_string()))?;
    let mut lines = Vec::new();
    collect_text_recursive(&root, &walker, &mut lines, 0);
    Ok(lines)
}

fn collect_text_recursive(
    el: &uiautomation::core::UIElement,
    walker: &uiautomation::core::UITreeWalker,
    out: &mut Vec<String>,
    depth: usize,
) {
    if depth > 5 || out.len() >= 120 {
        return;
    }
    let name = el.get_name().unwrap_or_default();
    let trimmed = name.trim();
    if !trimmed.is_empty() {
        let s = trimmed.to_string();
        if out.last() != Some(&s) {
            out.push(s);
        }
    }

    if let Ok(mut child) = walker.get_first_child(el) {
        loop {
            collect_text_recursive(&child, walker, out, depth + 1);
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
