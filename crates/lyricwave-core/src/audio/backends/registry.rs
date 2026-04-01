use crate::audio::AudioBackend;

use super::cpal_ffmpeg::CpalFfmpegBackend;

#[derive(Debug, Clone, Copy)]
pub struct AudioBackendDescriptor {
    pub id: &'static str,
    pub note: &'static str,
}

pub fn default_audio_backend() -> Box<dyn AudioBackend> {
    Box::new(CpalFfmpegBackend::new())
}

pub fn build_audio_backend(backend_id: &str) -> Result<Box<dyn AudioBackend>, String> {
    match backend_id {
        "cpal+ffmpeg" => Ok(Box::new(CpalFfmpegBackend::new())),
        _ => Err(format!("unknown audio backend: {backend_id}")),
    }
}

pub fn audio_backends() -> Vec<AudioBackendDescriptor> {
    vec![AudioBackendDescriptor {
        id: "cpal+ffmpeg",
        note: "CPAL device discovery + FFmpeg command strategy by OS",
    }]
}
