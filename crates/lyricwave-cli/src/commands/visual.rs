use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use anyhow::Result;
use lyricwave_core::visual::{
    ActiveVisualProcessInfo, VisualBackend, VisualCaptureRequest, VisualProcessSelector,
    VisualScope, VisualTarget, build_visual_backend, visual_backends,
};

pub fn list_backends() {
    println!("visual_backends:");
    for backend in visual_backends() {
        println!("- id={} note={}", backend.id, backend.note);
    }
}

pub fn list_displays(backend: &dyn VisualBackend) -> Result<()> {
    let displays = backend.list_displays()?;
    let caps = backend.capabilities();

    println!("backend: {}", backend.backend_name());
    println!(
        "capabilities: screen_capture={}, window_capture={}, per_app_capture={}, note={}",
        caps.screen_capture, caps.window_capture, caps.per_app_capture, caps.note
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

pub fn system(
    backend: &dyn VisualBackend,
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

        eprintln!("recording system visual... press Enter (or Ctrl+C) to stop");
        Some(flag)
    } else {
        None
    };

    let report = backend.capture_blocking(&VisualCaptureRequest {
        scope: VisualScope::System,
        target: VisualTarget::File(out.clone()),
        duration_secs: seconds,
        fps,
        display_hint: display,
        stop_flag,
    })?;

    eprintln!(
        "captured {} frames @ {}fps from '{}' out={}, matched={}",
        report.frames_captured,
        report.fps,
        report.selected_display.name,
        report.output_path.display(),
        report.matched_processes.join("; ")
    );

    Ok(())
}

pub fn app(
    backend: &dyn VisualBackend,
    out: PathBuf,
    seconds: Option<u32>,
    fps: Option<u32>,
    pids: Vec<u32>,
    names: Vec<String>,
) -> Result<()> {
    let mut selectors = Vec::new();
    selectors.extend(pids.into_iter().map(VisualProcessSelector::Pid));
    selectors.extend(names.into_iter().map(VisualProcessSelector::NameContains));

    if selectors.is_empty() {
        return Err(anyhow::anyhow!(
            "visual app requires at least one selector: --pid or --name"
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

        eprintln!("recording selected app visual... press Enter (or Ctrl+C) to stop");
        Some(flag)
    } else {
        None
    };

    let report = backend.capture_blocking(&VisualCaptureRequest {
        scope: VisualScope::Processes(selectors),
        target: VisualTarget::File(out),
        duration_secs: seconds,
        fps,
        display_hint: None,
        stop_flag,
    })?;

    eprintln!(
        "captured {} frames @ {}fps from '{}' out={}, matched={}",
        report.frames_captured,
        report.fps,
        report.selected_display.name,
        report.output_path.display(),
        report.matched_processes.join("; ")
    );

    Ok(())
}

pub fn apps_list(backend: &dyn VisualBackend) -> Result<()> {
    let apps = backend.list_active_visual_processes()?;
    if apps.is_empty() {
        println!("no active visual processes detected");
        return Ok(());
    }

    for app in apps {
        println!("- {}", app);
    }
    Ok(())
}

pub fn apps_split(
    backend: &dyn VisualBackend,
    out_dir: PathBuf,
    seconds: u32,
    fps: Option<u32>,
    pids: Vec<u32>,
    names: Vec<String>,
    all_active: bool,
) -> Result<()> {
    std::fs::create_dir_all(&out_dir)?;
    let active = backend.list_active_visual_processes()?;
    let selected = select_processes(&active, pids, names, all_active)?;

    if selected.is_empty() {
        return Err(anyhow::anyhow!(
            "no target processes selected; use --all-active or --pid/--name"
        ));
    }

    for (pid, name) in selected {
        let file_name = format!("{}-{}.mp4", sanitize_name(&name), pid);
        let file_path = out_dir.join(file_name);

        app(
            backend,
            file_path.clone(),
            Some(seconds),
            fps,
            vec![pid],
            vec![],
        )?;
        println!("- saved {}", file_path.display());
    }

    Ok(())
}

fn select_processes(
    active: &[ActiveVisualProcessInfo],
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

pub fn build_backend(backend_id: &str) -> Result<Box<dyn VisualBackend>> {
    build_visual_backend(backend_id)
        .map_err(anyhow::Error::msg)
        .map_err(|e| anyhow::anyhow!("failed to initialize visual backend '{}': {e}", backend_id))
}
