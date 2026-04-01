use std::thread;

use thiserror::Error;

use crate::audio::{AudioBackend, AudioError, CaptureReport, CaptureRequest};
use crate::video::{VideoBackend, VideoCaptureReport, VideoCaptureRequest, VideoError};

#[derive(Debug, Clone)]
pub struct RecordingRequest {
    pub audio: Option<CaptureRequest>,
    pub video: Option<VideoCaptureRequest>,
}

#[derive(Debug, Clone)]
pub struct RecordingReport {
    pub audio: Option<CaptureReport>,
    pub video: Option<VideoCaptureReport>,
}

#[derive(Debug, Error)]
pub enum RecordingError {
    #[error("invalid request: {0}")]
    InvalidRequest(String),

    #[error("audio capture failed: {0}")]
    Audio(#[from] AudioError),

    #[error("video capture failed: {0}")]
    Video(#[from] VideoError),

    #[error("recording thread failed: {0}")]
    Thread(String),
}

pub struct RecordingCoordinator<'a> {
    audio_backend: Option<&'a dyn AudioBackend>,
    video_backend: Option<&'a dyn VideoBackend>,
}

impl<'a> RecordingCoordinator<'a> {
    pub fn new(
        audio_backend: Option<&'a dyn AudioBackend>,
        video_backend: Option<&'a dyn VideoBackend>,
    ) -> Self {
        Self {
            audio_backend,
            video_backend,
        }
    }

    pub fn run_blocking(
        &self,
        request: RecordingRequest,
    ) -> Result<RecordingReport, RecordingError> {
        if request.audio.is_none() && request.video.is_none() {
            return Err(RecordingError::InvalidRequest(
                "at least one of audio/video must be requested".to_string(),
            ));
        }

        match (request.audio, request.video) {
            (Some(audio_req), Some(video_req)) => {
                let audio_backend = self.audio_backend.ok_or_else(|| {
                    RecordingError::InvalidRequest("audio backend is not configured".to_string())
                })?;
                let video_backend = self.video_backend.ok_or_else(|| {
                    RecordingError::InvalidRequest("video backend is not configured".to_string())
                })?;

                thread::scope(|scope| {
                    let audio_handle = scope.spawn(|| audio_backend.capture_blocking(&audio_req));
                    let video_handle = scope.spawn(|| video_backend.capture_blocking(&video_req));

                    let audio = audio_handle.join().map_err(|_| {
                        RecordingError::Thread("audio thread panicked".to_string())
                    })??;
                    let video = video_handle.join().map_err(|_| {
                        RecordingError::Thread("video thread panicked".to_string())
                    })??;

                    Ok(RecordingReport {
                        audio: Some(audio),
                        video: Some(video),
                    })
                })
            }
            (Some(audio_req), None) => {
                let audio_backend = self.audio_backend.ok_or_else(|| {
                    RecordingError::InvalidRequest("audio backend is not configured".to_string())
                })?;
                Ok(RecordingReport {
                    audio: Some(audio_backend.capture_blocking(&audio_req)?),
                    video: None,
                })
            }
            (None, Some(video_req)) => {
                let video_backend = self.video_backend.ok_or_else(|| {
                    RecordingError::InvalidRequest("video backend is not configured".to_string())
                })?;
                Ok(RecordingReport {
                    audio: None,
                    video: Some(video_backend.capture_blocking(&video_req)?),
                })
            }
            (None, None) => Err(RecordingError::InvalidRequest(
                "at least one of audio/video must be requested".to_string(),
            )),
        }
    }
}
