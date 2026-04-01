#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
mod unsupported;
#[cfg(target_os = "windows")]
mod windows;

use crate::video::{DisplayInfo, VideoCaptureReport, VideoCaptureRequest, VideoError};

#[cfg(target_os = "linux")]
pub fn capability_note() -> &'static str {
    linux::capability_note()
}
#[cfg(target_os = "macos")]
pub fn capability_note() -> &'static str {
    macos::capability_note()
}
#[cfg(target_os = "windows")]
pub fn capability_note() -> &'static str {
    windows::capability_note()
}
#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
pub fn capability_note() -> &'static str {
    unsupported::capability_note()
}

#[cfg(target_os = "linux")]
pub fn list_displays() -> Result<Vec<DisplayInfo>, VideoError> {
    linux::list_displays()
}
#[cfg(target_os = "macos")]
pub fn list_displays() -> Result<Vec<DisplayInfo>, VideoError> {
    macos::list_displays()
}
#[cfg(target_os = "windows")]
pub fn list_displays() -> Result<Vec<DisplayInfo>, VideoError> {
    windows::list_displays()
}
#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
pub fn list_displays() -> Result<Vec<DisplayInfo>, VideoError> {
    unsupported::list_displays()
}

#[cfg(target_os = "linux")]
pub fn capture_screen(request: &VideoCaptureRequest) -> Result<VideoCaptureReport, VideoError> {
    linux::capture_screen(request)
}
#[cfg(target_os = "macos")]
pub fn capture_screen(request: &VideoCaptureRequest) -> Result<VideoCaptureReport, VideoError> {
    macos::capture_screen(request)
}
#[cfg(target_os = "windows")]
pub fn capture_screen(request: &VideoCaptureRequest) -> Result<VideoCaptureReport, VideoError> {
    windows::capture_screen(request)
}
#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
pub fn capture_screen(request: &VideoCaptureRequest) -> Result<VideoCaptureReport, VideoError> {
    unsupported::capture_screen(request)
}
