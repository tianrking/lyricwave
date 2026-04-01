use crate::audio::{ActiveAudioProcessInfo, AudioError, CaptureReport, CaptureRequest};

pub fn capability_note() -> &'static str {
    "Unsupported OS for native loopback defaults; provide an explicit input device that carries mixed audio."
}

pub fn supports_per_app_capture() -> bool {
    false
}

pub fn list_active_audio_processes() -> Result<Vec<ActiveAudioProcessInfo>, AudioError> {
    Err(AudioError::NotImplemented {
        feature: "list active audio processes on this OS backend",
    })
}

pub fn capture_processes(_request: &CaptureRequest) -> Result<CaptureReport, AudioError> {
    Err(AudioError::NotImplemented {
        feature: "per-app capture on this OS backend",
    })
}
