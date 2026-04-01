use anyhow::Result;
use lyricwave_core::audio::AudioBackend;

pub fn list(backend: &dyn AudioBackend) -> Result<()> {
    let devices = backend.list_output_devices()?;
    let caps = backend.capabilities();

    println!("backend: {}", backend.backend_name());
    println!(
        "capabilities: system_loopback={}, per_app={}, note={}",
        caps.system_loopback_capture, caps.per_app_capture, caps.note
    );

    if devices.is_empty() {
        println!("no output devices found");
        return Ok(());
    }

    for device in devices {
        println!("- {device}");
    }

    Ok(())
}
