use crate::error::{CuError, Result};
use crate::screen::{cursor_position, set_cursor_position};
use crate::types::{MouseButton, Point};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_MOUSE, MOUSEEVENTF_ABSOLUTE, MOUSEEVENTF_LEFTDOWN,
    MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP, MOUSEEVENTF_MOVE,
    MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP, MOUSEEVENTF_VIRTUALDESK, MOUSEEVENTF_WHEEL,
    MOUSEINPUT,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetSystemMetrics, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN,
};

fn send_mouse(input: MOUSEINPUT) -> Result<()> {
    let _ = crate::desktop::attach_thread_to_input_desktop();
    unsafe {
        let inp = INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 { mi: input },
        };
        let sent = SendInput(&[inp], std::mem::size_of::<INPUT>() as i32);
        if sent == 0 {
            let last_err = windows::Win32::Foundation::GetLastError();
            return Err(CuError::Windows(format!("SendInput failed: {:?}", last_err)));
        }
    }
    Ok(())
}

fn button_flags(btn: MouseButton, down: bool) -> (u32, u32) {
    match (btn, down) {
        (MouseButton::Left, true) => (MOUSEEVENTF_LEFTDOWN.0, 0),
        (MouseButton::Left, false) => (MOUSEEVENTF_LEFTUP.0, 0),
        (MouseButton::Right, true) => (MOUSEEVENTF_RIGHTDOWN.0, 0),
        (MouseButton::Right, false) => (MOUSEEVENTF_RIGHTUP.0, 0),
        (MouseButton::Middle, true) => (MOUSEEVENTF_MIDDLEDOWN.0, 0),
        (MouseButton::Middle, false) => (MOUSEEVENTF_MIDDLEUP.0, 0),
    }
}

/// Maps physical screen coordinates (including negative multi-monitor offsets) to [0..65535] across the entire virtual desktop.
pub fn normalized_virtual_coords(p: Point) -> (i32, i32) {
    crate::desktop::enable_per_monitor_dpi_v2();
    unsafe {
        let vx = GetSystemMetrics(SM_XVIRTUALSCREEN);
        let vy = GetSystemMetrics(SM_YVIRTUALSCREEN);
        let vw = GetSystemMetrics(SM_CXVIRTUALSCREEN);
        let vh = GetSystemMetrics(SM_CYVIRTUALSCREEN);
        if vw > 1 && vh > 1 {
            let nx = (((p.x - vx) as f64) * 65535.0 / (vw as f64 - 1.0)).round() as i32;
            let ny = (((p.y - vy) as f64) * 65535.0 / (vh as f64 - 1.0)).round() as i32;
            return (nx.clamp(0, 65535), ny.clamp(0, 65535));
        }
    }
    if let Ok((w, h)) = crate::screen::screen_size() {
        if w > 1 && h > 1 {
            let nx = ((p.x as f64) * 65535.0 / (w as f64 - 1.0)).round() as i32;
            let ny = ((p.y as f64) * 65535.0 / (h as f64 - 1.0)).round() as i32;
            return (nx.clamp(0, 65535), ny.clamp(0, 65535));
        }
    }
    (0, 0)
}

pub fn move_to(p: Point) -> Result<()> {
    set_cursor_position(p)?;
    let (norm_x, norm_y) = normalized_virtual_coords(p);
    let _ = send_mouse(MOUSEINPUT {
        dx: norm_x,
        dy: norm_y,
        mouseData: 0,
        dwFlags: MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_VIRTUALDESK,
        time: 0,
        dwExtraInfo: 0,
    });
    Ok(())
}

pub fn move_relative(dx: i32, dy: i32) -> Result<Point> {
    let cur = cursor_position()?;
    let np = Point { x: cur.x + dx, y: cur.y + dy };
    move_to(np)?;
    Ok(np)
}

pub fn click(btn: MouseButton, p: Option<Point>) -> Result<()> {
    let target = if let Some(pt) = p {
        move_to(pt)?;
        std::thread::sleep(std::time::Duration::from_millis(35));
        pt
    } else {
        cursor_position().unwrap_or(Point { x: 0, y: 0 })
    };
    let (norm_x, norm_y) = normalized_virtual_coords(target);
    let (down_flag, _) = button_flags(btn, true);
    let (up_flag, _) = button_flags(btn, false);
    send_mouse(MOUSEINPUT {
        dx: norm_x,
        dy: norm_y,
        mouseData: 0,
        dwFlags: windows::Win32::UI::Input::KeyboardAndMouse::MOUSE_EVENT_FLAGS(down_flag) | MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_MOVE | MOUSEEVENTF_VIRTUALDESK,
        time: 0,
        dwExtraInfo: 0,
    })?;
    std::thread::sleep(std::time::Duration::from_millis(45));
    send_mouse(MOUSEINPUT {
        dx: norm_x,
        dy: norm_y,
        mouseData: 0,
        dwFlags: windows::Win32::UI::Input::KeyboardAndMouse::MOUSE_EVENT_FLAGS(up_flag) | MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_MOVE | MOUSEEVENTF_VIRTUALDESK,
        time: 0,
        dwExtraInfo: 0,
    })?;
    std::thread::sleep(std::time::Duration::from_millis(20));
    Ok(())
}

pub fn double_click(btn: MouseButton, p: Option<Point>) -> Result<()> {
    click(btn, p)?;
    std::thread::sleep(std::time::Duration::from_millis(80));
    click(btn, None)?;
    Ok(())
}

/// Triple-click at an optional point (or current cursor position) to select an entire line or paragraph like a human.
pub fn triple_click(btn: MouseButton, p: Option<Point>) -> Result<()> {
    click(btn, p)?;
    std::thread::sleep(std::time::Duration::from_millis(70));
    click(btn, None)?;
    std::thread::sleep(std::time::Duration::from_millis(70));
    click(btn, None)?;
    Ok(())
}

/// Smoothly glide the cursor from its current position to `to` using a cubic ease-in-out human trajectory over `duration_ms`.
/// Essential for triggering `:hover` menus, tooltips, and canvas scrubbers that ignore teleported cursors.
pub fn move_smooth(to: Point, duration_ms: u64) -> Result<()> {
    let from = cursor_position().unwrap_or(to);
    let dx = (to.x - from.x) as f64;
    let dy = (to.y - from.y) as f64;
    let dist = (dx * dx + dy * dy).sqrt();
    if dist < 2.0 || duration_ms == 0 {
        return move_to(to);
    }
    let steps = ((duration_ms / 8).clamp(6, 60)) as usize;
    let step_sleep = (duration_ms / (steps as u64)).max(2);
    for i in 1..=steps {
        let t = (i as f64) / (steps as f64);
        // Cubic ease-in-out: 3t^2 - 2t^3
        let ease = t * t * (3.0 - 2.0 * t);
        let x = (from.x as f64 + dx * ease).round() as i32;
        let y = (from.y as f64 + dy * ease).round() as i32;
        move_to(Point { x, y })?;
        std::thread::sleep(std::time::Duration::from_millis(step_sleep));
    }
    Ok(())
}

/// Long click (hold mouse button down for `duration_ms` before release).
/// Useful for opening press-and-hold menus, color pickers, or drag handles.
pub fn long_click(btn: MouseButton, p: Option<Point>, duration_ms: u64) -> Result<()> {
    if let Some(pt) = p {
        move_to(pt)?;
        std::thread::sleep(std::time::Duration::from_millis(30));
    }
    mouse_down(btn)?;
    std::thread::sleep(std::time::Duration::from_millis(duration_ms.max(50)));
    mouse_up(btn)?;
    Ok(())
}

/// Click while holding keyboard modifiers (e.g. `["ctrl"]`, `["shift"]`, `["alt"]`).
pub fn click_modified(btn: MouseButton, p: Option<Point>, modifiers: &[String]) -> Result<()> {
    if modifiers.is_empty() {
        return click(btn, p);
    }
    if let Some(pt) = p {
        move_to(pt)?;
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
    crate::keyboard::with_modifiers(modifiers, || click(btn, None))
}

/// Drag while holding keyboard modifiers (e.g. `Shift+Drag` for proportional resize/straight line, `Alt+Drag` to duplicate).
pub fn drag_modified(from: Point, to: Point, btn: MouseButton, modifiers: &[String]) -> Result<()> {
    if modifiers.is_empty() {
        return drag(from, to, btn);
    }
    move_to(from)?;
    std::thread::sleep(std::time::Duration::from_millis(30));
    crate::keyboard::with_modifiers(modifiers, || drag(from, to, btn))
}

/// Scroll at a specific `(x, y)` target (moving the cursor there first if provided) with optional keyboard modifiers (e.g. `Ctrl+Scroll` to zoom).
pub fn scroll_at(amount: i32, horizontal: bool, p: Option<Point>, modifiers: &[String]) -> Result<()> {
    if let Some(pt) = p {
        move_to(pt)?;
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
    if modifiers.is_empty() {
        scroll(amount, horizontal)
    } else {
        crate::keyboard::with_modifiers(modifiers, || scroll(amount, horizontal))
    }
}

pub fn mouse_down(btn: MouseButton) -> Result<()> {
    let cur = cursor_position().unwrap_or(Point { x: 0, y: 0 });
    let (norm_x, norm_y) = normalized_virtual_coords(cur);
    let (flag, _) = button_flags(btn, true);
    send_mouse(MOUSEINPUT {
        dx: norm_x,
        dy: norm_y,
        mouseData: 0,
        dwFlags: windows::Win32::UI::Input::KeyboardAndMouse::MOUSE_EVENT_FLAGS(flag) | MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_MOVE | MOUSEEVENTF_VIRTUALDESK,
        time: 0,
        dwExtraInfo: 0,
    })
}

pub fn mouse_up(btn: MouseButton) -> Result<()> {
    let cur = cursor_position().unwrap_or(Point { x: 0, y: 0 });
    let (norm_x, norm_y) = normalized_virtual_coords(cur);
    let (flag, _) = button_flags(btn, false);
    send_mouse(MOUSEINPUT {
        dx: norm_x,
        dy: norm_y,
        mouseData: 0,
        dwFlags: windows::Win32::UI::Input::KeyboardAndMouse::MOUSE_EVENT_FLAGS(flag) | MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_MOVE | MOUSEEVENTF_VIRTUALDESK,
        time: 0,
        dwExtraInfo: 0,
    })
}

pub fn drag(from: Point, to: Point, btn: MouseButton) -> Result<()> {
    move_to(from)?;
    std::thread::sleep(std::time::Duration::from_millis(40));
    mouse_down(btn)?;
    std::thread::sleep(std::time::Duration::from_millis(60));
    // move in steps
    let steps = 25;
    for i in 1..=steps {
        let t = i as f64 / steps as f64;
        let x = from.x as f64 + (to.x - from.x) as f64 * t;
        let y = from.y as f64 + (to.y - from.y) as f64 * t;
        move_to(Point { x: x.round() as i32, y: y.round() as i32 })?;
        std::thread::sleep(std::time::Duration::from_millis(8));
    }
    std::thread::sleep(std::time::Duration::from_millis(40));
    mouse_up(btn)?;
    Ok(())
}

pub fn drag_fast(from: Point, to: Point, btn: MouseButton) -> Result<()> {
    move_to(from)?;
    std::thread::sleep(std::time::Duration::from_millis(25));
    mouse_down(btn)?;
    std::thread::sleep(std::time::Duration::from_millis(30));
    let steps = 15;
    for i in 1..=steps {
        let t = i as f64 / steps as f64;
        let x = from.x as f64 + (to.x - from.x) as f64 * t;
        let y = from.y as f64 + (to.y - from.y) as f64 * t;
        move_to(Point { x: x.round() as i32, y: y.round() as i32 })?;
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    std::thread::sleep(std::time::Duration::from_millis(25));
    mouse_up(btn)?;
    std::thread::sleep(std::time::Duration::from_millis(10));
    Ok(())
}

/// Executes a polyline by drawing each segment as an independent drag with an explicit pause between segments so Paint never drops edges or merges clicks into double-clicks.
pub fn stroke_continuous(pts: &[Point], btn: MouseButton) -> Result<()> {
    if pts.len() < 2 {
        return Ok(());
    }
    for i in 0..pts.len() - 1 {
        let p1 = pts[i];
        let p2 = pts[i + 1];
        if p1.x == p2.x && p1.y == p2.y {
            continue;
        }
        move_to(p1)?;
        std::thread::sleep(std::time::Duration::from_millis(35));
        mouse_down(btn)?;
        std::thread::sleep(std::time::Duration::from_millis(45));

        let dx = (p2.x - p1.x) as f64;
        let dy = (p2.y - p1.y) as f64;
        let dist = (dx * dx + dy * dy).sqrt();
        let steps = ((dist / 10.0).ceil() as usize).clamp(8, 25);

        for s in 1..=steps {
            let t = s as f64 / steps as f64;
            let x = (p1.x as f64 + dx * t).round() as i32;
            let y = (p1.y as f64 + dy * t).round() as i32;
            move_to(Point { x, y })?;
            std::thread::sleep(std::time::Duration::from_millis(5));
        }

        std::thread::sleep(std::time::Duration::from_millis(30));
        mouse_up(btn)?;
        // Pause between segments so Paint commits the stroke and never triggers a double-click at the shared vertex
        std::thread::sleep(std::time::Duration::from_millis(35));
    }
    Ok(())
}

/// Renders an array of isolated pointillist / stipple dots with zero connecting lines.
/// Repeats the first dot once so Ribbon-to-Canvas focus handoff never drops a dot.
pub fn stipple_points(pts: &[Point], dot_radius: i32, btn: MouseButton) -> Result<()> {
    if pts.is_empty() {
        return Ok(());
    }
    let r = dot_radius.clamp(2, 8);
    let r_inner = (r / 2).max(1);

    // Focus-activation tap on the first point so Ribbon focus is returned to the canvas
    let first = pts[0];
    move_to(first)?;
    std::thread::sleep(std::time::Duration::from_millis(20));
    mouse_down(btn)?;
    std::thread::sleep(std::time::Duration::from_millis(22));
    move_to(Point { x: first.x + 2, y: first.y })?;
    std::thread::sleep(std::time::Duration::from_millis(12));
    mouse_up(btn)?;
    std::thread::sleep(std::time::Duration::from_millis(25));

    for &p in pts {
        move_to(Point { x: p.x - r, y: p.y })?;
        std::thread::sleep(std::time::Duration::from_millis(14));
        mouse_down(btn)?;
        std::thread::sleep(std::time::Duration::from_millis(22));

        // Outer ring + inner cross of the dot so DirectInk renders a bold, saturated, solid round dot
        let micro_pts = [
            Point { x: p.x, y: p.y - r },
            Point { x: p.x + r, y: p.y },
            Point { x: p.x, y: p.y + r },
            Point { x: p.x - r, y: p.y },
            Point { x: p.x - r_inner, y: p.y - r_inner },
            Point { x: p.x + r_inner, y: p.y + r_inner },
            Point { x: p.x + r_inner, y: p.y - r_inner },
            Point { x: p.x - r_inner, y: p.y + r_inner },
            p,
        ];
        for mp in micro_pts {
            move_to(mp)?;
            std::thread::sleep(std::time::Duration::from_millis(3));
        }

        std::thread::sleep(std::time::Duration::from_millis(14));
        mouse_up(btn)?;
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
    Ok(())
}

pub fn scroll(amount: i32, horizontal: bool) -> Result<()> {
    // amount in "clicks" * 120
    let delta = amount * 120;
    let flags = if horizontal {
        // MOUSEEVENTF_HWHEEL = 0x01000
        windows::Win32::UI::Input::KeyboardAndMouse::MOUSE_EVENT_FLAGS(0x01000)
    } else {
        MOUSEEVENTF_WHEEL
    };
    send_mouse(MOUSEINPUT {
        dx: 0,
        dy: 0,
        mouseData: delta as u32,
        dwFlags: flags,
        time: 0,
        dwExtraInfo: 0,
    })
}

pub fn click_at(btn: MouseButton, p: Point) -> Result<()> {
    click(btn, Some(p))
}
