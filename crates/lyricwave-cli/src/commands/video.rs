use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use anyhow::Result;
use lyricwave_core::video::{
    VideoBackend, VideoCaptureRequest, VideoScope, VideoTarget, build_video_backend, video_backends,
};

pub fn list_backends() {
    println!("video_backends:");
    for backend in video_backends() {
        println!("- id={} note={}", backend.id, backend.note);
    }
}

pub fn list_displays(backend: &dyn VideoBackend) -> Result<()> {
    let displays = backend.list_displays()?;
    let caps = backend.capabilities();

    println!("backend: {}", backend.backend_name());
    println!(
        "capabilities: screen_capture={}, window_capture={}, note={}",
        caps.screen_capture, caps.window_capture, caps.note
    );

    if displays.is_empty() {
        println!("no displays found");
        return Ok(());
    }

    for display in displays {
        println!("- {display}");
    }

    Ok(())
}

pub fn capture_screen(
    backend: &dyn VideoBackend,
    out: PathBuf,
    seconds: Option<u32>,
    fps: Option<u32>,
    display: Option<String>,
) -> Result<()> {
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

        eprintln!("recording screen... press Enter (or Ctrl+C) to stop");
        Some(flag)
    } else {
        None
    };

    let report = backend.capture_blocking(&VideoCaptureRequest {
        scope: VideoScope::Display,
        target: VideoTarget::File(out.clone()),
        duration_secs: seconds,
        fps,
        display_hint: display,
        stop_flag,
    })?;

    eprintln!(
        "captured {} frames @ {}fps from '{}', out={}, note={}",
        report.frames_captured,
        report.fps,
        report.selected_display.name,
        report.output_path.display(),
        report.backend_note
    );

    Ok(())
}

pub fn build_backend(backend_id: &str) -> Result<Box<dyn VideoBackend>> {
    build_video_backend(backend_id)
        .map_err(anyhow::Error::msg)
        .map_err(|e| anyhow::anyhow!("failed to initialize video backend '{}': {e}", backend_id))
}
