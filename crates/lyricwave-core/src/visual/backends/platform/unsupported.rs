use crate::visual::{DisplayInfo, VisualCaptureReport, VisualCaptureRequest, VisualError};

pub fn capability_note() -> &'static str {
    "Unsupported OS for native visual capture backend scaffold."
}

pub fn list_displays() -> Result<Vec<DisplayInfo>, VisualError> {
    Ok(vec![])
}

pub fn capture_display(
    _request: &VisualCaptureRequest,
) -> Result<VisualCaptureReport, VisualError> {
    Err(VisualError::NotImplemented {
        feature: "native screen recording on this OS",
    })
}
