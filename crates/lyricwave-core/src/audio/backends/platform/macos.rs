use crate::audio::CaptureRequest;

pub fn ffmpeg_input_args(request: &CaptureRequest) -> Vec<String> {
    let selector = request
        .input_device_hint
        .clone()
        .unwrap_or_else(|| ":0".to_string());

    vec![
        "-f".to_string(),
        "avfoundation".to_string(),
        "-i".to_string(),
        selector,
    ]
}

pub fn capability_note() -> &'static str {
    "macOS needs a loopback-capable input (e.g. BlackHole) as audio source"
}
