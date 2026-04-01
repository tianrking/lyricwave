use crate::audio::AudioBackend;

use super::cpal_native::CpalNativeBackend;

#[derive(Debug, Clone, Copy)]
pub struct AudioBackendDescriptor {
    pub id: &'static str,
    pub note: &'static str,
}

pub fn default_audio_backend() -> Box<dyn AudioBackend> {
    Box::new(CpalNativeBackend::new())
}

pub fn build_audio_backend(backend_id: &str) -> Result<Box<dyn AudioBackend>, String> {
    match backend_id {
        "cpal-native" => Ok(Box::new(CpalNativeBackend::new())),
        _ => Err(format!("unknown audio backend: {backend_id}")),
    }
}

pub fn audio_backends() -> Vec<AudioBackendDescriptor> {
    vec![AudioBackendDescriptor {
        id: "cpal-native",
        note: "CPAL native capture stream (requires proper loopback input routing)",
    }]
}
