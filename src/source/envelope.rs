use crate::{consts, BufferConsumer, Error, NoteEvent, NoteKind, Status};

const PEAK_AMPLITUDE: f32 = 1.0;

enum EnvelopeMode {
    Attack,
    Decay,
    Sustain,
    Release,
    Finished,
}

pub struct Envelope {
    attack_gradient: f32,
    decay_gradient: f32,
    sustain_multiplier: f32,
    release_gradient: f32,
    consumer: Box<dyn BufferConsumer + Send + 'static>,
    intermediate_buffer: Vec<f32>,
    mode: EnvelopeMode,
    samples_progress_in_mode: isize,
}

impl Envelope {
    pub fn from_adsr(
        attack_time: f32,
        decay_time: f32,
        sustain_multiplier: f32,
        release_time: f32,
        consumer: Box<dyn BufferConsumer + Send + 'static>,
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
            consumer,
            intermediate_buffer: vec![0.0; consts::BUFFER_SIZE * consts::CHANNEL_COUNT],
            mode: EnvelopeMode::Attack,
            samples_progress_in_mode: 0,
        }
    }
}

impl BufferConsumer for Envelope {
    fn duplicate(&self) -> Result<Box<dyn BufferConsumer + Send + 'static>, Error> {
        let consumer = self.consumer.duplicate()?;
        let envelope = Self {
            attack_gradient: self.attack_gradient,
            decay_gradient: self.decay_gradient,
            sustain_multiplier: self.sustain_multiplier,
            release_gradient: self.release_gradient,
            consumer,
            intermediate_buffer: vec![0.0; consts::BUFFER_SIZE * consts::CHANNEL_COUNT],
            mode: EnvelopeMode::Attack,
            samples_progress_in_mode: 0,
        };
        Ok(Box::new(envelope))
    }

    fn set_note(&mut self, event: NoteEvent) {
        match event.kind {
            NoteKind::NoteOn { .. } => {
                self.mode = EnvelopeMode::Attack;
                self.samples_progress_in_mode = 0;
            }
            NoteKind::NoteOff { .. } => {
                self.samples_progress_in_mode = match self.mode {
                    EnvelopeMode::Attack => {
                        let current_multiplier =
                            self.samples_progress_in_mode as f32 * self.attack_gradient;
                        ((current_multiplier - self.sustain_multiplier) / self.release_gradient)
                            as isize
                    }
                    EnvelopeMode::Decay => {
                        let current_multiplier = PEAK_AMPLITUDE
                            + self.samples_progress_in_mode as f32 * self.decay_gradient;
                        ((current_multiplier - self.sustain_multiplier) / self.release_gradient)
                            as isize
                    }
                    EnvelopeMode::Sustain => 0,
                    EnvelopeMode::Release => self.samples_progress_in_mode,
                    EnvelopeMode::Finished => {
                        (self.release_gradient * self.sustain_multiplier * PEAK_AMPLITUDE) as isize
                    }
                };
                self.mode = EnvelopeMode::Release;
            }
        };
        self.consumer.set_note(event);
    }

    fn fill_buffer(&mut self, buffer: &mut [f32]) -> Status {
        let buffer_size = buffer.len();
        let samples_in_buffer = buffer_size / consts::CHANNEL_COUNT;

        let mut intermediate_slice = &mut self.intermediate_buffer[0..buffer_size];
        intermediate_slice.fill(0.0);
        self.consumer.fill_buffer(&mut intermediate_slice);

        let mut samples_available = samples_in_buffer;
        while samples_available > 0 {
            let samples_left_in_mode: usize = match self.mode {
                EnvelopeMode::Attack => ((PEAK_AMPLITUDE / self.attack_gradient) as isize
                    - self.samples_progress_in_mode)
                    .max(0) as usize,
                EnvelopeMode::Decay => ((PEAK_AMPLITUDE * (self.sustain_multiplier - 1.0)
                    / self.decay_gradient) as isize
                    - self.samples_progress_in_mode)
                    .max(0) as usize,
                EnvelopeMode::Sustain => usize::MAX,
                EnvelopeMode::Release => ((PEAK_AMPLITUDE * self.sustain_multiplier
                    / self.release_gradient) as isize
                    - self.samples_progress_in_mode)
                    .max(0) as usize,
                EnvelopeMode::Finished => usize::MAX,
            };
            let samples_to_fill = samples_left_in_mode.min(samples_available);
            let buffer_index = consts::CHANNEL_COUNT * (samples_in_buffer - samples_available);
            let buffer_slice = &mut buffer[buffer_index..];
            let intermediate_slice = &self.intermediate_buffer[buffer_index..];
            match self.mode {
                EnvelopeMode::Attack => {
                    for i in 0..samples_to_fill {
                        let multiplier = (self.samples_progress_in_mode + i as isize) as f32
                            * self.attack_gradient;
                        buffer_slice[2 * i] += multiplier * intermediate_slice[2 * i];
                        buffer_slice[2 * i + 1] += multiplier * intermediate_slice[2 * i + 1];
                    }
                    if samples_to_fill == samples_left_in_mode {
                        self.mode = EnvelopeMode::Decay;
                        self.samples_progress_in_mode = 0;
                    } else {
                        self.samples_progress_in_mode += samples_to_fill as isize;
                    }
                }
                EnvelopeMode::Decay => {
                    for i in 0..samples_to_fill {
                        let multiplier = PEAK_AMPLITUDE
                            + (self.samples_progress_in_mode + i as isize) as f32
                                * self.decay_gradient;
                        buffer_slice[2 * i] += multiplier * intermediate_slice[2 * i];
                        buffer_slice[2 * i + 1] += multiplier * intermediate_slice[2 * i + 1];
                    }
                    if samples_to_fill == samples_left_in_mode {
                        self.mode = EnvelopeMode::Sustain;
                        self.samples_progress_in_mode = 0;
                    } else {
                        self.samples_progress_in_mode += samples_to_fill as isize;
                    }
                }
                EnvelopeMode::Sustain => {
                    for i in 0..samples_to_fill {
                        let multiplier = self.sustain_multiplier;
                        buffer_slice[2 * i] += multiplier * intermediate_slice[2 * i];
                        buffer_slice[2 * i + 1] += multiplier * intermediate_slice[2 * i + 1];
                    }
                    self.samples_progress_in_mode += samples_to_fill as isize;
                }
                EnvelopeMode::Release => {
                    for i in 0..samples_to_fill {
                        let multiplier = self.sustain_multiplier
                            + (self.samples_progress_in_mode + i as isize) as f32
                                * self.release_gradient;
                        buffer_slice[2 * i] += multiplier * intermediate_slice[2 * i];
                        buffer_slice[2 * i + 1] += multiplier * intermediate_slice[2 * i + 1];
                    }
                    if samples_to_fill == samples_left_in_mode {
                        self.mode = EnvelopeMode::Finished;
                        self.samples_progress_in_mode = 0;
                    } else {
                        self.samples_progress_in_mode += samples_to_fill as isize;
                    }
                }
                EnvelopeMode::Finished => {}
            };
            samples_available -= samples_to_fill;
        }

        match self.mode {
            EnvelopeMode::Finished => Status::Ended,
            _ => Status::Ok,
        }
    }
}
