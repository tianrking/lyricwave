use tokio::sync::broadcast;

use super::DaemonEvent;

#[derive(Clone)]
pub struct EventHub {
    tx: broadcast::Sender<DaemonEvent>,
}

impl EventHub {
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity);
        Self { tx }
    }

    pub fn publish(&self, evt: DaemonEvent) {
        let _ = self.tx.send(evt);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<DaemonEvent> {
        self.tx.subscribe()
    }
}
