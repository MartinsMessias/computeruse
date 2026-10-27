use crate::error::{CuError, Result};
use std::sync::atomic::{AtomicBool, Ordering};
use windows::Win32::System::StationsAndDesktops::{
    CloseDesktop, OpenInputDesktop, SetThreadDesktop, DESKTOP_ACCESS_FLAGS, DESKTOP_CONTROL_FLAGS,
};
use windows::Win32::UI::HiDpi::{
    SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
};

static DESKTOP_ATTACHED: AtomicBool = AtomicBool::new(false);
static DPI_INITIALIZED: AtomicBool = AtomicBool::new(false);

const DESKTOP_GENERIC_ACCESS: u32 = 0x01FF;

/// Enables Per-Monitor DPI Awareness V2 for the current process so all Win32 and UIA coordinates
/// map 1:1 to true physical pixels without Windows bitmap virtualization distortion.
pub fn enable_per_monitor_dpi_v2() {
    if !DPI_INITIALIZED.swap(true, Ordering::SeqCst) {
        unsafe {
            let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        }
    }
}

/// Attempt to attach the calling thread to the currently active input desktop (e.g. "Default").
/// This is critical when running from service or background processes where the thread
/// is not attached to the interactive desktop by default.
pub fn attach_thread_to_input_desktop() -> Result<()> {
    enable_per_monitor_dpi_v2();
    unsafe {
        let desk = OpenInputDesktop(
            DESKTOP_CONTROL_FLAGS(0),
            false,
            DESKTOP_ACCESS_FLAGS(DESKTOP_GENERIC_ACCESS),
        ).map_err(|e| CuError::Windows(format!("OpenInputDesktop failed: {}", e)))?;
        let res = SetThreadDesktop(desk);
        let _ = CloseDesktop(desk);
        if let Err(e) = res {
            return Err(CuError::Windows(format!("SetThreadDesktop failed: {}", e)));
        }
        DESKTOP_ATTACHED.store(true, Ordering::SeqCst);
        Ok(())
    }
}

/// Execute a function ensuring it runs in a thread attached to the active input desktop.
/// If the current thread can attach (or is already attached), runs directly.
/// If current thread fails due to ERROR_BUSY (it already created GUI objects/windows),
/// a dedicated worker thread is spawned to execute the work on the input desktop.
pub fn with_input_desktop<F, R>(f: F) -> Result<R>
where
    F: FnOnce() -> Result<R> + Send + 'static,
    R: Send + 'static,
{
    // Try attaching on current thread
    match attach_thread_to_input_desktop() {
        Ok(_) => f(),
        Err(e) => {
            let err_str = e.to_string();
            if err_str.contains("170") || err_str.contains("0x800700AA") || err_str.contains("in use") {
                // Spawn a clean thread without any window hooks
                let handle = std::thread::spawn(move || -> Result<R> {
                    let _ = attach_thread_to_input_desktop();
                    f()
                });
                handle
                    .join()
                    .map_err(|_| CuError::Internal("Thread panicked while executing desktop operation".into()))?
            } else {
                f()
            }
        }
    }
}
