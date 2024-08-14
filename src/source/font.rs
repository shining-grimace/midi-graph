use crate::{BufferConsumer, NoteConsumer, NoteEvent};

const SOURCE_CAPACITY: usize = 8;

pub struct SoundFont {
    active_count: usize,
    consumers: Vec<(u8, Box<dyn BufferConsumer + Send + 'static>)>,
}

impl SoundFont {
    pub fn new(consumer_spawner: fn() -> Box<dyn BufferConsumer + Send + 'static>) -> Self {
        let mut consumers = Vec::new();
        for _ in 0..SOURCE_CAPACITY {
            consumers.push((0, consumer_spawner()));
        }
        Self {
            active_count: 0,
            consumers,
        }
    }
}

impl NoteConsumer for SoundFont {
    fn restart_with_event(&mut self, event: &NoteEvent) {
        match event {
            NoteEvent::NoteOn(note) => {
                let same_notes = self.consumers[0..self.active_count]
                    .iter()
                    .filter(|(n, _)| *n == *note)
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
                self.consumers[self.active_count].0 = *note;
                self.consumers[self.active_count]
                    .1
                    .set_note(NoteEvent::NoteOn(*note));
                self.active_count += 1;
            }
            NoteEvent::NoteOff(note) => {
                let maybe_index = self.consumers[0..self.active_count]
                    .iter()
                    .position(|(n, _)| *n == *note);
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
                removed_source.1.set_note(NoteEvent::NoteOff(*note));
                self.consumers.push(removed_source);
                self.active_count -= 1;
            }
        }
    }

    fn fill_buffer(&mut self, buffer: &mut [f32]) {
        for i in 0..self.active_count {
            self.consumers[i].1.fill_buffer(buffer);
        }
    }
}
