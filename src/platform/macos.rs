use std::io::Cursor;

use image::{ImageFormat, RgbaImage};
use objc2::rc::Retained;
use objc2::{AnyThread, MainThreadMarker};
use objc2_app_kit::{
    NSAutoresizingMaskOptions, NSEvent, NSImage, NSImageScaling, NSImageView, NSScreen, NSView,
    NSWindowCollectionBehavior,
};
use objc2_foundation::NSData;
use winit::dpi::{LogicalPosition, LogicalSize, Position, Size};
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
use winit::window::Window;

/// winit's own transparency handling is what we want on macOS.
pub const WINIT_TRANSPARENT: bool = true;

/// Screen units per physical pixel.
pub fn units_per_pixel(scale_factor: f64) -> f64 {
    1.0 / scale_factor
}

/// Screen units per logical point (units already are points here).
pub fn units_per_logical(_scale_factor: f64) -> f64 {
    1.0
}

pub fn position(x: f64, y: f64) -> Position {
    LogicalPosition::new(x, y).into()
}

pub fn size(units: f64) -> Size {
    LogicalSize::new(units, units).into()
}

/// Cursor position in points, origin top-left of the primary screen.
pub fn cursor_position() -> (f64, f64) {
    let mtm = MainThreadMarker::new().expect("cursor queried on main thread");
    let p = NSEvent::mouseLocation();
    let primary_h = NSScreen::screens(mtm)
        .firstObject()
        .map(|s| s.frame().size.height)
        .unwrap_or(0.0);
    (p.x, primary_h - p.y)
}

pub struct Renderer {
    view: Retained<NSImageView>,
    images: Vec<Retained<NSImage>>,
    shown: Option<usize>,
}

impl Renderer {
    /// `frames` must already be scaled to the window's backing pixel size.
    pub fn new(window: &Window, frames: &[RgbaImage]) -> Self {
        let mtm = MainThreadMarker::new().expect("renderer created on main thread");
        let RawWindowHandle::AppKit(handle) = window.window_handle().unwrap().as_raw() else {
            unreachable!("not an AppKit window");
        };
        let ns_view: Retained<NSView> = unsafe { Retained::retain(handle.ns_view.as_ptr().cast()) }.unwrap();
        let ns_window = ns_view.window().expect("view is attached to a window");

        // Follow the user across Spaces and sit on top of full-screen apps.
        ns_window.setCollectionBehavior(
            NSWindowCollectionBehavior::CanJoinAllSpaces
                | NSWindowCollectionBehavior::FullScreenAuxiliary
                | NSWindowCollectionBehavior::Stationary
                | NSWindowCollectionBehavior::IgnoresCycle,
        );

        let view = NSImageView::initWithFrame(mtm.alloc(), ns_view.bounds());
        view.setImageScaling(NSImageScaling::ScaleAxesIndependently);
        view.setAutoresizingMask(
            NSAutoresizingMaskOptions::ViewWidthSizable | NSAutoresizingMaskOptions::ViewHeightSizable,
        );
        ns_view.addSubview(&view);

        Self { view, images: Self::encode(frames), shown: None }
    }

    fn encode(frames: &[RgbaImage]) -> Vec<Retained<NSImage>> {
        frames
            .iter()
            .map(|f| {
                let mut png = Vec::new();
                f.write_to(&mut Cursor::new(&mut png), ImageFormat::Png).expect("png encode");
                let data = NSData::with_bytes(&png);
                NSImage::initWithData(NSImage::alloc(), &data).expect("NSImage from png")
            })
            .collect()
    }

    /// Swap the sprite set (e.g. a different character). Same frame count and size expected.
    pub fn set_frames(&mut self, frames: &[RgbaImage]) {
        self.images = Self::encode(frames);
        self.shown = None;
    }

    pub fn show(&mut self, frame: usize) {
        if self.shown == Some(frame) {
            return;
        }
        self.view.setImage(Some(&self.images[frame]));
        self.shown = Some(frame);
    }
}
