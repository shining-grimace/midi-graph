use crate::{
    util::{smf_from_file, wav_from_file},
    BaseMixer, MidiSource, SquareWaveSource,
};
use cpal::traits::StreamTrait;
use std::time::Duration;

const MIDI_FILE: &'static str = "resources/MIDI_sample.mid";
const WAV_FILE: &'static str = "resources/snare.wav";

#[test]
fn can_decode_midi_file() {
    let smf = smf_from_file(MIDI_FILE);
    assert!(smf.is_ok());
}

#[test]
fn can_decode_wav_file() {
    let wav = wav_from_file(WAV_FILE);
    assert!(wav.is_ok());
}

#[test]
fn can_play_square_stream() {
    let smf = smf_from_file(MIDI_FILE).unwrap();
    let midi = MidiSource::new(smf, Box::new(SquareWaveSource::default()));
    let mixer = BaseMixer::from_source(Box::new(midi));
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
    let smf = smf_from_file(MIDI_FILE).unwrap();
    let sample = wav_from_file(WAV_FILE).unwrap();
    let midi = MidiSource::new(smf, Box::new(sample));
    let mixer = BaseMixer::from_source(Box::new(midi));
    let stream = mixer.open_stream();
    assert!(stream.is_ok());

    let stream = stream.unwrap();
    let playback = stream.play();
    assert!(playback.is_ok());

    std::thread::sleep(Duration::from_secs(3));
    let pause = stream.pause();
    assert!(pause.is_ok());
}
