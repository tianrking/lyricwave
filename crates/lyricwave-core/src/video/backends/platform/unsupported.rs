use crate::video::{DisplayInfo, VideoCaptureReport, VideoCaptureRequest, VideoError};

pub fn capability_note() -> &'static str {
    "Unsupported OS for native video capture backend scaffold."
}

pub fn list_displays() -> Result<Vec<DisplayInfo>, VideoError> {
    Ok(vec![])
}

pub fn capture_screen(_request: &VideoCaptureRequest) -> Result<VideoCaptureReport, VideoError> {
    Err(VideoError::NotImplemented {
        feature: "native screen recording on this OS",
    })
}
