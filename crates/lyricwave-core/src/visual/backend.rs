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
pub struct VisualBackendCapabilities {
    pub screen_capture: bool,
    pub window_capture: bool,
    pub per_app_capture: bool,
    pub note: &'static str,
}

#[derive(Debug, Clone)]
pub enum VisualTarget {
    File(PathBuf),
}

#[derive(Debug, Clone)]
pub enum VisualProcessSelector {
    Pid(u32),
    NameContains(String),
}

impl fmt::Display for VisualProcessSelector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pid(pid) => write!(f, "pid:{pid}"),
            Self::NameContains(name) => write!(f, "name:*{name}*"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ActiveVisualProcessInfo {
    pub pid: u32,
    pub name: String,
}

impl fmt::Display for ActiveVisualProcessInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (pid={})", self.name, self.pid)
    }
}

#[derive(Debug, Clone)]
pub enum VisualScope {
    System,
    Processes(Vec<VisualProcessSelector>),
}

#[derive(Debug, Clone)]
pub struct VisualCaptureRequest {
    pub scope: VisualScope,
    pub target: VisualTarget,
    pub duration_secs: Option<u32>,
    pub fps: Option<u32>,
    pub display_hint: Option<String>,
    pub stop_flag: Option<Arc<AtomicBool>>,
}

#[derive(Debug, Clone)]
pub struct VisualCaptureReport {
    pub frames_captured: usize,
    pub fps: u32,
    pub selected_display: DisplayInfo,
    pub output_path: PathBuf,
    pub backend_note: String,
    pub matched_processes: Vec<String>,
}

#[derive(Debug, Error)]
pub enum VisualError {
    #[error("{0}")]
    Message(String),

    #[error("feature not yet implemented on this backend: {feature}")]
    NotImplemented { feature: &'static str },
}

pub trait VisualBackend: Send + Sync {
    fn backend_name(&self) -> &'static str;
    fn capabilities(&self) -> VisualBackendCapabilities;
    fn list_displays(&self) -> Result<Vec<DisplayInfo>, VisualError>;
    fn list_active_visual_processes(&self) -> Result<Vec<ActiveVisualProcessInfo>, VisualError>;
    fn capture_blocking(
        &self,
        request: &VisualCaptureRequest,
    ) -> Result<VisualCaptureReport, VisualError>;
}
