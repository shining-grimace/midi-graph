use crate::{config, AudioSource};
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
    sources: Vec<(u8, Box<dyn AudioSource + Send + 'static>)>,
}

impl<'a> MidiTrackSource<'a> {
    pub fn new(
        smf: Arc<Smf<'a>>,
        track_no: usize,
        samples_per_tick: f64,
        source_spawner: fn() -> Box<dyn AudioSource + Send + 'static>,
    ) -> Self {
        let mut sources = Vec::new();
        for _ in 0..SOURCE_CAPACITY {
            sources.push((0, source_spawner()));
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

    // Update self when a new event was reached
    fn update_on_event(&mut self, event: &TrackEvent) {
        if let TrackEventKind::Midi {
            channel: _,
            message: MidiMessage::NoteOn { key, vel: _ },
        } = event.kind
        {
            let note = u8::from(key);
            self.on_note_on(note);
        }
        if let TrackEventKind::Midi {
            channel: _,
            message: MidiMessage::NoteOff { key, vel: _ },
        } = event.kind
        {
            let note = u8::from(key);
            self.on_note_off(note);
        }
    }

    fn write_buffer(&mut self, buffer: &mut [f32]) {
        for i in 0..self.active_count {
            self.sources[i].1.fill_buffer(buffer);
        }
    }
}

impl<'a> AudioSource for MidiTrackSource<'a> {
    fn on_note_on(&mut self, key: u8) {
        let same_notes = self.sources[0..self.active_count]
            .iter()
            .filter(|(note, _)| *note == key)
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
        self.sources[self.active_count].0 = key;
        self.sources[self.active_count].1.on_note_on(key);
        self.active_count += 1;
    }

    fn on_note_off(&mut self, key: u8) {
        let maybe_index = self.sources[0..self.active_count]
            .iter()
            .position(|(note, _)| *note == key);
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
        removed_source.1.on_note_off(key);
        self.sources.push(removed_source);
        self.active_count -= 1;
    }

    fn fill_buffer(&mut self, buffer: &mut [f32]) {
        let smf = Arc::clone(&self.smf);
        let track_data = &smf.tracks[self.track_no];
        if self.has_finished {
            return;
        }
        let next_event = &track_data[self.next_event_index];
        let event_ticks_delta = u32::from(next_event.delta) as isize;
        let ticks_until_event = event_ticks_delta - self.event_ticks_progress;
        let samples_until_event = (ticks_until_event as f64 * self.samples_per_tick) as usize;
        let samples_available_per_channel = buffer.len() / config::CHANNEL_COUNT;
        let samples_to_play_now = samples_until_event.min(samples_available_per_channel);

        #[cfg(debug_assertions)]
        assert_eq!(buffer.len() % config::CHANNEL_COUNT, 0);

        // Currently only-supported channel configuration
        #[cfg(debug_assertions)]
        assert_eq!(config::CHANNEL_COUNT, 2);

        if ticks_until_event > 0 {
            let ticks_available =
                ((samples_available_per_channel as f64) / self.samples_per_tick) as isize;
            self.event_ticks_progress += ticks_until_event.min(ticks_available);
            self.write_buffer(&mut buffer[0..(samples_to_play_now * config::CHANNEL_COUNT)]);
        }
        if self.event_ticks_progress >= event_ticks_delta {
            self.update_on_event(next_event);
            self.next_event_index += 1;
            self.event_ticks_progress = 0;
            let remaining_buffer = &mut buffer[(samples_to_play_now * config::CHANNEL_COUNT)..];
            if self.next_event_index >= track_data.len() {
                self.has_finished = true;
                return;
            }
            self.write_buffer(remaining_buffer);
        }
    }
}
