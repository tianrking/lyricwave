use std::fmt;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use thiserror::Error;

#[derive(Debug, Clone)]
pub struct DisplayInfo {
    pub id: String,
    pub name: String,
    pub is_primary: bool,
    pub width: u32,
    pub height: u32,
}

impl fmt::Display for DisplayInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let primary = if self.is_primary { " (primary)" } else { "" };
        write!(
            f,
            "{} [{}] {}x{}{}",
            self.name, self.id, self.width, self.height, primary
        )
    }
}

#[derive(Debug, Clone)]
pub struct VideoBackendCapabilities {
    pub screen_capture: bool,
    pub window_capture: bool,
    pub note: &'static str,
}

#[derive(Debug, Clone)]
pub enum VideoTarget {
    File(PathBuf),
}

#[derive(Debug, Clone)]
pub enum VideoScope {
    Display,
}

#[derive(Debug, Clone)]
pub struct VideoCaptureRequest {
    pub scope: VideoScope,
    pub target: VideoTarget,
    pub duration_secs: Option<u32>,
    pub fps: Option<u32>,
    pub display_hint: Option<String>,
    pub stop_flag: Option<Arc<AtomicBool>>,
}

#[derive(Debug, Clone)]
pub struct VideoCaptureReport {
    pub frames_captured: usize,
    pub fps: u32,
    pub selected_display: DisplayInfo,
    pub output_path: PathBuf,
    pub backend_note: String,
}

#[derive(Debug, Error)]
pub enum VideoError {
    #[error("{0}")]
    Message(String),

    #[error("feature not yet implemented on this backend: {feature}")]
    NotImplemented { feature: &'static str },
}

pub trait VideoBackend: Send + Sync {
    fn backend_name(&self) -> &'static str;
    fn capabilities(&self) -> VideoBackendCapabilities;
    fn list_displays(&self) -> Result<Vec<DisplayInfo>, VideoError>;
    fn capture_blocking(
        &self,
        request: &VideoCaptureRequest,
    ) -> Result<VideoCaptureReport, VideoError>;
}
