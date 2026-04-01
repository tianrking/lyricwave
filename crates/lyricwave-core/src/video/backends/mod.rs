mod native_platform;
mod platform;
mod registry;

pub use native_platform::NativePlatformVideoBackend;
pub use registry::{
    VideoBackendDescriptor, build_video_backend, default_video_backend, video_backends,
};
