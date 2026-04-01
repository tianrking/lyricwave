use crate::video::{DisplayInfo, VideoCaptureReport, VideoCaptureRequest, VideoError};

pub fn capability_note() -> &'static str {
    "Windows video backend scaffold is ready; DXGI/Windows Graphics Capture integration is the next step."
}

pub fn list_displays() -> Result<Vec<DisplayInfo>, VideoError> {
    Ok(vec![DisplayInfo {
        id: "windows-main".to_string(),
        name: "Main Display".to_string(),
        is_primary: true,
        width: 1920,
        height: 1080,
    }])
}

pub fn capture_screen(_request: &VideoCaptureRequest) -> Result<VideoCaptureReport, VideoError> {
    Err(VideoError::NotImplemented {
        feature: "windows native screen recording",
    })
}
