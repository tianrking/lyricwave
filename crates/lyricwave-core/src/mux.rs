use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Clone, Copy)]
pub enum MuxContainer {
    Mp4,
    Mkv,
}

#[derive(Debug, Clone)]
pub struct MuxRequest {
    pub audio_path: Option<PathBuf>,
    pub visual_path: Option<PathBuf>,
    pub output_path: PathBuf,
    pub container: MuxContainer,
}

#[derive(Debug, Clone)]
pub struct MuxReport {
    pub output_path: PathBuf,
    pub container: MuxContainer,
}

#[derive(Debug, Error)]
pub enum MuxError {
    #[error("{0}")]
    Message(String),

    #[error("feature not yet implemented: {feature}")]
    NotImplemented { feature: &'static str },
}

pub trait Muxer: Send + Sync {
    fn mux(&self, request: &MuxRequest) -> Result<MuxReport, MuxError>;
}

#[derive(Default)]
pub struct NativeMuxer;

impl Muxer for NativeMuxer {
    fn mux(&self, _request: &MuxRequest) -> Result<MuxReport, MuxError> {
        Err(MuxError::NotImplemented {
            feature: "native A/V mux (mp4/mkv)",
        })
    }
}
