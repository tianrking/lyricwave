use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageTag(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptEvent {
    pub source_text: String,
    pub translated_text: Option<String>,
    pub source_language: Option<LanguageTag>,
    pub target_language: Option<LanguageTag>,
    pub start_ms: u64,
    pub end_ms: u64,
    pub is_final: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DaemonEvent {
    Status {
        message: String,
        emitted_at_ms: u128,
    },
    Transcript {
        payload: TranscriptEvent,
        emitted_at_ms: u128,
    },
    Error {
        message: String,
        emitted_at_ms: u128,
    },
}

impl DaemonEvent {
    pub fn now_ms() -> u128 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_millis()
    }
}
