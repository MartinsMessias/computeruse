use computeruse::*;

fn main() {
    let (w, h) = ComputerUse::screen_size().unwrap();
    println!("screen: {}x{}", w, h);

    let shot = ComputerUse::screenshot_full(None).unwrap();
    println!("screenshot: {}x{} ({} bytes b64)", shot.width, shot.height, shot.png_base64.len());

    let p = ComputerUse::cursor_position().unwrap();
    println!("cursor: ({}, {})", p.x, p.y);

    ComputerUse::clipboard_set("hello from computeruse").unwrap();
    println!("clipboard: {}", ComputerUse::clipboard_get().unwrap());

    let focused = ComputerUse::uia_focused_element().unwrap();
    println!("focused: {} ({})", focused.name, focused.control_type);
}
