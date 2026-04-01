use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};
use lyricwave_core::audio::CaptureFormat;

#[derive(Parser, Debug)]
#[command(
    name = "lyricwave",
    version,
    about = "Cross-platform system audio capture and subtitle pipeline CLI"
)]
pub struct Cli {
    /// Audio backend id used for capture/device commands.
    #[arg(long, global = true, default_value = "cpal-native")]
    pub audio_backend: String,
    /// Visual backend id used for visual commands.
    #[arg(
        long = "visual-backend",
        global = true,
        default_value = "platform-native"
    )]
    pub visual_backend: String,
    /// Optional config file path (TOML).
    #[arg(long, global = true)]
    pub config: Option<PathBuf>,
    /// Profile name inside config file.
    #[arg(long, global = true, default_value = "default")]
    pub profile: String,
    /// Logging verbosity.
    #[arg(long, global = true, value_enum, default_value_t = LogLevelArg::Info)]
    pub log_level: LogLevelArg,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// List available audio backends
    Backends {
        #[command(subcommand)]
        command: BackendCommands,
    },
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
    /// Visual backend/device/capture commands
    Visual {
        #[command(subcommand)]
        command: VisualCommands,
    },
    /// Unified composition session (audio-only / visual-only / audio+visual)
    Record {
        #[command(subcommand)]
        command: RecordCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum ProviderCommands {
    /// List provider catalog and setup requirements
    List,
}

#[derive(Subcommand, Debug)]
pub enum BackendCommands {
    /// List audio backend catalog
    List,
}

#[derive(Subcommand, Debug)]
pub enum DeviceCommands {
    /// List output/input devices
    List,
}

#[derive(Subcommand, Debug)]
pub enum CaptureCommands {
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

        /// Platform input hint (macOS e.g. :0, Linux monitor source, Windows endpoint name).
        #[arg(long)]
        input_device: Option<String>,
        /// Disable loopback-first auto selection.
        #[arg(long, default_value_t = false)]
        no_prefer_loopback: bool,
    },
    /// Capture audio from selected app processes (macOS ScreenCaptureKit or Linux Pulse/PipeWire).
    App {
        /// Output wav file path.
        #[arg(long)]
        out: PathBuf,

        /// Capture duration in seconds; omit for manual stop.
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

        /// Match by process id (repeatable).
        #[arg(long)]
        pid: Vec<u32>,

        /// Match by application name contains (repeatable, case-insensitive).
        #[arg(long)]
        name: Vec<String>,
    },
    /// List active audio processes that backend can detect.
    AppsList,
    /// Capture multiple app processes into independent files, with optional mixed output.
    AppsSplit {
        /// Directory for per-process wav files.
        #[arg(long)]
        out_dir: PathBuf,
        /// Capture duration in seconds.
        #[arg(long)]
        seconds: u32,
        /// Optional target sample rate.
        #[arg(long)]
        sample_rate: Option<u32>,
        /// Optional target channel count.
        #[arg(long)]
        channels: Option<u16>,
        /// Capture format when writing files.
        #[arg(long, value_enum, default_value_t = FileFormatArg::Wav)]
        format: FileFormatArg,
        /// Match by process id (repeatable).
        #[arg(long)]
        pid: Vec<u32>,
        /// Match by application name contains (repeatable, case-insensitive).
        #[arg(long)]
        name: Vec<String>,
        /// Capture all currently active audio processes.
        #[arg(long, default_value_t = false)]
        all_active: bool,
        /// Optional mixed output file generated from captured split files.
        #[arg(long)]
        mix_out: Option<PathBuf>,
    },
}

#[derive(Subcommand, Debug)]
pub enum PipelineCommands {
    /// Demo translation pipeline using selected providers.
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
    /// File ASR with provider selection (local or online in future).
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
    /// End-to-end one-shot flow: capture system audio, transcribe, then translate.
    RunOnce {
        /// Capture duration in seconds.
        #[arg(long, default_value_t = 8)]
        seconds: u32,
        /// Optional path to keep captured wav. If omitted, uses temp file.
        #[arg(long)]
        audio_out: Option<PathBuf>,
        /// Keep temp captured file when --audio-out is not provided.
        #[arg(long, default_value_t = false)]
        keep_temp_audio: bool,
        /// ASR file provider id.
        #[arg(long, default_value = "vibevoice")]
        asr_provider: String,
        /// Required for vibevoice provider.
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
        /// Optional input device hint for capture backend.
        #[arg(long)]
        input_device: Option<String>,
        /// Disable loopback-first auto selection.
        #[arg(long, default_value_t = false)]
        no_prefer_loopback: bool,
        #[arg(long)]
        sample_rate: Option<u32>,
        #[arg(long)]
        channels: Option<u16>,
    },
}

#[derive(Subcommand, Debug)]
pub enum DaemonCommands {
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

#[derive(Subcommand, Debug)]
pub enum VisualCommands {
    /// List visual backend catalog
    Backends,
    /// List available displays
    Displays,
    /// Capture system visual output (display stream) to a file.
    System {
        /// Output file path (recommended extension: .mp4 / .mkv / image stream container)
        #[arg(long)]
        out: PathBuf,
        /// Capture duration in seconds. Omit for manual stop.
        #[arg(long)]
        seconds: Option<u32>,
        /// Target frames per second.
        #[arg(long)]
        fps: Option<u32>,
        /// Optional display hint.
        #[arg(long)]
        display: Option<String>,
    },
    /// Capture selected app visual output (future per-OS app/window routing).
    App {
        #[arg(long)]
        out: PathBuf,
        #[arg(long)]
        seconds: Option<u32>,
        #[arg(long)]
        fps: Option<u32>,
        #[arg(long)]
        pid: Vec<u32>,
        #[arg(long)]
        name: Vec<String>,
    },
    /// List active visual processes that backend can detect.
    AppsList,
    /// Capture multiple app visuals to independent files.
    AppsSplit {
        #[arg(long)]
        out_dir: PathBuf,
        #[arg(long)]
        seconds: u32,
        #[arg(long)]
        fps: Option<u32>,
        #[arg(long)]
        pid: Vec<u32>,
        #[arg(long)]
        name: Vec<String>,
        #[arg(long, default_value_t = false)]
        all_active: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum RecordCommands {
    /// Compose system audio + system visual in one session.
    System {
        /// Optional audio output wav file path.
        #[arg(long)]
        audio_out: Option<PathBuf>,
        /// Optional visual output file path.
        #[arg(long = "visual-out")]
        visual_out: Option<PathBuf>,
        /// Duration in seconds. Omit for manual stop.
        #[arg(long)]
        seconds: Option<u32>,
        /// Audio sample rate.
        #[arg(long)]
        sample_rate: Option<u32>,
        /// Audio channel count.
        #[arg(long)]
        channels: Option<u16>,
        /// Optional audio input hint.
        #[arg(long)]
        input_device: Option<String>,
        /// Disable loopback-first auto selection for audio.
        #[arg(long, default_value_t = false)]
        no_prefer_loopback: bool,
        /// Visual target fps.
        #[arg(long)]
        fps: Option<u32>,
        /// Optional display hint for visual capture.
        #[arg(long)]
        display: Option<String>,
    },
    /// Compose selected app audio + selected app visual in one session.
    App {
        #[arg(long)]
        audio_out: Option<PathBuf>,
        #[arg(long = "visual-out")]
        visual_out: Option<PathBuf>,
        #[arg(long)]
        seconds: Option<u32>,
        #[arg(long)]
        sample_rate: Option<u32>,
        #[arg(long)]
        channels: Option<u16>,
        #[arg(long)]
        fps: Option<u32>,
        #[arg(long)]
        pid: Vec<u32>,
        #[arg(long)]
        name: Vec<String>,
    },
    /// Compose selected app audio/visual into independent per-process files.
    AppsSplit {
        #[arg(long)]
        out_dir: PathBuf,
        #[arg(long)]
        seconds: u32,
        #[arg(long)]
        sample_rate: Option<u32>,
        #[arg(long)]
        channels: Option<u16>,
        #[arg(long)]
        fps: Option<u32>,
        #[arg(long)]
        pid: Vec<u32>,
        #[arg(long)]
        name: Vec<String>,
        #[arg(long, default_value_t = false)]
        all_active: bool,
        #[arg(long, default_value_t = false)]
        no_audio: bool,
        #[arg(long, default_value_t = false)]
        no_visual: bool,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum FileFormatArg {
    Wav,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum LogLevelArg {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl LogLevelArg {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warn => "warn",
            Self::Info => "info",
            Self::Debug => "debug",
            Self::Trace => "trace",
        }
    }
}

impl From<FileFormatArg> for CaptureFormat {
    fn from(value: FileFormatArg) -> Self {
        match value {
            FileFormatArg::Wav => CaptureFormat::Wav,
        }
    }
}
