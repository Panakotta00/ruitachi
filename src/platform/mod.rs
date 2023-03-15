pub mod common;

#[cfg(any(feature = "windows", feature = "wayland"))]
pub mod winit;

#[cfg(feature = "windows")]
pub mod windows;
#[cfg(feature = "windows")]
pub use crate::platform::windows::*;

#[cfg(feature = "wayland")]
pub mod wayland;
#[cfg(feature = "wayland")]
pub use crate::platform::wayland::*;
