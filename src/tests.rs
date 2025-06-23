use crate::{
    Balance, BaseMixer, NoteRange,
    generator::SquareWaveNode,
    group::FontNodeBuilder,
    util::{midi_builder_from_file, wav_from_file},
};
use std::time::Duration;

const MIDI_FILE: &'static str = "resources/sample-in-c.mid";
const WAV_FILE: &'static str = "resources/guitar-a2-48k-stereo.wav";

#[test]
fn can_decode_midi_file() {
    let midi_builder = midi_builder_from_file(None, MIDI_FILE);
    assert!(midi_builder.is_ok());
}

#[test]
fn can_decode_wav_file() {
    let wav = wav_from_file(WAV_FILE, 69, None, Balance::Both, None);
    assert!(wav.is_ok());
}

#[test]
fn can_play_square_stream() {
    let midi = midi_builder_from_file(None, MIDI_FILE)
        .unwrap()
        .add_channel_source(
            0,
            Box::new(
                FontNodeBuilder::new(None)
                    .add_range(
                        NoteRange::new_full_range(),
                        Box::new(SquareWaveNode::new(None, Balance::Both, 0.25, 0.125)),
                    )
                    .unwrap()
                    .build(),
            ),
        )
        .build()
        .unwrap();
    let mixer = BaseMixer::builder_with_default_registry()
        .unwrap()
        .set_initial_program(1, Box::new(midi))
        .start(Some(1));
    assert!(mixer.is_ok());

    std::thread::sleep(Duration::from_secs(3));
}

#[test]
fn can_play_wav_stream() {
    let midi = midi_builder_from_file(None, MIDI_FILE)
        .unwrap()
        .add_channel_source(
            0,
            Box::new(
                FontNodeBuilder::new(None)
                    .add_range(
                        NoteRange::new_full_range(),
                        Box::new(wav_from_file(WAV_FILE, 69, None, Balance::Both, None).unwrap()),
                    )
                    .unwrap()
                    .build(),
            ),
        )
        .build()
        .unwrap();
    let mixer = BaseMixer::builder_with_existing_registry()
        .set_initial_program(1, Box::new(midi))
        .start(Some(1));
    assert!(mixer.is_ok());

    std::thread::sleep(Duration::from_secs(3));
}
