use crate::{BufferConsumer, BufferConsumerNode, Error, Node, NodeEvent};
use crossbeam_channel::{unbounded, Receiver, Sender};
use std::ops::{Deref, DerefMut};

pub struct EventChannel(pub Sender<NodeEvent>);

impl Deref for EventChannel {
    type Target = Sender<NodeEvent>;

    fn deref(&self) -> &Sender<NodeEvent> {
        &self.0
    }
}

impl DerefMut for EventChannel {
    fn deref_mut(&mut self) -> &mut Sender<NodeEvent> {
        &mut self.0
    }
}

pub struct AsyncEventReceiver {
    node_id: u64,
    receiver: Receiver<NodeEvent>,
    consumer: Box<dyn BufferConsumerNode + Send + 'static>,
}

impl AsyncEventReceiver {
    pub fn new(
        node_id: Option<u64>,
        consumer: Box<dyn BufferConsumerNode + Send + 'static>,
    ) -> (EventChannel, Self) {
        let (sender, receiver) = unbounded();
        let async_receiver = AsyncEventReceiver {
            node_id: node_id.unwrap_or_else(<Self as Node>::new_node_id),
            receiver,
            consumer,
        };
        (EventChannel(sender), async_receiver)
    }
}

impl BufferConsumerNode for AsyncEventReceiver {}

impl Node for AsyncEventReceiver {
    fn get_node_id(&self) -> u64 {
        self.node_id
    }

    fn on_event(&mut self, _event: &NodeEvent) {}

    fn fill_buffer(&mut self, buffer: &mut [f32]) {
        for event in self.receiver.try_iter() {
            self.consumer.on_event(&event);
        }
        self.consumer.fill_buffer(buffer);
    }
}

impl BufferConsumer for AsyncEventReceiver {
    fn duplicate(&self) -> Result<Box<dyn BufferConsumerNode + Send + 'static>, Error> {
        Err(Error::User(
            "AsyncEventReceiver cannot be duplicated".to_owned(),
        ))
    }
}
