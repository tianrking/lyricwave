use crate::pipeline::Translator;

pub struct PassthroughTranslatorProvider;

impl Translator for PassthroughTranslatorProvider {
    fn name(&self) -> &'static str {
        "passthrough"
    }

    fn translate(&self, input: &str, _target_lang: &str) -> Result<String, String> {
        Ok(input.to_string())
    }
}
