use crate::audio::CaptureRequest;

pub fn ffmpeg_input_args(_request: &CaptureRequest) -> Vec<String> {
    vec![
        "-f".to_string(),
        "lavfi".to_string(),
        "-i".to_string(),
        "anullsrc".to_string(),
    ]
}

pub fn capability_note() -> &'static str {
    "Unsupported OS for system loopback template; using fallback source"
}
