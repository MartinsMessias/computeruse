use crate::error::{CuError, Result};
use crate::types::{DriveInfo, SystemInfo};
use std::path::Path;
use windows::core::PCWSTR;
use windows::Win32::Storage::FileSystem::{GetDiskFreeSpaceExW, GetLogicalDriveStringsW};
use windows::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};
use windows::Win32::System::SystemInformation::{
    GlobalMemoryStatusEx, GetTickCount64, MEMORYSTATUSEX,
};

/// Resolves a file extension or URI protocol scheme to its associated executable or default application.
/// Examples: `.pdf`, `.png`, `.docx`, `https`, `mailto`.
pub fn resolve_file_association(ext_or_protocol: &str) -> Result<String> {
    use windows::Win32::UI::Shell::{AssocQueryStringW, ASSOCF_NONE, ASSOCSTR_EXECUTABLE};
    let key = if ext_or_protocol.starts_with('.') || ext_or_protocol.contains(':') {
        ext_or_protocol.trim_end_matches(':').to_string()
    } else {
        format!(".{}", ext_or_protocol)
    };
    let wide_key: Vec<u16> = key.encode_utf16().chain(std::iter::once(0)).collect();
    let mut buf = vec![0u16; 512];
    let mut cch = buf.len() as u32;

    unsafe {
        let hr = AssocQueryStringW(
            ASSOCF_NONE,
            ASSOCSTR_EXECUTABLE,
            PCWSTR(wide_key.as_ptr()),
            PCWSTR::null(),
            Some(windows::core::PWSTR(buf.as_mut_ptr())),
            &mut cch,
        );
        if hr.is_err() {
            return Err(CuError::NotFound(format!(
                "No associated application for '{}'",
                ext_or_protocol
            )));
        }
    }
    let len = cch.saturating_sub(1) as usize;
    Ok(String::from_utf16_lossy(&buf[..len]).to_string())
}

/// Plays a system standard sound alert (`default`, `error`, `warning`, `question`, `asterisk`).
pub fn play_system_sound(sound_type: &str) -> Result<()> {
    windows::core::link!("user32.dll" "system" fn MessageBeep(utype: u32) -> windows::core::BOOL);
    let u_type = match sound_type.to_lowercase().as_str() {
        "error" | "hand" | "stop" => 0x00000010,
        "question" => 0x00000020,
        "warning" | "exclamation" => 0x00000030,
        "asterisk" | "info" => 0x00000040,
        _ => 0xFFFFFFFF, // default simple beep
    };
    unsafe {
        let _ = MessageBeep(u_type);
    }
    Ok(())
}

/// Query standard Windows environment variables or system-wide settings.
pub fn get_env_var(name: &str) -> Result<String> {
    std::env::var(name).map_err(|_| CuError::NotFound(format!("Environment variable '{}' not set", name)))
}

windows::core::link!("user32.dll" "system" fn LockWorkStation() -> windows::core::BOOL);

/// Retrieve system health, memory, CPU, uptime, and battery power information.
pub fn get_system_info() -> Result<SystemInfo> {
    unsafe {
        // Memory
        let mut mem = MEMORYSTATUSEX::default();
        mem.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;
        GlobalMemoryStatusEx(&mut mem)
            .map_err(|e| CuError::Windows(format!("GlobalMemoryStatusEx failed: {}", e)))?;

        let total_memory_mb = mem.ullTotalPhys / (1024 * 1024);
        let available_memory_mb = mem.ullAvailPhys / (1024 * 1024);
        let memory_load_percent = mem.dwMemoryLoad;

        // Uptime
        let uptime_ms = GetTickCount64();
        let uptime_seconds = uptime_ms / 1000;

        // CPU cores
        let cpu_cores = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);

        // Power status
        let mut power = SYSTEM_POWER_STATUS::default();
        let (battery_percent, is_charging) = if GetSystemPowerStatus(&mut power).is_ok() {
            let pct = if power.BatteryLifePercent <= 100 {
                Some(power.BatteryLifePercent)
            } else {
                None // 255 means unknown or no battery (desktop)
            };
            let charging = match power.ACLineStatus {
                1 => Some(true),
                0 => Some(false),
                _ => None,
            };
            (pct, charging)
        } else {
            (None, None)
        };

        Ok(SystemInfo {
            os_name: "Windows".into(),
            cpu_cores,
            total_memory_mb,
            available_memory_mb,
            memory_load_percent,
            uptime_seconds,
            battery_percent,
            is_charging,
        })
    }
}

/// Retrieve logical disk drives and their capacity and free space.
pub fn get_drives() -> Result<Vec<DriveInfo>> {
    unsafe {
        let mut buffer = [0u16; 512];
        let len = GetLogicalDriveStringsW(Some(&mut buffer));
        if len == 0 {
            return Err(CuError::Windows("GetLogicalDriveStringsW failed".into()));
        }

        let mut drives = Vec::new();
        let slice = &buffer[..len as usize];
        for drive_chars in slice.split(|&c| c == 0) {
            if drive_chars.is_empty() {
                continue;
            }
            let drive_str = String::from_utf16_lossy(drive_chars);
            let wide: Vec<u16> = drive_str.encode_utf16().chain(std::iter::once(0)).collect();

            let mut free_available = 0u64;
            let mut total_bytes = 0u64;
            let mut total_free = 0u64;

            if GetDiskFreeSpaceExW(
                PCWSTR(wide.as_ptr()),
                Some(&mut free_available),
                Some(&mut total_bytes),
                Some(&mut total_free),
            ).is_ok() {
                drives.push(DriveInfo {
                    name: drive_str.trim_end_matches('\\').to_string(),
                    total_bytes,
                    free_bytes: total_free,
                    available_bytes: free_available,
                });
            }
        }
        Ok(drives)
    }
}

/// Lock the Windows workstation immediately.
pub fn lock() -> Result<()> {
    unsafe {
        let ok = LockWorkStation();
        if !ok.as_bool() {
            return Err(CuError::Windows("LockWorkStation failed".into()));
        }
        Ok(())
    }
}

/// Get standard Windows special directory path by name:
/// "desktop", "downloads", "documents", "appdata", "temp", "userprofile", "windows"
pub fn get_special_folder(name: &str) -> Result<String> {
    let lower = name.trim().to_lowercase();
    let path = match lower.as_str() {
        "temp" => std::env::temp_dir().to_string_lossy().to_string(),
        "userprofile" | "home" => std::env::var("USERPROFILE").map_err(|_| CuError::NotFound("USERPROFILE not set".into()))?,
        "appdata" => std::env::var("APPDATA").map_err(|_| CuError::NotFound("APPDATA not set".into()))?,
        "localappdata" => std::env::var("LOCALAPPDATA").map_err(|_| CuError::NotFound("LOCALAPPDATA not set".into()))?,
        "desktop" => {
            let user = std::env::var("USERPROFILE").map_err(|_| CuError::NotFound("USERPROFILE not set".into()))?;
            let p = Path::new(&user).join("Desktop");
            p.to_string_lossy().to_string()
        }
        "downloads" => {
            let user = std::env::var("USERPROFILE").map_err(|_| CuError::NotFound("USERPROFILE not set".into()))?;
            let p = Path::new(&user).join("Downloads");
            p.to_string_lossy().to_string()
        }
        "documents" => {
            let user = std::env::var("USERPROFILE").map_err(|_| CuError::NotFound("USERPROFILE not set".into()))?;
            let p = Path::new(&user).join("Documents");
            p.to_string_lossy().to_string()
        }
        "windows" | "systemroot" => std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".to_string()),
        _ => return Err(CuError::InvalidArg(format!("Unknown special folder '{}'. Supported: desktop, downloads, documents, appdata, localappdata, temp, userprofile, windows", name))),
    };
    Ok(path)
}
