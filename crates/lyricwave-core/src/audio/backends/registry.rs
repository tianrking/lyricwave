use crate::audio::AudioBackend;

use super::cpal_ffmpeg::CpalFfmpegBackend;

pub fn default_audio_backend() -> Box<dyn AudioBackend> {
    Box::new(CpalFfmpegBackend::new())
}

pub fn audio_backend_names() -> Vec<&'static str> {
    vec!["cpal+ffmpeg"]
}
