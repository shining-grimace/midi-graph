use crate::{BufferConsumer, NoteEvent, NoteRange};

pub struct RangeData {
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
                println!("WARNING: Soundfont: Note turning on, but was already on");
                return;
            }
            if self.active_count >= self.consumers.len() {
                #[cfg(debug_assertions)]
                println!("WARNING: Soundfont: Note turning on, but all consumers in use");
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
                    println!("WARNING: Soundfont: Note turning off, but was not on");
                    return;
                }
            };
            if source_index >= self.active_count {
                #[cfg(debug_assertions)]
                println!("WARNING: Soundfont: Note turning off, but source not in use");
            }
            let mut removed_source = self.consumers.remove(source_index);
            removed_source.1.set_note(NoteEvent::NoteOff(note));
            self.consumers.push(removed_source);
            self.active_count -= 1;
        }
    }
}
