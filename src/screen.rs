use crate::error::{CuError, Result};
use crate::types::{Monitor, Point, Rect, Screenshot};
use image::{ImageBuffer, Rgba};
use std::ptr;
use windows::Win32::Foundation::{LPARAM, RECT};
use windows::Win32::Graphics::Gdi::{
    BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, EnumDisplayMonitors, GetDC,
    GetDIBits, GetMonitorInfoW, ReleaseDC, SelectObject, BITMAPINFO,
    BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, HDC, HMONITOR, MONITORINFOEXW, SRCCOPY,
};
use windows::Win32::UI::HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI};
use windows::Win32::UI::WindowsAndMessaging::{
    GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN,
};

pub fn screen_size() -> Result<(u32, u32)> {
    unsafe {
        let w = GetSystemMetrics(SM_CXSCREEN);
        let h = GetSystemMetrics(SM_CYSCREEN);
        Ok((w as u32, h as u32))
    }
}

pub fn list_monitors() -> Result<Vec<Monitor>> {
    unsafe {
        let mut monitors: Vec<Monitor> = Vec::new();
        let ptr = &mut monitors as *mut Vec<Monitor>;
        unsafe extern "system" fn cb(hmon: HMONITOR, _hdc: HDC, _lprect: *mut RECT, lparam: LPARAM) -> windows::core::BOOL {
            let monitors = &mut *(lparam.0 as *mut Vec<Monitor>);
            let mut mi = MONITORINFOEXW::default();
            mi.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;
            if GetMonitorInfoW(hmon, &mut mi.monitorInfo).as_bool() {
                let r = mi.monitorInfo.rcMonitor;
                let wa = mi.monitorInfo.rcWork;
                let primary = (mi.monitorInfo.dwFlags & 1) != 0;
                let mut scale = 1.0f64;
                let mut dpi_x = 0u32;
                let mut dpi_y = 0u32;
                let _ = GetDpiForMonitor(hmon, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y);
                if dpi_x > 0 {
                    scale = dpi_x as f64 / 96.0;
                }
                let name = String::from_utf16_lossy(&mi.szDevice)
                    .trim_end_matches('\0')
                    .to_string();
                monitors.push(Monitor {
                    index: monitors.len() as u32,
                    name,
                    rect: Rect {
                        x: r.left,
                        y: r.top,
                        width: (r.right - r.left) as u32,
                        height: (r.bottom - r.top) as u32,
                    },
                    work_area: Rect {
                        x: wa.left,
                        y: wa.top,
                        width: (wa.right - wa.left) as u32,
                        height: (wa.bottom - wa.top) as u32,
                    },
                    is_primary: primary,
                    scale_factor: scale,
                });
            }
            windows::core::BOOL(1)
        }
        let _ = EnumDisplayMonitors(None, None, Some(cb), LPARAM(ptr as isize));
        Ok(monitors)
    }
}

pub fn screenshot_full(monitor: Option<u32>) -> Result<Screenshot> {
    let monitors = list_monitors()?;
    let m = if let Some(idx) = monitor {
        monitors
            .iter()
            .find(|mm| mm.index == idx)
            .cloned()
            .ok_or_else(|| CuError::NotFound(format!("monitor {}", idx)))?
    } else {
        monitors
            .iter()
            .find(|mm| mm.is_primary)
            .cloned()
            .or_else(|| monitors.first().cloned())
            .ok_or_else(|| CuError::NotFound("no monitors".into()))?
    };
    screenshot_region(Rect {
        x: m.rect.x,
        y: m.rect.y,
        width: m.rect.width,
        height: m.rect.height,
    }, Some(m.index))
}

pub fn screenshot_region(region: Rect, monitor: Option<u32>) -> Result<Screenshot> {
    let _ = crate::desktop::attach_thread_to_input_desktop();
    unsafe {
        let hdc_screen = GetDC(None);
        if hdc_screen.is_invalid() {
            return Err(CuError::Windows("GetDC failed".into()));
        }
        let hdc_mem = CreateCompatibleDC(Some(hdc_screen));
        let hbmp = CreateCompatibleBitmap(hdc_screen, region.width as i32, region.height as i32);
        let old = SelectObject(hdc_mem, hbmp.into());
        let ok = BitBlt(
            hdc_mem,
            0,
            0,
            region.width as i32,
            region.height as i32,
            Some(hdc_screen),
            region.x,
            region.y,
            SRCCOPY,
        );
        // Deselect hbmp before GetDIBits or cleanup per Win32 GDI spec
        SelectObject(hdc_mem, old);
        if ok.is_err() {
            let _ = DeleteObject(hbmp.into()).as_bool();
            let _ = DeleteDC(hdc_mem).as_bool();
            ReleaseDC(None, hdc_screen);
            return Err(CuError::Windows(format!("BitBlt failed: {:?}", ok.err())));
        }

        let mut bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: region.width as i32,
                biHeight: -(region.height as i32),
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut buf: Vec<u8> = vec![0u8; (region.width * region.height * 4) as usize];
        let lines = GetDIBits(
            hdc_mem,
            hbmp,
            0,
            region.height,
            Some(buf.as_mut_ptr() as *mut _),
            &mut bmi,
            DIB_RGB_COLORS,
        );
        let _ = DeleteObject(hbmp.into()).as_bool();
        let _ = DeleteDC(hdc_mem).as_bool();
        ReleaseDC(None, hdc_screen);

        if lines == 0 {
            return Err(CuError::Windows("GetDIBits failed".into()));
        }

        // Convert BGRA -> RGBA
        let (chunks, _) = buf.as_chunks_mut::<4>();
        for chunk in chunks {
            let b = chunk[0];
            let r = chunk[2];
            chunk[0] = r;
            chunk[2] = b;
            chunk[3] = 255;
        }

        let img: ImageBuffer<Rgba<u8>, Vec<u8>> =
            ImageBuffer::from_raw(region.width, region.height, buf)
                .ok_or_else(|| CuError::Image("from_raw failed".into()))?;
        let mut png: Vec<u8> = Vec::new();
        image::DynamicImage::ImageRgba8(img)
            .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
            .map_err(|e| CuError::Image(e.to_string()))?;

        use base64::Engine;
        let b64 = base64::engine::general_purpose::STANDARD.encode(&png);

        Ok(Screenshot {
            width: region.width,
            height: region.height,
            png_base64: b64,
            monitor,
            region: Some(region),
        })
    }
}

pub fn screenshot_window(hwnd: isize) -> Result<Screenshot> {
    let w = crate::window::find_window_by_hwnd(hwnd)?;
    screenshot_region(w.rect, None)
}

pub fn screenshot_active_window() -> Result<Screenshot> {
    let w = crate::window::foreground_window()?;
    screenshot_region(w.rect, None)
}

pub fn screenshot_all_monitors() -> Result<Screenshot> {
    let monitors = list_monitors()?;
    if monitors.is_empty() {
        return Err(CuError::NotFound("no monitors".into()));
    }
    let mut min_x = i32::MAX;
    let mut min_y = i32::MAX;
    let mut max_x = i32::MIN;
    let mut max_y = i32::MIN;
    for m in &monitors {
        min_x = min_x.min(m.rect.x);
        min_y = min_y.min(m.rect.y);
        max_x = max_x.max(m.rect.x + m.rect.width as i32);
        max_y = max_y.max(m.rect.y + m.rect.height as i32);
    }
    let region = Rect {
        x: min_x,
        y: min_y,
        width: (max_x - min_x) as u32,
        height: (max_y - min_y) as u32,
    };
    screenshot_region(region, None)
}

pub fn cursor_position() -> Result<Point> {
    let _ = crate::desktop::attach_thread_to_input_desktop();
    use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;
    unsafe {
        let mut pt = windows::Win32::Foundation::POINT::default();
        GetCursorPos(&mut pt).map_err(|e| CuError::Windows(e.to_string()))?;
        Ok(Point { x: pt.x, y: pt.y })
    }
}

pub fn set_cursor_position(p: Point) -> Result<()> {
    let _ = crate::desktop::attach_thread_to_input_desktop();
    use windows::Win32::UI::WindowsAndMessaging::SetCursorPos;
    unsafe {
        SetCursorPos(p.x, p.y).map_err(|e| CuError::Windows(e.to_string()))?;
    }
    Ok(())
}

/// Aguarda de forma eficiente até que a região especificada da tela apresente alteração visual de pixels,
/// ou até que o timeout_ms seja atingido. Usa checksum visual FNV-1a direto sobre o buffer GDI (~1.5ms/tick)
/// sem incorrer em compressão PNG ou alocação Base64.
pub fn wait_for_screen_change(region: Option<Rect>, timeout_ms: u64, poll_interval_ms: u64) -> Result<bool> {
    let reg = match region {
        Some(r) => r,
        None => {
            let (w, h) = screen_size()?;
            Rect { x: 0, y: 0, width: w, height: h }
        }
    };

    let initial_hash = crate::pixel::region_visual_checksum(reg)?;
    let start = std::time::Instant::now();
    let interval = std::time::Duration::from_millis(poll_interval_ms.max(15));

    while start.elapsed().as_millis() < timeout_ms as u128 {
        std::thread::sleep(interval);
        let cur_hash = crate::pixel::region_visual_checksum(reg)?;
        if cur_hash != initial_hash {
            return Ok(true);
        }
    }
    Ok(false)
}

#[allow(dead_code)]
fn _use_ptr() { let _ = ptr::null::<i32>(); }
