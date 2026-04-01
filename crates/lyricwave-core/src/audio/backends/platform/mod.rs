#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
mod unsupported;
#[cfg(target_os = "windows")]
mod windows;

use crate::audio::{AudioError, CaptureReport, CaptureRequest};

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
pub fn supports_per_app_capture() -> bool {
    linux::supports_per_app_capture()
}
#[cfg(target_os = "macos")]
pub fn supports_per_app_capture() -> bool {
    macos::supports_per_app_capture()
}
#[cfg(target_os = "windows")]
pub fn supports_per_app_capture() -> bool {
    windows::supports_per_app_capture()
}
#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
pub fn supports_per_app_capture() -> bool {
    unsupported::supports_per_app_capture()
}

#[cfg(target_os = "linux")]
pub fn capture_processes(request: &CaptureRequest) -> Result<CaptureReport, AudioError> {
    linux::capture_processes(request)
}
#[cfg(target_os = "macos")]
pub fn capture_processes(request: &CaptureRequest) -> Result<CaptureReport, AudioError> {
    macos::capture_processes(request)
}
#[cfg(target_os = "windows")]
pub fn capture_processes(request: &CaptureRequest) -> Result<CaptureReport, AudioError> {
    windows::capture_processes(request)
}
#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
pub fn capture_processes(request: &CaptureRequest) -> Result<CaptureReport, AudioError> {
    unsupported::capture_processes(request)
}
