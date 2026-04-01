use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::Ordering;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde_json::Value;

use crate::audio::{
    AudioError, CaptureFormat, CaptureReport, CaptureRequest, CaptureScope, CaptureTarget,
    InputDeviceInfo, ProcessSelector,
};

pub fn capability_note() -> &'static str {
    "Linux native capture uses CPAL input stream for system capture; per-app capture is available with PulseAudio/PipeWire via pactl+parecord."
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

    if !matches!(request.target, CaptureTarget::File(_)) {
        return Err(AudioError::Message(
            "process capture currently supports file target only".to_string(),
        ));
    }
    if !matches!(request.format, CaptureFormat::Wav) {
        return Err(AudioError::Message(
            "process capture currently supports WAV output only".to_string(),
        ));
    }

    ensure_command_exists("pactl")?;
    ensure_command_exists("parecord")?;

    let sink_inputs = list_sink_inputs()?;
    let matched = match_sink_inputs(&sink_inputs, selectors);
    if matched.is_empty() {
        return Err(AudioError::Message(format!(
            "no playing sink-inputs matched selectors: {}",
            selectors
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        )));
    }

    let default_sink = get_default_sink_name()?;
    let capture_sink = format!("lyricwave_capture_{}", now_millis());
    let module_id = load_null_sink(&capture_sink)?;

    let mut moved_back = false;
    let capture_result = (|| {
        move_sink_inputs_to_capture(&matched, &capture_sink)?;
        let run_result = run_parecord_for_request(request, &capture_sink);
        move_sink_inputs_back(&matched, &default_sink)?;
        moved_back = true;
        run_result
    })();

    if !moved_back {
        let _ = move_sink_inputs_back(&matched, &default_sink);
    }
    let _ = unload_module(&module_id);

    capture_result?;

    let (captured_samples, sample_rate, channels) = match &request.target {
        CaptureTarget::File(path) => read_wav_meta(path)?,
        CaptureTarget::StdoutPcm => (
            0,
            request.sample_rate.unwrap_or(48000),
            request.channels.unwrap_or(2),
        ),
    };

    Ok(CaptureReport {
        captured_samples,
        sample_rate,
        channels,
        selected_input_device: InputDeviceInfo {
            id: format!("pulse-monitor:{capture_sink}.monitor"),
            name: format!("{} monitor", capture_sink),
            is_default: false,
            loopback_score: 100,
            is_loopback_candidate: true,
        },
        selection_reason: "selected linux per-app monitor sink".to_string(),
        matched_processes: matched
            .iter()
            .map(|m| format!("pid={} name={} sink_input={}", m.pid, m.name, m.index))
            .collect(),
    })
}

#[derive(Debug, Clone)]
struct SinkInput {
    index: u32,
    pid: u32,
    name: String,
}

fn ensure_command_exists(cmd: &str) -> Result<(), AudioError> {
    let status = Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {cmd} >/dev/null 2>&1"))
        .status()
        .map_err(|e| AudioError::Message(format!("failed to probe command '{cmd}': {e}")))?;

    if status.success() {
        Ok(())
    } else {
        Err(AudioError::Message(format!(
            "required command '{cmd}' not found; install PulseAudio/PipeWire client tools"
        )))
    }
}

fn list_sink_inputs() -> Result<Vec<SinkInput>, AudioError> {
    let output = Command::new("pactl")
        .args(["-f", "json", "list", "sink-inputs"])
        .output()
        .map_err(|e| AudioError::Message(format!("failed to run pactl list sink-inputs: {e}")))?;

    if !output.status.success() {
        return Err(AudioError::Message(format!(
            "pactl list sink-inputs failed with code {:?}",
            output.status.code()
        )));
    }

    let parsed: Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| AudioError::Message(format!("invalid pactl sink-input json: {e}")))?;

    let Some(arr) = parsed.as_array() else {
        return Err(AudioError::Message(
            "unexpected pactl sink-input json shape".to_string(),
        ));
    };

    let mut out = Vec::new();
    for item in arr {
        let Some(index_u64) = item.get("index").and_then(Value::as_u64) else {
            continue;
        };
        let props = item
            .get("properties")
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default();

        let pid = props
            .get("application.process.id")
            .and_then(Value::as_str)
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(0);

        let name = props
            .get("application.name")
            .and_then(Value::as_str)
            .or_else(|| props.get("media.name").and_then(Value::as_str))
            .unwrap_or("unknown")
            .to_string();

        out.push(SinkInput {
            index: index_u64 as u32,
            pid,
            name,
        });
    }

    Ok(out)
}

fn match_sink_inputs(inputs: &[SinkInput], selectors: &[ProcessSelector]) -> Vec<SinkInput> {
    let mut matched = Vec::new();

    for input in inputs {
        let ok = selectors.iter().any(|selector| match selector {
            ProcessSelector::Pid(pid) => input.pid == *pid,
            ProcessSelector::NameContains(name) => {
                input.name.to_lowercase().contains(&name.to_lowercase())
            }
        });
        if ok {
            matched.push(input.clone());
        }
    }

    matched
}

fn get_default_sink_name() -> Result<String, AudioError> {
    let output = Command::new("pactl")
        .arg("get-default-sink")
        .output()
        .map_err(|e| AudioError::Message(format!("failed to get default sink: {e}")))?;

    if !output.status.success() {
        return Err(AudioError::Message(
            "pactl get-default-sink failed".to_string(),
        ));
    }

    let sink = String::from_utf8(output.stdout)
        .map_err(|e| AudioError::Message(format!("invalid default sink output: {e}")))?
        .trim()
        .to_string();

    if sink.is_empty() {
        return Err(AudioError::Message(
            "default sink is empty; Pulse/PipeWire may be unavailable".to_string(),
        ));
    }

    Ok(sink)
}

fn load_null_sink(sink_name: &str) -> Result<String, AudioError> {
    let output = Command::new("pactl")
        .args([
            "load-module",
            "module-null-sink",
            &format!("sink_name={sink_name}"),
            "sink_properties=device.description=LyricwaveCapture",
        ])
        .output()
        .map_err(|e| AudioError::Message(format!("failed to load null sink module: {e}")))?;

    if !output.status.success() {
        return Err(AudioError::Message(
            "pactl load-module module-null-sink failed".to_string(),
        ));
    }

    let id = String::from_utf8(output.stdout)
        .map_err(|e| AudioError::Message(format!("invalid module id output: {e}")))?
        .trim()
        .to_string();

    if id.is_empty() {
        return Err(AudioError::Message(
            "pactl returned empty module id for null sink".to_string(),
        ));
    }

    Ok(id)
}

fn move_sink_inputs_to_capture(inputs: &[SinkInput], sink_name: &str) -> Result<(), AudioError> {
    for input in inputs {
        let status = Command::new("pactl")
            .args(["move-sink-input", &input.index.to_string(), sink_name])
            .status()
            .map_err(|e| {
                AudioError::Message(format!(
                    "failed to move sink input {} to capture sink: {e}",
                    input.index
                ))
            })?;
        if !status.success() {
            return Err(AudioError::Message(format!(
                "pactl move-sink-input {} -> {} failed",
                input.index, sink_name
            )));
        }
    }

    Ok(())
}

fn move_sink_inputs_back(inputs: &[SinkInput], sink_name: &str) -> Result<(), AudioError> {
    for input in inputs {
        let status = Command::new("pactl")
            .args(["move-sink-input", &input.index.to_string(), sink_name])
            .status()
            .map_err(|e| {
                AudioError::Message(format!(
                    "failed to restore sink input {} to default sink: {e}",
                    input.index
                ))
            })?;
        if !status.success() {
            return Err(AudioError::Message(format!(
                "failed to restore sink input {} back to {}",
                input.index, sink_name
            )));
        }
    }

    Ok(())
}

fn run_parecord_for_request(request: &CaptureRequest, sink_name: &str) -> Result<(), AudioError> {
    let out_path = match &request.target {
        CaptureTarget::File(path) => path,
        CaptureTarget::StdoutPcm => {
            return Err(AudioError::Message(
                "process capture only supports file target".to_string(),
            ));
        }
    };

    let mut cmd = Command::new("parecord");
    cmd.args(["-d", &format!("{sink_name}.monitor")])
        .args(["--file-format=wav"])
        .arg(out_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    if let Some(rate) = request.sample_rate {
        cmd.args(["--rate", &rate.to_string()]);
    }
    if let Some(ch) = request.channels {
        cmd.args(["--channels", &ch.to_string()]);
    }

    let mut child = cmd
        .spawn()
        .map_err(|e| AudioError::Message(format!("failed to start parecord: {e}")))?;

    wait_or_stop_capture(&mut child, request)?;

    Ok(())
}

fn wait_or_stop_capture(child: &mut Child, request: &CaptureRequest) -> Result<(), AudioError> {
    if let Some(duration) = request.duration_secs {
        thread::sleep(Duration::from_secs(duration as u64));
    } else if let Some(flag) = &request.stop_flag {
        while !flag.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_millis(50));
        }
    } else {
        return Err(AudioError::Message(
            "manual capture requires stop flag when duration is not set".to_string(),
        ));
    }

    terminate_child(child)?;

    let status = child
        .wait()
        .map_err(|e| AudioError::Message(format!("failed waiting for parecord: {e}")))?;
    if status.success() {
        return Ok(());
    }

    match status.code() {
        Some(143) | Some(130) | Some(0) | None => Ok(()),
        Some(code) => Err(AudioError::Message(format!(
            "parecord exited with code {code}"
        ))),
    }
}

fn terminate_child(child: &mut Child) -> Result<(), AudioError> {
    #[cfg(unix)]
    {
        let status = Command::new("kill")
            .args(["-TERM", &child.id().to_string()])
            .status()
            .map_err(|e| AudioError::Message(format!("failed to signal parecord: {e}")))?;
        if !status.success() {
            let _ = child.kill();
        }
    }

    #[cfg(not(unix))]
    {
        child
            .kill()
            .map_err(|e| AudioError::Message(format!("failed to stop parecord: {e}")))?;
    }

    Ok(())
}

fn unload_module(module_id: &str) -> Result<(), AudioError> {
    let status = Command::new("pactl")
        .args(["unload-module", module_id])
        .status()
        .map_err(|e| AudioError::Message(format!("failed to unload module {module_id}: {e}")))?;
    if !status.success() {
        return Err(AudioError::Message(format!(
            "pactl unload-module {module_id} failed"
        )));
    }

    Ok(())
}

fn read_wav_meta(path: &Path) -> Result<(usize, u32, u16), AudioError> {
    let reader = hound::WavReader::open(path).map_err(|e| {
        AudioError::Message(format!("failed to open wav output {}: {e}", path.display()))
    })?;
    let spec = reader.spec();
    let samples = reader.duration() as usize;
    Ok((samples, spec.sample_rate, spec.channels))
}

fn now_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}
