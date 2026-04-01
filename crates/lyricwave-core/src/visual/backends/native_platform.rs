use crate::visual::{
    ActiveVisualProcessInfo, DisplayInfo, VisualBackend, VisualBackendCapabilities,
    VisualCaptureReport, VisualCaptureRequest, VisualError,
};

use super::platform;

pub struct NativePlatformVisualBackend;

impl NativePlatformVisualBackend {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NativePlatformVisualBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl VisualBackend for NativePlatformVisualBackend {
    fn backend_name(&self) -> &'static str {
        "platform-native"
    }

    fn capabilities(&self) -> VisualBackendCapabilities {
        VisualBackendCapabilities {
            screen_capture: true,
            window_capture: false,
            per_app_capture: false,
            note: platform::capability_note(),
        }
    }

    fn list_displays(&self) -> Result<Vec<DisplayInfo>, VisualError> {
        platform::list_displays()
    }

    fn list_active_visual_processes(&self) -> Result<Vec<ActiveVisualProcessInfo>, VisualError> {
        platform::list_active_visual_processes()
    }

    fn capture_blocking(
        &self,
        request: &VisualCaptureRequest,
    ) -> Result<VisualCaptureReport, VisualError> {
        platform::capture_display(request)
    }
}
