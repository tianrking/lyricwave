use cpal::traits::{DeviceTrait, HostTrait};

use super::{AudioBackend, AudioError, CaptureConfig, CaptureStream, DeviceInfo};

pub struct CpalBackend;

impl CpalBackend {
    pub fn new() -> Self {
        Self
    }
}

impl AudioBackend for CpalBackend {
    fn backend_name(&self) -> &'static str {
        "cpal"
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

    fn start_system_capture(&self, _config: CaptureConfig) -> Result<CaptureStream, AudioError> {
        // CPAL gives us device enumeration cross-platform. Actual loopback capture is
        // platform-specific (WASAPI loopback / CoreAudio virtual device / PipeWire monitor).
        Err(AudioError::NotImplemented {
            feature: "system loopback capture",
        })
    }
}
