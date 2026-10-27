use crate::error::{CuError, Result};
use crate::types::{Rect, WindowInfo};
use std::mem;
use windows::Win32::Foundation::{HWND, LPARAM, RECT};
// (OpenProcess import removed)
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetClassNameW, GetForegroundWindow, GetWindowRect, GetWindowThreadProcessId, GetWindowTextLengthW,
    GetWindowTextW, IsIconic, IsWindowVisible, IsZoomed, SetForegroundWindow, ShowWindow,
    SW_MAXIMIZE, SW_MINIMIZE, SW_RESTORE, SetWindowPos, HWND_TOPMOST, HWND_NOTOPMOST,
    SWP_NOMOVE, SWP_NOSIZE, SetLayeredWindowAttributes, LWA_ALPHA, GetWindowLongPtrW, SetWindowLongPtrW,
    GWL_EXSTYLE, WS_EX_LAYERED,
};

pub fn list_windows(visible_only: bool) -> Result<Vec<WindowInfo>> {
    let _ = crate::desktop::attach_thread_to_input_desktop();
    unsafe {
        let mut out: Vec<WindowInfo> = Vec::new();
        let ptr = &mut out as *mut Vec<WindowInfo>;
        unsafe extern "system" fn cb(hwnd: HWND, lparam: LPARAM) -> windows::core::BOOL {
            let out = &mut *(lparam.0 as *mut Vec<WindowInfo>);
            if !IsWindowVisible(hwnd).as_bool() {
                // still include if not visible_only filter handled by caller; here include all but mark
            }
            let len = GetWindowTextLengthW(hwnd);
            let mut title_buf = vec![0u16; (len + 1) as usize];
            let got = GetWindowTextW(hwnd, &mut title_buf);
            let title = String::from_utf16_lossy(&title_buf[..got as usize]).to_string();
            // class name
            let mut cls_buf = vec![0u16; 256];
            let clen = GetClassNameW(hwnd, &mut cls_buf);
            let class_name = String::from_utf16_lossy(&cls_buf[..clen as usize]).to_string();

            let mut pid = 0u32;
            GetWindowThreadProcessId(hwnd, Some(&mut pid));

            let mut r = RECT::default();
            let _ = GetWindowRect(hwnd, &mut r);

            let visible = IsWindowVisible(hwnd).as_bool();
            let minimized = IsIconic(hwnd).as_bool();
            let maximized = IsZoomed(hwnd).as_bool();

            out.push(WindowInfo {
                hwnd: hwnd.0 as isize,
                title,
                class_name,
                pid,
                rect: Rect {
                    x: r.left,
                    y: r.top,
                    width: (r.right - r.left) as u32,
                    height: (r.bottom - r.top) as u32,
                },
                is_visible: visible,
                is_minimized: minimized,
                is_maximized: maximized,
            });
            windows::core::BOOL(1)
        }
        let _ = EnumWindows(Some(cb), LPARAM(ptr as isize));
        if visible_only {
            out.retain(|w| w.is_visible && !w.title.is_empty());
        }
        Ok(out)
    }
}

pub fn find_window(title_contains: &str) -> Result<WindowInfo> {
    let wins = list_windows(false)?;
    let lower = title_contains.to_lowercase();
    wins.into_iter()
        .find(|w| w.title.to_lowercase().contains(&lower))
        .ok_or_else(|| CuError::NotFound(format!("window '{}'", title_contains)))
}

pub fn find_window_by_hwnd(hwnd: isize) -> Result<WindowInfo> {
    let wins = list_windows(false)?;
    wins.into_iter()
        .find(|w| w.hwnd == hwnd)
        .ok_or_else(|| CuError::NotFound(format!("hwnd {}", hwnd)))
}

pub fn foreground_window() -> Result<WindowInfo> {
    let _ = crate::desktop::attach_thread_to_input_desktop();
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() {
            return Err(CuError::NotFound("no foreground window".into()));
        }
        // Try enumerated list first; fall back to building from hwnd.
        if let Ok(w) = find_window_by_hwnd(hwnd.0 as isize) {
            return Ok(w);
        }
        // Build directly
        let mut title_buf = vec![0u16; 512];
        let got = GetWindowTextW(hwnd, &mut title_buf);
        let title = String::from_utf16_lossy(&title_buf[..got as usize]).to_string();
        let mut cls_buf = vec![0u16; 256];
        let clen = GetClassNameW(hwnd, &mut cls_buf);
        let class_name = String::from_utf16_lossy(&cls_buf[..clen as usize]).to_string();
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        let mut r = RECT::default();
        let _ = GetWindowRect(hwnd, &mut r);
        Ok(WindowInfo {
            hwnd: hwnd.0 as isize,
            title,
            class_name,
            pid,
            rect: Rect {
                x: r.left,
                y: r.top,
                width: (r.right - r.left) as u32,
                height: (r.bottom - r.top) as u32,
            },
            is_visible: true,
            is_minimized: IsIconic(hwnd).as_bool(),
            is_maximized: IsZoomed(hwnd).as_bool(),
        })
    }
}

pub fn focus_window(hwnd: isize) -> Result<()> {
    unsafe {
        let h = HWND(hwnd as *mut _);
        // restore if minimized
        if IsIconic(h).as_bool() {
            let _ = ShowWindow(h, SW_RESTORE).as_bool();
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        // AttachThreadInput trick to allow focus
        use windows::Win32::UI::WindowsAndMessaging::AllowSetForegroundWindow;
        let _ = AllowSetForegroundWindow(0xFFFFFFFF); // ASFW_ANY
        let _ = SetWindowPos(h, Some(HWND_TOPMOST), 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE);
        let _ = SetWindowPos(h, Some(HWND_NOTOPMOST), 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE);
        let ok = SetForegroundWindow(h);
        if !ok.as_bool() {
            // fallback: try again after alt key tap
            // simulate alt press/release
            crate::keyboard::key_press("alt").ok();
            std::thread::sleep(std::time::Duration::from_millis(50));
            let _ = SetForegroundWindow(h);
        }
        Ok(())
    }
}

pub fn focus_by_title(title_contains: &str) -> Result<WindowInfo> {
    let w = find_window(title_contains)?;
    focus_window(w.hwnd)?;
    std::thread::sleep(std::time::Duration::from_millis(150));
    Ok(w)
}

pub fn minimize_window(hwnd: isize) -> Result<()> {
    unsafe {
        let h = HWND(hwnd as *mut _);
        let _ = ShowWindow(h, SW_MINIMIZE).as_bool();
    }
    Ok(())
}

pub fn maximize_window(hwnd: isize) -> Result<()> {
    unsafe {
        let h = HWND(hwnd as *mut _);
        let _ = ShowWindow(h, SW_MAXIMIZE).as_bool();
    }
    Ok(())
}

pub fn restore_window(hwnd: isize) -> Result<()> {
    unsafe {
        let h = HWND(hwnd as *mut _);
        let _ = ShowWindow(h, SW_RESTORE).as_bool();
    }
    Ok(())
}

pub fn close_window(hwnd: isize) -> Result<()> {
    use windows::Win32::UI::WindowsAndMessaging::PostMessageW;
    use windows::Win32::Foundation::WPARAM;
    unsafe {
        let h = HWND(hwnd as *mut _);
        // WM_CLOSE = 0x0010
        let _ = PostMessageW(Some(h), 0x0010, WPARAM(0), LPARAM(0));
    }
    Ok(())
}

pub fn move_resize_window(hwnd: isize, rect: Rect) -> Result<()> {
    use windows::Win32::UI::WindowsAndMessaging::MoveWindow;
    unsafe {
        let h = HWND(hwnd as *mut _);
        let _ = MoveWindow(h, rect.x, rect.y, rect.width as i32, rect.height as i32, true);
    }
    Ok(())
}

pub fn wait_for_window(title_contains: &str, timeout_ms: u64) -> Result<WindowInfo> {
    let start = std::time::Instant::now();
    loop {
        if let Ok(w) = find_window(title_contains) {
            return Ok(w);
        }
        if start.elapsed().as_millis() as u64 >= timeout_ms {
            return Err(CuError::NotFound(format!("window '{}' not found within {}ms", title_contains, timeout_ms)));
        }
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
}

pub fn set_topmost(hwnd: isize, topmost: bool) -> Result<()> {
    unsafe {
        let h = HWND(hwnd as *mut _);
        let insert_after = if topmost { HWND_TOPMOST } else { HWND_NOTOPMOST };
        SetWindowPos(h, Some(insert_after), 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE)
            .map_err(|e| CuError::Windows(format!("SetWindowPos topmost failed: {}", e)))?;
        Ok(())
    }
}

pub fn set_opacity(hwnd: isize, alpha: u8) -> Result<()> {
    use windows::Win32::Foundation::COLORREF;
    unsafe {
        let h = HWND(hwnd as *mut _);
        let ex_style = GetWindowLongPtrW(h, GWL_EXSTYLE);
        SetWindowLongPtrW(h, GWL_EXSTYLE, ex_style | (WS_EX_LAYERED.0 as isize));
        SetLayeredWindowAttributes(h, COLORREF(0), alpha, LWA_ALPHA)
            .map_err(|e| CuError::Windows(format!("SetLayeredWindowAttributes failed: {}", e)))?;
        Ok(())
    }
}

// suppress unused
#[allow(dead_code)]
fn _use_mem() { let _ = mem::size_of::<i32>(); }
