mod backend;
mod backends;

pub use backend::{
    DisplayInfo, VideoBackend, VideoBackendCapabilities, VideoCaptureReport, VideoCaptureRequest,
    VideoError, VideoScope, VideoTarget,
};
pub use backends::{
    NativePlatformVideoBackend, VideoBackendDescriptor, build_video_backend, default_video_backend,
    video_backends,
};
