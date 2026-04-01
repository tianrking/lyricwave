use cpal::traits::{DeviceTrait, HostTrait};

use crate::audio::{
    AudioBackend, AudioError, BackendCapabilities, CaptureRequest, CaptureTarget, CommandSpec,
    DeviceInfo,
};

use super::platform;

pub struct CpalFfmpegBackend;

impl CpalFfmpegBackend {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CpalFfmpegBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioBackend for CpalFfmpegBackend {
    fn backend_name(&self) -> &'static str {
        "cpal+ffmpeg"
    }

    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities {
            system_loopback_capture: true,
            per_app_capture: false,
            note: platform::capability_note(),
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

        args.extend(platform::ffmpeg_input_args(request));
        platform::append_common_ffmpeg_args(&mut args, request);

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
