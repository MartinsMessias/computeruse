use crate::error::{CuError, Result};
use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
use windows::Win32::Media::Audio::{eMultimedia, eRender, IMMDeviceEnumerator, MMDeviceEnumerator};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_ALL, COINIT_MULTITHREADED,
};

struct ComGuard {
    should_uninit: bool,
}
impl ComGuard {
    fn new() -> Self {
        let should_uninit = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED).is_ok() };
        ComGuard { should_uninit }
    }
}
impl Drop for ComGuard {
    fn drop(&mut self) {
        if self.should_uninit {
            unsafe {
                CoUninitialize();
            }
        }
    }
}

/// Emit a native Win32 hardware/system tone without spawning external processes.
pub fn beep(freq_hz: u32, duration_ms: u32) -> Result<()> {
    windows::core::link!("kernel32.dll" "system" fn Beep(dwfreq: u32, dwduration: u32) -> windows::core::BOOL);
    let freq = freq_hz.clamp(37, 32767);
    let dur = duration_ms.clamp(1, 10000);
    unsafe {
        let _ = Beep(freq, dur);
    }
    Ok(())
}

fn get_endpoint_volume() -> Result<IAudioEndpointVolume> {
    unsafe {
        let enumerator: IMMDeviceEnumerator =
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)
                .map_err(|e| CuError::Windows(format!("CoCreateInstance MMDeviceEnumerator failed: {}", e)))?;
        let default_device = enumerator
            .GetDefaultAudioEndpoint(eRender, eMultimedia)
            .map_err(|e| CuError::Windows(format!("GetDefaultAudioEndpoint failed: {}", e)))?;
        let endpoint_volume: IAudioEndpointVolume = default_device
            .Activate(CLSCTX_ALL, None)
            .map_err(|e| CuError::Windows(format!("Activate IAudioEndpointVolume failed: {}", e)))?;
        Ok(endpoint_volume)
    }
}

/// Get the master volume level as a percentage (0 to 100).
pub fn get_volume() -> Result<u32> {
    let _com = ComGuard::new();
    let ep = get_endpoint_volume()?;
    unsafe {
        let level = ep.GetMasterVolumeLevelScalar()
            .map_err(|e| CuError::Windows(format!("GetMasterVolumeLevelScalar failed: {}", e)))?;
        Ok((level * 100.0).round() as u32)
    }
}

/// Set the master volume level as a percentage (0 to 100).
pub fn set_volume(percent: u32) -> Result<()> {
    let _com = ComGuard::new();
    let ep = get_endpoint_volume()?;
    let scalar = (percent.min(100) as f32) / 100.0;
    unsafe {
        ep.SetMasterVolumeLevelScalar(scalar, std::ptr::null())
            .map_err(|e| CuError::Windows(format!("SetMasterVolumeLevelScalar failed: {}", e)))?;
        Ok(())
    }
}

/// Get the master mute status (true = muted).
pub fn get_mute() -> Result<bool> {
    let _com = ComGuard::new();
    let ep = get_endpoint_volume()?;
    unsafe {
        let mute = ep.GetMute()
            .map_err(|e| CuError::Windows(format!("GetMute failed: {}", e)))?;
        Ok(mute.as_bool())
    }
}

/// Set the master mute status.
pub fn set_mute(muted: bool) -> Result<()> {
    let _com = ComGuard::new();
    let ep = get_endpoint_volume()?;
    unsafe {
        ep.SetMute(muted, std::ptr::null())
            .map_err(|e| CuError::Windows(format!("SetMute failed: {}", e)))?;
        Ok(())
    }
}

/// Toggle the master mute status. Returns the new mute state.
pub fn toggle_mute() -> Result<bool> {
    let current = get_mute()?;
    set_mute(!current)?;
    Ok(!current)
}
