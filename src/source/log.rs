use midly::{MetaMessage, MidiMessage, Smf, TrackEvent, TrackEventKind};

pub fn log_loaded_midi(smf: &Smf) {
    for track_events in smf.tracks.iter() {
        for event in track_events.iter() {
            if let Some(message) = get_log_for_event(event) {
                println!("{}", message);
            }
        }
    }
    println!("MIDI file loaded.");
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
            MetaMessage::CuePoint(_) => None,
            MetaMessage::ProgramName(_) => None,
            MetaMessage::DeviceName(_) => None,
            MetaMessage::MidiChannel(_) => None,
            MetaMessage::MidiPort(_) => None,
            MetaMessage::EndOfTrack => None,
            MetaMessage::Tempo(_) => None,
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
