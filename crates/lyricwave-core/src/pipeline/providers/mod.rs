mod mock_asr;
mod mock_translator;
mod passthrough_translator;
mod registry;
mod types;
mod vibevoice_file_asr;

pub use mock_asr::MockAsrProvider;
pub use mock_translator::MockTranslatorProvider;
pub use passthrough_translator::PassthroughTranslatorProvider;
pub use registry::{
    FileAsrBuildContext, asr_file_providers, asr_text_providers, build_file_asr, build_text_asr,
    build_translator, translator_providers,
};
pub use types::{ProviderDescriptor, ProviderMode};
pub use vibevoice_file_asr::VibeVoiceFileAsrProvider;
