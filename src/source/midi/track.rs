use crate::{consts, NoteConsumer, NoteEvent, NoteKind};
use midly::{MidiMessage, Smf, TrackEvent, TrackEventKind};
use std::sync::Arc;

struct NoteEventOnChannel {
    channel: usize,
    event: NoteEvent,
}

pub struct MidiTrackSource<'a> {
    smf: Arc<Smf<'a>>,
    track_no: usize,
    channel_no: usize,
    samples_per_tick: f64,
    has_finished: bool,
    next_event_index: usize,
    event_ticks_progress: isize,
    source: Box<dyn NoteConsumer + Send + 'static>,
}

impl<'a> MidiTrackSource<'a> {
    pub fn new(
        smf: Arc<Smf<'a>>,
        track_no: usize,
        channel_no: usize,
        samples_per_tick: f64,
        note_consumer: Box<dyn NoteConsumer + Send + 'static>,
    ) -> Self {
        Self {
            smf,
            track_no,
            channel_no,
            samples_per_tick,
            has_finished: false,
            next_event_index: 0,
            event_ticks_progress: 0,
            source: note_consumer,
        }
    }

    fn on_event_reached(&mut self, event: &Option<NoteEventOnChannel>) {
        match event {
            None => {}
            Some(e) => {
                if e.channel != self.channel_no {
                    return;
                }
                self.source.restart_with_event(&e.event);
            }
        }
    }

    fn note_event_from_midi_event(event: &TrackEvent) -> Option<NoteEventOnChannel> {
        match event.kind {
            TrackEventKind::Midi {
                channel,
                message: MidiMessage::NoteOn { key, vel: _ },
            } => Some(NoteEventOnChannel {
                channel: u8::from(channel) as usize,
                event: NoteEvent {
                    kind: NoteKind::NoteOn(u8::from(key)),
                },
            }),
            TrackEventKind::Midi {
                channel,
                message: MidiMessage::NoteOff { key, vel: _ },
            } => Some(NoteEventOnChannel {
                channel: u8::from(channel) as usize,
                event: NoteEvent {
                    kind: NoteKind::NoteOff(u8::from(key)),
                },
            }),
            _ => None,
        }
    }

    fn write_buffer(&mut self, buffer: &mut [f32]) {
        self.source.fill_buffer(buffer);
    }

    pub fn fill_buffer(&mut self, buffer: &mut [f32]) {
        #[cfg(debug_assertions)]
        assert_eq!(buffer.len() % consts::CHANNEL_COUNT, 0);

        // Currently only-supported channel configuration
        #[cfg(debug_assertions)]
        assert_eq!(consts::CHANNEL_COUNT, 2);

        let smf = Arc::clone(&self.smf);
        let track_data = &smf.tracks[self.track_no];
        if self.has_finished {
            return;
        }

        loop {
            let next_event = &track_data[self.next_event_index];
            let event_ticks_delta = u32::from(next_event.delta) as isize;
            let ticks_until_event = event_ticks_delta - self.event_ticks_progress;
            let samples_until_event = (ticks_until_event as f64 * self.samples_per_tick) as usize;
            let samples_available_per_channel = buffer.len() / consts::CHANNEL_COUNT;
            if samples_until_event > samples_available_per_channel {
                self.write_buffer(buffer);
                self.event_ticks_progress +=
                    (samples_available_per_channel as f64 / self.samples_per_tick) as isize;
                return;
            }

            let buffer_samples_to_fill = samples_until_event * consts::CHANNEL_COUNT;
            self.write_buffer(&mut buffer[0..buffer_samples_to_fill]);
            self.event_ticks_progress = 0;
            self.next_event_index += 1;
            if self.next_event_index >= track_data.len() {
                self.has_finished = true;
                return;
            }

            let note_event = Self::note_event_from_midi_event(next_event);
            self.on_event_reached(&note_event);
        }
    }
}
