use crate::{
    util::{midi_builder_from_file, wav_from_file},
    BaseMixer, NoteRange, SoundFontBuilder, SquareWaveSource,
};
use cpal::traits::StreamTrait;
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
    let wav = wav_from_file(WAV_FILE, 69, None, None);
    assert!(wav.is_ok());
}

#[test]
fn can_play_square_stream() {
    let midi = midi_builder_from_file(None, MIDI_FILE)
        .unwrap()
        .add_channel_font(
            0,
            SoundFontBuilder::new()
                .add_range(
                    NoteRange::new_full_range(),
                    Box::new(SquareWaveSource::new(None, 0.25, 0.125)),
                )
                .unwrap()
                .build(),
        )
        .build()
        .unwrap();
    let mixer = BaseMixer::from_consumer(Box::new(midi));
    let stream = mixer.open_stream();
    assert!(stream.is_ok());

    let stream = stream.unwrap();
    let playback = stream.play();
    assert!(playback.is_ok());

    std::thread::sleep(Duration::from_secs(3));
    let pause = stream.pause();
    assert!(pause.is_ok());
}

#[test]
fn can_play_wav_stream() {
    let midi = midi_builder_from_file(None, MIDI_FILE)
        .unwrap()
        .add_channel_font(
            0,
            SoundFontBuilder::new()
                .add_range(
                    NoteRange::new_full_range(),
                    Box::new(wav_from_file(WAV_FILE, 69, None, None).unwrap()),
                )
                .unwrap()
                .build(),
        )
        .build()
        .unwrap();
    let mixer = BaseMixer::from_consumer(Box::new(midi));

    let stream = mixer.open_stream();
    assert!(stream.is_ok());

    let stream = stream.unwrap();
    let playback = stream.play();
    assert!(playback.is_ok());

    std::thread::sleep(Duration::from_secs(3));
    let pause = stream.pause();
    assert!(pause.is_ok());
}
