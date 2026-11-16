//! Platform-specific bits: global cursor position, window tweaks, and sprite rendering.
//!
//! Coordinates are in "screen units": logical points on macOS, physical pixels on Windows.
//! That matches what each platform's cursor API returns and what winit expects for positions there.

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::*;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use windows::*;
