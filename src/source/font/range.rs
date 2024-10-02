use crate::{BufferConsumerNode, NoteEvent, NoteKind, NoteRange, Status};
use std::cell::RefCell;

pub struct RangeData {
    pub range: NoteRange,
    pub active_count: usize,
    pub consumers: Vec<(u8, Box<dyn BufferConsumerNode + Send + 'static>)>,
    ended_notes: RefCell<Vec<u8>>,
}

impl RangeData {
    pub fn new(
        range: NoteRange,
        consumers: Vec<(u8, Box<dyn BufferConsumerNode + Send + 'static>)>,
    ) -> Self {
        Self {
            range,
            active_count: 0,
            consumers,
            ended_notes: RefCell::new(vec![]),
        }
    }

    pub fn turn_note_on(&mut self, note: u8, vel: f32) {
        if !self.range.contains(note) {
            return;
        }
        let existing_index = self.consumers[0..self.active_count]
            .iter()
            .position(|(n, _)| *n == note);
        match existing_index {
            Some(index) => {
                let event = NoteEvent {
                    kind: NoteKind::NoteOn { note, vel },
                };
                self.consumers[index].1.on_event(&event);
            }
            None => {
                if self.active_count >= self.consumers.len() {
                    #[cfg(debug_assertions)]
                    println!("WARNING: Soundfont: Note turning on, but all consumers in use");
                    return;
                }
                let event = NoteEvent {
                    kind: NoteKind::NoteOn { note, vel },
                };
                self.consumers[self.active_count].0 = note;
                self.consumers[self.active_count].1.on_event(&event);
                self.active_count += 1;
            }
        }
    }

    pub fn turn_note_off(&mut self, note: u8, vel: f32) {
        if !self.range.contains(note) {
            return;
        }
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
        let event = NoteEvent {
            kind: NoteKind::NoteOff { note, vel },
        };
        self.consumers[source_index].1.on_event(&event);
    }

    fn remove_note(&mut self, note: u8) {
        let maybe_index = self.consumers[0..self.active_count]
            .iter()
            .position(|(n, _)| *n == note);
        let source_index = match maybe_index {
            Some(index) => index,
            None => {
                #[cfg(debug_assertions)]
                println!("WARNING: Soundfont: Removing a note that was not on");
                return;
            }
        };
        if source_index >= self.active_count {
            #[cfg(debug_assertions)]
            println!("WARNING: Soundfont: Removing a note, but source not in use");
        }
        let removed_source = self.consumers.remove(source_index);
        self.consumers.push(removed_source);
        self.active_count -= 1;
    }

    pub fn fill_buffer(&mut self, buffer: &mut [f32]) {
        {
            let mut ended_notes = self.ended_notes.borrow_mut();
            for (note, consumer) in self.consumers.iter_mut().take(self.active_count) {
                let status = consumer.fill_buffer(buffer);
                if status == Status::Ended {
                    ended_notes.push(*note);
                }
            }
        }
        let ended_note_count = self.ended_notes.borrow().len();
        for i in 0..ended_note_count {
            let note = {
                let n = self.ended_notes.borrow()[i];
                n
            };
            self.remove_note(note);
        }
        self.ended_notes.borrow_mut().clear();
    }
}
