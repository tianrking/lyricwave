use std::env;
use std::path::PathBuf;

use crate::pipeline::{AsrEngine, AsrFileEngine, Translator};

use super::deepl_translator::DeepLTranslatorProvider;
use super::libretranslate_translator::LibreTranslateProvider;
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

pub struct TranslatorBuildContext {
    pub deepl_api_key: Option<String>,
    pub deepl_base_url: Option<String>,
    pub libretranslate_base_url: Option<String>,
    pub libretranslate_api_key: Option<String>,
}

impl TranslatorBuildContext {
    pub fn from_env() -> Self {
        Self {
            deepl_api_key: env::var("DEEPL_API_KEY").ok(),
            deepl_base_url: env::var("DEEPL_BASE_URL").ok(),
            libretranslate_base_url: env::var("LIBRETRANSLATE_BASE_URL").ok(),
            libretranslate_api_key: env::var("LIBRETRANSLATE_API_KEY").ok(),
        }
    }
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
            note: "DeepL API translation (requires DEEPL_API_KEY).",
        },
        ProviderDescriptor {
            id: "libretranslate",
            capability: "text->text",
            mode: ProviderMode::OnlineApi,
            requires_setup: false,
            note: "LibreTranslate HTTP translation (self-hosted recommended).",
        },
    ]
}

pub fn build_text_asr(provider_id: &str) -> Result<Box<dyn AsrEngine>, String> {
    match provider_id {
        "mock" => Ok(Box::new(MockAsrProvider)),
        _ => Err(format!("unknown text ASR provider: {provider_id}")),
    }
}

pub fn build_translator(
    provider_id: &str,
    ctx: TranslatorBuildContext,
) -> Result<Box<dyn Translator>, String> {
    match provider_id {
        "mock" => Ok(Box::new(MockTranslatorProvider)),
        "passthrough" => Ok(Box::new(PassthroughTranslatorProvider)),
        "deepl" => {
            let api_key = ctx
                .deepl_api_key
                .ok_or_else(|| "missing DEEPL_API_KEY for deepl provider".to_string())?;
            let base_url = ctx
                .deepl_base_url
                .unwrap_or_else(|| "https://api-free.deepl.com".to_string());
            Ok(Box::new(DeepLTranslatorProvider { api_key, base_url }))
        }
        "libretranslate" => {
            let base_url = ctx
                .libretranslate_base_url
                .unwrap_or_else(|| "http://127.0.0.1:5000".to_string());
            Ok(Box::new(LibreTranslateProvider {
                base_url,
                api_key: ctx.libretranslate_api_key,
            }))
        }
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
