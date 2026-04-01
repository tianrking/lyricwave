use std::collections::{HashSet, VecDeque};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use wasapi::{AudioClient, Direction, SampleType, StreamMode, WaveFormat, initialize_mta};

use crate::audio::{
    ActiveAudioProcessInfo, AudioError, CaptureFormat, CaptureReport, CaptureRequest, CaptureScope,
    CaptureTarget, InputDeviceInfo, ProcessSelector,
};

pub fn capability_note() -> &'static str {
    "Windows supports per-app capture via native WASAPI process loopback (single/multiple process selectors)."
}

pub fn supports_per_app_capture() -> bool {
    true
}

pub fn list_active_audio_processes() -> Result<Vec<ActiveAudioProcessInfo>, AudioError> {
    initialize_mta().ok();

    let enumerator = wasapi::DeviceEnumerator::new()
        .map_err(|e| AudioError::Message(format!("create wasapi enumerator failed: {e}")))?;
    let devices = enumerator
        .get_device_collection(&Direction::Render)
        .map_err(|e| AudioError::Message(format!("enumerate render devices failed: {e}")))?;

    let mut active_pids = HashSet::<u32>::new();
    for device in &devices {
        let dev = match device {
            Ok(d) => d,
            Err(_) => continue,
        };
        let manager = match dev.get_iaudiosessionmanager() {
            Ok(v) => v,
            Err(_) => continue,
        };
        let sessions = match manager.get_audiosessionenumerator() {
            Ok(v) => v,
            Err(_) => continue,
        };
        let count = sessions.get_count().unwrap_or(0);
        for idx in 0..count {
            if let Ok(control) = sessions.get_session(idx)
                && let Ok(pid) = control.get_process_id()
                && pid > 0
            {
                active_pids.insert(pid);
            }
        }
    }

    let names = tasklist_rows()?;
    let mut map = std::collections::BTreeMap::<u32, String>::new();
    for (name, pid) in names {
        if active_pids.contains(&pid) {
            map.entry(pid).or_insert(name);
        }
    }

    Ok(map
        .into_iter()
        .map(|(pid, name)| ActiveAudioProcessInfo { pid, name })
        .collect())
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
        CaptureTarget::File(path) => path.clone(),
        CaptureTarget::StdoutPcm => {
            return Err(AudioError::Message(
                "windows process capture currently supports file target only".to_string(),
            ));
        }
    };

    if !matches!(request.format, CaptureFormat::Wav) {
        return Err(AudioError::Message(
            "windows process capture currently supports WAV output only".to_string(),
        ));
    }

    let sample_rate = request.sample_rate.unwrap_or(48_000);
    let channels = request.channels.unwrap_or(2);
    let pids = resolve_target_pids(selectors)?;
    if pids.is_empty() {
        return Err(AudioError::Message(
            "no processes matched selectors".to_string(),
        ));
    }

    let stop = Arc::new(AtomicBool::new(false));
    let started_at_ms = now_millis();
    let mut worker_handles = Vec::new();
    let mut raw_paths = Vec::new();

    for pid in &pids {
        let raw_path =
            std::env::temp_dir().join(format!("lyricwave-win-pid-{pid}-{}.raw", now_millis()));
        raw_paths.push((*pid, raw_path.clone()));

        let stop_clone = Arc::clone(&stop);
        let pid_copy = *pid;
        worker_handles.push(thread::spawn(move || {
            capture_one_process_to_raw(pid_copy, sample_rate, channels, &raw_path, stop_clone)
        }));
    }

    wait_until_done(request, &stop);
    stop.store(true, Ordering::Relaxed);

    let mut errors = Vec::new();
    for handle in worker_handles {
        match handle.join() {
            Ok(Ok(())) => {}
            Ok(Err(e)) => errors.push(e.to_string()),
            Err(_) => errors.push("capture thread panicked".to_string()),
        }
    }

    if !errors.is_empty() {
        return Err(AudioError::Message(format!(
            "windows process capture failed: {}",
            errors.join("; ")
        )));
    }

    let mut streams = Vec::new();
    for (_pid, raw) in &raw_paths {
        streams.push(read_raw_f32(raw)?);
    }

    let mixed = mix_streams(&streams);
    write_wav_i16(&out_path, sample_rate, channels, &mixed)?;

    for (_pid, raw) in &raw_paths {
        let _ = std::fs::remove_file(raw);
    }
    let ended_at_ms = now_millis();

    Ok(CaptureReport {
        captured_samples: mixed.len(),
        sample_rate,
        channels,
        started_at_ms,
        ended_at_ms,
        selected_input_device: InputDeviceInfo {
            id: "wasapi-process-loopback".to_string(),
            name: "WASAPI Process Loopback".to_string(),
            is_default: false,
            loopback_score: 100,
            is_loopback_candidate: true,
        },
        selection_reason: "selected Windows native WASAPI process loopback".to_string(),
        matched_processes: pids.iter().map(|p| format!("pid={p}")).collect(),
    })
}

fn capture_one_process_to_raw(
    pid: u32,
    sample_rate: u32,
    channels: u16,
    raw_path: &PathBuf,
    stop: Arc<AtomicBool>,
) -> Result<(), AudioError> {
    initialize_mta().ok();

    let sample_rate_usize = usize::try_from(sample_rate).map_err(|_| {
        AudioError::Message(format!("invalid sample_rate for WASAPI: {sample_rate}"))
    })?;
    let channels_usize = usize::from(channels);
    let desired_format = WaveFormat::new(
        32,
        32,
        &SampleType::Float,
        sample_rate_usize,
        channels_usize,
        None,
    );
    let mut audio_client =
        AudioClient::new_application_loopback_client(pid, true).map_err(|e| {
            AudioError::Message(format!("create loopback client for pid {pid} failed: {e}"))
        })?;

    let mode = StreamMode::EventsShared {
        autoconvert: true,
        buffer_duration_hns: 0,
    };
    audio_client
        .initialize_client(&desired_format, &Direction::Capture, &mode)
        .map_err(|e| {
            AudioError::Message(format!("initialize audio client pid {pid} failed: {e}"))
        })?;

    let event = audio_client
        .set_get_eventhandle()
        .map_err(|e| AudioError::Message(format!("set event handle pid {pid} failed: {e}")))?;

    let capture_client = audio_client
        .get_audiocaptureclient()
        .map_err(|e| AudioError::Message(format!("get capture client pid {pid} failed: {e}")))?;

    let mut file = std::fs::File::create(raw_path).map_err(|e| {
        AudioError::Message(format!(
            "create raw file failed {}: {e}",
            raw_path.display()
        ))
    })?;

    let mut queue = VecDeque::<u8>::new();

    audio_client
        .start_stream()
        .map_err(|e| AudioError::Message(format!("start stream pid {pid} failed: {e}")))?;

    while !stop.load(Ordering::Relaxed) {
        let packet = capture_client
            .get_next_packet_size()
            .map_err(|e| AudioError::Message(format!("get packet size pid {pid} failed: {e}")))?
            .unwrap_or(0);

        if packet > 0 {
            capture_client
                .read_from_device_to_deque(&mut queue)
                .map_err(|e| {
                    AudioError::Message(format!("read capture data pid {pid} failed: {e}"))
                })?;
        }

        while queue.len() >= 4096 {
            let mut chunk = vec![0u8; 4096];
            for b in &mut chunk {
                *b = queue
                    .pop_front()
                    .ok_or_else(|| AudioError::Message("capture queue underflow".to_string()))?;
            }
            file.write_all(&chunk)
                .map_err(|e| AudioError::Message(format!("write raw chunk failed: {e}")))?;
        }

        let _ = event.wait_for_event(250);
    }

    audio_client
        .stop_stream()
        .map_err(|e| AudioError::Message(format!("stop stream pid {pid} failed: {e}")))?;

    while !queue.is_empty() {
        let mut chunk = Vec::with_capacity(queue.len());
        while let Some(b) = queue.pop_front() {
            chunk.push(b);
        }
        file.write_all(&chunk)
            .map_err(|e| AudioError::Message(format!("flush raw chunk failed: {e}")))?;
    }

    file.flush()
        .map_err(|e| AudioError::Message(format!("flush raw file failed: {e}")))?;

    Ok(())
}

fn wait_until_done(request: &CaptureRequest, stop: &Arc<AtomicBool>) {
    if let Some(secs) = request.duration_secs {
        thread::sleep(Duration::from_secs(secs as u64));
        return;
    }

    let started = Instant::now();
    loop {
        if stop.load(Ordering::Relaxed) {
            break;
        }
        if let Some(flag) = &request.stop_flag {
            if flag.load(Ordering::Relaxed) {
                break;
            }
        }
        if started.elapsed() > Duration::from_secs(24 * 60 * 60) {
            break;
        }
        thread::sleep(Duration::from_millis(50));
    }
}

fn resolve_target_pids(selectors: &[ProcessSelector]) -> Result<Vec<u32>, AudioError> {
    let mut pids = HashSet::new();

    for selector in selectors {
        if let ProcessSelector::Pid(pid) = selector {
            pids.insert(*pid);
        }
    }

    let name_queries: Vec<String> = selectors
        .iter()
        .filter_map(|s| match s {
            ProcessSelector::NameContains(name) => Some(name.to_lowercase()),
            _ => None,
        })
        .collect();

    if !name_queries.is_empty() {
        for (name, pid) in tasklist_rows()? {
            let lowered = name.to_lowercase();
            if name_queries.iter().any(|q| lowered.contains(q)) {
                pids.insert(pid);
            }
        }
    }

    let mut out = pids.into_iter().collect::<Vec<_>>();
    out.sort_unstable();
    Ok(out)
}

fn tasklist_rows() -> Result<Vec<(String, u32)>, AudioError> {
    let out = Command::new("tasklist")
        .args(["/FO", "CSV", "/NH"])
        .output()
        .map_err(|e| AudioError::Message(format!("run tasklist failed: {e}")))?;

    if !out.status.success() {
        return Err(AudioError::Message(
            "tasklist returned non-zero status".to_string(),
        ));
    }

    let text = String::from_utf8(out.stdout)
        .map_err(|e| AudioError::Message(format!("tasklist output is not utf8: {e}")))?;

    let mut rows = Vec::new();
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let cols = parse_simple_csv_line(line);
        if cols.len() < 2 {
            continue;
        }

        let name = cols[0].clone();
        let pid = cols[1].replace(',', "").parse::<u32>().unwrap_or(0);
        if pid > 0 {
            rows.push((name, pid));
        }
    }

    Ok(rows)
}

fn parse_simple_csv_line(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_quotes = false;

    for ch in line.chars() {
        match ch {
            '"' => {
                in_quotes = !in_quotes;
            }
            ',' if !in_quotes => {
                out.push(cur.trim().trim_matches('"').to_string());
                cur.clear();
            }
            _ => cur.push(ch),
        }
    }

    out.push(cur.trim().trim_matches('"').to_string());
    out
}

fn read_raw_f32(path: &PathBuf) -> Result<Vec<f32>, AudioError> {
    let bytes = std::fs::read(path)
        .map_err(|e| AudioError::Message(format!("read raw {} failed: {e}", path.display())))?;

    if bytes.len() % 4 != 0 {
        return Err(AudioError::Message(format!(
            "raw capture size is not float32-aligned: {}",
            path.display()
        )));
    }

    let mut out = Vec::with_capacity(bytes.len() / 4);
    for chunk in bytes.chunks_exact(4) {
        out.push(f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]));
    }
    Ok(out)
}

fn mix_streams(streams: &[Vec<f32>]) -> Vec<f32> {
    let max_len = streams.iter().map(Vec::len).max().unwrap_or(0);
    if max_len == 0 {
        return Vec::new();
    }

    let mut mixed = vec![0.0f32; max_len];
    let mut counts = vec![0u32; max_len];

    for stream in streams {
        for (i, sample) in stream.iter().enumerate() {
            mixed[i] += *sample;
            counts[i] += 1;
        }
    }

    for i in 0..max_len {
        if counts[i] > 0 {
            mixed[i] /= counts[i] as f32;
        }
    }

    mixed
}

fn write_wav_i16(
    path: &PathBuf,
    sample_rate: u32,
    channels: u16,
    samples: &[f32],
) -> Result<(), AudioError> {
    let spec = hound::WavSpec {
        channels,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = hound::WavWriter::create(path, spec)
        .map_err(|e| AudioError::Message(format!("create wav {} failed: {e}", path.display())))?;

    for sample in samples {
        let clamped = sample.clamp(-1.0, 1.0);
        let v = (clamped * i16::MAX as f32).round() as i16;
        writer
            .write_sample(v)
            .map_err(|e| AudioError::Message(format!("write wav sample failed: {e}")))?;
    }

    writer
        .finalize()
        .map_err(|e| AudioError::Message(format!("finalize wav failed: {e}")))?;

    Ok(())
}

fn now_millis() -> u128 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}
