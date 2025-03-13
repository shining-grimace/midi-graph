use crate::{Error, Node, NodeEvent};

pub struct NullSource {
    node_id: u64,
}

impl NullSource {
    pub fn new(node_id: Option<u64>) -> Self {
        Self {
            node_id: node_id.unwrap_or_else(<Self as Node>::new_node_id),
        }
    }
}

impl Node for NullSource {
    fn get_node_id(&self) -> u64 {
        self.node_id
    }

    fn duplicate(&self) -> Result<Box<dyn Node + Send + 'static>, Error> {
        let source = Self::new(Some(self.node_id));
        Ok(Box::new(source))
    }

    fn on_event(&mut self, _event: &NodeEvent) {}

    fn fill_buffer(&mut self, _buffer: &mut [f32]) {}

    fn replace_children(
        &mut self,
        children: &[Box<dyn Node + Send + 'static>],
    ) -> Result<(), Error> {
        match children.is_empty() {
            true => Ok(()),
            false => Err(Error::User("NullSource cannot have children".to_owned()))
        }
    }
}
