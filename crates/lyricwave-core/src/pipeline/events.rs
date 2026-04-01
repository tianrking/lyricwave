use std::time::Duration;

#[derive(Debug, Clone)]
pub struct LanguageTag(pub String);

#[derive(Debug, Clone)]
pub struct TranscriptEvent {
    pub source_text: String,
    pub translated_text: Option<String>,
    pub source_language: Option<LanguageTag>,
    pub target_language: Option<LanguageTag>,
    pub start: Duration,
    pub end: Duration,
    pub is_final: bool,
}
