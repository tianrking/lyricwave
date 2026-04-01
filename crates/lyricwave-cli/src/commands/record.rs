use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use anyhow::Result;
use lyricwave_core::audio::{
    ActiveAudioProcessInfo, CaptureFormat, CaptureRequest, CaptureScope, CaptureTarget,
    ProcessSelector, build_audio_backend,
};
use lyricwave_core::composition::{CompositionCoordinator, CompositionRequest};
use lyricwave_core::visual::{
    ActiveVisualProcessInfo, VisualCaptureRequest, VisualProcessSelector, VisualScope,
    VisualTarget, build_visual_backend,
};

#[allow(clippy::too_many_arguments)]
pub fn run(
    audio_backend_id: &str,
    visual_backend_id: &str,
    audio_out: Option<PathBuf>,
    visual_out: Option<PathBuf>,
    seconds: Option<u32>,
    sample_rate: Option<u32>,
    channels: Option<u16>,
    input_device: Option<String>,
    no_prefer_loopback: bool,
    fps: Option<u32>,
    display: Option<String>,
) -> Result<()> {
    if audio_out.is_none() && visual_out.is_none() {
        return Err(anyhow::anyhow!(
            "record run requires at least one output: --audio-out and/or --visual-out"
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

    let visual_backend = if visual_out.is_some() {
        Some(
            build_visual_backend(visual_backend_id)
                .map_err(anyhow::Error::msg)
                .map_err(|e| {
                    anyhow::anyhow!("failed to init visual backend '{}': {e}", visual_backend_id)
                })?,
        )
    } else {
        None
    };

    let request = CompositionRequest {
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
        visual: visual_out.map(|path| VisualCaptureRequest {
            scope: VisualScope::System,
            target: VisualTarget::File(path),
            duration_secs: seconds,
            fps,
            display_hint: display,
            stop_flag,
        }),
    };

    let coordinator =
        CompositionCoordinator::new(audio_backend.as_deref(), visual_backend.as_deref());
    let report = coordinator
        .run_blocking(request)
        .map_err(|e| anyhow::anyhow!("composition session failed: {e}"))?;

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

    if let Some(visual) = report.visual {
        eprintln!(
            "visual done: {} frames @ {}fps, display='{}', out={}",
            visual.frames_captured,
            visual.fps,
            visual.selected_display.name,
            visual.output_path.display()
        );
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn run_app(
    audio_backend_id: &str,
    visual_backend_id: &str,
    audio_out: Option<PathBuf>,
    visual_out: Option<PathBuf>,
    seconds: Option<u32>,
    sample_rate: Option<u32>,
    channels: Option<u16>,
    fps: Option<u32>,
    pids: Vec<u32>,
    names: Vec<String>,
) -> Result<()> {
    if audio_out.is_none() && visual_out.is_none() {
        return Err(anyhow::anyhow!(
            "record app requires at least one output: --audio-out and/or --visual-out"
        ));
    }

    let mut audio_selectors = Vec::new();
    audio_selectors.extend(pids.iter().copied().map(ProcessSelector::Pid));
    audio_selectors.extend(names.iter().cloned().map(ProcessSelector::NameContains));
    if audio_selectors.is_empty() {
        return Err(anyhow::anyhow!(
            "record app requires at least one selector: --pid or --name"
        ));
    }

    let mut visual_selectors = Vec::new();
    visual_selectors.extend(pids.into_iter().map(VisualProcessSelector::Pid));
    visual_selectors.extend(names.into_iter().map(VisualProcessSelector::NameContains));

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

        eprintln!("recording app composition... press Enter (or Ctrl+C) to stop");
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

    let visual_backend = if visual_out.is_some() {
        Some(
            build_visual_backend(visual_backend_id)
                .map_err(anyhow::Error::msg)
                .map_err(|e| {
                    anyhow::anyhow!("failed to init visual backend '{}': {e}", visual_backend_id)
                })?,
        )
    } else {
        None
    };

    let request = CompositionRequest {
        audio: audio_out.map(|path| CaptureRequest {
            scope: CaptureScope::Processes(audio_selectors),
            target: CaptureTarget::File(path),
            duration_secs: seconds,
            sample_rate,
            channels,
            format: CaptureFormat::Wav,
            input_device_hint: None,
            prefer_loopback: false,
            stop_flag: stop_flag.clone(),
        }),
        visual: visual_out.map(|path| VisualCaptureRequest {
            scope: VisualScope::Processes(visual_selectors),
            target: VisualTarget::File(path),
            duration_secs: seconds,
            fps,
            display_hint: None,
            stop_flag,
        }),
    };

    let coordinator =
        CompositionCoordinator::new(audio_backend.as_deref(), visual_backend.as_deref());
    let report = coordinator
        .run_blocking(request)
        .map_err(|e| anyhow::anyhow!("composition app session failed: {e}"))?;

    if let Some(audio) = report.audio {
        eprintln!(
            "audio done: {} samples @ {}Hz {}ch, matched={}",
            audio.captured_samples,
            audio.sample_rate,
            audio.channels,
            audio.matched_processes.join("; ")
        );
    }

    if let Some(visual) = report.visual {
        eprintln!(
            "visual done: {} frames @ {}fps, out={}, matched={}",
            visual.frames_captured,
            visual.fps,
            visual.output_path.display(),
            visual.matched_processes.join("; ")
        );
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn run_apps_split(
    audio_backend_id: &str,
    visual_backend_id: &str,
    out_dir: PathBuf,
    seconds: u32,
    sample_rate: Option<u32>,
    channels: Option<u16>,
    fps: Option<u32>,
    pids: Vec<u32>,
    names: Vec<String>,
    all_active: bool,
    no_audio: bool,
    no_visual: bool,
) -> Result<()> {
    if no_audio && no_visual {
        return Err(anyhow::anyhow!(
            "record apps-split cannot disable both streams; remove --no-audio or --no-visual"
        ));
    }

    std::fs::create_dir_all(&out_dir)?;

    let audio_backend = build_audio_backend(audio_backend_id)
        .map_err(anyhow::Error::msg)
        .map_err(|e| anyhow::anyhow!("failed to init audio backend '{}': {e}", audio_backend_id))?;
    let visual_backend = build_visual_backend(visual_backend_id)
        .map_err(anyhow::Error::msg)
        .map_err(|e| {
            anyhow::anyhow!("failed to init visual backend '{}': {e}", visual_backend_id)
        })?;

    let active_audio = audio_backend
        .list_active_audio_processes()
        .unwrap_or_else(|_| vec![]);
    let active_visual = visual_backend
        .list_active_visual_processes()
        .unwrap_or_else(|_| vec![]);

    let selected = select_processes_union(&active_audio, &active_visual, pids, names, all_active);
    if selected.is_empty() {
        return Err(anyhow::anyhow!(
            "no target processes selected; use --all-active or --pid/--name"
        ));
    }

    for (pid, name) in selected {
        let safe = sanitize_name(&name);
        let audio_out = if no_audio {
            None
        } else {
            Some(out_dir.join(format!("{safe}-{pid}.wav")))
        };
        let visual_out = if no_visual {
            None
        } else {
            Some(out_dir.join(format!("{safe}-{pid}.mp4")))
        };

        run_app(
            audio_backend_id,
            visual_backend_id,
            audio_out.clone(),
            visual_out.clone(),
            Some(seconds),
            sample_rate,
            channels,
            fps,
            vec![pid],
            vec![],
        )?;

        if let Some(path) = audio_out {
            println!("- saved audio {}", path.display());
        }
        if let Some(path) = visual_out {
            println!("- saved visual {}", path.display());
        }
    }

    Ok(())
}

fn select_processes_union(
    active_audio: &[ActiveAudioProcessInfo],
    active_visual: &[ActiveVisualProcessInfo],
    pids: Vec<u32>,
    names: Vec<String>,
    all_active: bool,
) -> Vec<(u32, String)> {
    let mut selected = BTreeMap::<u32, String>::new();

    if all_active {
        for app in active_audio {
            selected.insert(app.pid, app.name.clone());
        }
        for app in active_visual {
            selected.entry(app.pid).or_insert_with(|| app.name.clone());
        }
    }

    for pid in pids {
        if let Some(app) = active_audio.iter().find(|a| a.pid == pid) {
            selected.insert(pid, app.name.clone());
        } else if let Some(app) = active_visual.iter().find(|a| a.pid == pid) {
            selected.insert(pid, app.name.clone());
        } else {
            selected.insert(pid, format!("pid-{pid}"));
        }
    }

    let lowered_queries: Vec<String> = names.into_iter().map(|n| n.to_lowercase()).collect();
    if !lowered_queries.is_empty() {
        for app in active_audio {
            if lowered_queries
                .iter()
                .any(|q| app.name.to_lowercase().contains(q))
            {
                selected.insert(app.pid, app.name.clone());
            }
        }
        for app in active_visual {
            if lowered_queries
                .iter()
                .any(|q| app.name.to_lowercase().contains(q))
            {
                selected.entry(app.pid).or_insert_with(|| app.name.clone());
            }
        }
    }

    selected.into_iter().collect()
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
