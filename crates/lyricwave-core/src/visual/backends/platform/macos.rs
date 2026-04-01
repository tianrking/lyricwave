use crate::visual::{DisplayInfo, VisualCaptureReport, VisualCaptureRequest, VisualError};

pub fn capability_note() -> &'static str {
    "macOS visual backend scaffold is ready; ScreenCaptureKit recorder integration is the next step."
}

pub fn list_displays() -> Result<Vec<DisplayInfo>, VisualError> {
    Ok(vec![DisplayInfo {
        id: "macos-main".to_string(),
        name: "Main Display".to_string(),
        is_primary: true,
        width: 1920,
        height: 1080,
    }])
}

pub fn capture_display(
    _request: &VisualCaptureRequest,
) -> Result<VisualCaptureReport, VisualError> {
    Err(VisualError::NotImplemented {
        feature: "macOS ScreenCaptureKit screen recording",
    })
}
