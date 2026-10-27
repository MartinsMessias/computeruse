use std::f64::consts::PI;
use crate::error::Result;
use crate::mouse;
use crate::types::{MouseButton, Point, Rect};
use crate::uia;
use crate::window;

/// A dynamic drawing canvas mapped to an application window or UI element.
/// Coordinates are fully normalized (0.0 .. 1.0) relative to the detected surface,
/// immune to window movement, DPI scaling, and screen repositioning.
#[derive(Debug, Clone)]
pub struct DynamicCanvas {
    pub origin_x: i32,
    pub origin_y: i32,
    pub width: u32,
    pub height: u32,
    pub aspect_ratio: f64,
}

impl DynamicCanvas {
    /// Creates a canvas from an explicit bounding rectangle.
    pub fn new(rect: Rect) -> Self {
        let width = rect.width.max(10);
        let height = rect.height.max(10);
        let aspect_ratio = (width as f64) / (height as f64);
        Self {
            origin_x: rect.x,
            origin_y: rect.y,
            width,
            height,
            aspect_ratio,
        }
    }

    /// Automatically discovers the dynamic drawing surface of a window by HWND.
    /// First attempts to find a UI element with AutomationId "image" or name containing "Canvas".
    /// If not found, falls back to probing the window center, or using an inner window rect.
    pub fn find_in_window(hwnd: isize) -> Result<Self> {
        let win = window::find_window_by_hwnd(hwnd)?;
        let r = win.rect;

        // Strategy 1: Look for AutomationId "image" (Standard in Paint and several drawing tools)
        if let Ok(els) = uia::find_by_automation_id("image") {
            for el in els {
                if el.rect.width > 200 && el.rect.height > 200 {
                    // Check if it's within the window bounds
                    if el.rect.x >= r.x && el.rect.y >= r.y {
                        return Ok(Self::new(el.rect));
                    }
                }
            }
        }

        // Strategy 2: Probe the center of the window via UIA element_at_point
        let cx = r.x + (r.width as i32 / 2);
        let cy = r.y + (r.height as i32 / 2);
        if let Ok(el) = uia::element_at_point(cx, cy) {
            if el.rect.width > 200 && el.rect.height > 200 {
                return Ok(Self::new(el.rect));
            }
        }

        // Strategy 3: Window fallback with ribbon offset (typical 150px top toolbar)
        let ribbon_offset = (r.height as f64 * 0.18).min(180.0) as u32;
        let canvas_rect = Rect {
            x: r.x + 20,
            y: r.y + ribbon_offset as i32,
            width: r.width.saturating_sub(40).max(100),
            height: r.height.saturating_sub(ribbon_offset + 30).max(100),
        };
        Ok(Self::new(canvas_rect))
    }

    /// Converts normalized (rx, ry) in 0.0..1.0 to absolute screen Point.
    pub fn pt(&self, rx: f64, ry: f64) -> Point {
        let x = self.origin_x + ((self.width as f64) * rx.clamp(0.0, 1.0)).round() as i32;
        let y = self.origin_y + ((self.height as f64) * ry.clamp(0.0, 1.0)).round() as i32;
        Point { x, y }
    }

    /// Converts polar coordinates relative to a normalized center (cx, cy) with aspect ratio correction.
    /// `radius` is relative to canvas height (0.0 .. 1.0).
    pub fn polar_pt(&self, cx: f64, cy: f64, radius: f64, angle_rad: f64) -> Point {
        let rx = cx + (radius / self.aspect_ratio) * angle_rad.cos();
        let ry = cy + radius * angle_rad.sin();
        self.pt(rx, ry)
    }

    /// Draws a straight line between two normalized points.
    pub fn stroke_line(&self, from_rx: f64, from_ry: f64, to_rx: f64, to_ry: f64) -> Result<()> {
        let p1 = self.pt(from_rx, from_ry);
        let p2 = self.pt(to_rx, to_ry);
        mouse::stroke_continuous(&[p1, p2], MouseButton::Left)
    }

    /// Draws a continuous polyline connecting multiple normalized points.
    pub fn stroke_polyline(&self, points: &[(f64, f64)]) -> Result<()> {
        if points.len() < 2 {
            return Ok(());
        }
        let mut screen_pts: Vec<Point> = points.iter().map(|&(rx, ry)| self.pt(rx, ry)).collect();
        // Repeat the first edge at the end so if Paint consumes the very first drag to focus the canvas, the closing stroke redraws it 100%!
        if screen_pts.len() >= 3 {
            let p0 = screen_pts[0];
            let p_last = screen_pts[screen_pts.len() - 1];
            if p0.x == p_last.x && p0.y == p_last.y {
                let p1 = screen_pts[1];
                screen_pts.push(p1);
            }
        }
        mouse::stroke_continuous(&screen_pts, MouseButton::Left)
    }

    /// Draws a smooth cubic Bézier curve between (p0) and (p3) with control points (p1) and (p2).
    pub fn stroke_bezier(
        &self,
        p0: (f64, f64),
        p1: (f64, f64),
        p2: (f64, f64),
        p3: (f64, f64),
        steps: usize,
    ) -> Result<()> {
        let steps = steps.max(8);
        let mut pts = Vec::with_capacity(steps + 1);
        for i in 0..=steps {
            let t = i as f64 / steps as f64;
            let u = 1.0 - t;
            let tt = t * t;
            let uu = u * u;
            let uuu = uu * u;
            let ttt = tt * t;

            let rx = uuu * p0.0 + 3.0 * uu * t * p1.0 + 3.0 * u * tt * p2.0 + ttt * p3.0;
            let ry = uuu * p0.1 + 3.0 * uu * t * p1.1 + 3.0 * u * tt * p2.1 + ttt * p3.1;
            pts.push((rx, ry));
        }
        self.stroke_polyline(&pts)
    }

    /// Draws a regular polygon with aspect ratio compensation.
    pub fn stroke_polygon(
        &self,
        cx: f64,
        cy: f64,
        radius: f64,
        sides: usize,
        rotation: f64,
    ) -> Result<()> {
        let mut pts = Vec::with_capacity(sides + 1);
        for i in 0..=sides {
            let angle = rotation + (i as f64 / sides as f64) * 2.0 * PI;
            let rx = cx + (radius / self.aspect_ratio) * angle.cos();
            let ry = cy + radius * angle.sin();
            pts.push((rx, ry));
        }
        self.stroke_polyline(&pts)
    }

    /// Draws an N-pointed star with outer and inner radii.
    pub fn stroke_star(
        &self,
        cx: f64,
        cy: f64,
        r_outer: f64,
        r_inner: f64,
        points: usize,
        rotation: f64,
    ) -> Result<()> {
        let total_pts = points * 2;
        let mut pts = Vec::with_capacity(total_pts + 1);
        for i in 0..=total_pts {
            let angle = rotation + (i as f64 / total_pts as f64) * 2.0 * PI;
            let r = if i % 2 == 0 { r_outer } else { r_inner };
            let rx = cx + (r / self.aspect_ratio) * angle.cos();
            let ry = cy + r * angle.sin();
            pts.push((rx, ry));
        }
        self.stroke_polyline(&pts)
    }

    /// Draws an arc or circle slice between start_rad and end_rad.
    pub fn stroke_arc(
        &self,
        cx: f64,
        cy: f64,
        radius: f64,
        start_rad: f64,
        end_rad: f64,
        steps: usize,
    ) -> Result<()> {
        let steps = steps.max(6);
        let mut pts = Vec::with_capacity(steps + 1);
        for i in 0..=steps {
            let t = i as f64 / steps as f64;
            let angle = start_rad + t * (end_rad - start_rad);
            let rx = cx + (radius / self.aspect_ratio) * angle.cos();
            let ry = cy + radius * angle.sin();
            pts.push((rx, ry));
        }
        self.stroke_polyline(&pts)
    }

    /// Draws an Archimedean spiral.
    pub fn stroke_spiral(
        &self,
        cx: f64,
        cy: f64,
        start_r: f64,
        end_r: f64,
        turns: f64,
        steps: usize,
    ) -> Result<()> {
        let mut pts = Vec::with_capacity(steps + 1);
        for i in 0..=steps {
            let t = i as f64 / steps as f64;
            let angle = t * turns * 2.0 * PI;
            let r = start_r + t * (end_r - start_r);
            let rx = cx + (r / self.aspect_ratio) * angle.cos();
            let ry = cy + r * angle.sin();
            pts.push((rx, ry));
        }
        self.stroke_polyline(&pts)
    }

    /// Draws a mechanical gear with teeth.
    pub fn stroke_gear(
        &self,
        cx: f64,
        cy: f64,
        r_outer: f64,
        r_inner: f64,
        teeth: usize,
        rotation: f64,
    ) -> Result<()> {
        let total = teeth * 4;
        let mut pts = Vec::with_capacity(total + 1);
        for i in 0..=total {
            let angle = rotation + (i as f64 / total as f64) * 2.0 * PI;
            let step = i % 4;
            let r = if step == 1 || step == 2 { r_outer } else { r_inner };
            let rx = cx + (r / self.aspect_ratio) * angle.cos();
            let ry = cy + r * angle.sin();
            pts.push((rx, ry));
        }
        self.stroke_polyline(&pts)
    }

    /// Draws stroked vector uppercase letters and digits at a normalized origin.
    pub fn stroke_text(&self, start_x: f64, start_y: f64, text: &str, scale: f64) -> Result<()> {
        let char_w = scale / self.aspect_ratio;
        let char_h = scale * 1.5;
        let spacing = char_w * 1.35;
        let mut cur_x = start_x;

        for ch in text.chars() {
            let x0 = cur_x;
            let x1 = cur_x + char_w;
            let xm = cur_x + char_w * 0.5;
            let y0 = start_y;
            let y1 = start_y + char_h;
            let ym = start_y + char_h * 0.5;

            match ch.to_ascii_uppercase() {
                'A' => {
                    self.stroke_line(x0, y1, xm, y0)?;
                    self.stroke_line(xm, y0, x1, y1)?;
                    self.stroke_line(x0 + char_w * 0.2, ym, x1 - char_w * 0.2, ym)?;
                }
                'B' => {
                    self.stroke_line(x0, y0, x0, y1)?;
                    self.stroke_line(x0, y0, xm, y0)?;
                    self.stroke_line(xm, y0, x1, ym * 0.5)?;
                    self.stroke_line(x1, ym * 0.5, xm, ym)?;
                    self.stroke_line(xm, ym, x0, ym)?;
                    self.stroke_line(xm, ym, x1, ym + (y1 - ym) * 0.5)?;
                    self.stroke_line(x1, ym + (y1 - ym) * 0.5, xm, y1)?;
                    self.stroke_line(xm, y1, x0, y1)?;
                }
                'C' => {
                    self.stroke_line(x1, y0, x0, y0)?;
                    self.stroke_line(x0, y0, x0, y1)?;
                    self.stroke_line(x0, y1, x1, y1)?;
                }
                'D' => {
                    self.stroke_line(x0, y0, x0, y1)?;
                    self.stroke_line(x0, y0, xm, y0)?;
                    self.stroke_line(xm, y0, x1, ym)?;
                    self.stroke_line(x1, ym, xm, y1)?;
                    self.stroke_line(xm, y1, x0, y1)?;
                }
                'E' => {
                    self.stroke_line(x1, y0, x0, y0)?;
                    self.stroke_line(x0, y0, x0, y1)?;
                    self.stroke_line(x0, y1, x1, y1)?;
                    self.stroke_line(x0, ym, xm, ym)?;
                }
                'F' => {
                    self.stroke_line(x1, y0, x0, y0)?;
                    self.stroke_line(x0, y0, x0, y1)?;
                    self.stroke_line(x0, ym, xm, ym)?;
                }
                'G' => {
                    self.stroke_line(x1, y0, x0, y0)?;
                    self.stroke_line(x0, y0, x0, y1)?;
                    self.stroke_line(x0, y1, x1, y1)?;
                    self.stroke_line(x1, y1, x1, ym)?;
                    self.stroke_line(x1, ym, xm, ym)?;
                }
                'H' => {
                    self.stroke_line(x0, y0, x0, y1)?;
                    self.stroke_line(x1, y0, x1, y1)?;
                    self.stroke_line(x0, ym, x1, ym)?;
                }
                'I' => {
                    self.stroke_line(x0, y0, x1, y0)?;
                    self.stroke_line(xm, y0, xm, y1)?;
                    self.stroke_line(x0, y1, x1, y1)?;
                }
                'L' => {
                    self.stroke_line(x0, y0, x0, y1)?;
                    self.stroke_line(x0, y1, x1, y1)?;
                }
                'M' => {
                    self.stroke_line(x0, y1, x0, y0)?;
                    self.stroke_line(x0, y0, xm, ym)?;
                    self.stroke_line(xm, ym, x1, y0)?;
                    self.stroke_line(x1, y0, x1, y1)?;
                }
                'N' => {
                    self.stroke_line(x0, y1, x0, y0)?;
                    self.stroke_line(x0, y0, x1, y1)?;
                    self.stroke_line(x1, y1, x1, y0)?;
                }
                'O' => {
                    self.stroke_line(x0, y0, x1, y0)?;
                    self.stroke_line(x1, y0, x1, y1)?;
                    self.stroke_line(x1, y1, x0, y1)?;
                    self.stroke_line(x0, y1, x0, y0)?;
                }
                'P' => {
                    self.stroke_line(x0, y1, x0, y0)?;
                    self.stroke_line(x0, y0, x1, y0)?;
                    self.stroke_line(x1, y0, x1, ym)?;
                    self.stroke_line(x1, ym, x0, ym)?;
                }
                'R' => {
                    self.stroke_line(x0, y1, x0, y0)?;
                    self.stroke_line(x0, y0, x1, y0)?;
                    self.stroke_line(x1, y0, x1, ym)?;
                    self.stroke_line(x1, ym, x0, ym)?;
                    self.stroke_line(xm, ym, x1, y1)?;
                }
                'S' => {
                    self.stroke_line(x1, y0, x0, y0)?;
                    self.stroke_line(x0, y0, x0, ym)?;
                    self.stroke_line(x0, ym, x1, ym)?;
                    self.stroke_line(x1, ym, x1, y1)?;
                    self.stroke_line(x1, y1, x0, y1)?;
                }
                'T' => {
                    self.stroke_line(x0, y0, x1, y0)?;
                    self.stroke_line(xm, y0, xm, y1)?;
                }
                'U' => {
                    self.stroke_line(x0, y0, x0, y1)?;
                    self.stroke_line(x0, y1, x1, y1)?;
                    self.stroke_line(x1, y1, x1, y0)?;
                }
                'V' => {
                    self.stroke_line(x0, y0, xm, y1)?;
                    self.stroke_line(xm, y1, x1, y0)?;
                }
                'W' => {
                    self.stroke_line(x0, y0, x0 + char_w * 0.25, y1)?;
                    self.stroke_line(x0 + char_w * 0.25, y1, xm, ym)?;
                    self.stroke_line(xm, ym, x0 + char_w * 0.75, y1)?;
                    self.stroke_line(x0 + char_w * 0.75, y1, x1, y0)?;
                }
                'X' => {
                    self.stroke_line(x0, y0, x1, y1)?;
                    self.stroke_line(x0, y1, x1, y0)?;
                }
                'Y' => {
                    self.stroke_line(x0, y0, xm, ym)?;
                    self.stroke_line(x1, y0, xm, ym)?;
                    self.stroke_line(xm, ym, xm, y1)?;
                }
                'Z' => {
                    self.stroke_line(x0, y0, x1, y0)?;
                    self.stroke_line(x1, y0, x0, y1)?;
                    self.stroke_line(x0, y1, x1, y1)?;
                }
                '0'..='9' => {
                    // Generic box for numbers or simple representation
                    self.stroke_line(x0, y0, x1, y0)?;
                    self.stroke_line(x1, y0, x1, y1)?;
                    self.stroke_line(x1, y1, x0, y1)?;
                    self.stroke_line(x0, y1, x0, y0)?;
                    self.stroke_line(x0, y1, x1, y0)?;
                }
                '-' => {
                    self.stroke_line(x0, ym, x1, ym)?;
                }
                '.' => {
                    self.stroke_line(xm - 0.002, y1, xm + 0.002, y1)?;
                }
                ' ' => {}
                _ => {}
            }
            cur_x += spacing;
        }
        Ok(())
    }
}
