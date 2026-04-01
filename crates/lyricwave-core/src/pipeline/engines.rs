pub trait AsrEngine: Send + Sync {
    fn name(&self) -> &'static str;
    fn transcribe_text(&self, input: &str) -> String;
}

pub trait Translator: Send + Sync {
    fn name(&self) -> &'static str;
    fn translate(&self, input: &str, target_lang: &str) -> String;
}

pub struct MockAsrEngine;

impl AsrEngine for MockAsrEngine {
    fn name(&self) -> &'static str {
        "mock-asr"
    }

    fn transcribe_text(&self, input: &str) -> String {
        input.trim().to_string()
    }
}

pub struct MockTranslator;

impl Translator for MockTranslator {
    fn name(&self) -> &'static str {
        "mock-translator"
    }

    fn translate(&self, input: &str, target_lang: &str) -> String {
        format!("[{target_lang}] {input}")
    }
}
