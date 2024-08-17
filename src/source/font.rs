use crate::{BufferConsumer, NoteConsumer, NoteEvent, NoteRange};

const SOURCE_CAPACITY: usize = 8;

struct RangeData {
    pub range: NoteRange,
    pub active_count: usize,
    pub consumers: Vec<(u8, Box<dyn BufferConsumer + Send + 'static>)>,
}

impl RangeData {
    pub fn turn_note_on(&mut self, note: u8) {
        if self.range.contains(note) {
            let same_notes = self.consumers[0..self.active_count]
                .iter()
                .filter(|(n, _)| *n == note)
                .count();
            if same_notes > 0 {
                #[cfg(debug_assertions)]
                println!("Note turning on, but was already on");
                return;
            }
            if self.active_count >= self.consumers.len() {
                #[cfg(debug_assertions)]
                println!("Note turning on, but all consumers in use");
                return;
            }
            self.consumers[self.active_count].0 = note;
            self.consumers[self.active_count]
                .1
                .set_note(NoteEvent::NoteOn(note));
            self.active_count += 1;
        }
    }

    pub fn turn_note_off(&mut self, note: u8) {
        if self.range.contains(note) {
            let maybe_index = self.consumers[0..self.active_count]
                .iter()
                .position(|(n, _)| *n == note);
            let source_index = match maybe_index {
                Some(index) => index,
                None => {
                    #[cfg(debug_assertions)]
                    println!("Note turning off, but was not on");
                    return;
                }
            };
            if source_index >= self.active_count {
                #[cfg(debug_assertions)]
                println!("Note turning off, but source not in use");
            }
            let mut removed_source = self.consumers.remove(source_index);
            removed_source.1.set_note(NoteEvent::NoteOff(note));
            self.consumers.push(removed_source);
            self.active_count -= 1;
        }
    }
}

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
        consumer_spawner: fn() -> Box<dyn BufferConsumer + Send + 'static>,
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
        let note: u8 = match event {
            NoteEvent::NoteOn(note) => *note,
            NoteEvent::NoteOff(note) => *note,
        };
        let turning_on = match event {
            NoteEvent::NoteOn(_) => true,
            NoteEvent::NoteOff(_) => false,
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
