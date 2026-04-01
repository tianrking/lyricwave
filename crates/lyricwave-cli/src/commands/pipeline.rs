use std::path::PathBuf;

use anyhow::{Context, Result};
use lyricwave_core::pipeline::{
    FileAsrBuildContext, MockAsrProvider, MockTranslatorProvider, TranslatorBuildContext,
    build_file_asr, build_text_asr, build_translator,
};
use lyricwave_core::service::PipelineService;

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
