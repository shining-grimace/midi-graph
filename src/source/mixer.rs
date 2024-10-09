use crate::{consts, BufferConsumer, BufferConsumerNode, Error, Node, NodeEvent, Status};

pub struct MixerSource {
    node_id: u64,
    balance: f32,
    consumer_0: Box<dyn BufferConsumerNode + Send + 'static>,
    consumer_1: Box<dyn BufferConsumerNode + Send + 'static>,
    intermediate_buffer_0: Vec<f32>,
    intermediate_buffer_1: Vec<f32>,
}

impl MixerSource {
    pub fn new(
        node_id: Option<u64>,
        balance: f32,
        consumer_0: Box<dyn BufferConsumerNode + Send + 'static>,
        consumer_1: Box<dyn BufferConsumerNode + Send + 'static>,
    ) -> Self {
        Self {
            node_id: node_id.unwrap_or_else(|| <Self as Node>::new_node_id()),
            balance,
            consumer_0,
            consumer_1,
            intermediate_buffer_0: vec![0.0; consts::BUFFER_SIZE * consts::CHANNEL_COUNT],
            intermediate_buffer_1: vec![0.0; consts::BUFFER_SIZE * consts::CHANNEL_COUNT],
        }
    }
}

impl BufferConsumerNode for MixerSource {}

impl Node for MixerSource {
    fn get_node_id(&self) -> u64 {
        self.node_id
    }

    fn on_event(&mut self, event: &NodeEvent) {
        self.consumer_0.on_event(event);
        self.consumer_1.on_event(event);
    }
}

impl BufferConsumer for MixerSource {
    fn duplicate(&self) -> Result<Box<dyn BufferConsumerNode + Send + 'static>, Error> {
        let consumer_0 = self.consumer_0.duplicate()?;
        let consumer_1 = self.consumer_1.duplicate()?;
        let mixer = Self::new(Some(self.node_id), self.balance, consumer_0, consumer_1);
        Ok(Box::new(mixer))
    }

    fn fill_buffer(&mut self, buffer: &mut [f32]) -> Status {
        let buffer_size = buffer.len();
        let sample_count = buffer_size / consts::CHANNEL_COUNT;
        let mut intermediate_slice_0 = &mut self.intermediate_buffer_0[0..buffer_size];
        let mut intermediate_slice_1 = &mut self.intermediate_buffer_1[0..buffer_size];
        intermediate_slice_0.fill(0.0);
        intermediate_slice_1.fill(0.0);
        let status_0 = self.consumer_0.fill_buffer(&mut intermediate_slice_0);
        let status_1 = self.consumer_1.fill_buffer(&mut intermediate_slice_1);
        let multiplier_0 = 1.0 - self.balance;
        for i in 0..sample_count {
            let mut index = i * 2;
            buffer[index] += multiplier_0 * intermediate_slice_0[index]
                + self.balance * intermediate_slice_1[index];
            index += 1;
            buffer[index] += multiplier_0 * intermediate_slice_0[index]
                + self.balance * intermediate_slice_1[index];
        }

        match (status_0, status_1) {
            (Status::Ended, Status::Ended) => Status::Ended,
            _ => Status::Ok,
        }
    }
}
