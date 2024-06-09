mod track;
mod util;

use crate::{AudioSource, Error};
use midly::{MidiMessage, Smf, TrackEvent, TrackEventKind};
use track::MidiTrackSource;

#[cfg(debug_assertions)]
use crate::source::log;

const PLAYBACK_TRACK: usize = 1;

pub struct MidiSource<'a> {
    smf: Smf<'a>,
    source: Box<MidiTrackSource>,
    has_finished: bool,
    next_event_index: usize,
    samples_per_tick: f64,
    event_ticks_progress: isize,
}

impl<'a> MidiSource<'a> {
    pub fn new(
        smf: Smf<'a>,
        source_spawner: fn() -> Box<dyn AudioSource + Send + 'static>,
    ) -> Result<Self, Error> {
        #[cfg(debug_assertions)]
        log::log_loaded_midi(&smf, PLAYBACK_TRACK);

        let samples_per_tick = util::get_samples_per_tick(&smf)?;
        let source = MidiTrackSource::new(source_spawner);

        Ok(Self {
            smf,
            source: Box::new(source),
            has_finished: false,
            next_event_index: 0,
            samples_per_tick,
            event_ticks_progress: 0,
        })
    }

    // Update self when a new event was reached
    fn update_on_event(&mut self, event: TrackEvent) {
        if let TrackEventKind::Midi {
            channel: _,
            message: MidiMessage::NoteOn { key, vel: _ },
        } = event.kind
        {
            let note = u8::from(key);
            self.source.on_note_on(note);
        }
        if let TrackEventKind::Midi {
            channel: _,
            message: MidiMessage::NoteOff { key, vel: _ },
        } = event.kind
        {
            let note = u8::from(key);
            self.source.on_note_off(note);
        }
    }
}

impl<'a> AudioSource for MidiSource<'a> {
    fn on_note_on(&mut self, key: u8) {
        self.has_finished = false;
        self.next_event_index = 0;
        self.event_ticks_progress = 0;
    }

    fn on_note_off(&mut self, key: u8) {
        self.has_finished = true;
    }

    fn fill_buffer(&mut self, key: u8, buffer: &mut [f32]) {
        if self.has_finished {
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
            self.source
                .fill_buffer(0, &mut buffer[0..samples_to_play_now]);
        }
        if self.event_ticks_progress >= event_ticks_delta {
            self.update_on_event(*next_event);
            self.next_event_index += 1;
            self.event_ticks_progress = 0;
            let remaining_buffer = &mut buffer[samples_to_play_now..];
            if self.next_event_index >= self.smf.tracks[PLAYBACK_TRACK].len() {
                self.has_finished = true;
                return;
            }
            self.fill_buffer(key, remaining_buffer);
        }
    }
}
