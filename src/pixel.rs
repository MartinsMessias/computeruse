use crate::error::{CuError, Result};
use crate::types::{Point, Rect};
use windows::Win32::Graphics::Gdi::{
    BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDC, GetDIBits,
    ReleaseDC, SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, SRCCOPY,
};

/// Reads the true DWM-composed RGB pixel at `p` using a 1x1 GDI BitBlt + GetDIBits capture.
pub fn get_pixel(p: Point) -> Result<(u8, u8, u8)> {
    let buf = capture_raw_bgra(Rect {
        x: p.x,
        y: p.y,
        width: 1,
        height: 1,
    })?;
    Ok((buf[2], buf[1], buf[0]))
}

/// Captures raw BGRA bytes for a screen region using a single GDI BitBlt + GetDIBits call.
fn capture_raw_bgra(region: Rect) -> Result<Vec<u8>> {
    let _ = crate::desktop::attach_thread_to_input_desktop();
    let w = region.width.max(1);
    let h = region.height.max(1);
    unsafe {
        let hdc_screen = GetDC(None);
        if hdc_screen.is_invalid() {
            return Err(CuError::Windows("GetDC failed".into()));
        }
        let hdc_mem = CreateCompatibleDC(Some(hdc_screen));
        let hbmp = CreateCompatibleBitmap(hdc_screen, w as i32, h as i32);
        let old = SelectObject(hdc_mem, hbmp.into());
        let ok = BitBlt(
            hdc_mem,
            0,
            0,
            w as i32,
            h as i32,
            Some(hdc_screen),
            region.x,
            region.y,
            SRCCOPY,
        );
        // Deselect bitmap before calling GetDIBits per Win32 GDI specification
        SelectObject(hdc_mem, old);
        if ok.is_err() {
            let _ = DeleteObject(hbmp.into()).as_bool();
            let _ = DeleteDC(hdc_mem).as_bool();
            ReleaseDC(None, hdc_screen);
            return Err(CuError::Windows("BitBlt failed".into()));
        }
        let mut bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: w as i32,
                biHeight: -(h as i32),
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut buf = vec![0u8; (w * h * 4) as usize];
        let lines = GetDIBits(
            hdc_mem,
            hbmp,
            0,
            h,
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
        Ok(buf)
    }
}

/// Scans a rectangular screen region in <3ms for the first pixel matching `target_rgb` within `tolerance`.
pub fn find_color_in_region(
    region: Rect,
    target_rgb: (u8, u8, u8),
    tolerance: u8,
) -> Result<Option<Point>> {
    let w = region.width.max(1);
    let h = region.height.max(1);
    let buf = capture_raw_bgra(region)?;
    let (tr, tg, tb) = target_rgb;
    let tol = tolerance as i32;

    for y in 0..h {
        for x in 0..w {
            let idx = ((y * w + x) * 4) as usize;
            let b = buf[idx] as i32;
            let g = buf[idx + 1] as i32;
            let r = buf[idx + 2] as i32;
            if (r - tr as i32).abs() <= tol
                && (g - tg as i32).abs() <= tol
                && (b - tb as i32).abs() <= tol
            {
                return Ok(Some(Point {
                    x: region.x + x as i32,
                    y: region.y + y as i32,
                }));
            }
        }
    }
    Ok(None)
}

/// Counts the percentage (0.0 .. 1.0) of pixels in `region` matching `target_rgb` within `tolerance`.
pub fn count_color_ratio(
    region: Rect,
    target_rgb: (u8, u8, u8),
    tolerance: u8,
) -> Result<f64> {
    let w = region.width.max(1);
    let h = region.height.max(1);
    let total_pixels = (w * h) as f64;
    let buf = capture_raw_bgra(region)?;
    let (tr, tg, tb) = target_rgb;
    let tol = tolerance as i32;
    let mut matched = 0usize;

    for y in 0..h {
        for x in 0..w {
            let idx = ((y * w + x) * 4) as usize;
            let b = buf[idx] as i32;
            let g = buf[idx + 1] as i32;
            let r = buf[idx + 2] as i32;
            if (r - tr as i32).abs() <= tol
                && (g - tg as i32).abs() <= tol
                && (b - tb as i32).abs() <= tol
            {
                matched += 1;
            }
        }
    }
    Ok((matched as f64) / total_pixels)
}

/// Computes a fast 64-bit FNV-1a visual fingerprint of a screen region (strided every 4th pixel).
pub fn region_visual_checksum(region: Rect) -> Result<u64> {
    let buf = capture_raw_bgra(region)?;
    let mut hash: u64 = 0xcbf29ce484222325;
    for chunk in buf.chunks_exact(16) {
        hash ^= (chunk[0] as u64) | ((chunk[1] as u64) << 8) | ((chunk[2] as u64) << 16);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    Ok(hash)
}
