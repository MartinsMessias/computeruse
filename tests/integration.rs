use computeruse::*;

#[test]
fn screen_size_nonzero() {
    let (w, h) = ComputerUse::screen_size().unwrap();
    assert!(w > 0 && h > 0);
}

#[test]
fn monitors_nonempty() {
    let m = ComputerUse::list_monitors().unwrap();
    assert!(!m.is_empty());
    assert!(m.iter().any(|x| x.is_primary));
}

#[test]
fn cursor_roundtrip() {
    let p = ComputerUse::cursor_position().unwrap();
    ComputerUse::set_cursor_position(p).unwrap();
    let p2 = ComputerUse::cursor_position().unwrap();
    assert!((p.x - p2.x).abs() <= 2 && (p.y - p2.y).abs() <= 2);
}

#[test]
fn clipboard_roundtrip() {
    let msg = "cu-test-123";
    ComputerUse::clipboard_set(msg).unwrap();
    let got = ComputerUse::clipboard_get().unwrap();
    assert_eq!(got, msg);
}

#[test]
fn screenshot_nonempty() {
    let s = ComputerUse::screenshot_full(None).unwrap();
    assert!(s.width > 0 && s.height > 0);
    assert!(!s.png_base64.is_empty());
}

#[test]
fn windows_list() {
    let w = ComputerUse::list_windows(true).unwrap();
    assert!(!w.is_empty());
}

#[test]
fn screenshot_window() {
    let fg = ComputerUse::foreground_window().unwrap();
    let s = ComputerUse::screenshot_window(fg.hwnd).unwrap();
    assert!(s.width > 0 && s.height > 0);
}

#[test]
fn screenshot_all_monitors() {
    let s = ComputerUse::screenshot_all_monitors().unwrap();
    assert!(s.width > 0 && s.height > 0);
}

#[test]
fn list_processes_nonempty() {
    let p = ComputerUse::list_processes().unwrap();
    assert!(!p.is_empty());
    assert!(p.iter().any(|x| x.exe_name.to_lowercase().contains("cu") || x.exe_name.to_lowercase().contains("opencode") || x.exe_name.to_lowercase().contains("explorer")));
}

#[test]
fn uia_element_at_point() {
    let e = ComputerUse::uia_element_at_point(100, 100).unwrap();
    assert!(!e.control_type.is_empty());
}

#[test]
fn system_info_valid() {
    let s = ComputerUse::system_info().unwrap();
    assert!(s.total_memory_mb > 0);
    assert!(s.cpu_cores > 0);
    assert!(s.uptime_seconds > 0);
}

#[test]
fn system_drives_nonempty() {
    let d = ComputerUse::system_drives().unwrap();
    assert!(!d.is_empty());
    assert!(d.iter().any(|drive| drive.total_bytes > 0));
}

#[test]
fn special_folders_resolve() {
    let desk = ComputerUse::system_special_folder("desktop").unwrap();
    assert!(!desk.is_empty());
    let temp = ComputerUse::system_special_folder("temp").unwrap();
    assert!(!temp.is_empty());
}

#[test]
fn audio_volume_query() {
    // Audio device may not be present in all CI environments, so test gracefully
    if let Ok(vol) = ComputerUse::audio_get_volume() {
        assert!(vol <= 100);
    }
}
