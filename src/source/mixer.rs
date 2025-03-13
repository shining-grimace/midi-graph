use crate::{consts, Error, Node, NodeControlEvent, NodeEvent};

pub struct MixerSource {
    node_id: u64,
    balance: f32,
    consumer_0: Box<dyn Node + Send + 'static>,
    consumer_1: Box<dyn Node + Send + 'static>,
    intermediate_buffer: Vec<f32>,
}

impl MixerSource {
    pub fn new(
        node_id: Option<u64>,
        balance: f32,
        consumer_0: Box<dyn Node + Send + 'static>,
        consumer_1: Box<dyn Node + Send + 'static>,
    ) -> Self {
        Self {
            node_id: node_id.unwrap_or_else(<Self as Node>::new_node_id),
            balance,
            consumer_0,
            consumer_1,
            intermediate_buffer: vec![0.0; consts::BUFFER_SIZE * consts::CHANNEL_COUNT],
        }
    }
}

impl Node for MixerSource {
    fn get_node_id(&self) -> u64 {
        self.node_id
    }

    fn duplicate(&self) -> Result<Box<dyn Node + Send + 'static>, Error> {
        let consumer_0 = self.consumer_0.duplicate()?;
        let consumer_1 = self.consumer_1.duplicate()?;
        let mixer = Self::new(Some(self.node_id), self.balance, consumer_0, consumer_1);
        Ok(Box::new(mixer))
    }

    fn on_event(&mut self, event: &NodeEvent) {
        if let NodeEvent::NodeControl {
            node_id,
            event: NodeControlEvent::MixerBalance(balance),
        } = event
        {
            if *node_id == self.node_id {
                self.balance = *balance;
                return;
            }
        }
        self.consumer_0.on_event(event);
        self.consumer_1.on_event(event);
    }

    fn fill_buffer(&mut self, buffer: &mut [f32]) {
        let buffer_size = buffer.len();
        let sample_count = buffer_size / consts::CHANNEL_COUNT;
        let intermediate_slice = &mut self.intermediate_buffer[0..buffer_size];
        intermediate_slice.fill(0.0);
        self.consumer_0.fill_buffer(intermediate_slice);
        let multiplier_0 = 1.0 - self.balance;
        for i in 0..sample_count {
            let index = i * 2;
            buffer[index] += multiplier_0 * intermediate_slice[index];
            buffer[index + 1] += multiplier_0 * intermediate_slice[index + 1];
        }
        intermediate_slice.fill(0.0);
        self.consumer_1.fill_buffer(intermediate_slice);
        for i in 0..sample_count {
            let index = i * 2;
            buffer[index] += self.balance * intermediate_slice[index];
            buffer[index + 1] += self.balance * intermediate_slice[index + 1];
        }
    }

    fn replace_children(
        &mut self,
        children: &[Box<dyn Node + Send + 'static>],
    ) -> Result<(), Error> {
        if children.len() != 2 {
            return Err(Error::User("Mixer requires two children".to_owned()));
        }
        self.consumer_0 = children[0].duplicate()?;
        self.consumer_1 = children[1].duplicate()?;
        Ok(())
    }
}
