use std::io::Write;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use crate::audio::selection::select_input_device;
use crate::audio::{
    AudioBackend, AudioError, BackendCapabilities, CaptureFormat, CaptureReport, CaptureRequest,
    CaptureScope, CaptureTarget, InputDeviceInfo, OutputDeviceInfo,
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
            per_app_capture: platform::supports_per_app_capture(),
            note: platform::capability_note(),
        }
    }

    fn list_output_devices(&self) -> Result<Vec<OutputDeviceInfo>, AudioError> {
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
                .unwrap_or_else(|_| format!("unknown-output-device-{idx}"));
            devices.push(OutputDeviceInfo {
                id: format!("cpal-output-{idx}"),
                is_default: name == default_name,
                name,
            });
        }

        Ok(devices)
    }

    fn list_input_devices(&self) -> Result<Vec<InputDeviceInfo>, AudioError> {
        list_input_devices_with_host(&cpal::default_host())
    }

    fn capture_blocking(&self, request: &CaptureRequest) -> Result<CaptureReport, AudioError> {
        if let CaptureScope::Processes(_) = &request.scope {
            return platform::capture_processes(request);
        }

        let host = cpal::default_host();
        let inputs = list_input_devices_with_host(&host)?;
        let selected = select_input_device(
            &inputs,
            request.input_device_hint.as_deref(),
            request.prefer_loopback,
        )
        .ok_or_else(|| {
            AudioError::Message(
                "no input device found; configure a loopback-capable input first".to_string(),
            )
        })?;

        let device = find_input_device_by_name(&host, &selected.info.name)?;

        let config = device
            .default_input_config()
            .map_err(|e| AudioError::Message(format!("failed to get default input config: {e}")))?;

        let sample_rate = request.sample_rate.unwrap_or(config.sample_rate().0);
        let channels = request.channels.unwrap_or(config.channels());
        let stop_flag = request.stop_flag.clone();

        let captured = Arc::new(Mutex::new(Vec::<f32>::new()));
        let captured_clone = Arc::clone(&captured);

        let err_fn = |err| eprintln!("audio stream error: {err}");
        let mut stream_config: cpal::StreamConfig = config.clone().into();
        stream_config.sample_rate = cpal::SampleRate(sample_rate);
        stream_config.channels = channels;

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
        if let Some(duration) = request.duration_secs {
            std::thread::sleep(Duration::from_secs(duration as u64));
        } else if let Some(flag) = stop_flag {
            while !flag.load(Ordering::Relaxed) {
                std::thread::sleep(Duration::from_millis(50));
            }
        } else {
            return Err(AudioError::Message(
                "manual capture requires stop flag when duration is not set".to_string(),
            ));
        }
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
            selected_input_device: selected.info,
            selection_reason: selected.reason,
            matched_processes: Vec::new(),
        })
    }
}

fn list_input_devices_with_host(host: &cpal::Host) -> Result<Vec<InputDeviceInfo>, AudioError> {
    let default_name = host
        .default_input_device()
        .and_then(|d| d.name().ok())
        .unwrap_or_default();

    let mut devices = Vec::new();
    for (idx, device) in host
        .input_devices()
        .map_err(|e| AudioError::Message(format!("failed to read input devices: {e}")))?
        .enumerate()
    {
        let name = device
            .name()
            .unwrap_or_else(|_| format!("unknown-input-device-{idx}"));
        let score = crate::audio::selection::loopback_score(&name);
        devices.push(InputDeviceInfo {
            id: format!("cpal-input-{idx}"),
            is_default: name == default_name,
            is_loopback_candidate: score > 0,
            loopback_score: score,
            name,
        });
    }

    Ok(devices)
}

fn find_input_device_by_name(
    host: &cpal::Host,
    wanted_name: &str,
) -> Result<cpal::Device, AudioError> {
    host.input_devices()
        .map_err(|e| AudioError::Message(format!("failed to list input devices: {e}")))?
        .find(|device| device.name().ok().as_deref() == Some(wanted_name))
        .ok_or_else(|| {
            AudioError::Message(format!(
                "selected input device '{wanted_name}' is no longer available"
            ))
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
