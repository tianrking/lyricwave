use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
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

pub fn apps_list(backend: &dyn AudioBackend) -> Result<()> {
    let apps = backend.list_active_audio_processes()?;
    if apps.is_empty() {
        println!("no active audio processes detected");
        return Ok(());
    }

    for app in apps {
        println!("- {}", app);
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn apps_split(
    backend: &dyn AudioBackend,
    out_dir: PathBuf,
    seconds: u32,
    sample_rate: Option<u32>,
    channels: Option<u16>,
    format: CaptureFormat,
    pids: Vec<u32>,
    names: Vec<String>,
    all_active: bool,
    mix_out: Option<PathBuf>,
) -> Result<()> {
    if !matches!(format, CaptureFormat::Wav) {
        return Err(anyhow::anyhow!(
            "apps-split currently supports WAV format only"
        ));
    }

    std::fs::create_dir_all(&out_dir)?;
    let active = backend.list_active_audio_processes()?;
    let selected = select_processes(&active, pids, names, all_active)?;

    if selected.is_empty() {
        return Err(anyhow::anyhow!(
            "no target processes selected; use --all-active or --pid/--name"
        ));
    }

    let mut split_files = Vec::new();
    for (pid, name) in selected {
        let file_name = format!("{}-{}.wav", sanitize_name(&name), pid);
        let file_path = out_dir.join(file_name);

        let report = app(
            backend,
            file_path.clone(),
            Some(seconds),
            sample_rate,
            channels,
            format,
            vec![pid],
            vec![],
        )?;
        println!(
            "- saved {} ({} samples @ {}Hz {}ch)",
            file_path.display(),
            report.captured_samples,
            report.sample_rate,
            report.channels
        );
        split_files.push(file_path);
    }

    if let Some(mix_path) = mix_out {
        mix_wav_files(&split_files, &mix_path)?;
        println!("- mixed file {}", mix_path.display());
    }

    Ok(())
}

fn select_processes(
    active: &[lyricwave_core::audio::ActiveAudioProcessInfo],
    pids: Vec<u32>,
    names: Vec<String>,
    all_active: bool,
) -> Result<Vec<(u32, String)>> {
    let mut selected = BTreeMap::<u32, String>::new();

    if all_active {
        for app in active {
            selected.insert(app.pid, app.name.clone());
        }
    }

    for pid in pids {
        if let Some(app) = active.iter().find(|a| a.pid == pid) {
            selected.insert(pid, app.name.clone());
        } else {
            selected.insert(pid, format!("pid-{pid}"));
        }
    }

    let lowered_queries: Vec<String> = names.into_iter().map(|n| n.to_lowercase()).collect();
    for app in active {
        let lowered = app.name.to_lowercase();
        if lowered_queries.iter().any(|q| lowered.contains(q)) {
            selected.insert(app.pid, app.name.clone());
        }
    }

    Ok(selected.into_iter().collect())
}

fn sanitize_name(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
            out.push(ch);
        } else if ch.is_whitespace() {
            out.push('_');
        }
    }
    let trimmed = out.trim_matches('_');
    if trimmed.is_empty() {
        "app".to_string()
    } else {
        trimmed.to_string()
    }
}

fn mix_wav_files(inputs: &[PathBuf], out: &Path) -> Result<()> {
    if inputs.is_empty() {
        return Err(anyhow::anyhow!("no split wav files to mix"));
    }

    let mut spec: Option<hound::WavSpec> = None;
    let mut tracks = Vec::<Vec<i16>>::new();
    for path in inputs {
        let mut r = hound::WavReader::open(path)
            .map_err(|e| anyhow::anyhow!("open wav {} failed: {e}", path.display()))?;
        let rs = r.spec();
        if let Some(s) = spec {
            if rs.sample_rate != s.sample_rate || rs.channels != s.channels {
                return Err(anyhow::anyhow!(
                    "split wav formats differ; cannot mix safely"
                ));
            }
        } else {
            spec = Some(rs);
        }

        let mut buf = Vec::with_capacity(r.duration() as usize);
        for sample in r.samples::<i16>() {
            buf.push(sample.unwrap_or(0));
        }
        tracks.push(buf);
    }
    let spec = spec.ok_or_else(|| anyhow::anyhow!("failed to read wav spec from split files"))?;
    let samples_per_file = tracks.iter().map(Vec::len).max().unwrap_or(0);

    let mut writer = hound::WavWriter::create(out, spec)?;
    for idx in 0..samples_per_file {
        let mut sum = 0f32;
        let mut count = 0f32;
        for t in &tracks {
            if idx < t.len() {
                sum += t[idx] as f32 / i16::MAX as f32;
                count += 1.0;
            }
        }
        let mixed = if count > 0.0 { sum / count } else { 0.0 };
        let s = (mixed.clamp(-1.0, 1.0) * i16::MAX as f32).round() as i16;
        writer.write_sample(s)?;
    }
    writer.finalize()?;
    Ok(())
}
