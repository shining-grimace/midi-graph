use crate::{BufferConsumer, BufferConsumerNode, Error, Node, NodeEvent};

pub struct NullSource {
    node_id: u64,
}

impl NullSource {
    pub fn new(node_id: Option<u64>) -> Self {
        Self {
            node_id: node_id.unwrap_or_else(|| <Self as Node>::new_node_id()),
        }
    }
}

impl BufferConsumerNode for NullSource {}

impl Node for NullSource {
    fn get_node_id(&self) -> u64 {
        self.node_id
    }

    fn on_event(&mut self, _event: &NodeEvent) {}

    fn fill_buffer(&mut self, _buffer: &mut [f32]) {}
}

impl BufferConsumer for NullSource {
    fn duplicate(&self) -> Result<Box<dyn BufferConsumerNode + Send + 'static>, Error> {
        let source = Self::new(Some(self.node_id));
        Ok(Box::new(source))
    }
}
