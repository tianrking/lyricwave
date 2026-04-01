use std::fmt;

use thiserror::Error;

#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub id: String,
    pub name: String,
    pub is_default_output: bool,
}

impl fmt::Display for DeviceInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let default_tag = if self.is_default_output { " (default)" } else { "" };
        write!(f, "{} [{}]{}", self.name, self.id, default_tag)
    }
}

#[derive(Debug, Clone)]
pub struct CaptureConfig {
    pub sample_rate: Option<u32>,
    pub channels: Option<u16>,
    pub output_device_id: Option<String>,
}

#[derive(Debug)]
pub struct CaptureStream {
    pub backend_name: &'static str,
}

#[derive(Debug, Error)]
pub enum AudioError {
    #[error("{0}")]
    Message(String),

    #[error("feature not yet implemented on this backend: {feature}")]
    NotImplemented { feature: &'static str },
}

pub trait AudioBackend: Send + Sync {
    fn backend_name(&self) -> &'static str;
    fn list_output_devices(&self) -> Result<Vec<DeviceInfo>, AudioError>;
    fn start_system_capture(&self, _config: CaptureConfig) -> Result<CaptureStream, AudioError> {
        Err(AudioError::NotImplemented {
            feature: "system loopback capture",
        })
    }
}
