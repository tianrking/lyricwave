use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, Result};
use lyricwave_core::audio::{AudioBackend, CaptureFormat, CaptureRequest, CaptureTarget};

#[allow(clippy::too_many_arguments)]
pub fn system(
    backend: &dyn AudioBackend,
    out: Option<PathBuf>,
    stdout: bool,
    seconds: Option<u32>,
    sample_rate: Option<u32>,
    channels: Option<u16>,
    format: CaptureFormat,
    ffmpeg_bin: String,
    input_device: Option<String>,
) -> Result<()> {
    let target = if stdout {
        CaptureTarget::StdoutPcm
    } else {
        let path = out.context("--out is required when --stdout is not set")?;
        CaptureTarget::File(path)
    };

    let request = CaptureRequest {
        target,
        duration_secs: seconds,
        sample_rate,
        channels,
        format,
        ffmpeg_bin,
        input_device_hint: input_device,
    };

    let spec = backend.build_capture_command(&request)?;
    eprintln!("running: {} {}", spec.program, spec.args.join(" "));

    let status = Command::new(&spec.program)
        .args(&spec.args)
        .status()
        .with_context(|| format!("failed to start {}", spec.program))?;

    if !status.success() {
        anyhow::bail!("capture process exited with status: {status}");
    }

    Ok(())
}
