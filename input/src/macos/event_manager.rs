use tokio::sync::{mpsc::{self, UnboundedReceiver}, oneshot::{self, Receiver}};

use crate::{EventWriter, event::Event};
use std::io::{Error, ErrorKind};

pub struct EventManager {
    writer: EventWriter,
    event_receiver: UnboundedReceiver<Result<Event, Error>>,
    watcher_receiver: Receiver<Error>,
}

impl EventManager {
    pub async fn new() -> Result<Self, Error> {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        let (watcher_sender, watcher_receiver) = oneshot::channel();

        let writer = EventWriter::new().await?;
        Ok(EventManager {
            writer,
            event_receiver,
            watcher_receiver,
        })
    }

    pub async fn read(&mut self) -> Result<Event, Error> {
        if let Ok(err) = self.watcher_receiver.try_recv() {
            return Err(err);
        }

        self.event_receiver
            .recv()
            .await
            .ok_or_else(|| Error::new(ErrorKind::Other, "All devices closed"))?
    }

    pub async fn write(&mut self, event: Event) -> Result<(), Error> {
        self.writer.write(event).await
    }
}
