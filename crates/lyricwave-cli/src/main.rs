use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use lyricwave_core::audio::{
    AudioBackend, CaptureFormat, CaptureRequest, CaptureTarget, CpalBackend,
};
use lyricwave_core::pipeline::{
    DaemonEvent, asr_file_providers, asr_text_providers, build_file_asr_vibevoice, build_text_asr,
    build_translator, translator_providers, MockAsrEngine, MockTranslator,
};
use lyricwave_core::service::PipelineService;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::sync::broadcast;

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
    /// List available ASR/translation providers
    Providers {
        #[command(subcommand)]
        command: ProviderCommands,
    },
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
enum ProviderCommands {
    /// List provider catalog and setup requirements
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
        #[arg(long, default_value = "mock")]
        asr_provider: String,
        #[arg(long, default_value = "mock")]
        translator_provider: String,
    },
    /// Offline ASR through an external VibeVoice checkout.
    AsrFile {
        #[arg(long)]
        audio: PathBuf,
        #[arg(long, default_value = "vibevoice")]
        asr_provider: String,
        #[arg(long)]
        vibevoice_dir: Option<PathBuf>,
        #[arg(long, default_value = "microsoft/VibeVoice-ASR")]
        model_path: String,
        #[arg(long, default_value = "python")]
        python_bin: String,
        #[arg(long, default_value = "auto")]
        source_lang: String,
        #[arg(long, default_value = "zh")]
        target_lang: String,
        #[arg(long, default_value = "mock")]
        translator_provider: String,
        #[arg(long, default_value_t = false)]
        no_translate: bool,
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
    /// Serve daemon events over TCP JSON lines for overlay clients.
    Serve {
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
        #[arg(long, default_value_t = 7878)]
        port: u16,
        #[arg(long, default_value = "auto")]
        source_lang: String,
        #[arg(long, default_value = "zh")]
        target_lang: String,
        #[arg(long, default_value_t = 1000)]
        interval_ms: u64,
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
        Commands::Providers { command } => match command {
            ProviderCommands::List => cmd_providers_list(),
        },
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
                asr_provider,
                translator_provider,
            } => cmd_pipeline_demo(
                &text,
                &source_lang,
                &target_lang,
                &asr_provider,
                &translator_provider,
            )?,
            PipelineCommands::AsrFile {
                audio,
                asr_provider,
                vibevoice_dir,
                model_path,
                python_bin,
                source_lang,
                target_lang,
                translator_provider,
                no_translate,
            } => cmd_pipeline_asr_file_vibevoice(
                &audio,
                &asr_provider,
                vibevoice_dir.as_ref(),
                &model_path,
                &python_bin,
                &source_lang,
                &target_lang,
                &translator_provider,
                no_translate,
            )?,
        },
        Commands::Daemon { command } => match command {
            DaemonCommands::Run {
                source_lang,
                target_lang,
                interval_ms,
                count,
            } => cmd_daemon_run(&source_lang, &target_lang, interval_ms, count)?,
            DaemonCommands::Serve {
                host,
                port,
                source_lang,
                target_lang,
                interval_ms,
            } => {
                let rt =
                    tokio::runtime::Runtime::new().context("failed to create tokio runtime")?;
                rt.block_on(cmd_daemon_serve(
                    &host,
                    port,
                    &source_lang,
                    &target_lang,
                    interval_ms,
                ))?;
            }
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

fn cmd_providers_list() {
    println!("text_asr:");
    for p in asr_text_providers() {
        println!(
            "- id={} mode={:?} setup_required={} note={}",
            p.id, p.mode, p.requires_setup, p.note
        );
    }

    println!("file_asr:");
    for p in asr_file_providers() {
        println!(
            "- id={} mode={:?} setup_required={} note={}",
            p.id, p.mode, p.requires_setup, p.note
        );
    }

    println!("translator:");
    for p in translator_providers() {
        println!(
            "- id={} mode={:?} setup_required={} note={}",
            p.id, p.mode, p.requires_setup, p.note
        );
    }
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

fn cmd_pipeline_demo(
    text: &str,
    source_lang: &str,
    target_lang: &str,
    asr_provider: &str,
    translator_provider: &str,
) -> Result<()> {
    let asr = build_text_asr(asr_provider).map_err(anyhow::Error::msg)?;
    let translator = build_translator(translator_provider).map_err(anyhow::Error::msg)?;
    let transcribed = asr.transcribe_text(text);
    let translated = translator.translate(&transcribed, target_lang);

    let service = PipelineService::new(
        lyricwave_core::pipeline::MockAsrEngine,
        lyricwave_core::pipeline::MockTranslator,
        64,
    );
    let mut evt = service.process_text(&transcribed, source_lang, target_lang);
    evt.translated_text = Some(translated);

    println!("asr_provider: {}", asr.name());
    println!("translator_provider: {}", translator.name());
    println!("source: {}", evt.source_text);
    println!("translation: {}", evt.translated_text.unwrap_or_default());
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn cmd_pipeline_asr_file_vibevoice(
    audio: &PathBuf,
    asr_provider: &str,
    vibevoice_dir: Option<&PathBuf>,
    model_path: &str,
    python_bin: &str,
    source_lang: &str,
    target_lang: &str,
    translator_provider: &str,
    no_translate: bool,
) -> Result<()> {
    let asr = match asr_provider {
        "vibevoice" => {
            let repo_dir = vibevoice_dir
                .cloned()
                .context("--vibevoice-dir is required when --asr-provider vibevoice")?;
            build_file_asr_vibevoice(python_bin.to_string(), repo_dir, model_path.to_string())
        }
        _ => anyhow::bail!("unknown file ASR provider: {asr_provider}"),
    };

    let source_text = asr
        .transcribe_file(audio)
        .map_err(anyhow::Error::msg)
        .with_context(|| format!("{} failed for {}", asr.name(), audio.display()))?;

    println!("asr_provider: {}", asr.name());
    println!("source: {source_text}");

    if !no_translate {
        let translator = build_translator(translator_provider).map_err(anyhow::Error::msg)?;
        let translated = translator.translate(&source_text, target_lang);
        println!("translator: {}", translator.name());
        println!("translation: {translated}");
    }

    println!("source_lang_hint: {source_lang}");
    println!("target_lang: {target_lang}");
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

async fn cmd_daemon_serve(
    host: &str,
    port: u16,
    source_lang: &str,
    target_lang: &str,
    interval_ms: u64,
) -> Result<()> {
    let bind_addr = format!("{host}:{port}");
    let listener = TcpListener::bind(&bind_addr)
        .await
        .with_context(|| format!("failed to bind daemon server at {bind_addr}"))?;

    println!("daemon tcp json stream listening on {bind_addr}");

    let (tx, _) = broadcast::channel::<String>(256);

    let producer_tx = tx.clone();
    let source = source_lang.to_string();
    let target = target_lang.to_string();
    tokio::spawn(async move {
        let service = PipelineService::new(MockAsrEngine, MockTranslator, 128);

        let started = DaemonEvent::Status {
            message: "daemon started".to_string(),
            emitted_at_ms: DaemonEvent::now_ms(),
        };
        if let Ok(line) = serde_json::to_string(&started) {
            let _ = producer_tx.send(line);
        }

        let mut idx: u64 = 1;
        loop {
            let text = format!("live line {idx}");
            let transcript = service.process_text(&text, &source, &target);
            let evt = DaemonEvent::Transcript {
                payload: transcript,
                emitted_at_ms: DaemonEvent::now_ms(),
            };
            if let Ok(line) = serde_json::to_string(&evt) {
                let _ = producer_tx.send(line);
            }
            idx = idx.saturating_add(1);
            tokio::time::sleep(Duration::from_millis(interval_ms)).await;
        }
    });

    loop {
        let (mut socket, peer) = listener.accept().await?;
        let mut rx = tx.subscribe();
        println!("client connected: {peer}");
        tokio::spawn(async move {
            while let Ok(line) = rx.recv().await {
                if socket.write_all(line.as_bytes()).await.is_err() {
                    break;
                }
                if socket.write_all(b"\n").await.is_err() {
                    break;
                }
            }
        });
    }
}
