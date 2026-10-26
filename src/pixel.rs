use crate::error::{CuError, Result};
use crate::types::Point;
use windows::Win32::Graphics::Gdi::{GetDC, GetPixel, ReleaseDC};

pub fn get_pixel(p: Point) -> Result<(u8, u8, u8)> {
    let _ = crate::desktop::attach_thread_to_input_desktop();
    unsafe {
        let hdc = GetDC(None);
        if hdc.0.is_null() {
            return Err(CuError::Windows("GetDC failed".into()));
        }
        let color = GetPixel(hdc, p.x, p.y);
        let _ = ReleaseDC(None, hdc);
        if color.0 == 0xFFFFFFFF {
            return Err(CuError::Windows("GetPixel failed".into()));
        }
        let c = color.0;
        let r = (c & 0xFF) as u8;
        let g = ((c >> 8) & 0xFF) as u8;
        let b = ((c >> 16) & 0xFF) as u8;
        Ok((r, g, b))
    }
}
