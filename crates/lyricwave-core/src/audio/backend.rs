use std::fmt;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use thiserror::Error;

#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub id: String,
    pub name: String,
    pub is_default_output: bool,
}

impl fmt::Display for DeviceInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let default_tag = if self.is_default_output {
            " (default)"
        } else {
            ""
        };
        write!(f, "{} [{}]{}", self.name, self.id, default_tag)
    }
}

#[derive(Debug, Clone)]
pub struct BackendCapabilities {
    pub system_loopback_capture: bool,
    pub per_app_capture: bool,
    pub note: &'static str,
}

#[derive(Debug, Clone)]
pub enum CaptureTarget {
    File(PathBuf),
    StdoutPcm,
}

#[derive(Debug, Clone, Copy)]
pub enum CaptureFormat {
    Wav,
    Flac,
    PcmS16Le,
}

#[derive(Debug, Clone)]
pub struct CaptureRequest {
    pub target: CaptureTarget,
    pub duration_secs: Option<u32>,
    pub sample_rate: Option<u32>,
    pub channels: Option<u16>,
    pub format: CaptureFormat,
    pub input_device_hint: Option<String>,
    pub stop_flag: Option<Arc<AtomicBool>>,
}

#[derive(Debug, Clone)]
pub struct CaptureReport {
    pub captured_samples: usize,
    pub sample_rate: u32,
    pub channels: u16,
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
    fn capabilities(&self) -> BackendCapabilities;
    fn list_output_devices(&self) -> Result<Vec<DeviceInfo>, AudioError>;
    fn capture_blocking(&self, request: &CaptureRequest) -> Result<CaptureReport, AudioError>;
}
