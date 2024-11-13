use crate::{consts::PLAYBACK_SAMPLE_RATE, Error};
use midly::{Fps, MetaMessage, MidiMessage, Smf, Timing, TrackEvent, TrackEventKind};

pub fn get_samples_per_tick(smf: &Smf) -> Result<f64, Error> {
    match smf.header.timing {
        Timing::Metrical(ticks_per_beat) => {
            let found_micros_per_beat: Option<f64> =
                scan_for_data(smf, |event_kind| match event_kind {
                    TrackEventKind::Meta(MetaMessage::Tempo(micros)) => {
                        Some(u32::from(*micros) as f64)
                    }
                    _ => None,
                });
            let micros_per_beat: f64 = match found_micros_per_beat {
                Some(micros) => micros,
                None => {
                    // TODO - This is a fallback for Ardour not exporting
                    // tempo meta events. This is not ideal.
                    println!("WARNING: MIDI: Tempo meta event not found, assuming 120 BPM");
                    1000000.0 / (120.0 / 60.0)
                }
            };
            let samples_per_micro = (PLAYBACK_SAMPLE_RATE as f64) / 1000000.0;
            let samples_per_beat = samples_per_micro * micros_per_beat;
            let samples_per_tick = samples_per_beat / (u16::from(ticks_per_beat) as f64);
            Ok(samples_per_tick)
        }
        Timing::Timecode(fps, sub) => {
            let samples_per_second: f64 = PLAYBACK_SAMPLE_RATE as f64;
            let frames_per_second: f64 = match fps {
                Fps::Fps24 => 24.0,
                Fps::Fps25 => 25.0,
                Fps::Fps29 => 30.0 / 1.001,
                Fps::Fps30 => 30.0,
            };
            let ticks_per_second = 1.0 / (frames_per_second * (sub as f64));
            let samples_per_tick = samples_per_second / ticks_per_second;
            Ok(samples_per_tick)
        }
    }
}

fn scan_for_data<T>(smf: &Smf, extractor: fn(&TrackEventKind) -> Option<T>) -> Option<T> {
    for track in smf.tracks.iter() {
        for event in track.iter() {
            if let Some(value) = extractor(&event.kind) {
                return Some(value);
            }
        }
    }
    None
}

pub fn choose_track_index(smf: &Smf) -> Result<usize, Error> {
    if smf.tracks.is_empty() {
        return Err(Error::User("MIDI: No tracks in MIDI file".to_owned()));
    }
    for (i, track) in smf.tracks.iter().enumerate() {
        let any_note_on_events = track.iter().any(|event| {
            matches!(
                event,
                TrackEvent {
                    kind: TrackEventKind::Midi {
                        message: MidiMessage::NoteOn { key: _, vel: _ },
                        ..
                    },
                    ..
                }
            )
        });
        if any_note_on_events {
            return Ok(i);
        }
    }
    Err(Error::User(
        "MIDI: No tracks found with key on events".to_owned(),
    ))
}
