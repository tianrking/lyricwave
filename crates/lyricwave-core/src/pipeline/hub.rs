use tokio::sync::broadcast;

use super::TranscriptEvent;

#[derive(Clone)]
pub struct EventHub {
    tx: broadcast::Sender<TranscriptEvent>,
}

impl EventHub {
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity);
        Self { tx }
    }

    pub fn publish(&self, evt: TranscriptEvent) {
        let _ = self.tx.send(evt);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<TranscriptEvent> {
        self.tx.subscribe()
    }
}
