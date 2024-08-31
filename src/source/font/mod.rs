mod range;

use crate::{BufferConsumer, NoteConsumer, NoteEvent, NoteKind, NoteRange};
use range::RangeData;

const SOURCE_CAPACITY: usize = 8;

pub struct SoundFontBuilder {
    ranges: Vec<RangeData>,
}

impl SoundFontBuilder {
    pub fn new() -> Self {
        Self { ranges: vec![] }
    }

    pub fn add_range(
        mut self,
        range: NoteRange,
        consumer_spawner: impl Fn() -> Box<dyn BufferConsumer + Send + 'static>,
    ) -> Self {
        let mut consumers = Vec::new();
        for _ in 0..SOURCE_CAPACITY {
            consumers.push((0, consumer_spawner()));
        }
        self.ranges.push(RangeData {
            range,
            active_count: 0,
            consumers,
        });
        self
    }

    pub fn build(self) -> SoundFont {
        SoundFont::new(self.ranges)
    }
}

pub struct SoundFont {
    ranges: Vec<RangeData>,
}

impl SoundFont {
    fn new(ranges: Vec<RangeData>) -> Self {
        Self { ranges }
    }
}

impl NoteConsumer for SoundFont {
    fn restart_with_event(&mut self, event: &NoteEvent) {
        let note: u8 = match event.kind {
            NoteKind::NoteOn(note) => note,
            NoteKind::NoteOff(note) => note,
        };
        let turning_on = match event.kind {
            NoteKind::NoteOn(_) => true,
            NoteKind::NoteOff(_) => false,
        };
        if turning_on {
            for range_data in self.ranges.iter_mut() {
                range_data.turn_note_on(note);
            }
        } else {
            for range_data in self.ranges.iter_mut() {
                range_data.turn_note_off(note);
            }
        }
    }

    fn fill_buffer(&mut self, buffer: &mut [f32]) {
        for range_data in self.ranges.iter_mut() {
            for i in 0..range_data.active_count {
                range_data.consumers[i].1.fill_buffer(buffer);
            }
        }
    }
}
