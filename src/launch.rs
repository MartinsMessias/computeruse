use crate::error::{CuError, Result};
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::Foundation::HWND;
use windows::Win32::System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE};
use windows::core::PCWSTR;

pub fn launch(target: &str, params: Option<&str>) -> Result<()> {
    launch_with_dir(target, params, None)
}

pub fn launch_with_dir(target: &str, params: Option<&str>, working_dir: Option<&str>) -> Result<()> {
    let trimmed = target.trim();
    let is_uri_or_doc = trimmed.contains("://")
        || trimmed.starts_with("ms-")
        || trimmed.starts_with("mailto:")
        || (!trimmed.to_lowercase().ends_with(".exe")
            && !trimmed.to_lowercase().ends_with(".bat")
            && !trimmed.to_lowercase().ends_with(".cmd")
            && trimmed.contains('.'));

    // Fast & synchronous OS process creation for executables and Windows 11 AppExecutionAliases (mspaint, notepad, calc)
    if !is_uri_or_doc {
        let mut cmd = std::process::Command::new(trimmed);
        if let Some(p) = params.filter(|s| !s.trim().is_empty()) {
            for arg in p.split_whitespace() {
                cmd.arg(arg);
            }
        }
        if let Some(dir) = working_dir.filter(|d| !d.trim().is_empty()) {
            cmd.current_dir(dir);
        }
        if cmd.spawn().is_ok() {
            return Ok(());
        }
    }

    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE);
        let op: Vec<u16> = "open\0".encode_utf16().collect();
        let file: Vec<u16> = trimmed.encode_utf16().chain(std::iter::once(0)).collect();
        let prm: Vec<u16> = params.unwrap_or("").encode_utf16().chain(std::iter::once(0)).collect();
        let dir_buf: Option<Vec<u16>> = working_dir.map(|d| d.encode_utf16().chain(std::iter::once(0)).collect());
        let dir_ptr = match dir_buf {
            Some(ref b) => PCWSTR(b.as_ptr()),
            None => PCWSTR(std::ptr::null()),
        };
        let r = ShellExecuteW(
            Some(HWND(std::ptr::null_mut())),
            PCWSTR(op.as_ptr()),
            PCWSTR(file.as_ptr()),
            PCWSTR(prm.as_ptr()),
            dir_ptr,
            windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL,
        );
        // ShellExecute returns HINSTANCE > 32 on success
        if (r.0 as usize) <= 32 {
            return Err(CuError::Windows(format!("ShellExecute failed: {}", r.0 as usize)));
        }
        // Allow asynchronous COM/DDE Shell activation to complete before short-lived CLI processes exit
        std::thread::sleep(std::time::Duration::from_millis(120));
        Ok(())
    }
}
