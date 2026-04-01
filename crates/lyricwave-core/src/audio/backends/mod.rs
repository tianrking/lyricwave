mod cpal_ffmpeg;
mod platform;
mod registry;

pub use cpal_ffmpeg::CpalFfmpegBackend;
pub use registry::{
    AudioBackendDescriptor, audio_backends, build_audio_backend, default_audio_backend,
};
