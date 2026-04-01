use std::path::PathBuf;

use super::{
    AsrEngine, AsrFileEngine, MockAsrEngine, MockTranslator, Translator, VibeVoiceAsrEngine,
};

#[derive(Debug, Clone, Copy)]
pub enum ProviderMode {
    LocalOffline,
    OnlineApi,
    Hybrid,
}

#[derive(Debug, Clone, Copy)]
pub struct ProviderDescriptor {
    pub id: &'static str,
    pub capability: &'static str,
    pub mode: ProviderMode,
    pub requires_setup: bool,
    pub note: &'static str,
}

pub fn asr_text_providers() -> Vec<ProviderDescriptor> {
    vec![ProviderDescriptor {
        id: "mock",
        capability: "text->text",
        mode: ProviderMode::LocalOffline,
        requires_setup: false,
        note: "Built-in testing provider.",
    }]
}

pub fn asr_file_providers() -> Vec<ProviderDescriptor> {
    vec![
        ProviderDescriptor {
            id: "vibevoice",
            capability: "audio-file->text",
            mode: ProviderMode::LocalOffline,
            requires_setup: true,
            note: "External microsoft/VibeVoice checkout and model environment required.",
        },
        ProviderDescriptor {
            id: "openai",
            capability: "audio-file->text",
            mode: ProviderMode::OnlineApi,
            requires_setup: true,
            note: "Planned provider (API key based).",
        },
        ProviderDescriptor {
            id: "deepgram",
            capability: "audio-file->text",
            mode: ProviderMode::OnlineApi,
            requires_setup: true,
            note: "Planned provider (API key based).",
        },
    ]
}

pub fn translator_providers() -> Vec<ProviderDescriptor> {
    vec![
        ProviderDescriptor {
            id: "mock",
            capability: "text->text",
            mode: ProviderMode::LocalOffline,
            requires_setup: false,
            note: "Built-in testing translator.",
        },
        ProviderDescriptor {
            id: "passthrough",
            capability: "text->text",
            mode: ProviderMode::LocalOffline,
            requires_setup: false,
            note: "Returns source text without translation.",
        },
        ProviderDescriptor {
            id: "deepl",
            capability: "text->text",
            mode: ProviderMode::OnlineApi,
            requires_setup: true,
            note: "Planned provider (API key based).",
        },
    ]
}

pub fn build_text_asr(provider_id: &str) -> Result<Box<dyn AsrEngine>, String> {
    match provider_id {
        "mock" => Ok(Box::new(MockAsrEngine)),
        _ => Err(format!("unknown text ASR provider: {provider_id}")),
    }
}

pub fn build_translator(provider_id: &str) -> Result<Box<dyn Translator>, String> {
    match provider_id {
        "mock" => Ok(Box::new(MockTranslator)),
        "passthrough" => Ok(Box::new(PassthroughTranslator)),
        _ => Err(format!("unknown translator provider: {provider_id}")),
    }
}

pub fn build_file_asr_vibevoice(
    python_bin: String,
    repo_dir: PathBuf,
    model_path: String,
) -> Box<dyn AsrFileEngine> {
    Box::new(VibeVoiceAsrEngine {
        python_bin,
        repo_dir,
        model_path,
    })
}

pub struct PassthroughTranslator;

impl Translator for PassthroughTranslator {
    fn name(&self) -> &'static str {
        "passthrough"
    }

    fn translate(&self, input: &str, _target_lang: &str) -> String {
        input.to_string()
    }
}
