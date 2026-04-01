use crate::video::{DisplayInfo, VideoCaptureReport, VideoCaptureRequest, VideoError};

pub fn capability_note() -> &'static str {
    "Linux video backend scaffold is ready; screen capture implementation will be added via native pipeline (PipeWire/X11/Wayland)."
}

pub fn list_displays() -> Result<Vec<DisplayInfo>, VideoError> {
    Ok(vec![DisplayInfo {
        id: "linux-main".to_string(),
        name: "Main Display".to_string(),
        is_primary: true,
        width: 1920,
        height: 1080,
    }])
}

pub fn capture_screen(_request: &VideoCaptureRequest) -> Result<VideoCaptureReport, VideoError> {
    Err(VideoError::NotImplemented {
        feature: "linux native screen capture",
    })
}
