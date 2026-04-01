use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use anyhow::Result;
use lyricwave_core::audio::{
    AudioBackend, CaptureFormat, CaptureReport, CaptureRequest, CaptureScope, CaptureTarget,
    ProcessSelector,
};

#[allow(clippy::too_many_arguments)]
pub fn system(
    backend: &dyn AudioBackend,
    out: Option<PathBuf>,
    stdout: bool,
    seconds: Option<u32>,
    sample_rate: Option<u32>,
    channels: Option<u16>,
    format: CaptureFormat,
    input_device: Option<String>,
    prefer_loopback: bool,
) -> Result<CaptureReport> {
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

        eprintln!("recording... press Enter (or Ctrl+C) to stop");
        Some(flag)
    } else {
        None
    };

    let target = if stdout {
        CaptureTarget::StdoutPcm
    } else {
        let path =
            out.ok_or_else(|| anyhow::anyhow!("--out is required when --stdout is not set"))?;
        CaptureTarget::File(path)
    };

    let request = CaptureRequest {
        scope: CaptureScope::System,
        target,
        duration_secs: seconds,
        sample_rate,
        channels,
        format,
        input_device_hint: input_device,
        prefer_loopback,
        stop_flag,
    };
    Ok(backend.capture_blocking(&request)?)
}

#[allow(clippy::too_many_arguments)]
pub fn app(
    backend: &dyn AudioBackend,
    out: PathBuf,
    seconds: Option<u32>,
    sample_rate: Option<u32>,
    channels: Option<u16>,
    format: CaptureFormat,
    pids: Vec<u32>,
    names: Vec<String>,
) -> Result<CaptureReport> {
    let mut selectors = Vec::new();
    selectors.extend(pids.into_iter().map(ProcessSelector::Pid));
    selectors.extend(names.into_iter().map(ProcessSelector::NameContains));

    if selectors.is_empty() {
        return Err(anyhow::anyhow!(
            "capture app requires at least one selector: --pid or --name"
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

        eprintln!("recording selected app audio... press Enter (or Ctrl+C) to stop");
        Some(flag)
    } else {
        None
    };

    let request = CaptureRequest {
        scope: CaptureScope::Processes(selectors),
        target: CaptureTarget::File(out),
        duration_secs: seconds,
        sample_rate,
        channels,
        format,
        input_device_hint: None,
        prefer_loopback: false,
        stop_flag,
    };

    Ok(backend.capture_blocking(&request)?)
}
