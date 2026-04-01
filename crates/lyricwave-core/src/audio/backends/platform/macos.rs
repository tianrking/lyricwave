use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;

use serde_json::Value;

use crate::audio::{
    AudioError, CaptureFormat, CaptureReport, CaptureRequest, CaptureScope, CaptureTarget,
    InputDeviceInfo, ProcessSelector,
};

pub fn capability_note() -> &'static str {
    "macOS supports per-app capture via ScreenCaptureKit helper (requires Screen Recording permission)."
}

pub fn supports_per_app_capture() -> bool {
    true
}

pub fn capture_processes(request: &CaptureRequest) -> Result<CaptureReport, AudioError> {
    let selectors = match &request.scope {
        CaptureScope::Processes(items) if !items.is_empty() => items.as_slice(),
        CaptureScope::Processes(_) => {
            return Err(AudioError::Message(
                "process capture requires at least one selector".to_string(),
            ));
        }
        CaptureScope::System => {
            return Err(AudioError::Message(
                "internal error: process capture called with system scope".to_string(),
            ));
        }
    };

    let out_path = match &request.target {
        CaptureTarget::File(path) => path,
        CaptureTarget::StdoutPcm => {
            return Err(AudioError::Message(
                "macOS process capture currently supports file target only".to_string(),
            ));
        }
    };
    if !matches!(request.format, CaptureFormat::Wav) {
        return Err(AudioError::Message(
            "macOS process capture currently supports WAV output only".to_string(),
        ));
    }

    if request.duration_secs.is_none() {
        return Err(AudioError::Message(
            "macOS process capture currently requires --seconds".to_string(),
        ));
    }

    let helper = ensure_helper_binary()?;
    let sample_rate = request.sample_rate.unwrap_or(48000);
    let channels = request.channels.unwrap_or(2);

    let mut cmd = Command::new(&helper);
    cmd.arg("--out")
        .arg(out_path)
        .arg("--sample-rate")
        .arg(sample_rate.to_string())
        .arg("--channels")
        .arg(channels.to_string());

    if let Some(secs) = request.duration_secs {
        cmd.arg("--seconds").arg(secs.to_string());
    }
    for selector in selectors {
        match selector {
            ProcessSelector::Pid(pid) => {
                cmd.arg("--pid").arg(pid.to_string());
            }
            ProcessSelector::NameContains(name) => {
                cmd.arg("--name").arg(name);
            }
        }
    }

    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = cmd
        .spawn()
        .map_err(|e| AudioError::Message(format!("failed to start macOS SC helper: {e}")))?;

    if let Some(flag) = &request.stop_flag {
        while !flag.load(Ordering::Relaxed) {
            if let Some(status) = child
                .try_wait()
                .map_err(|e| AudioError::Message(format!("failed while waiting helper: {e}")))?
            {
                if !status.success() {
                    let out = child.wait_with_output().map_err(|e| {
                        AudioError::Message(format!("failed to collect helper output: {e}"))
                    })?;
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    return Err(AudioError::Message(format!(
                        "macOS SC helper failed early: {stderr}"
                    )));
                }
                break;
            }
            thread::sleep(Duration::from_millis(50));
        }
    }

    let output = child
        .wait_with_output()
        .map_err(|e| AudioError::Message(format!("failed waiting helper output: {e}")))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AudioError::Message(format!(
            "macOS SC helper failed: {}",
            stderr.trim()
        )));
    }

    let json: Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| AudioError::Message(format!("invalid helper json output: {e}")))?;

    let captured_samples = json
        .get("captured_samples")
        .and_then(Value::as_u64)
        .unwrap_or(0) as usize;
    let sample_rate = json
        .get("sample_rate")
        .and_then(Value::as_u64)
        .unwrap_or(sample_rate as u64) as u32;
    let channels = json
        .get("channels")
        .and_then(Value::as_u64)
        .unwrap_or(channels as u64) as u16;
    let matched_processes = json
        .get("matched_processes")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(Value::as_str)
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    Ok(CaptureReport {
        captured_samples,
        sample_rate,
        channels,
        selected_input_device: InputDeviceInfo {
            id: "screencapturekit-app-audio".to_string(),
            name: "ScreenCaptureKit App Audio".to_string(),
            is_default: false,
            loopback_score: 100,
            is_loopback_candidate: true,
        },
        selection_reason: "selected macOS ScreenCaptureKit app-audio path".to_string(),
        matched_processes,
    })
}

fn ensure_helper_binary() -> Result<PathBuf, AudioError> {
    let cache_dir = std::env::temp_dir()
        .join("lyricwave")
        .join("macos_sc_helper");
    std::fs::create_dir_all(&cache_dir)
        .map_err(|e| AudioError::Message(format!("failed to create helper cache dir: {e}")))?;

    let source_path = cache_dir.join("sc_process_audio.swift");
    let bin_path = cache_dir.join("sc_process_audio");
    let source = include_str!("macos_sc_capture.swift");

    let should_write_source = match std::fs::read_to_string(&source_path) {
        Ok(existing) => existing != source,
        Err(_) => true,
    };
    if should_write_source {
        std::fs::write(&source_path, source)
            .map_err(|e| AudioError::Message(format!("failed to write helper source: {e}")))?;
    }

    let should_build = if !bin_path.exists() {
        true
    } else {
        let src_m = modified_time(&source_path).unwrap_or(std::time::SystemTime::UNIX_EPOCH);
        let bin_m = modified_time(&bin_path).unwrap_or(std::time::SystemTime::UNIX_EPOCH);
        src_m > bin_m
    };
    if !should_build {
        return Ok(bin_path);
    }

    let output = Command::new("xcrun")
        .args([
            "swiftc",
            "-parse-as-library",
            "-O",
            source_path.to_string_lossy().as_ref(),
            "-o",
        ])
        .arg(&bin_path)
        .output()
        .map_err(|e| AudioError::Message(format!("failed to run swiftc: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AudioError::Message(format!(
            "failed to compile ScreenCaptureKit helper: {}",
            stderr.trim()
        )));
    }

    Ok(bin_path)
}

fn modified_time(path: &Path) -> Option<std::time::SystemTime> {
    std::fs::metadata(path).ok()?.modified().ok()
}
