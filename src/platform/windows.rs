use std::ffi::c_void;

use image::RgbaImage;
use windows::Win32::Foundation::{COLORREF, HWND, POINT, SIZE};
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, GetDC, ReleaseDC, SelectObject,
    AC_SRC_ALPHA, AC_SRC_OVER, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, BLENDFUNCTION, DIB_RGB_COLORS,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetCursorPos, GetWindowLongPtrW, SetWindowLongPtrW, UpdateLayeredWindow, GWL_EXSTYLE, ULW_ALPHA,
    WS_EX_LAYERED,
};
use winit::dpi::{PhysicalPosition, PhysicalSize, Position, Size};
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
use winit::window::Window;

/// We paint through UpdateLayeredWindow, so winit's DWM blur-behind transparency must stay off.
pub const WINIT_TRANSPARENT: bool = false;

/// Screen units are physical pixels on Windows.
pub fn units_per_pixel(_scale_factor: f64) -> f64 {
    1.0
}

/// Screen units per logical point, so the cat grows with the DPI setting.
pub fn units_per_logical(scale_factor: f64) -> f64 {
    scale_factor
}

pub fn position(x: f64, y: f64) -> Position {
    PhysicalPosition::new(x.round() as i32, y.round() as i32).into()
}

pub fn size(units: f64) -> Size {
    PhysicalSize::new(units.round() as u32, units.round() as u32).into()
}

/// Cursor position in physical pixels, virtual-screen coordinates.
pub fn cursor_position() -> (f64, f64) {
    let mut p = POINT::default();
    // Safe: POINT is a plain out-param.
    let _ = unsafe { GetCursorPos(&mut p) };
    (p.x as f64, p.y as f64)
}

pub struct Renderer {
    hwnd: HWND,
    /// Premultiplied BGRA, one Vec per frame.
    frames: Vec<Vec<u8>>,
    px: i32,
    shown: Option<usize>,
}

impl Renderer {
    pub fn new(window: &Window, frames: &[RgbaImage]) -> Self {
        let RawWindowHandle::Win32(handle) = window.window_handle().unwrap().as_raw() else {
            unreachable!("not a Win32 window");
        };
        let hwnd = HWND(handle.hwnd.get() as *mut c_void);
        unsafe {
            let ex = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
            SetWindowLongPtrW(hwnd, GWL_EXSTYLE, ex | WS_EX_LAYERED.0 as isize);
        }
        let px = frames[0].width() as i32;
        Self { hwnd, frames: Self::premultiply(frames), px, shown: None }
    }

    fn premultiply(frames: &[RgbaImage]) -> Vec<Vec<u8>> {
        frames
            .iter()
            .map(|f| {
                f.pixels()
                    .flat_map(|p| {
                        let [r, g, b, a] = p.0;
                        let pm = |c: u8| ((c as u32 * a as u32) / 255) as u8;
                        [pm(b), pm(g), pm(r), a]
                    })
                    .collect()
            })
            .collect()
    }

    /// Swap the sprite set (e.g. a different character). Same frame count and size expected.
    pub fn set_frames(&mut self, frames: &[RgbaImage]) {
        self.frames = Self::premultiply(frames);
        self.shown = None;
    }

    pub fn show(&mut self, frame: usize) {
        if self.shown == Some(frame) {
            return;
        }
        self.shown = Some(frame);
        let src = &self.frames[frame];
        let px = self.px;
        unsafe {
            let screen_dc = GetDC(None);
            let mem_dc = CreateCompatibleDC(Some(screen_dc));
            let mut info = BITMAPINFO::default();
            info.bmiHeader = BITMAPINFOHEADER {
                biSize: size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: px,
                biHeight: -px, // top-down
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            };
            let mut bits: *mut c_void = std::ptr::null_mut();
            let Ok(bitmap) = CreateDIBSection(Some(screen_dc), &info, DIB_RGB_COLORS, &mut bits, None, 0) else {
                let _ = DeleteDC(mem_dc);
                ReleaseDC(None, screen_dc);
                return;
            };
            std::ptr::copy_nonoverlapping(src.as_ptr(), bits as *mut u8, src.len());
            let old = SelectObject(mem_dc, bitmap.into());

            let blend = BLENDFUNCTION {
                BlendOp: AC_SRC_OVER as u8,
                BlendFlags: 0,
                SourceConstantAlpha: 255,
                AlphaFormat: AC_SRC_ALPHA as u8,
            };
            let size = SIZE { cx: px, cy: px };
            let origin = POINT { x: 0, y: 0 };
            let _ = UpdateLayeredWindow(
                self.hwnd,
                None,
                None,
                Some(&size),
                Some(mem_dc),
                Some(&origin),
                COLORREF(0),
                Some(&blend),
                ULW_ALPHA,
            );

            SelectObject(mem_dc, old);
            let _ = DeleteObject(bitmap.into());
            let _ = DeleteDC(mem_dc);
            ReleaseDC(None, screen_dc);
        }
    }
}
