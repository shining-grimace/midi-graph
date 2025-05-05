use crate::{Error, Event, Message, Node};

pub struct Polyphony {
    node_id: u64,
    consumers: Vec<Box<dyn Node + Send + 'static>>,
    next_on_index: usize,
}

impl Polyphony {
    pub fn new(
        node_id: Option<u64>,
        max_voices: usize,
        consumer: Box<dyn Node + Send + 'static>,
    ) -> Result<Self, Error> {
        if max_voices < 1 {
            return Err(Error::User(format!(
                "Cannot form Polyphony with {} voices",
                max_voices
            )));
        }
        let mut consumers = (0..(max_voices - 1))
            .map(|_| consumer.duplicate())
            .collect::<Result<Vec<Box<dyn Node + Send + 'static>>, Error>>()?;
        consumers.push(consumer);
        Ok(Self {
            node_id: node_id.unwrap_or_else(<Self as Node>::new_node_id),
            consumers,
            next_on_index: 0,
        })
    }
}

impl Node for Polyphony {
    fn get_node_id(&self) -> u64 {
        self.node_id
    }

    fn set_node_id(&mut self, node_id: u64) {
        self.node_id = node_id;
    }

    fn duplicate(&self) -> Result<Box<dyn Node + Send + 'static>, Error> {
        let consumers = self
            .consumers
            .iter()
            .map(|consumer| consumer.duplicate())
            .collect::<Result<Vec<Box<dyn Node + Send + 'static>>, Error>>()?;
        let polyphony = Self {
            node_id: self.node_id,
            consumers,
            next_on_index: 0,
        };
        Ok(Box::new(polyphony))
    }

    fn on_event(&mut self, event: &Message) {
        let was_consumed = if event.target.influences(self.node_id) {
            match event.data {
                Event::NoteOn { .. } => {
                    self.consumers[self.next_on_index].on_event(event);
                    self.next_on_index = (self.next_on_index + 1) % self.consumers.len();
                    true
                }
                Event::NoteOff { .. } => {
                    for consumer in self.consumers.iter_mut() {
                        consumer.on_event(event);
                    }
                    true
                }
                _ => false
            }
        } else {
            true
        };
        if event.target.propagates_from(self.node_id, was_consumed) {
            for consumer in self.consumers.iter_mut() {
                consumer.on_event(event);
            }
        }
    }

    fn fill_buffer(&mut self, buffer: &mut [f32]) {
        for consumer in self.consumers.iter_mut() {
            consumer.fill_buffer(buffer);
        }
    }

    fn replace_children(
        &mut self,
        children: &[Box<dyn Node + Send + 'static>],
    ) -> Result<(), Error> {
        if children.len() != 1 {
            return Err(Error::User(
                "Polyphony requires one child which will be duplicated as needed".to_owned(),
            ));
        }

        self.consumers = (0..(self.consumers.len()))
            .map(|_| children[0].duplicate())
            .collect::<Result<Vec<Box<dyn Node + Send + 'static>>, Error>>()?;
        self.next_on_index = 0;
        Ok(())
    }
}
