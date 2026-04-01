mod backend;
mod backends;

pub use backend::{
    AudioBackend, AudioError, BackendCapabilities, CaptureFormat, CaptureRequest, CaptureTarget,
    CommandSpec, DeviceInfo,
};
pub use backends::{
    AudioBackendDescriptor, CpalFfmpegBackend, audio_backends, build_audio_backend,
    default_audio_backend,
};
