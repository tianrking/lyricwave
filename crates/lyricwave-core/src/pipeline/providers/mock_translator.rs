use crate::pipeline::Translator;

pub struct MockTranslatorProvider;

impl Translator for MockTranslatorProvider {
    fn name(&self) -> &'static str {
        "mock-translator"
    }

    fn translate(&self, input: &str, target_lang: &str) -> Result<String, String> {
        Ok(format!("[{target_lang}] {input}"))
    }
}
