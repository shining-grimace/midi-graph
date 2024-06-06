mod util;

use crate::{AudioSource, Error};
use midly::{MidiMessage, Smf, TrackEventKind};

#[cfg(debug_assertions)]
use crate::source::log;

const PLAYBACK_TRACK: usize = 1;

pub struct MidiSource<'a> {
    smf: Smf<'a>,
    source: Box<dyn AudioSource + Send + 'static>,
    has_finished: bool,
    next_event_index: usize,
    samples_per_tick: f64,
    event_ticks_progress: isize,
    current_relative_pitch: f32,
}

impl<'a> MidiSource<'a> {
    pub fn new(smf: Smf<'a>, source: Box<dyn AudioSource + Send + 'static>) -> Result<Self, Error> {
        #[cfg(debug_assertions)]
        log::log_loaded_midi(&smf, PLAYBACK_TRACK);

        let samples_per_tick = util::get_samples_per_tick(&smf)?;

        Ok(Self {
            smf,
            source,
            has_finished: false,
            next_event_index: 0,
            samples_per_tick,
            event_ticks_progress: 0,
            current_relative_pitch: 0.0,
        })
    }
}

impl<'a> AudioSource for MidiSource<'a> {
    fn is_completed(&self) -> bool {
        self.has_finished
    }

    fn rewind(&mut self) {
        self.has_finished = false;
        self.next_event_index = 0;
        self.source.rewind();
    }

    fn fill_buffer(&mut self, relative_pitch: f32, buffer: &mut [f32]) {
        if self.has_finished {
            buffer.fill(0.0);
            return;
        }
        let next_event = &self.smf.tracks[PLAYBACK_TRACK][self.next_event_index];
        let event_ticks_delta = u32::from(next_event.delta) as isize;
        let ticks_until_event = event_ticks_delta - self.event_ticks_progress;
        let samples_until_event = (ticks_until_event as f64 * self.samples_per_tick) as usize;
        let samples_to_play_now = samples_until_event.min(buffer.len());
        if ticks_until_event > 0 {
            let ticks_available = ((buffer.len() as f64) / self.samples_per_tick) as isize;
            self.event_ticks_progress += ticks_until_event.min(ticks_available);
            self.source.fill_buffer(
                self.current_relative_pitch,
                &mut buffer[0..samples_to_play_now],
            );
        }
        if self.event_ticks_progress >= event_ticks_delta {
            self.next_event_index += 1;
            self.event_ticks_progress = 0;
            let remaining_buffer = &mut buffer[samples_to_play_now..];
            if self.next_event_index >= self.smf.tracks[PLAYBACK_TRACK].len() {
                self.has_finished = true;
                remaining_buffer.fill(0.0);
                return;
            }
            let new_event = &self.smf.tracks[PLAYBACK_TRACK][self.next_event_index];
            if let TrackEventKind::Midi {
                channel: _,
                message: MidiMessage::NoteOn { key, vel: _ },
            } = new_event.kind
            {
                let note_relative_to_a440 = u8::from(key) as f32 - 69.0;
                self.source.rewind();
                self.current_relative_pitch = note_relative_to_a440;
            }
            self.fill_buffer(self.current_relative_pitch, remaining_buffer);
        }
    }
}
