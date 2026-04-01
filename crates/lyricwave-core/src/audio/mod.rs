mod backend;
mod cpal_backend;

pub use backend::{AudioBackend, AudioError, CaptureConfig, CaptureStream, DeviceInfo};
pub use cpal_backend::CpalBackend;
