use crate::{
    font::SoundFontBuilder,
    generator::SquareWaveSource,
    util::{midi_builder_from_file, wav_from_file},
    Balance, BaseMixer, NoteRange
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
                SoundFontBuilder::new(None)
                    .add_range(
                        NoteRange::new_full_range(),
                        Box::new(SquareWaveSource::new(None, Balance::Both, 0.25, 0.125)),
                    )
                    .unwrap()
                    .build(),
            ),
        )
        .build()
        .unwrap();
    let mixer = BaseMixer::start_single_program(Box::new(midi));
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
                SoundFontBuilder::new(None)
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
    let mixer = BaseMixer::start_single_program(Box::new(midi));
    assert!(mixer.is_ok());

    std::thread::sleep(Duration::from_secs(3));
}
