use std::io::Write;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use crate::audio::{
    AudioBackend, AudioError, BackendCapabilities, CaptureFormat, CaptureReport, CaptureRequest,
    CaptureTarget, DeviceInfo,
};

use super::platform;

pub struct CpalNativeBackend;

impl CpalNativeBackend {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CpalNativeBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioBackend for CpalNativeBackend {
    fn backend_name(&self) -> &'static str {
        "cpal-native"
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

    fn capture_blocking(&self, request: &CaptureRequest) -> Result<CaptureReport, AudioError> {
        let host = cpal::default_host();
        let device = select_input_device(&host, request.input_device_hint.as_deref())?;

        let config = device
            .default_input_config()
            .map_err(|e| AudioError::Message(format!("failed to get default input config: {e}")))?;
        let sample_rate = config.sample_rate().0;
        let channels = config.channels();
        let duration = request.duration_secs.unwrap_or(5);

        let captured = Arc::new(Mutex::new(Vec::<f32>::new()));
        let captured_clone = Arc::clone(&captured);

        let err_fn = |err| eprintln!("audio stream error: {err}");
        let stream_config: cpal::StreamConfig = config.clone().into();

        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => device
                .build_input_stream(
                    &stream_config,
                    move |data: &[f32], _| {
                        if let Ok(mut out) = captured_clone.lock() {
                            out.extend_from_slice(data);
                        }
                    },
                    err_fn,
                    None,
                )
                .map_err(|e| {
                    AudioError::Message(format!("failed to build f32 input stream: {e}"))
                })?,
            cpal::SampleFormat::I16 => {
                let captured_i16 = Arc::clone(&captured);
                device
                    .build_input_stream(
                        &stream_config,
                        move |data: &[i16], _| {
                            if let Ok(mut out) = captured_i16.lock() {
                                out.extend(data.iter().map(|v| *v as f32 / i16::MAX as f32));
                            }
                        },
                        err_fn,
                        None,
                    )
                    .map_err(|e| {
                        AudioError::Message(format!("failed to build i16 input stream: {e}"))
                    })?
            }
            cpal::SampleFormat::U16 => {
                let captured_u16 = Arc::clone(&captured);
                device
                    .build_input_stream(
                        &stream_config,
                        move |data: &[u16], _| {
                            if let Ok(mut out) = captured_u16.lock() {
                                out.extend(
                                    data.iter()
                                        .map(|v| (*v as f32 / u16::MAX as f32) * 2.0 - 1.0),
                                );
                            }
                        },
                        err_fn,
                        None,
                    )
                    .map_err(|e| {
                        AudioError::Message(format!("failed to build u16 input stream: {e}"))
                    })?
            }
            sample_format => {
                return Err(AudioError::Message(format!(
                    "unsupported sample format: {sample_format:?}"
                )));
            }
        };

        stream
            .play()
            .map_err(|e| AudioError::Message(format!("failed to start input stream: {e}")))?;
        std::thread::sleep(Duration::from_secs(duration as u64));
        drop(stream);

        let samples = captured
            .lock()
            .map_err(|_| AudioError::Message("failed to lock captured buffer".to_string()))?
            .clone();

        if samples.is_empty() {
            return Err(AudioError::Message(
                "captured audio is empty; check input routing or loopback device".to_string(),
            ));
        }

        write_capture_output(request, &samples, sample_rate, channels)?;

        Ok(CaptureReport {
            captured_samples: samples.len(),
            sample_rate,
            channels,
        })
    }
}

fn select_input_device(host: &cpal::Host, hint: Option<&str>) -> Result<cpal::Device, AudioError> {
    if let Some(hint_text) = hint {
        let query = hint_text.to_lowercase();
        let mut devices = host
            .input_devices()
            .map_err(|e| AudioError::Message(format!("failed to list input devices: {e}")))?;
        if let Some(device) = devices.find(|d| {
            d.name()
                .map(|n| n.to_lowercase().contains(&query))
                .unwrap_or(false)
        }) {
            return Ok(device);
        }
    }

    host.default_input_device().ok_or_else(|| {
        AudioError::Message(
            "no default input device found; configure loopback-capable input first".to_string(),
        )
    })
}

fn write_capture_output(
    request: &CaptureRequest,
    samples: &[f32],
    sample_rate: u32,
    channels: u16,
) -> Result<(), AudioError> {
    match &request.target {
        CaptureTarget::File(path) => {
            if !matches!(request.format, CaptureFormat::Wav) {
                return Err(AudioError::Message(
                    "native backend currently supports file output in WAV format only".to_string(),
                ));
            }

            let spec = hound::WavSpec {
                channels,
                sample_rate,
                bits_per_sample: 16,
                sample_format: hound::SampleFormat::Int,
            };
            let mut writer = hound::WavWriter::create(path, spec)
                .map_err(|e| AudioError::Message(format!("failed to create wav writer: {e}")))?;

            for sample in samples {
                writer
                    .write_sample(f32_to_i16(*sample))
                    .map_err(|e| AudioError::Message(format!("failed to write wav sample: {e}")))?;
            }

            writer
                .finalize()
                .map_err(|e| AudioError::Message(format!("failed to finalize wav file: {e}")))?;
        }
        CaptureTarget::StdoutPcm => {
            let mut out = std::io::stdout().lock();
            for sample in samples {
                let bytes = f32_to_i16(*sample).to_le_bytes();
                out.write_all(&bytes)
                    .map_err(|e| AudioError::Message(format!("stdout write failed: {e}")))?;
            }
            out.flush()
                .map_err(|e| AudioError::Message(format!("stdout flush failed: {e}")))?;
        }
    }

    Ok(())
}

fn f32_to_i16(sample: f32) -> i16 {
    let clamped = sample.clamp(-1.0, 1.0);
    (clamped * i16::MAX as f32).round() as i16
}
