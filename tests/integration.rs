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

#[test]
fn som_perception_snapshot() {
    let mut env = AgentEnvironment::new();
    let obs = env.observe().unwrap();
    assert!(!obs.window_title.is_empty() || obs.window_hwnd == 0);
    assert!(!obs.compact_prompt.is_empty());
}

#[test]
fn dynamic_canvas_math_coordinates() {
    let canvas = DynamicCanvas::new(Rect {
        x: 100,
        y: 200,
        width: 1000,
        height: 500,
    });
    assert!((canvas.aspect_ratio - 2.0).abs() < 0.01);
    let center = canvas.pt(0.5, 0.5);
    assert_eq!(center.x, 600);
    assert_eq!(center.y, 450);

    let polar = canvas.polar_pt(0.5, 0.5, 0.2, 0.0);
    // (0.5 + 0.2/2.0) = 0.6 -> 100 + 600 = 700
    assert_eq!(polar.x, 700);
    assert_eq!(polar.y, 450);
}

#[test]
fn pillar1_som_categorized_truncation() {
    use computeruse::som::{InteractiveElement, SomSnapshot};
    let snap = SomSnapshot {
        hwnd: 1234,
        title: "TestApp".into(),
        window_rect: Rect { x: 0, y: 0, width: 800, height: 600 },
        elements: vec![
            InteractiveElement {
                mark_id: 1,
                name: "Submit".into(),
                control_type: "Button".into(),
                automation_id: "btnSubmit".into(),
                value: None,
                rect: Rect { x: 10, y: 10, width: 80, height: 30 },
                center: Point { x: 50, y: 25 },
                rel_center: (0.06, 0.04),
                is_enabled: true,
            },
            InteractiveElement {
                mark_id: 2,
                name: "Search Query".into(),
                control_type: "Edit".into(),
                automation_id: "txtSearch".into(),
                value: Some("rust".into()),
                rect: Rect { x: 100, y: 10, width: 200, height: 30 },
                center: Point { x: 200, y: 25 },
                rel_center: (0.25, 0.04),
                is_enabled: true,
            },
            InteractiveElement {
                mark_id: 3,
                name: "This is a very long passive status text that exceeds thirty five characters".into(),
                control_type: "Text".into(),
                automation_id: "lblStatus".into(),
                value: None,
                rect: Rect { x: 10, y: 500, width: 600, height: 20 },
                center: Point { x: 310, y: 510 },
                rel_center: (0.38, 0.85),
                is_enabled: true,
            },
        ],
    };
    let digest = snap.to_compact_prompt();
    assert!(digest.contains("Tools/Buttons (1): [01] \"Submit\" (50,25)"));
    assert!(digest.contains("Inputs/Controls (1): [02] \"Search Query\"=\"rust\" (200,25)"));
    assert!(digest.contains("Menus/Other (1): [03] \"This is a very long passive stat...\" (310,510)"));
}

#[test]
fn pillar2_flexible_batch_action_parsing() {
    let a1 = serde_json::json!({"action": "click_name", "name": "File"});
    let a2 = serde_json::json!({"type": "type", "text": "Hello World"});
    let a3 = serde_json::json!({"ClickMark": {"mark_id": 7}});
    let a4 = serde_json::json!({"type": "wait_for_change", "timeout_ms": 500});

    assert!(matches!(AgentAction::from_flexible_json(&a1).unwrap(), AgentAction::ClickName { ref name } if name == "File"));
    assert!(matches!(AgentAction::from_flexible_json(&a2).unwrap(), AgentAction::TypeText { ref text } if text == "Hello World"));
    assert!(matches!(AgentAction::from_flexible_json(&a3).unwrap(), AgentAction::ClickMark { mark_id: 7 }));
    assert!(matches!(AgentAction::from_flexible_json(&a4).unwrap(), AgentAction::WaitForChange { timeout_ms: 500 }));
}

#[test]
fn pillar3_per_monitor_dpi_v2_and_virtual_desktop() {
    computeruse::desktop::enable_per_monitor_dpi_v2();
    let attached = computeruse::desktop::attach_thread_to_input_desktop();
    assert!(attached.is_ok());
}

#[test]
fn pillar4_reactive_wait_for_change() {
    let mut env = AgentEnvironment::new().with_settle_delay(10);
    let (delta, obs, elapsed) = env.wait_for_change(60).unwrap();
    assert!(elapsed >= 20);
    assert!(!obs.compact_prompt.is_empty());
    assert!(!delta.summary.is_empty());
}

#[test]
fn keyboard_punctuation_vk_resolution() {
    for key in ["+", "plus", "-", "minus", "=", "/", ".", ",", ";", "'", "[", "]", "\\", "`", "apps"] {
        assert!(computeruse::keyboard::resolve_vk(key).is_some(), "Failed to resolve VK for '{}'", key);
    }
}

#[test]
fn uia_read_window_text_nonempty() {
    let lines = ComputerUse::uia_read_window_text(0).unwrap();
    assert!(!lines.is_empty());
}

#[test]
fn som_annotated_image_generation() {
    let fg = ComputerUse::foreground_window().unwrap();
    let som = ComputerUse::som_snapshot(fg.hwnd).unwrap();
    let png_bytes = som.generate_annotated_image().unwrap();
    assert!(png_bytes.len() > 100);
}

#[test]
fn pixel_visual_checksum_and_color_search() {
    let reg = Rect { x: 50, y: 50, width: 64, height: 64 };
    let h = ComputerUse::region_visual_checksum(reg).unwrap();
    assert!(h != 0);
    let sample = ComputerUse::get_pixel(Point { x: 60, y: 60 }).unwrap();
    let found = ComputerUse::find_color_in_region(reg, sample, 5).unwrap();
    assert!(found.is_some());
}
