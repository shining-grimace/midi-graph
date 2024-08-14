use crate::{BufferConsumer, NoteConsumer, NoteEvent};

pub struct SoundFont {
    consumer: Box<dyn BufferConsumer + Send + 'static>,
}

impl SoundFont {
    pub fn new(consumer: Box<dyn BufferConsumer + Send + 'static>) -> Self {
        Self { consumer }
    }
}

impl NoteConsumer for SoundFont {
    fn restart_with_event(&mut self, event: NoteEvent) {
        self.consumer.set_note(event);
    }

    fn fill_buffer(&mut self, buffer: &mut [f32]) {
        self.consumer.fill_buffer(buffer);
    }
}
