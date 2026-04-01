use crate::audio::{AudioError, CaptureReport, CaptureRequest};

pub fn capability_note() -> &'static str {
    "Windows native capture uses CPAL input stream; configure loopback-capable endpoint routing for system mix."
}

pub fn supports_per_app_capture() -> bool {
    false
}

pub fn capture_processes(_request: &CaptureRequest) -> Result<CaptureReport, AudioError> {
    Err(AudioError::NotImplemented {
        feature: "per-app capture on Windows backend",
    })
}
