use midly::{num::u24, Fps, MetaMessage, MidiMessage, Smf, Timing, TrackEvent, TrackEventKind};

pub fn log_loaded_midi(smf: &Smf) {
    println!("MIDI: {}", get_log_for_header(smf));
    for track_events in smf.tracks.iter() {
        for event in track_events.iter() {
            if let Some(message) = get_log_for_event(event) {
                println!("MIDI: {}", message);
            }
        }
    }
    println!("MIDI: File loaded.");
    if smf.tracks.is_empty() {
        println!("WARNING: MIDI: There are no tracks to play.");
    } else {
        for (index, track) in smf.tracks.iter().enumerate() {
            println!("MIDI: Track {} has {} events.", index, track.len());
        }
    }
}

fn get_log_for_header(smf: &Smf) -> String {
    let timing = smf.header.timing;
    match timing {
        Timing::Metrical(ticks_per_beat) => {
            format!("Metrical timing: {} ticks/beat", ticks_per_beat)
        }
        Timing::Timecode(fps, tpf) => match fps {
            Fps::Fps24 => format!("Timecode timing, 24 fps with {} ticks/frame", tpf),
            Fps::Fps25 => format!("Timecode timing, 25 fps with {} ticks/frame", tpf),
            Fps::Fps29 => format!("Timecode timing, 29 fps with {} ticks/frame", tpf),
            Fps::Fps30 => format!("Timecode timing, 30 fps with {} ticks/frame", tpf),
        },
    }
}

fn get_log_for_event(event: &TrackEvent) -> Option<String> {
    match event.kind {
        TrackEventKind::Midi {
            channel: _,
            message,
        } => match message {
            MidiMessage::NoteOn { key: _, vel: _ } => None,
            MidiMessage::NoteOff { key: _, vel: _ } => None,
            MidiMessage::PitchBend { bend: _ } => None,
            MidiMessage::Aftertouch { key: _, vel: _ } => None,
            MidiMessage::ChannelAftertouch { vel: _ } => None,
            MidiMessage::Controller {
                controller: _,
                value: _,
            } => None,
            MidiMessage::ProgramChange { program: _ } => None,
        },
        TrackEventKind::SysEx(_) => None,
        TrackEventKind::Escape(_) => None,
        TrackEventKind::Meta(message) => match message {
            MetaMessage::TrackNumber(_) => None,
            MetaMessage::Text(_) => None,
            MetaMessage::Copyright(_) => None,
            MetaMessage::TrackName(_) => None,
            MetaMessage::InstrumentName(_) => None,
            MetaMessage::Lyric(_) => None,
            MetaMessage::Marker(_) => None,
            MetaMessage::CuePoint(cue_point) => {
                let string = std::str::from_utf8(cue_point).unwrap();
                Some(format!("Cue label: {}", string))
            }
            MetaMessage::ProgramName(_) => None,
            MetaMessage::DeviceName(_) => None,
            MetaMessage::MidiChannel(_) => None,
            MetaMessage::MidiPort(_) => None,
            MetaMessage::EndOfTrack => None,
            MetaMessage::Tempo(micros_per_beat) => Some(log_tempo(micros_per_beat)),
            MetaMessage::SmpteOffset(_) => None,
            MetaMessage::TimeSignature(_, _, _, _) => None,
            MetaMessage::KeySignature(sharps, major) => Some(log_key(sharps, major)),
            MetaMessage::SequencerSpecific(_) => None,
            MetaMessage::Unknown(_, _) => Some("Unknown message".to_owned()),
        },
    }
}

fn log_key(sharps: i8, major: bool) -> String {
    let key_type = match major {
        true => "major",
        false => "minor",
    };
    match sharps < 0 {
        true => format!("Key: {} flats ({})", -sharps, key_type),
        false => format!("Key: {} sharps ({})", sharps, key_type),
    }
}

fn log_tempo(micros_per_beat: u24) -> String {
    let bpm = 60000000.0 / (u32::from(micros_per_beat) as f64);
    format!("Tempo: {} BPM", bpm)
}
