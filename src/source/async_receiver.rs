use crate::{BufferConsumer, Error, NoteConsumer, NoteEvent, Status};
use crossbeam_channel::{unbounded, Receiver, Sender};

pub struct AsyncEventReceiver {
    receiver: Receiver<NoteEvent>,
    consumer: Box<dyn NoteConsumer + Send + 'static>,
}

impl AsyncEventReceiver {
    pub fn new(consumer: Box<dyn NoteConsumer + Send + 'static>) -> (Sender<NoteEvent>, Self) {
        let (sender, receiver) = unbounded();
        let async_receiver = AsyncEventReceiver { receiver, consumer };
        (sender, async_receiver)
    }
}

impl BufferConsumer for AsyncEventReceiver {
    fn duplicate(&self) -> Result<Box<dyn BufferConsumer + Send + 'static>, Error> {
        Err(Error::User(
            "AsyncEventReceiver cannot be duplicated".to_owned(),
        ))
    }

    fn set_note(&mut self, _event: NoteEvent) {}

    fn fill_buffer(&mut self, buffer: &mut [f32]) -> Status {
        for event in self.receiver.try_iter() {
            self.consumer.restart_with_event(&event);
        }
        self.consumer.fill_buffer(buffer);
        Status::Ok
    }
}
