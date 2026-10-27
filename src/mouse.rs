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
