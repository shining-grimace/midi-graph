use crate::{Error, Node, NodeEvent};
use crossbeam_channel::{unbounded, Receiver, Sender};
use std::ops::{Deref, DerefMut};

pub struct EventChannel {
    pub for_node_id: u64,
    pub sender: Sender<NodeEvent>,
}

impl Deref for EventChannel {
    type Target = Sender<NodeEvent>;

    fn deref(&self) -> &Sender<NodeEvent> {
        &self.sender
    }
}

impl DerefMut for EventChannel {
    fn deref_mut(&mut self) -> &mut Sender<NodeEvent> {
        &mut self.sender
    }
}

pub struct AsyncEventReceiver {
    node_id: u64,
    receiver: Receiver<NodeEvent>,
    consumer: Box<dyn Node + Send + 'static>,
}

impl AsyncEventReceiver {
    pub fn new(
        node_id: Option<u64>,
        consumer: Box<dyn Node + Send + 'static>,
    ) -> (EventChannel, Self) {
        let (sender, receiver) = unbounded();
        let async_receiver = AsyncEventReceiver {
            node_id: node_id.unwrap_or_else(<Self as Node>::new_node_id),
            receiver,
            consumer,
        };
        let channel = EventChannel {
            for_node_id: async_receiver.node_id,
            sender,
        };
        (channel, async_receiver)
    }
}

impl Node for AsyncEventReceiver {
    fn get_node_id(&self) -> u64 {
        self.node_id
    }

    fn duplicate(&self) -> Result<Box<dyn Node + Send + 'static>, Error> {
        Err(Error::User(
            "AsyncEventReceiver cannot be duplicated".to_owned(),
        ))
    }

    fn on_event(&mut self, _event: &NodeEvent) {}

    fn fill_buffer(&mut self, buffer: &mut [f32]) {
        for event in self.receiver.try_iter() {
            self.consumer.on_event(&event);
        }
        self.consumer.fill_buffer(buffer);
    }
}
