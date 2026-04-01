use crate::pipeline::AsrEngine;

pub struct MockAsrProvider;

impl AsrEngine for MockAsrProvider {
    fn name(&self) -> &'static str {
        "mock-asr"
    }

    fn transcribe_text(&self, input: &str) -> String {
        input.trim().to_string()
    }
}
