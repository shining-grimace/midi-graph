use crate::{constants::PLAYBACK_SAMPLE_RATE, Error};
use midly::{Fps, MetaMessage, Smf, Timing, TrackEventKind};

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
            match found_micros_per_beat {
                Some(micros_per_beat) => {
                    let samples_per_micro = (PLAYBACK_SAMPLE_RATE as f64) / 1000000.0;
                    let samples_per_beat = samples_per_micro * (micros_per_beat as f64);
                    let samples_per_tick = samples_per_beat / (u16::from(ticks_per_beat) as f64);
                    Ok(samples_per_tick)
                }
                None => Err(Error::User("No tempo information found".to_owned())),
            }
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
    return None;
}
