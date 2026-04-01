#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
mod unsupported;
#[cfg(target_os = "windows")]
mod windows;

use crate::visual::{DisplayInfo, VisualCaptureReport, VisualCaptureRequest, VisualError};

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
pub fn list_displays() -> Result<Vec<DisplayInfo>, VisualError> {
    linux::list_displays()
}
#[cfg(target_os = "macos")]
pub fn list_displays() -> Result<Vec<DisplayInfo>, VisualError> {
    macos::list_displays()
}
#[cfg(target_os = "windows")]
pub fn list_displays() -> Result<Vec<DisplayInfo>, VisualError> {
    windows::list_displays()
}
#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
pub fn list_displays() -> Result<Vec<DisplayInfo>, VisualError> {
    unsupported::list_displays()
}

#[cfg(target_os = "linux")]
pub fn capture_display(request: &VisualCaptureRequest) -> Result<VisualCaptureReport, VisualError> {
    linux::capture_display(request)
}
#[cfg(target_os = "macos")]
pub fn capture_display(request: &VisualCaptureRequest) -> Result<VisualCaptureReport, VisualError> {
    macos::capture_display(request)
}
#[cfg(target_os = "windows")]
pub fn capture_display(request: &VisualCaptureRequest) -> Result<VisualCaptureReport, VisualError> {
    windows::capture_display(request)
}
#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
pub fn capture_display(request: &VisualCaptureRequest) -> Result<VisualCaptureReport, VisualError> {
    unsupported::capture_display(request)
}
