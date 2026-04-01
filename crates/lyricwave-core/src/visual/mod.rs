mod backend;
mod backends;

pub use backend::{
    ActiveVisualProcessInfo, DisplayInfo, VisualBackend, VisualBackendCapabilities,
    VisualCaptureReport, VisualCaptureRequest, VisualError, VisualProcessSelector, VisualScope,
    VisualTarget,
};
pub use backends::{
    NativePlatformVisualBackend, VisualBackendDescriptor, build_visual_backend,
    default_visual_backend, visual_backends,
};
