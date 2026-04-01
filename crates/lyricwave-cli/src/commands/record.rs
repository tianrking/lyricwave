use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use anyhow::Result;
use lyricwave_core::audio::{
    CaptureFormat, CaptureRequest, CaptureScope, CaptureTarget, build_audio_backend,
};
use lyricwave_core::recording::{RecordingCoordinator, RecordingRequest};
use lyricwave_core::video::{VideoCaptureRequest, VideoScope, VideoTarget, build_video_backend};

#[allow(clippy::too_many_arguments)]
pub fn run(
    audio_backend_id: &str,
    video_backend_id: &str,
    audio_out: Option<PathBuf>,
    video_out: Option<PathBuf>,
    seconds: Option<u32>,
    sample_rate: Option<u32>,
    channels: Option<u16>,
    input_device: Option<String>,
    no_prefer_loopback: bool,
    fps: Option<u32>,
    display: Option<String>,
) -> Result<()> {
    if audio_out.is_none() && video_out.is_none() {
        return Err(anyhow::anyhow!(
            "record run requires at least one output: --audio-out and/or --video-out"
        ));
    }

    let stop_flag = if seconds.is_none() {
        let flag = Arc::new(AtomicBool::new(false));

        let enter_flag = Arc::clone(&flag);
        thread::spawn(move || {
            let mut line = String::new();
            let _ = std::io::stdin().read_line(&mut line);
            enter_flag.store(true, Ordering::Relaxed);
        });

        let ctrlc_flag = Arc::clone(&flag);
        let _ = ctrlc::set_handler(move || {
            ctrlc_flag.store(true, Ordering::Relaxed);
        });

        eprintln!("recording session... press Enter (or Ctrl+C) to stop");
        Some(flag)
    } else {
        None
    };

    let audio_backend = if audio_out.is_some() {
        Some(
            build_audio_backend(audio_backend_id)
                .map_err(anyhow::Error::msg)
                .map_err(|e| {
                    anyhow::anyhow!("failed to init audio backend '{}': {e}", audio_backend_id)
                })?,
        )
    } else {
        None
    };

    let video_backend = if video_out.is_some() {
        Some(
            build_video_backend(video_backend_id)
                .map_err(anyhow::Error::msg)
                .map_err(|e| {
                    anyhow::anyhow!("failed to init video backend '{}': {e}", video_backend_id)
                })?,
        )
    } else {
        None
    };

    let request = RecordingRequest {
        audio: audio_out.map(|path| CaptureRequest {
            scope: CaptureScope::System,
            target: CaptureTarget::File(path),
            duration_secs: seconds,
            sample_rate,
            channels,
            format: CaptureFormat::Wav,
            input_device_hint: input_device,
            prefer_loopback: !no_prefer_loopback,
            stop_flag: stop_flag.clone(),
        }),
        video: video_out.map(|path| VideoCaptureRequest {
            scope: VideoScope::Display,
            target: VideoTarget::File(path),
            duration_secs: seconds,
            fps,
            display_hint: display,
            stop_flag,
        }),
    };

    let coordinator = RecordingCoordinator::new(audio_backend.as_deref(), video_backend.as_deref());
    let report = coordinator
        .run_blocking(request)
        .map_err(|e| anyhow::anyhow!("record session failed: {e}"))?;

    if let Some(audio) = report.audio {
        eprintln!(
            "audio done: {} samples @ {}Hz {}ch, input='{}', reason={}",
            audio.captured_samples,
            audio.sample_rate,
            audio.channels,
            audio.selected_input_device.name,
            audio.selection_reason
        );
    }

    if let Some(video) = report.video {
        eprintln!(
            "video done: {} frames @ {}fps, display='{}', out={}",
            video.frames_captured,
            video.fps,
            video.selected_display.name,
            video.output_path.display()
        );
    }

    Ok(())
}
