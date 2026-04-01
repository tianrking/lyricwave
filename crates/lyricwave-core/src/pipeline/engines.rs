use std::path::Path;

pub trait AsrEngine: Send + Sync {
    fn name(&self) -> &'static str;
    fn transcribe_text(&self, input: &str) -> String;
}

pub trait AsrFileEngine: Send + Sync {
    fn name(&self) -> &'static str;
    fn transcribe_file(&self, input_audio: &Path) -> Result<String, String>;
}

pub trait Translator: Send + Sync {
    fn name(&self) -> &'static str;
    fn translate(&self, input: &str, target_lang: &str) -> Result<String, String>;
}
