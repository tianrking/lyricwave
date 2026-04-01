use std::thread;

use thiserror::Error;

use crate::audio::{AudioBackend, AudioError, CaptureReport, CaptureRequest};
use crate::visual::{VisualBackend, VisualCaptureReport, VisualCaptureRequest, VisualError};

#[derive(Debug, Clone)]
pub struct CompositionRequest {
    pub audio: Option<CaptureRequest>,
    pub visual: Option<VisualCaptureRequest>,
}

#[derive(Debug, Clone)]
pub struct CompositionReport {
    pub audio: Option<CaptureReport>,
    pub visual: Option<VisualCaptureReport>,
}

#[derive(Debug, Error)]
pub enum CompositionError {
    #[error("invalid request: {0}")]
    InvalidRequest(String),

    #[error("audio capture failed: {0}")]
    Audio(#[from] AudioError),

    #[error("visual capture failed: {0}")]
    Visual(#[from] VisualError),

    #[error("composition thread failed: {0}")]
    Thread(String),
}

pub struct CompositionCoordinator<'a> {
    audio_backend: Option<&'a dyn AudioBackend>,
    visual_backend: Option<&'a dyn VisualBackend>,
}

impl<'a> CompositionCoordinator<'a> {
    pub fn new(
        audio_backend: Option<&'a dyn AudioBackend>,
        visual_backend: Option<&'a dyn VisualBackend>,
    ) -> Self {
        Self {
            audio_backend,
            visual_backend,
        }
    }

    pub fn run_blocking(
        &self,
        request: CompositionRequest,
    ) -> Result<CompositionReport, CompositionError> {
        if request.audio.is_none() && request.visual.is_none() {
            return Err(CompositionError::InvalidRequest(
                "at least one of audio/visual must be requested".to_string(),
            ));
        }

        match (request.audio, request.visual) {
            (Some(audio_req), Some(visual_req)) => {
                let audio_backend = self.audio_backend.ok_or_else(|| {
                    CompositionError::InvalidRequest("audio backend is not configured".to_string())
                })?;
                let visual_backend = self.visual_backend.ok_or_else(|| {
                    CompositionError::InvalidRequest("visual backend is not configured".to_string())
                })?;

                thread::scope(|scope| {
                    let audio_handle = scope.spawn(|| audio_backend.capture_blocking(&audio_req));
                    let visual_handle =
                        scope.spawn(|| visual_backend.capture_blocking(&visual_req));

                    let audio = audio_handle.join().map_err(|_| {
                        CompositionError::Thread("audio thread panicked".to_string())
                    })??;
                    let visual = visual_handle.join().map_err(|_| {
                        CompositionError::Thread("visual thread panicked".to_string())
                    })??;

                    Ok(CompositionReport {
                        audio: Some(audio),
                        visual: Some(visual),
                    })
                })
            }
            (Some(audio_req), None) => {
                let audio_backend = self.audio_backend.ok_or_else(|| {
                    CompositionError::InvalidRequest("audio backend is not configured".to_string())
                })?;
                Ok(CompositionReport {
                    audio: Some(audio_backend.capture_blocking(&audio_req)?),
                    visual: None,
                })
            }
            (None, Some(visual_req)) => {
                let visual_backend = self.visual_backend.ok_or_else(|| {
                    CompositionError::InvalidRequest("visual backend is not configured".to_string())
                })?;
                Ok(CompositionReport {
                    audio: None,
                    visual: Some(visual_backend.capture_blocking(&visual_req)?),
                })
            }
            (None, None) => Err(CompositionError::InvalidRequest(
                "at least one of audio/visual must be requested".to_string(),
            )),
        }
    }
}
