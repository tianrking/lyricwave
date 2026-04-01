use crate::pipeline::{AsrEngine, DaemonEvent, EventHub, LanguageTag, TranscriptEvent, Translator};

pub struct PipelineService<A, T>
where
    A: AsrEngine,
    T: Translator,
{
    pub asr: A,
    pub translator: T,
    pub hub: EventHub,
}

impl<A, T> PipelineService<A, T>
where
    A: AsrEngine,
    T: Translator,
{
    pub fn new(asr: A, translator: T, hub_capacity: usize) -> Self {
        Self {
            asr,
            translator,
            hub: EventHub::new(hub_capacity),
        }
    }

    pub fn process_text(
        &self,
        text: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> TranscriptEvent {
        let transcribed = self.asr.transcribe_text(text);
        let translated_text = match self.translator.translate(&transcribed, target_lang) {
            Ok(text) => Some(text),
            Err(err) => {
                self.hub.publish(DaemonEvent::Error {
                    message: format!("translator {} failed: {err}", self.translator.name()),
                    emitted_at_ms: DaemonEvent::now_ms(),
                });
                None
            }
        };

        let evt = TranscriptEvent {
            source_text: transcribed,
            translated_text,
            source_language: Some(LanguageTag(source_lang.to_string())),
            target_language: Some(LanguageTag(target_lang.to_string())),
            start_ms: 0,
            end_ms: 1200,
            is_final: true,
        };

        self.hub.publish(DaemonEvent::Transcript {
            payload: evt.clone(),
            emitted_at_ms: DaemonEvent::now_ms(),
        });

        evt
    }
}
