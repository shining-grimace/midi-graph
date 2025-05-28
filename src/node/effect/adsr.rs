use crate::{Error, Event, GraphNode, Message, Node, consts};

const PEAK_AMPLITUDE: f32 = 1.0;

enum EnvelopeMode {
    Attack,
    Decay,
    Sustain,
    Release,
    Finished,
}

pub struct AdsrEnvelope {
    node_id: u64,
    attack_gradient: f32,
    decay_gradient: f32,
    sustain_multiplier: f32,
    release_gradient: f32,
    consumer: GraphNode,
    intermediate_buffer: Vec<f32>,
    mode: EnvelopeMode,
    samples_progress_in_mode: isize,
}

impl AdsrEnvelope {
    pub fn from_parameters(
        node_id: Option<u64>,
        attack_time: f32,
        decay_time: f32,
        sustain_multiplier: f32,
        release_time: f32,
        consumer: GraphNode,
    ) -> Self {
        let attack_gradient = PEAK_AMPLITUDE / (attack_time * consts::PLAYBACK_SAMPLE_RATE as f32);
        let decay_gradient = (sustain_multiplier - PEAK_AMPLITUDE)
            / (decay_time * consts::PLAYBACK_SAMPLE_RATE as f32);
        let release_gradient =
            (0.0 - sustain_multiplier) / (release_time * consts::PLAYBACK_SAMPLE_RATE as f32);
        Self {
            node_id: node_id.unwrap_or_else(<Self as Node>::new_node_id),
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

    fn release(&mut self) {
        self.samples_progress_in_mode = match self.mode {
            EnvelopeMode::Attack => {
                let current_multiplier =
                    self.samples_progress_in_mode as f32 * self.attack_gradient;
                ((current_multiplier - self.sustain_multiplier) / self.release_gradient) as isize
            }
            EnvelopeMode::Decay => {
                let current_multiplier =
                    PEAK_AMPLITUDE + self.samples_progress_in_mode as f32 * self.decay_gradient;
                ((current_multiplier - self.sustain_multiplier) / self.release_gradient) as isize
            }
            EnvelopeMode::Sustain => 0,
            EnvelopeMode::Release => self.samples_progress_in_mode,
            EnvelopeMode::Finished => {
                (self.release_gradient * self.sustain_multiplier * PEAK_AMPLITUDE) as isize
            }
        };
        self.mode = EnvelopeMode::Release;
    }
}

impl Node for AdsrEnvelope {
    fn get_node_id(&self) -> u64 {
        self.node_id
    }

    fn set_node_id(&mut self, node_id: u64) {
        self.node_id = node_id;
    }

    fn duplicate(&self) -> Result<GraphNode, Error> {
        let consumer = self.consumer.duplicate()?;
        let envelope = Self {
            node_id: self.node_id,
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

    fn try_consume_event(&mut self, event: &Message) -> bool {
        match event.data {
            Event::NoteOn { .. } => {
                self.mode = EnvelopeMode::Attack;
                self.samples_progress_in_mode = 0;
            }
            Event::NoteOff { .. } => {
                self.release();
            }
            _ => {}
        }
        // AdsrEnvelope does not consume any events, but listens to notes
        false
    }

    fn propagate(&mut self, event: &Message) {
        self.consumer.on_event(event);
    }

    fn fill_buffer(&mut self, buffer: &mut [f32]) {
        let buffer_size = buffer.len();
        let samples_in_buffer = buffer_size / consts::CHANNEL_COUNT;

        let intermediate_slice = &mut self.intermediate_buffer[0..buffer_size];
        intermediate_slice.fill(0.0);
        self.consumer.fill_buffer(intermediate_slice);

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
    }

    fn replace_children(&mut self, children: &[GraphNode]) -> Result<(), Error> {
        if children.len() != 1 {
            return Err(Error::User("AdsrEnvelope requires one child".to_owned()));
        }
        self.consumer = children[0].duplicate()?;
        Ok(())
    }
}
