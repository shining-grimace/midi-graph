use crate::{BufferConsumer, BufferConsumerNode, Error, Node, NodeEvent, NoteConsumerNode, Status};
use crossbeam_channel::{unbounded, Receiver, Sender};

pub struct AsyncEventReceiver {
    node_id: u64,
    receiver: Receiver<NodeEvent>,
    consumer: Box<dyn NoteConsumerNode + Send + 'static>,
}

impl AsyncEventReceiver {
    pub fn new(
        node_id: Option<u64>,
        consumer: Box<dyn NoteConsumerNode + Send + 'static>,
    ) -> (Sender<NodeEvent>, Self) {
        let (sender, receiver) = unbounded();
        let async_receiver = AsyncEventReceiver {
            node_id: node_id.unwrap_or_else(|| <Self as Node>::new_node_id()),
            receiver,
            consumer,
        };
        (sender, async_receiver)
    }
}

impl BufferConsumerNode for AsyncEventReceiver {}

impl Node for AsyncEventReceiver {
    fn get_node_id(&self) -> u64 {
        self.node_id
    }

    fn on_event(&mut self, _event: &NodeEvent) {}
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
