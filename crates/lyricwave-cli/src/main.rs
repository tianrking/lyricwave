mod cli;
mod commands;

use anyhow::{Context, Result};
use clap::Parser;
use lyricwave_core::audio::build_audio_backend;

use crate::cli::{
    BackendCommands, CaptureCommands, Cli, Commands, DaemonCommands, DeviceCommands,
    PipelineCommands, ProviderCommands, RecordCommands, VisualCommands,
};

fn main() -> Result<()> {
    let cli = Cli::parse();

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
                        sample_rate,
                        channels,
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
                        sample_rate,
                        channels,
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
                        sample_rate,
                        channels,
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
            } => commands::pipeline::asr_file(
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
                    &asr_provider,
                    vibevoice_dir.as_ref(),
                    &model_path,
                    &python_bin,
                    &source_lang,
                    &target_lang,
                    &translator_provider,
                    input_device,
                    !no_prefer_loopback,
                    sample_rate,
                    channels,
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
            VisualCommands::CaptureDisplay {
                out,
                seconds,
                fps,
                display,
            } => {
                let backend = commands::visual::build_backend(&cli.visual_backend)?;
                commands::visual::capture_display(backend.as_ref(), out, seconds, fps, display)?;
            }
        },
        Commands::Record { command } => match command {
            RecordCommands::Run {
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
                sample_rate,
                channels,
                input_device,
                no_prefer_loopback,
                fps,
                display,
            )?,
        },
    }

    Ok(())
}

fn build_backend(backend_id: &str) -> Result<Box<dyn lyricwave_core::audio::AudioBackend>> {
    build_audio_backend(backend_id)
        .map_err(anyhow::Error::msg)
        .with_context(|| format!("failed to initialize audio backend '{backend_id}'"))
}
