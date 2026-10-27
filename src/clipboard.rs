use crate::error::{CuError, Result};
use windows::Win32::Foundation::{HGLOBAL, HWND};
use windows::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, GetClipboardData, OpenClipboard, SetClipboardData,
};
use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};

const CF_UNICODETEXT: u32 = 13;

struct ClipboardGuard;

impl ClipboardGuard {
    fn open() -> Result<Self> {
        let _ = crate::desktop::attach_thread_to_input_desktop();
        unsafe {
            let mut last_err = None;
            for _ in 0..8 {
                match OpenClipboard(Some(HWND(std::ptr::null_mut()))) {
                    Ok(()) => return Ok(Self),
                    Err(e) => {
                        last_err = Some(e);
                        std::thread::sleep(std::time::Duration::from_millis(10));
                    }
                }
            }
            Err(CuError::Clipboard(format!(
                "OpenClipboard failed after retries: {:?}",
                last_err
            )))
        }
    }
}

impl Drop for ClipboardGuard {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseClipboard();
        }
    }
}

pub fn get_text() -> Result<String> {
    let _guard = ClipboardGuard::open()?;
    unsafe {
        let handle = GetClipboardData(CF_UNICODETEXT)
            .map_err(|_| CuError::Clipboard("no text data".into()))?;
        let ptr = GlobalLock(HGLOBAL(handle.0));
        if ptr.is_null() {
            return Err(CuError::Clipboard("GlobalLock failed".into()));
        }
        let mut len = 0usize;
        let wptr = ptr as *const u16;
        while *wptr.add(len) != 0 {
            len += 1;
            if len > 1_000_000 {
                break;
            }
        }
        let slice = std::slice::from_raw_parts(wptr, len);
        let text = String::from_utf16_lossy(slice).to_string();
        let _ = GlobalUnlock(HGLOBAL(handle.0));
        Ok(text)
    }
}

pub fn set_text(text: &str) -> Result<()> {
    let _guard = ClipboardGuard::open()?;
    unsafe {
        EmptyClipboard()?;
        let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
        let bytes = wide.len() * 2;
        let hmem: HGLOBAL = GlobalAlloc(GMEM_MOVEABLE, bytes)?;
        let ptr = GlobalLock(hmem);
        if ptr.is_null() {
            return Err(CuError::Clipboard("GlobalLock failed".into()));
        }
        std::ptr::copy_nonoverlapping(wide.as_ptr(), ptr as *mut u16, wide.len());
        let _ = GlobalUnlock(hmem);
        let h: windows::Win32::Foundation::HANDLE = windows::Win32::Foundation::HANDLE(hmem.0);
        SetClipboardData(CF_UNICODETEXT, Some(h))
            .map_err(|_| CuError::Clipboard("SetClipboardData failed".into()))?;
        Ok(())
    }
}
