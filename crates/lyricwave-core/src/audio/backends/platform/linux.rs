use crate::audio::CaptureRequest;

pub fn ffmpeg_input_args(request: &CaptureRequest) -> Vec<String> {
    vec![
        "-f".to_string(),
        "pulse".to_string(),
        "-i".to_string(),
        request
            .input_device_hint
            .clone()
            .unwrap_or_else(|| "default".to_string()),
    ]
}

pub fn capability_note() -> &'static str {
    "Linux uses PulseAudio/PipeWire source; monitor source may be required"
}
