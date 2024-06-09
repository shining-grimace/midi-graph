use crate::AudioSource;

const SOURCE_CAPACITY: usize = 8;

pub struct MidiTrackSource {
    active_count: usize,
    sources: Vec<(u8, Box<dyn AudioSource + Send + 'static>)>,
}

impl MidiTrackSource {
    pub fn new(source_spawner: fn() -> Box<dyn AudioSource + Send + 'static>) -> Self {
        let mut sources = Vec::new();
        for _ in 0..SOURCE_CAPACITY {
            sources.push((0, source_spawner()));
        }
        Self {
            active_count: 0,
            sources,
        }
    }
}

impl AudioSource for MidiTrackSource {
    fn on_note_on(&mut self, key: u8) {
        let same_notes = self.sources[0..self.active_count]
            .iter()
            .filter(|(note, _)| *note == key)
            .count();
        if same_notes > 0 {
            #[cfg(debug_assertions)]
            println!("Note turning on, but was already on");
            return;
        }
        if self.active_count >= self.sources.len() {
            #[cfg(debug_assertions)]
            println!("Note turning on, but all sources in use");
            return;
        }
        self.sources[self.active_count].0 = key;
        self.sources[self.active_count].1.on_note_on(key);
        self.active_count += 1;
    }

    fn on_note_off(&mut self, key: u8) {
        let maybe_index = self.sources[0..self.active_count]
            .iter()
            .position(|(note, _)| *note == key);
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
        let mut removed_source = self.sources.remove(source_index);
        removed_source.1.on_note_off(key);
        self.sources.push(removed_source);
        self.active_count -= 1;
    }

    fn fill_buffer(&mut self, key: u8, buffer: &mut [f32]) {
        for i in 0..self.active_count {
            let note = self.sources[i].0;
            self.sources[i].1.fill_buffer(note, buffer);
        }
    }
}
