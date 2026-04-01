use crate::pipeline::Translator;

pub struct MockTranslatorProvider;

impl Translator for MockTranslatorProvider {
    fn name(&self) -> &'static str {
        "mock-translator"
    }

    fn translate(&self, input: &str, target_lang: &str) -> String {
        format!("[{target_lang}] {input}")
    }
}
