use crate::video::VideoBackend;

use super::native_platform::NativePlatformVideoBackend;

#[derive(Debug, Clone, Copy)]
pub struct VideoBackendDescriptor {
    pub id: &'static str,
    pub note: &'static str,
}

pub fn default_video_backend() -> Box<dyn VideoBackend> {
    Box::new(NativePlatformVideoBackend::new())
}

pub fn build_video_backend(backend_id: &str) -> Result<Box<dyn VideoBackend>, String> {
    match backend_id {
        "platform-native" => Ok(Box::new(NativePlatformVideoBackend::new())),
        _ => Err(format!("unknown video backend: {backend_id}")),
    }
}

pub fn video_backends() -> Vec<VideoBackendDescriptor> {
    vec![VideoBackendDescriptor {
        id: "platform-native",
        note: "Native platform video backend scaffold (screen capture implementation in progress)",
    }]
}
