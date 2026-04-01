use crate::visual::{
    ActiveVisualProcessInfo, DisplayInfo, VisualCaptureReport, VisualCaptureRequest, VisualError,
};

pub fn capability_note() -> &'static str {
    "Windows visual backend scaffold is ready; DXGI/Windows Graphics Capture integration is the next step."
}

pub fn list_displays() -> Result<Vec<DisplayInfo>, VisualError> {
    Ok(vec![DisplayInfo {
        id: "windows-main".to_string(),
        name: "Main Display".to_string(),
        is_primary: true,
        width: 1920,
        height: 1080,
    }])
}

pub fn list_active_visual_processes() -> Result<Vec<ActiveVisualProcessInfo>, VisualError> {
    Ok(vec![])
}

pub fn capture_display(
    _request: &VisualCaptureRequest,
) -> Result<VisualCaptureReport, VisualError> {
    Err(VisualError::NotImplemented {
        feature: "windows native screen recording",
    })
}
