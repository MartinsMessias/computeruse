use crate::error::{CuError, Result};
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::Foundation::HWND;
use windows::core::PCWSTR;

pub fn launch(target: &str, params: Option<&str>) -> Result<()> {
    unsafe {
        let op: Vec<u16> = "open\0".encode_utf16().collect();
        let file: Vec<u16> = target.encode_utf16().chain(std::iter::once(0)).collect();
        let prm: Vec<u16> = params.unwrap_or("").encode_utf16().chain(std::iter::once(0)).collect();
        let r = ShellExecuteW(
            Some(HWND(std::ptr::null_mut())),
            PCWSTR(op.as_ptr()),
            PCWSTR(file.as_ptr()),
            PCWSTR(prm.as_ptr()),
            PCWSTR(std::ptr::null()),
            windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL,
        );
        // ShellExecute returns HINSTANCE > 32 on success
        if (r.0 as usize) <= 32 {
            return Err(CuError::Windows(format!("ShellExecute failed: {}", r.0 as usize)));
        }
        Ok(())
    }
}
