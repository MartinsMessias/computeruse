use crate::error::{CuError, Result};
use crate::types::{Rect, UiElementInfo};
use uiautomation::core::UIAutomation;
use uiautomation::controls::ControlType;

fn to_info(el: &uiautomation::core::UIElement) -> UiElementInfo {
    let name = el.get_name().unwrap_or_default();
    let automation_id = el.get_automation_id().unwrap_or_default();
    let control_type = format!("{:?}", el.get_control_type().unwrap_or(ControlType::Custom));
    let class_name = String::new();
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
    let el = automation
        .element_from_point(uiautomation::types::Point::new(x, y))
        .map_err(|e| CuError::Windows(e.to_string()))?;
    Ok(to_info(&el))
}

pub fn focused_element() -> Result<UiElementInfo> {
    let automation = UIAutomation::new().map_err(|e| CuError::Windows(e.to_string()))?;
    let el = automation.get_focused_element().map_err(|e| CuError::Windows(e.to_string()))?;
    Ok(to_info(&el))
}

pub fn find_by_name(name: &str) -> Result<Vec<UiElementInfo>> {
    let automation = UIAutomation::new().map_err(|e| CuError::Windows(e.to_string()))?;
    let root = automation.get_root_element().map_err(|e| CuError::Windows(e.to_string()))?;
    let cond = automation
        .create_property_condition(
            uiautomation::types::UIProperty::Name,
            uiautomation::variants::Variant::from(name.to_string()),
            None,
        )
        .map_err(|e| CuError::Windows(e.to_string()))?;
    let els = root
        .find_all(uiautomation::types::TreeScope::Descendants, &cond)
        .map_err(|e| CuError::Windows(e.to_string()))?;
    Ok(els.iter().map(to_info).collect())
}

pub fn find_by_automation_id(id: &str) -> Result<Vec<UiElementInfo>> {
    let automation = UIAutomation::new().map_err(|e| CuError::Windows(e.to_string()))?;
    let root = automation.get_root_element().map_err(|e| CuError::Windows(e.to_string()))?;
    let cond = automation
        .create_property_condition(
            uiautomation::types::UIProperty::AutomationId,
            uiautomation::variants::Variant::from(id.to_string()),
            None,
        )
        .map_err(|e| CuError::Windows(e.to_string()))?;
    let els = root
        .find_all(uiautomation::types::TreeScope::Descendants, &cond)
        .map_err(|e| CuError::Windows(e.to_string()))?;
    Ok(els.iter().map(to_info).collect())
}

pub fn click_by_name(name: &str) -> Result<UiElementInfo> {
    let automation = UIAutomation::new().map_err(|e| CuError::Windows(e.to_string()))?;
    let root = automation.get_root_element().map_err(|e| CuError::Windows(e.to_string()))?;
    let cond = automation
        .create_property_condition(
            uiautomation::types::UIProperty::Name,
            uiautomation::variants::Variant::from(name.to_string()),
            None,
        )
        .map_err(|e| CuError::Windows(e.to_string()))?;
    let el = root
        .find_first(uiautomation::types::TreeScope::Descendants, &cond)
        .map_err(|e| CuError::Windows(e.to_string()))?;
    el.click().map_err(|e| CuError::Windows(e.to_string()))?;
    Ok(to_info(&el))
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
