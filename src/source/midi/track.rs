use crate::{config, BufferConsumer, NoteConsumer, NoteEvent};
use midly::{MidiMessage, Smf, TrackEvent, TrackEventKind};
use std::sync::Arc;

const SOURCE_CAPACITY: usize = 8;

pub struct MidiTrackSource<'a> {
    smf: Arc<Smf<'a>>,
    track_no: usize,
    samples_per_tick: f64,
    has_finished: bool,
    active_count: usize,
    next_event_index: usize,
    event_ticks_progress: isize,
    sources: Vec<(u8, Box<dyn NoteConsumer + Send + 'static>)>,
}

impl<'a> MidiTrackSource<'a> {
    pub fn new(
        smf: Arc<Smf<'a>>,
        track_no: usize,
        samples_per_tick: f64,
        note_consumer_spawner: fn() -> Box<dyn NoteConsumer + Send + 'static>,
    ) -> Self {
        let mut sources = Vec::new();
        for _ in 0..SOURCE_CAPACITY {
            sources.push((0, note_consumer_spawner()));
        }
        Self {
            smf,
            track_no,
            samples_per_tick,
            has_finished: false,
            active_count: 0,
            next_event_index: 0,
            event_ticks_progress: 0,
            sources,
        }
    }

    fn on_event_reached(&mut self, event: &Option<NoteEvent>) {
        match event {
            None => {}
            Some(NoteEvent::NoteOn(note)) => {
                let same_notes = self.sources[0..self.active_count]
                    .iter()
                    .filter(|(n, _)| *n == *note)
                    .count();
                if same_notes > 0 {
                    #[cfg(debug_assertions)]
                    println!("Note turning on, but was already on");
                    return;
                }
                if self.active_count >= self.sources.len() {
                    #[cfg(debug_assertions)]
                    println!("Note turning on, but all sources in use");
                    return;
                }
                self.sources[self.active_count].0 = *note;
                self.sources[self.active_count]
                    .1
                    .restart_with_event(NoteEvent::NoteOn(*note));
                self.active_count += 1;
            }
            Some(NoteEvent::NoteOff(note)) => {
                let maybe_index = self.sources[0..self.active_count]
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
                let mut removed_source = self.sources.remove(source_index);
                removed_source
                    .1
                    .restart_with_event(NoteEvent::NoteOff(*note));
                self.sources.push(removed_source);
                self.active_count -= 1;
            }
        }
    }

    fn note_event_from_midi_event(event: &TrackEvent) -> Option<NoteEvent> {
        match event.kind {
            TrackEventKind::Midi {
                channel: _,
                message: MidiMessage::NoteOn { key, vel: _ },
            } => Some(NoteEvent::NoteOn(u8::from(key))),
            TrackEventKind::Midi {
                channel: _,
                message: MidiMessage::NoteOff { key, vel: _ },
            } => Some(NoteEvent::NoteOff(u8::from(key))),
            _ => None,
        }
    }

    fn write_buffer(&mut self, buffer: &mut [f32]) {
        for i in 0..self.active_count {
            self.sources[i].1.fill_buffer(buffer);
        }
    }
}

impl<'a> BufferConsumer for MidiTrackSource<'a> {
    fn set_note(&mut self, _event: NoteEvent) {}

    fn fill_buffer(&mut self, buffer: &mut [f32]) {
        #[cfg(debug_assertions)]
        assert_eq!(buffer.len() % config::CHANNEL_COUNT, 0);

        // Currently only-supported channel configuration
        #[cfg(debug_assertions)]
        assert_eq!(config::CHANNEL_COUNT, 2);

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
            let samples_available_per_channel = buffer.len() / config::CHANNEL_COUNT;
            if samples_until_event > samples_available_per_channel {
                self.write_buffer(buffer);
                self.event_ticks_progress +=
                    (samples_available_per_channel as f64 / self.samples_per_tick) as isize;
                return;
            }

            let buffer_samples_to_fill = samples_until_event * config::CHANNEL_COUNT;
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
