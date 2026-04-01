use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use lyricwave_core::audio::{
    AudioBackend, CaptureFormat, CaptureRequest, CaptureTarget, CpalBackend,
};
use lyricwave_core::pipeline::{DaemonEvent, MockAsrEngine, MockTranslator};
use lyricwave_core::service::PipelineService;

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
    /// Run subtitle + translation pipeline demo
    Pipeline {
        #[command(subcommand)]
        command: PipelineCommands,
    },
    /// Emit daemon events as JSON lines for overlays
    Daemon {
        #[command(subcommand)]
        command: DaemonCommands,
    },
}

#[derive(Subcommand, Debug)]
enum DeviceCommands {
    /// List output devices
    List,
}

#[derive(Subcommand, Debug)]
enum CaptureCommands {
    /// Start system capture to file or stdout
    System {
        /// Output file path. Required unless --stdout is set.
        #[arg(long)]
        out: Option<PathBuf>,

        /// Stream raw PCM to stdout instead of writing file.
        #[arg(long, default_value_t = false)]
        stdout: bool,

        /// Capture duration in seconds.
        #[arg(long)]
        seconds: Option<u32>,

        /// Optional target sample rate.
        #[arg(long)]
        sample_rate: Option<u32>,

        /// Optional target channel count.
        #[arg(long)]
        channels: Option<u16>,

        /// Capture format when writing files.
        #[arg(long, value_enum, default_value_t = FileFormatArg::Wav)]
        format: FileFormatArg,

        /// FFmpeg executable path.
        #[arg(long, default_value = "ffmpeg")]
        ffmpeg_bin: String,

        /// Platform input hint (macOS e.g. :0, Linux monitor source, Windows endpoint name).
        #[arg(long)]
        input_device: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
enum PipelineCommands {
    /// Demo translation pipeline using mock engines.
    Demo {
        #[arg(long)]
        text: String,
        #[arg(long, default_value = "auto")]
        source_lang: String,
        #[arg(long, default_value = "zh")]
        target_lang: String,
    },
}

#[derive(Subcommand, Debug)]
enum DaemonCommands {
    /// Run a mock daemon stream for overlay integration.
    Run {
        #[arg(long, default_value = "auto")]
        source_lang: String,
        #[arg(long, default_value = "zh")]
        target_lang: String,
        #[arg(long, default_value_t = 800)]
        interval_ms: u64,
        #[arg(long, default_value_t = 8)]
        count: u32,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum FileFormatArg {
    Wav,
    Flac,
}

impl From<FileFormatArg> for CaptureFormat {
    fn from(value: FileFormatArg) -> Self {
        match value {
            FileFormatArg::Wav => CaptureFormat::Wav,
            FileFormatArg::Flac => CaptureFormat::Flac,
        }
    }
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
                out,
                stdout,
                seconds,
                sample_rate,
                channels,
                format,
                ffmpeg_bin,
                input_device,
            } => cmd_capture_system(
                &backend,
                out,
                stdout,
                seconds,
                sample_rate,
                channels,
                format.into(),
                ffmpeg_bin,
                input_device,
            )?,
        },
        Commands::Pipeline { command } => match command {
            PipelineCommands::Demo {
                text,
                source_lang,
                target_lang,
            } => cmd_pipeline_demo(&text, &source_lang, &target_lang)?,
        },
        Commands::Daemon { command } => match command {
            DaemonCommands::Run {
                source_lang,
                target_lang,
                interval_ms,
                count,
            } => cmd_daemon_run(&source_lang, &target_lang, interval_ms, count)?,
        },
    }

    Ok(())
}

fn cmd_devices_list(backend: &dyn AudioBackend) -> Result<()> {
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

#[allow(clippy::too_many_arguments)]
fn cmd_capture_system(
    backend: &dyn AudioBackend,
    out: Option<PathBuf>,
    stdout: bool,
    seconds: Option<u32>,
    sample_rate: Option<u32>,
    channels: Option<u16>,
    format: CaptureFormat,
    ffmpeg_bin: String,
    input_device: Option<String>,
) -> Result<()> {
    let target = if stdout {
        CaptureTarget::StdoutPcm
    } else {
        let path = out.context("--out is required when --stdout is not set")?;
        CaptureTarget::File(path)
    };

    let request = CaptureRequest {
        target,
        duration_secs: seconds,
        sample_rate,
        channels,
        format,
        ffmpeg_bin,
        input_device_hint: input_device,
    };

    let spec = backend.build_capture_command(&request)?;
    eprintln!("running: {} {}", spec.program, spec.args.join(" "));

    let status = Command::new(&spec.program)
        .args(&spec.args)
        .status()
        .with_context(|| format!("failed to start {}", spec.program))?;

    if !status.success() {
        anyhow::bail!("capture process exited with status: {status}");
    }

    Ok(())
}

fn cmd_pipeline_demo(text: &str, source_lang: &str, target_lang: &str) -> Result<()> {
    let service = PipelineService::new(MockAsrEngine, MockTranslator, 64);
    let evt = service.process_text(text, source_lang, target_lang);
    println!("source: {}", evt.source_text);
    println!("translation: {}", evt.translated_text.unwrap_or_default());
    Ok(())
}

fn cmd_daemon_run(
    source_lang: &str,
    target_lang: &str,
    interval_ms: u64,
    count: u32,
) -> Result<()> {
    let service = PipelineService::new(MockAsrEngine, MockTranslator, 128);

    let status_evt = DaemonEvent::Status {
        message: "daemon started".to_string(),
        emitted_at_ms: DaemonEvent::now_ms(),
    };
    println!("{}", serde_json::to_string(&status_evt)?);

    for idx in 1..=count {
        let text = format!("sample line {idx}");
        let transcript = service.process_text(&text, source_lang, target_lang);
        let evt = DaemonEvent::Transcript {
            payload: transcript,
            emitted_at_ms: DaemonEvent::now_ms(),
        };
        println!("{}", serde_json::to_string(&evt)?);
        thread::sleep(Duration::from_millis(interval_ms));
    }

    let done_evt = DaemonEvent::Status {
        message: "daemon finished".to_string(),
        emitted_at_ms: DaemonEvent::now_ms(),
    };
    println!("{}", serde_json::to_string(&done_evt)?);

    Ok(())
}
