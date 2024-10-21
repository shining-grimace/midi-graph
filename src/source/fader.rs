use crate::{consts, BufferConsumer, BufferConsumerNode, ControlEvent, Error, Node, NodeEvent};

pub struct Fader {
    node_id: u64,
    duration_seconds: f32,
    from_volume: f32,
    to_volume: f32,
    progress_seconds: f32,
    consumer: Box<dyn BufferConsumerNode + Send + 'static>,
    intermediate_buffer: Vec<f32>,
}

impl Fader {
    pub fn new(
        node_id: Option<u64>,
        initial_volume: f32,
        consumer: Box<dyn BufferConsumerNode + Send + 'static>,
    ) -> Self {
        Self {
            node_id: node_id.unwrap_or_else(|| <Self as Node>::new_node_id()),
            duration_seconds: 0.0,
            from_volume: initial_volume,
            to_volume: initial_volume,
            progress_seconds: 0.0,
            consumer,
            intermediate_buffer: vec![0.0; consts::BUFFER_SIZE * consts::CHANNEL_COUNT],
        }
    }
}

impl BufferConsumerNode for Fader {}

impl Node for Fader {
    fn get_node_id(&self) -> u64 {
        self.node_id
    }

    fn on_event(&mut self, event: &NodeEvent) {
        match event {
            NodeEvent::Control {
                node_id,
                event: ControlEvent::Fade { from, to, seconds },
            } => {
                if *node_id == self.node_id {
                    self.from_volume = *from;
                    self.to_volume = *to;
                    self.duration_seconds = *seconds;
                    self.progress_seconds = 0.0;
                    return;
                }
            }
            _ => {}
        }
        self.consumer.on_event(event);
    }

    fn fill_buffer(&mut self, buffer: &mut [f32]) {
        self.intermediate_buffer.fill(0.0);
        self.consumer
            .fill_buffer(self.intermediate_buffer.as_mut_slice());

        if self.progress_seconds >= self.duration_seconds {
            for i in 0..buffer.len() {
                buffer[i] += self.intermediate_buffer[i] * self.to_volume;
            }
            return;
        }

        let samples_to_fade = (((self.duration_seconds - self.progress_seconds)
            * consts::PLAYBACK_SAMPLE_RATE as f32) as usize)
            .min(buffer.len() / consts::CHANNEL_COUNT);
        let fade_gradient_per_sample = (self.to_volume - self.from_volume)
            / self.duration_seconds
            / (consts::PLAYBACK_SAMPLE_RATE as f32);
        let base_volume = self.from_volume
            + (self.progress_seconds / self.duration_seconds) * (self.to_volume - self.from_volume);

        for i in 0..samples_to_fade {
            let volume = base_volume + (i as f32) * fade_gradient_per_sample;
            buffer[2 * i] += self.intermediate_buffer[2 * i] * volume;
            buffer[2 * i + 1] += self.intermediate_buffer[2 * i + 1] * volume;
        }

        for i in (2 * samples_to_fade)..buffer.len() {
            buffer[i] += self.intermediate_buffer[i] * self.to_volume;
        }

        self.progress_seconds = (self.progress_seconds
            + ((buffer.len() / consts::CHANNEL_COUNT) as f32)
                / consts::PLAYBACK_SAMPLE_RATE as f32)
            .min(self.duration_seconds);
    }
}

impl BufferConsumer for Fader {
    fn duplicate(&self) -> Result<Box<dyn BufferConsumerNode + Send + 'static>, Error> {
        let consumer = self.consumer.duplicate()?;
        let fader = Self {
            node_id: self.node_id,
            duration_seconds: self.duration_seconds,
            from_volume: self.from_volume,
            to_volume: self.to_volume,
            progress_seconds: self.progress_seconds,
            consumer,
            intermediate_buffer: vec![0.0; consts::BUFFER_SIZE * consts::CHANNEL_COUNT],
        };
        Ok(Box::new(fader))
    }
}
