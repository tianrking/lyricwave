use anyhow::Result;
use lyricwave_core::audio::AudioBackend;

pub fn list(backend: &dyn AudioBackend) -> Result<()> {
    let outputs = backend.list_output_devices()?;
    let inputs = backend.list_input_devices()?;
    let caps = backend.capabilities();

    println!("backend: {}", backend.backend_name());
    println!(
        "capabilities: system_loopback={}, per_app={}, note={}",
        caps.system_loopback_capture, caps.per_app_capture, caps.note
    );

    println!("output devices:");
    if outputs.is_empty() {
        println!("- (none)");
    } else {
        for device in outputs {
            println!("- {device}");
        }
    }

    println!("input devices:");
    if inputs.is_empty() {
        println!("- (none)");
    } else {
        for device in inputs {
            println!("- {device}");
        }
    }

    Ok(())
}
