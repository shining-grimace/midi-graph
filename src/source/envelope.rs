use crate::{consts, BufferConsumer, Error, NoteEvent, Status};

const PEAK_AMPLITUDE: f32 = 1.0;

pub struct Envelope {
    attack_gradient: f32,
    decay_gradient: f32,
    sustain_multiplier: f32,
    release_gradient: f32,
    source: Box<dyn BufferConsumer + Send + 'static>,
    intermediate_buffer: Vec<f32>,
}

impl Envelope {
    pub fn from_adsr(
        attack_time: f32,
        decay_time: f32,
        sustain_multiplier: f32,
        release_time: f32,
        source: Box<dyn BufferConsumer + Send + 'static>,
    ) -> Self {
        let attack_gradient = PEAK_AMPLITUDE / (attack_time * consts::PLAYBACK_SAMPLE_RATE as f32);
        let decay_gradient = (sustain_multiplier - PEAK_AMPLITUDE)
            / (decay_time * consts::PLAYBACK_SAMPLE_RATE as f32);
        let release_gradient =
            (0.0 - sustain_multiplier) / (release_time * consts::PLAYBACK_SAMPLE_RATE as f32);
        Self {
            attack_gradient,
            decay_gradient,
            sustain_multiplier,
            release_gradient,
            source,
            intermediate_buffer: vec![0.0; consts::BUFFER_SIZE],
        }
    }
}

impl BufferConsumer for Envelope {
    fn duplicate(&self) -> Result<Box<dyn BufferConsumer + Send + 'static>, Error> {
        let source = self.source.duplicate()?;
        let envelope = Self {
            attack_gradient: self.attack_gradient,
            decay_gradient: self.decay_gradient,
            sustain_multiplier: self.sustain_multiplier,
            release_gradient: self.release_gradient,
            source,
            intermediate_buffer: vec![0.0; consts::BUFFER_SIZE],
        };
        Ok(Box::new(envelope))
    }

    fn set_note(&mut self, event: NoteEvent) {
        self.source.set_note(event);
    }

    fn fill_buffer(&mut self, buffer: &mut [f32]) -> Status {
        let buffer_size = buffer.len();
        let mut intermediate_slice = &mut self.intermediate_buffer[0..buffer_size];
        intermediate_slice.fill(0.0);
        self.source.fill_buffer(&mut intermediate_slice);
        for (i, sample) in intermediate_slice.iter().enumerate() {
            buffer[i] += sample;
        }
        Status::Ok
    }
}
