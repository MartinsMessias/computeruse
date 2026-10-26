use crate::error::{CuError, Result};
use crate::screen::{cursor_position, set_cursor_position};
use crate::types::{MouseButton, Point};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_MOUSE, MOUSEEVENTF_LEFTDOWN,
    MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP, MOUSEEVENTF_RIGHTDOWN,
    MOUSEEVENTF_RIGHTUP, MOUSEEVENTF_WHEEL, MOUSEINPUT,
};

fn send_mouse(input: MOUSEINPUT) -> Result<()> {
    unsafe {
        let inp = INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 { mi: input },
        };
        let sent = SendInput(&[inp], std::mem::size_of::<INPUT>() as i32);
        if sent == 0 {
            return Err(CuError::Windows("SendInput failed".into()));
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

pub fn move_to(p: Point) -> Result<()> {
    set_cursor_position(p)
}

pub fn move_relative(dx: i32, dy: i32) -> Result<Point> {
    let cur = cursor_position()?;
    let np = Point { x: cur.x + dx, y: cur.y + dy };
    set_cursor_position(np)?;
    Ok(np)
}

pub fn click(btn: MouseButton, p: Option<Point>) -> Result<()> {
    if let Some(pt) = p {
        set_cursor_position(pt)?;
        std::thread::sleep(std::time::Duration::from_millis(30));
    }
    let (down_flag, _) = button_flags(btn, true);
    let (up_flag, _) = button_flags(btn, false);
    send_mouse(MOUSEINPUT {
        dx: 0,
        dy: 0,
        mouseData: 0,
        dwFlags: windows::Win32::UI::Input::KeyboardAndMouse::MOUSE_EVENT_FLAGS(down_flag),
        time: 0,
        dwExtraInfo: 0,
    })?;
    std::thread::sleep(std::time::Duration::from_millis(40));
    send_mouse(MOUSEINPUT {
        dx: 0,
        dy: 0,
        mouseData: 0,
        dwFlags: windows::Win32::UI::Input::KeyboardAndMouse::MOUSE_EVENT_FLAGS(up_flag),
        time: 0,
        dwExtraInfo: 0,
    })?;
    Ok(())
}

pub fn double_click(btn: MouseButton, p: Option<Point>) -> Result<()> {
    click(btn, p)?;
    std::thread::sleep(std::time::Duration::from_millis(80));
    click(btn, None)?;
    Ok(())
}

pub fn mouse_down(btn: MouseButton) -> Result<()> {
    let (flag, _) = button_flags(btn, true);
    send_mouse(MOUSEINPUT {
        dx: 0, dy: 0, mouseData: 0,
        dwFlags: windows::Win32::UI::Input::KeyboardAndMouse::MOUSE_EVENT_FLAGS(flag),
        time: 0, dwExtraInfo: 0,
    })
}

pub fn mouse_up(btn: MouseButton) -> Result<()> {
    let (flag, _) = button_flags(btn, false);
    send_mouse(MOUSEINPUT {
        dx: 0, dy: 0, mouseData: 0,
        dwFlags: windows::Win32::UI::Input::KeyboardAndMouse::MOUSE_EVENT_FLAGS(flag),
        time: 0, dwExtraInfo: 0,
    })
}

pub fn drag(from: Point, to: Point, btn: MouseButton) -> Result<()> {
    set_cursor_position(from)?;
    std::thread::sleep(std::time::Duration::from_millis(40));
    mouse_down(btn)?;
    std::thread::sleep(std::time::Duration::from_millis(60));
    // move in steps
    let steps = 20;
    for i in 1..=steps {
        let t = i as f64 / steps as f64;
        let x = from.x as f64 + (to.x - from.x) as f64 * t;
        let y = from.y as f64 + (to.y - from.y) as f64 * t;
        set_cursor_position(Point { x: x as i32, y: y as i32 })?;
        std::thread::sleep(std::time::Duration::from_millis(8));
    }
    std::thread::sleep(std::time::Duration::from_millis(40));
    mouse_up(btn)?;
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
