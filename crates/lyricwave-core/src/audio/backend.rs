use std::fmt;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use thiserror::Error;

#[derive(Debug, Clone)]
pub struct OutputDeviceInfo {
    pub id: String,
    pub name: String,
    pub is_default: bool,
}

impl fmt::Display for OutputDeviceInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let default_tag = if self.is_default { " (default)" } else { "" };
        write!(f, "{} [{}]{}", self.name, self.id, default_tag)
    }
}

#[derive(Debug, Clone)]
pub struct InputDeviceInfo {
    pub id: String,
    pub name: String,
    pub is_default: bool,
    pub loopback_score: i32,
    pub is_loopback_candidate: bool,
}

impl fmt::Display for InputDeviceInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let default_tag = if self.is_default { " (default)" } else { "" };
        let loopback_tag = if self.is_loopback_candidate {
            format!(" [loopback-score:{}]", self.loopback_score)
        } else {
            String::new()
        };
        write!(
            f,
            "{} [{}]{}{}",
            self.name, self.id, default_tag, loopback_tag
        )
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

#[derive(Debug, Clone)]
pub enum ProcessSelector {
    Pid(u32),
    NameContains(String),
}

impl fmt::Display for ProcessSelector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pid(pid) => write!(f, "pid:{pid}"),
            Self::NameContains(name) => write!(f, "name:*{name}*"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum CaptureScope {
    System,
    Processes(Vec<ProcessSelector>),
}

#[derive(Debug, Clone, Copy)]
pub enum CaptureFormat {
    Wav,
    Flac,
    PcmS16Le,
}

#[derive(Debug, Clone)]
pub struct CaptureRequest {
    pub scope: CaptureScope,
    pub target: CaptureTarget,
    pub duration_secs: Option<u32>,
    pub sample_rate: Option<u32>,
    pub channels: Option<u16>,
    pub format: CaptureFormat,
    pub input_device_hint: Option<String>,
    pub prefer_loopback: bool,
    pub stop_flag: Option<Arc<AtomicBool>>,
}

#[derive(Debug, Clone)]
pub struct CaptureReport {
    pub captured_samples: usize,
    pub sample_rate: u32,
    pub channels: u16,
    pub selected_input_device: InputDeviceInfo,
    pub selection_reason: String,
    pub matched_processes: Vec<String>,
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
    fn list_output_devices(&self) -> Result<Vec<OutputDeviceInfo>, AudioError>;
    fn list_input_devices(&self) -> Result<Vec<InputDeviceInfo>, AudioError>;
    fn capture_blocking(&self, request: &CaptureRequest) -> Result<CaptureReport, AudioError>;
}
