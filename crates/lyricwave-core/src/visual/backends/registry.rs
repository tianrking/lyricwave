use crate::visual::VisualBackend;

use super::native_platform::NativePlatformVisualBackend;

#[derive(Debug, Clone, Copy)]
pub struct VisualBackendDescriptor {
    pub id: &'static str,
    pub note: &'static str,
}

pub fn default_visual_backend() -> Box<dyn VisualBackend> {
    Box::new(NativePlatformVisualBackend::new())
}

pub fn build_visual_backend(backend_id: &str) -> Result<Box<dyn VisualBackend>, String> {
    match backend_id {
        "platform-native" => Ok(Box::new(NativePlatformVisualBackend::new())),
        _ => Err(format!("unknown visual backend: {backend_id}")),
    }
}

pub fn visual_backends() -> Vec<VisualBackendDescriptor> {
    vec![VisualBackendDescriptor {
        id: "platform-native",
        note: "Native platform visual backend scaffold (screen capture implementation in progress)",
    }]
}
