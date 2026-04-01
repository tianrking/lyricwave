use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use lyricwave_core::audio::{AudioBackend, CaptureFormat};
use lyricwave_core::pipeline::{
    FileAsrBuildContext, MockAsrProvider, MockTranslatorProvider, TranslatorBuildContext,
    build_file_asr, build_text_asr, build_translator,
};
use lyricwave_core::service::PipelineService;
use serde_json::json;

use crate::commands::capture;

pub fn demo(
    text: &str,
    source_lang: &str,
    target_lang: &str,
    asr_provider: &str,
    translator_provider: &str,
) -> Result<()> {
    let asr = build_text_asr(asr_provider).map_err(anyhow::Error::msg)?;
    let translator = build_translator(translator_provider, TranslatorBuildContext::from_env())
        .map_err(anyhow::Error::msg)?;
    let transcribed = asr.transcribe_text(text);
    let translated = translator
        .translate(&transcribed, target_lang)
        .map_err(anyhow::Error::msg)?;

    let service = PipelineService::new(MockAsrProvider, MockTranslatorProvider, 64);
    let mut evt = service.process_text(&transcribed, source_lang, target_lang);
    evt.translated_text = Some(translated);

    println!("asr_provider: {}", asr.name());
    println!("translator_provider: {}", translator.name());
    println!("source: {}", evt.source_text);
    println!("translation: {}", evt.translated_text.unwrap_or_default());
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn asr_file(
    audio: &Path,
    asr_provider: &str,
    vibevoice_dir: Option<&PathBuf>,
    model_path: &str,
    python_bin: &str,
    source_lang: &str,
    target_lang: &str,
    translator_provider: &str,
    no_translate: bool,
) -> Result<()> {
    let asr = build_file_asr(
        asr_provider,
        FileAsrBuildContext {
            python_bin: python_bin.to_string(),
            vibevoice_dir: vibevoice_dir.cloned(),
            model_path: model_path.to_string(),
        },
    )
    .map_err(anyhow::Error::msg)?;

    let source_text = asr
        .transcribe_file(audio)
        .map_err(anyhow::Error::msg)
        .with_context(|| format!("{} failed for {}", asr.name(), audio.display()))?;

    println!("asr_provider: {}", asr.name());
    println!("source: {source_text}");

    if !no_translate {
        let translator = build_translator(translator_provider, TranslatorBuildContext::from_env())
            .map_err(anyhow::Error::msg)?;
        let translated = translator
            .translate(&source_text, target_lang)
            .map_err(anyhow::Error::msg)?;
        println!("translator: {}", translator.name());
        println!("translation: {translated}");
    }

    println!("source_lang_hint: {source_lang}");
    println!("target_lang: {target_lang}");
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn run_once(
    backend: &dyn AudioBackend,
    seconds: u32,
    audio_out: Option<&PathBuf>,
    keep_temp_audio: bool,
    asr_provider: &str,
    vibevoice_dir: Option<&PathBuf>,
    model_path: &str,
    python_bin: &str,
    source_lang: &str,
    target_lang: &str,
    translator_provider: &str,
    input_device: Option<String>,
    prefer_loopback: bool,
    sample_rate: Option<u32>,
    channels: Option<u16>,
) -> Result<()> {
    let asr = build_file_asr(
        asr_provider,
        FileAsrBuildContext {
            python_bin: python_bin.to_string(),
            vibevoice_dir: vibevoice_dir.cloned(),
            model_path: model_path.to_string(),
        },
    )
    .map_err(anyhow::Error::msg)?;
    let translator = build_translator(translator_provider, TranslatorBuildContext::from_env())
        .map_err(anyhow::Error::msg)?;

    let explicit_output = audio_out.cloned();
    let generated_temp = explicit_output.is_none();
    let capture_path = explicit_output.unwrap_or_else(temp_capture_path);

    let capture_report = capture::system(
        backend,
        Some(capture_path.clone()),
        false,
        Some(seconds),
        sample_rate,
        channels,
        CaptureFormat::Wav,
        input_device,
        prefer_loopback,
    )?;

    let source_text = asr
        .transcribe_file(&capture_path)
        .map_err(anyhow::Error::msg)
        .with_context(|| format!("{} failed for {}", asr.name(), capture_path.display()))?;
    let translated = translator
        .translate(&source_text, target_lang)
        .map_err(anyhow::Error::msg)?;

    let output = json!({
        "capture_file": capture_path.display().to_string(),
        "capture": {
            "samples": capture_report.captured_samples,
            "sample_rate": capture_report.sample_rate,
            "channels": capture_report.channels,
            "selected_input_device": {
                "id": capture_report.selected_input_device.id,
                "name": capture_report.selected_input_device.name,
                "is_default": capture_report.selected_input_device.is_default,
                "loopback_score": capture_report.selected_input_device.loopback_score,
                "is_loopback_candidate": capture_report.selected_input_device.is_loopback_candidate
            },
            "selection_reason": capture_report.selection_reason
        },
        "asr_provider": asr.name(),
        "translator_provider": translator.name(),
        "source_lang_hint": source_lang,
        "target_lang": target_lang,
        "source_text": source_text,
        "translated_text": translated,
    });
    println!("{}", serde_json::to_string_pretty(&output)?);

    if generated_temp && !keep_temp_audio {
        let _ = std::fs::remove_file(&capture_path);
    }

    Ok(())
}

fn temp_capture_path() -> PathBuf {
    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    std::env::temp_dir().join(format!("lyricwave-capture-{ms}.wav"))
}
