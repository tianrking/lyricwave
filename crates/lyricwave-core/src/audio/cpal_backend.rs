use cpal::traits::{DeviceTrait, HostTrait};

use super::{
    AudioBackend, AudioError, BackendCapabilities, CaptureRequest, CaptureTarget, CommandSpec,
    DeviceInfo,
};

pub struct CpalBackend;

impl CpalBackend {
    pub fn new() -> Self {
        Self
    }
}

impl AudioBackend for CpalBackend {
    fn backend_name(&self) -> &'static str {
        "cpal+ffmpeg"
    }

    fn capabilities(&self) -> BackendCapabilities {
        #[cfg(target_os = "macos")]
        let note = "macOS needs a loopback-capable input (e.g. BlackHole) as audio source";
        #[cfg(target_os = "windows")]
        let note = "Windows uses WASAPI input; loopback behavior depends on FFmpeg/device support";
        #[cfg(target_os = "linux")]
        let note = "Linux uses PulseAudio/PipeWire source; monitor source may be required";

        BackendCapabilities {
            system_loopback_capture: true,
            per_app_capture: false,
            note,
        }
    }

    fn list_output_devices(&self) -> Result<Vec<DeviceInfo>, AudioError> {
        let host = cpal::default_host();
        let default_name = host
            .default_output_device()
            .and_then(|d| d.name().ok())
            .unwrap_or_default();

        let mut devices = Vec::new();
        for (idx, device) in host
            .output_devices()
            .map_err(|e| AudioError::Message(format!("failed to read output devices: {e}")))?
            .enumerate()
        {
            let name = device
                .name()
                .unwrap_or_else(|_| format!("unknown-device-{idx}"));
            devices.push(DeviceInfo {
                id: format!("cpal-output-{idx}"),
                is_default_output: name == default_name,
                name,
            });
        }

        Ok(devices)
    }

    fn build_capture_command(&self, request: &CaptureRequest) -> Result<CommandSpec, AudioError> {
        let mut args = vec![
            "-hide_banner".to_string(),
            "-loglevel".to_string(),
            "warning".to_string(),
            "-y".to_string(),
        ];

        #[cfg(target_os = "windows")]
        {
            args.push("-f".to_string());
            args.push("wasapi".to_string());
            args.push("-i".to_string());
            args.push(
                request
                    .input_device_hint
                    .clone()
                    .unwrap_or_else(|| "default".to_string()),
            );
        }

        #[cfg(target_os = "macos")]
        {
            args.push("-f".to_string());
            args.push("avfoundation".to_string());
            args.push("-i".to_string());
            let audio_selector = request
                .input_device_hint
                .as_deref()
                .unwrap_or(":0")
                .to_string();
            args.push(audio_selector);
        }

        #[cfg(target_os = "linux")]
        {
            args.push("-f".to_string());
            args.push("pulse".to_string());
            args.push("-i".to_string());
            args.push(
                request
                    .input_device_hint
                    .clone()
                    .unwrap_or_else(|| "default".to_string()),
            );
        }

        if let Some(sample_rate) = request.sample_rate {
            args.push("-ar".to_string());
            args.push(sample_rate.to_string());
        }

        if let Some(channels) = request.channels {
            args.push("-ac".to_string());
            args.push(channels.to_string());
        }

        if let Some(duration) = request.duration_secs {
            args.push("-t".to_string());
            args.push(duration.to_string());
        }

        match &request.target {
            CaptureTarget::File(path) => {
                args.push("-f".to_string());
                args.push(request.format.ffmpeg_name().to_string());
                args.push(path.display().to_string());
            }
            CaptureTarget::StdoutPcm => {
                args.push("-f".to_string());
                args.push("s16le".to_string());
                args.push("-".to_string());
            }
        }

        Ok(CommandSpec {
            program: request.ffmpeg_bin.clone(),
            args,
        })
    }
}
