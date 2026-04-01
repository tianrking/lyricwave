use std::path::PathBuf;

use crate::pipeline::{AsrEngine, AsrFileEngine, Translator};

use super::mock_asr::MockAsrProvider;
use super::mock_translator::MockTranslatorProvider;
use super::passthrough_translator::PassthroughTranslatorProvider;
use super::types::{ProviderDescriptor, ProviderMode};
use super::vibevoice_file_asr::VibeVoiceFileAsrProvider;

pub struct FileAsrBuildContext {
    pub python_bin: String,
    pub vibevoice_dir: Option<PathBuf>,
    pub model_path: String,
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
        "mock" => Ok(Box::new(MockAsrProvider)),
        _ => Err(format!("unknown text ASR provider: {provider_id}")),
    }
}

pub fn build_translator(provider_id: &str) -> Result<Box<dyn Translator>, String> {
    match provider_id {
        "mock" => Ok(Box::new(MockTranslatorProvider)),
        "passthrough" => Ok(Box::new(PassthroughTranslatorProvider)),
        _ => Err(format!("unknown translator provider: {provider_id}")),
    }
}

pub fn build_file_asr(
    provider_id: &str,
    ctx: FileAsrBuildContext,
) -> Result<Box<dyn AsrFileEngine>, String> {
    match provider_id {
        "vibevoice" => {
            let repo_dir = ctx.vibevoice_dir.ok_or_else(|| {
                "--vibevoice-dir is required when --asr-provider vibevoice".to_string()
            })?;
            Ok(Box::new(VibeVoiceFileAsrProvider {
                python_bin: ctx.python_bin,
                repo_dir,
                model_path: ctx.model_path,
            }))
        }
        _ => Err(format!("unknown file ASR provider: {provider_id}")),
    }
}
