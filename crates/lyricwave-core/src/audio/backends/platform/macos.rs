use crate::audio::{AudioError, CaptureReport, CaptureRequest};

pub fn capability_note() -> &'static str {
    "macOS native capture uses CoreAudio through CPAL input stream; configure a loopback-capable input (e.g. BlackHole)."
}

pub fn supports_per_app_capture() -> bool {
    false
}

pub fn capture_processes(_request: &CaptureRequest) -> Result<CaptureReport, AudioError> {
    Err(AudioError::NotImplemented {
        feature: "per-app capture on macOS backend",
    })
}
