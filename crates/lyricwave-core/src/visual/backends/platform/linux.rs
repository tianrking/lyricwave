use crate::visual::{DisplayInfo, VisualCaptureReport, VisualCaptureRequest, VisualError};

pub fn capability_note() -> &'static str {
    "Linux visual backend scaffold is ready; screen capture implementation will be added via native pipeline (PipeWire/X11/Wayland)."
}

pub fn list_displays() -> Result<Vec<DisplayInfo>, VisualError> {
    Ok(vec![DisplayInfo {
        id: "linux-main".to_string(),
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
        feature: "linux native screen capture",
    })
}
