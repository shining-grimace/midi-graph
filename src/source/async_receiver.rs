use crate::{BufferConsumer, BufferConsumerNode, Error, Node, NoteConsumerNode, NoteEvent, Status};
use crossbeam_channel::{unbounded, Receiver, Sender};

pub struct AsyncEventReceiver {
    receiver: Receiver<NoteEvent>,
    consumer: Box<dyn NoteConsumerNode + Send + 'static>,
}

impl AsyncEventReceiver {
    pub fn new(consumer: Box<dyn NoteConsumerNode + Send + 'static>) -> (Sender<NoteEvent>, Self) {
        let (sender, receiver) = unbounded();
        let async_receiver = AsyncEventReceiver { receiver, consumer };
        (sender, async_receiver)
    }
}

impl BufferConsumerNode for AsyncEventReceiver {}

impl Node for AsyncEventReceiver {
    fn on_event(&mut self, _event: &NoteEvent) {}
}

impl BufferConsumer for AsyncEventReceiver {
    fn duplicate(&self) -> Result<Box<dyn BufferConsumerNode + Send + 'static>, Error> {
        Err(Error::User(
            "AsyncEventReceiver cannot be duplicated".to_owned(),
        ))
    }

    fn fill_buffer(&mut self, buffer: &mut [f32]) -> Status {
        for event in self.receiver.try_iter() {
            self.consumer.on_event(&event);
        }
        self.consumer.fill_buffer(buffer);
        Status::Ok
    }
}
