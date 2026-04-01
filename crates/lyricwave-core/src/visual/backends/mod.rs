mod native_platform;
mod platform;
mod registry;

pub use native_platform::NativePlatformVisualBackend;
pub use registry::{
    VisualBackendDescriptor, build_visual_backend, default_visual_backend, visual_backends,
};
