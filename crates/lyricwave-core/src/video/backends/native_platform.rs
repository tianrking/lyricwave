use crate::video::{
    DisplayInfo, VideoBackend, VideoBackendCapabilities, VideoCaptureReport, VideoCaptureRequest,
    VideoError,
};

use super::platform;

pub struct NativePlatformVideoBackend;

impl NativePlatformVideoBackend {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NativePlatformVideoBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl VideoBackend for NativePlatformVideoBackend {
    fn backend_name(&self) -> &'static str {
        "platform-native"
    }

    fn capabilities(&self) -> VideoBackendCapabilities {
        VideoBackendCapabilities {
            screen_capture: true,
            window_capture: false,
            note: platform::capability_note(),
        }
    }

    fn list_displays(&self) -> Result<Vec<DisplayInfo>, VideoError> {
        platform::list_displays()
    }

    fn capture_blocking(
        &self,
        request: &VideoCaptureRequest,
    ) -> Result<VideoCaptureReport, VideoError> {
        platform::capture_screen(request)
    }
}
