use anyhow::Result;
use clap::{Parser, Subcommand};
use lyricwave_core::audio::{AudioBackend, CaptureConfig, CpalBackend};

#[derive(Parser, Debug)]
#[command(
    name = "lyricwave",
    version,
    about = "Cross-platform system audio capture and subtitle pipeline CLI"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Inspect audio devices and backend capability
    Devices {
        #[command(subcommand)]
        command: DeviceCommands,
    },
    /// Capture system output (loopback)
    Capture {
        #[command(subcommand)]
        command: CaptureCommands,
    },
}

#[derive(Subcommand, Debug)]
enum DeviceCommands {
    /// List output devices
    List,
}

#[derive(Subcommand, Debug)]
enum CaptureCommands {
    /// Start system loopback capture (stub in v0)
    System {
        /// Optional target sample rate
        #[arg(long)]
        sample_rate: Option<u32>,

        /// Optional target channels
        #[arg(long)]
        channels: Option<u16>,

        /// Optional output device id
        #[arg(long)]
        device_id: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let backend = CpalBackend::new();

    match cli.command {
        Commands::Devices { command } => match command {
            DeviceCommands::List => cmd_devices_list(&backend)?,
        },
        Commands::Capture { command } => match command {
            CaptureCommands::System {
                sample_rate,
                channels,
                device_id,
            } => cmd_capture_system(&backend, sample_rate, channels, device_id)?,
        },
    }

    Ok(())
}

fn cmd_devices_list(backend: &dyn AudioBackend) -> Result<()> {
    let devices = backend.list_output_devices()?;
    println!("backend: {}", backend.backend_name());

    if devices.is_empty() {
        println!("no output devices found");
        return Ok(());
    }

    for device in devices {
        println!("- {device}");
    }

    Ok(())
}

fn cmd_capture_system(
    backend: &dyn AudioBackend,
    sample_rate: Option<u32>,
    channels: Option<u16>,
    device_id: Option<String>,
) -> Result<()> {
    let config = CaptureConfig {
        sample_rate,
        channels,
        output_device_id: device_id,
    };

    match backend.start_system_capture(config) {
        Ok(stream) => {
            println!("system capture started via {}", stream.backend_name);
            Ok(())
        }
        Err(err) => {
            println!("capture is not available yet: {err}");
            println!("next step: implement per-platform loopback backend");
            Ok(())
        }
    }
}
