mod cli;
mod commands;

use anyhow::{Context, Result};
use clap::Parser;
use lyricwave_core::audio::build_audio_backend;

use crate::cli::{
    BackendCommands, CaptureCommands, Cli, Commands, DaemonCommands, DeviceCommands,
    PipelineCommands, ProviderCommands,
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
                    )?;
                    eprintln!(
                        "captured {} samples @ {}Hz ({} channels)",
                        report.captured_samples, report.sample_rate, report.channels
                    );
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
    }

    Ok(())
}

fn build_backend(backend_id: &str) -> Result<Box<dyn lyricwave_core::audio::AudioBackend>> {
    build_audio_backend(backend_id)
        .map_err(anyhow::Error::msg)
        .with_context(|| format!("failed to initialize audio backend '{backend_id}'"))
}
