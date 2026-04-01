mod cli;
mod commands;
mod config;

use anyhow::{Context, Result};
use clap::Parser;
use log::{debug, info};
use lyricwave_core::audio::build_audio_backend;

use crate::cli::{
    BackendCommands, CaptureCommands, Cli, Commands, DaemonCommands, DeviceCommands,
    PipelineCommands, ProviderCommands, RecordCommands, VisualCommands,
};
use crate::config::ResolvedConfig;

fn main() {
    if let Err(err) = run() {
        eprintln!("{}", humanize_error(&err));
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    init_logger(cli.log_level.as_str());
    let cfg = ResolvedConfig::load(cli.config.as_deref(), &cli.profile)?;
    if let Some(path) = &cfg.source_path {
        info!(
            "using profile '{}' from {}",
            cfg.profile_name,
            path.display()
        );
    } else {
        info!("using built-in defaults (no config file)");
    }
    debug!("effective profile values: {:?}", cfg.values);

    match cli.command {
        Commands::Backends { command } => match command {
            BackendCommands::List => commands::backends::list(),
        },
        Commands::Providers { command } => match command {
            ProviderCommands::List => commands::providers::list(),
        },
        Commands::Devices { command } => {
            let backend = build_backend(&cli.audio_backend)?;
            match command {
                DeviceCommands::List => commands::devices::list(backend.as_ref())?,
            }
        }
        Commands::Capture { command } => {
            let backend = build_backend(&cli.audio_backend)?;
            match command {
                CaptureCommands::System {
                    out,
                    stdout,
                    seconds,
                    sample_rate,
                    channels,
                    format,
                    input_device,
                    no_prefer_loopback,
                } => {
                    let report = commands::capture::system(
                        backend.as_ref(),
                        out,
                        stdout,
                        seconds,
                        sample_rate.or(cfg.values.sample_rate),
                        channels.or(cfg.values.channels),
                        format.into(),
                        input_device,
                        !no_prefer_loopback,
                    )?;
                    eprintln!(
                        "captured {} samples @ {}Hz ({} channels), input='{}', reason={}",
                        report.captured_samples,
                        report.sample_rate,
                        report.channels,
                        report.selected_input_device.name,
                        report.selection_reason
                    );
                }
                CaptureCommands::App {
                    out,
                    seconds,
                    sample_rate,
                    channels,
                    format,
                    pid,
                    name,
                } => {
                    let report = commands::capture::app(
                        backend.as_ref(),
                        out,
                        seconds,
                        sample_rate.or(cfg.values.sample_rate),
                        channels.or(cfg.values.channels),
                        format.into(),
                        pid,
                        name,
                    )?;
                    eprintln!(
                        "captured {} samples @ {}Hz ({} channels), matched={}",
                        report.captured_samples,
                        report.sample_rate,
                        report.channels,
                        report.matched_processes.join("; ")
                    );
                }
                CaptureCommands::AppsList => {
                    commands::capture::apps_list(backend.as_ref())?;
                }
                CaptureCommands::AppsSplit {
                    out_dir,
                    seconds,
                    sample_rate,
                    channels,
                    format,
                    pid,
                    name,
                    all_active,
                    mix_out,
                } => {
                    commands::capture::apps_split(
                        backend.as_ref(),
                        out_dir,
                        seconds,
                        sample_rate.or(cfg.values.sample_rate),
                        channels.or(cfg.values.channels),
                        format.into(),
                        pid,
                        name,
                        all_active,
                        mix_out,
                    )?;
                }
            }
        }
        Commands::Pipeline { command } => match command {
            PipelineCommands::Demo {
                text,
                source_lang,
                target_lang,
                asr_provider,
                translator_provider,
            } => commands::pipeline::demo(
                &text,
                cfg.values.source_lang.as_deref().unwrap_or(&source_lang),
                cfg.values.target_lang.as_deref().unwrap_or(&target_lang),
                cfg.values.asr_provider.as_deref().unwrap_or(&asr_provider),
                cfg.values
                    .translator_provider
                    .as_deref()
                    .unwrap_or(&translator_provider),
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
            } => commands::pipeline::asr_file(
                &audio,
                cfg.values.asr_provider.as_deref().unwrap_or(&asr_provider),
                vibevoice_dir.as_ref().or(cfg.values.vibevoice_dir.as_ref()),
                cfg.values.model_path.as_deref().unwrap_or(&model_path),
                cfg.values.python_bin.as_deref().unwrap_or(&python_bin),
                cfg.values.source_lang.as_deref().unwrap_or(&source_lang),
                cfg.values.target_lang.as_deref().unwrap_or(&target_lang),
                cfg.values
                    .translator_provider
                    .as_deref()
                    .unwrap_or(&translator_provider),
                no_translate,
            )?,
            PipelineCommands::RunOnce {
                seconds,
                audio_out,
                keep_temp_audio,
                asr_provider,
                vibevoice_dir,
                model_path,
                python_bin,
                source_lang,
                target_lang,
                translator_provider,
                input_device,
                no_prefer_loopback,
                sample_rate,
                channels,
            } => {
                let backend = build_backend(&cli.audio_backend)?;
                commands::pipeline::run_once(
                    backend.as_ref(),
                    seconds,
                    audio_out.as_ref(),
                    keep_temp_audio,
                    cfg.values.asr_provider.as_deref().unwrap_or(&asr_provider),
                    vibevoice_dir.as_ref().or(cfg.values.vibevoice_dir.as_ref()),
                    cfg.values.model_path.as_deref().unwrap_or(&model_path),
                    cfg.values.python_bin.as_deref().unwrap_or(&python_bin),
                    cfg.values.source_lang.as_deref().unwrap_or(&source_lang),
                    cfg.values.target_lang.as_deref().unwrap_or(&target_lang),
                    cfg.values
                        .translator_provider
                        .as_deref()
                        .unwrap_or(&translator_provider),
                    input_device,
                    !no_prefer_loopback,
                    sample_rate.or(cfg.values.sample_rate),
                    channels.or(cfg.values.channels),
                )?
            }
        },
        Commands::Daemon { command } => match command {
            DaemonCommands::Run {
                source_lang,
                target_lang,
                interval_ms,
                count,
            } => commands::daemon::run(&source_lang, &target_lang, interval_ms, count)?,
            DaemonCommands::Serve {
                host,
                port,
                source_lang,
                target_lang,
                interval_ms,
            } => {
                let rt =
                    tokio::runtime::Runtime::new().context("failed to create tokio runtime")?;
                rt.block_on(commands::daemon::serve(
                    &host,
                    port,
                    &source_lang,
                    &target_lang,
                    interval_ms,
                ))?;
            }
        },
        Commands::Visual { command } => match command {
            VisualCommands::Backends => commands::visual::list_backends(),
            VisualCommands::Displays => {
                let backend = commands::visual::build_backend(&cli.visual_backend)?;
                commands::visual::list_displays(backend.as_ref())?;
            }
            VisualCommands::System {
                out,
                seconds,
                fps,
                display,
            } => {
                let backend = commands::visual::build_backend(&cli.visual_backend)?;
                commands::visual::system(
                    backend.as_ref(),
                    out,
                    seconds,
                    fps.or(cfg.values.fps),
                    display,
                )?;
            }
            VisualCommands::App {
                out,
                seconds,
                fps,
                pid,
                name,
            } => {
                let backend = commands::visual::build_backend(&cli.visual_backend)?;
                commands::visual::app(
                    backend.as_ref(),
                    out,
                    seconds,
                    fps.or(cfg.values.fps),
                    pid,
                    name,
                )?;
            }
            VisualCommands::AppsList => {
                let backend = commands::visual::build_backend(&cli.visual_backend)?;
                commands::visual::apps_list(backend.as_ref())?;
            }
            VisualCommands::AppsSplit {
                out_dir,
                seconds,
                fps,
                pid,
                name,
                all_active,
            } => {
                let backend = commands::visual::build_backend(&cli.visual_backend)?;
                commands::visual::apps_split(
                    backend.as_ref(),
                    out_dir,
                    seconds,
                    fps.or(cfg.values.fps),
                    pid,
                    name,
                    all_active,
                )?;
            }
        },
        Commands::Record { command } => match command {
            RecordCommands::System {
                audio_out,
                visual_out,
                seconds,
                sample_rate,
                channels,
                input_device,
                no_prefer_loopback,
                fps,
                display,
            } => commands::record::run(
                &cli.audio_backend,
                &cli.visual_backend,
                audio_out,
                visual_out,
                seconds,
                sample_rate.or(cfg.values.sample_rate),
                channels.or(cfg.values.channels),
                input_device,
                no_prefer_loopback,
                fps.or(cfg.values.fps),
                display,
            )?,
            RecordCommands::App {
                audio_out,
                visual_out,
                seconds,
                sample_rate,
                channels,
                fps,
                pid,
                name,
            } => commands::record::run_app(
                &cli.audio_backend,
                &cli.visual_backend,
                audio_out,
                visual_out,
                seconds,
                sample_rate.or(cfg.values.sample_rate),
                channels.or(cfg.values.channels),
                fps.or(cfg.values.fps),
                pid,
                name,
            )?,
            RecordCommands::AppsSplit {
                out_dir,
                seconds,
                sample_rate,
                channels,
                fps,
                pid,
                name,
                all_active,
                no_audio,
                no_visual,
            } => commands::record::run_apps_split(
                &cli.audio_backend,
                &cli.visual_backend,
                out_dir,
                seconds,
                sample_rate.or(cfg.values.sample_rate),
                channels.or(cfg.values.channels),
                fps.or(cfg.values.fps),
                pid,
                name,
                all_active,
                no_audio,
                no_visual,
            )?,
        },
    }

    Ok(())
}

fn init_logger(level: &str) {
    let env = env_logger::Env::default().default_filter_or(level);
    let _ = env_logger::Builder::from_env(env).is_test(false).try_init();
}

fn humanize_error(err: &anyhow::Error) -> String {
    let text = format!("{err:#}");
    if text.contains("NotImplemented") || text.contains("feature not yet implemented") {
        return format!(
            "This feature is not implemented on your current backend/OS yet.\nDetails: {text}\nHint: try system-level capture first (capture/visual/record system)."
        );
    }
    if text.contains("Screen Recording") || text.contains("SCStreamErrorDomain") {
        return format!(
            "Permission issue: please grant Screen Recording permission to your terminal/host app, then retry.\nDetails: {text}"
        );
    }
    format!("Error: {text}")
}

fn build_backend(backend_id: &str) -> Result<Box<dyn lyricwave_core::audio::AudioBackend>> {
    build_audio_backend(backend_id)
        .map_err(anyhow::Error::msg)
        .with_context(|| format!("failed to initialize audio backend '{backend_id}'"))
}
