mod range;

use crate::{Error, Node, NodeEvent, NoteRange};
use range::RangeData;

const SOURCE_CAPACITY: usize = 8;

pub struct SoundFontBuilder {
    node_id: Option<u64>,
    ranges: Vec<RangeData>,
}

impl Default for SoundFontBuilder {
    fn default() -> Self {
        Self::new(None)
    }
}

impl SoundFontBuilder {
    pub fn new(node_id: Option<u64>) -> Self {
        Self {
            node_id,
            ranges: vec![],
        }
    }

    pub fn add_range(
        mut self,
        range: NoteRange,
        consumer: Box<dyn Node + Send + 'static>,
    ) -> Result<Self, Error> {
        let mut consumers = Vec::new();
        for _ in 0..SOURCE_CAPACITY {
            consumers.push(consumer.duplicate()?);
        }
        self.ranges.push(RangeData::new(range, consumers));
        Ok(self)
    }

    pub fn build(self) -> SoundFont {
        SoundFont::new(self.node_id, self.ranges)
    }
}

pub struct SoundFont {
    node_id: u64,
    ranges: Vec<RangeData>,
}

impl SoundFont {
    fn new(node_id: Option<u64>, ranges: Vec<RangeData>) -> Self {
        Self {
            node_id: node_id.unwrap_or_else(<Self as Node>::new_node_id),
            ranges,
        }
    }
}

impl Node for SoundFont {
    fn get_node_id(&self) -> u64 {
        self.node_id
    }

    fn duplicate(&self) -> Result<Box<dyn Node + Send + 'static>, Error> {
        Err(Error::User("SoundFont cannot be duplicated".to_owned()))
    }

    fn on_event(&mut self, event: &NodeEvent) {
        for range_data in self.ranges.iter_mut() {
            range_data.on_event(event);
        }
    }

    fn fill_buffer(&mut self, buffer: &mut [f32]) {
        for range_data in self.ranges.iter_mut() {
            range_data.fill_buffer(buffer);
        }
    }

    fn replace_children(
        &mut self,
        _children: &[Box<dyn Node + Send + 'static>],
    ) -> Result<(), Error> {
        Err(Error::User("SoundFont does not support replacing its children".to_owned()))
    }
}
