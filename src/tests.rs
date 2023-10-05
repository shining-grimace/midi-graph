
use crate::{
    MidiProcessor,
    SquareAudio,
    WavAudio,
    util::{smf_from_file, wav_from_file}
};
use std::time::Duration;
use cpal::traits::StreamTrait;

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
    let midi = MidiProcessor::from_file(smf);
    let streamer = SquareAudio::default();
    let stream = midi.open_stream(streamer);
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
    let midi = MidiProcessor::from_file(smf);
    let (header, data) = wav_from_file(WAV_FILE).unwrap();
    let streamer = WavAudio::new_from_data(header, data);
    let stream = midi.open_stream(streamer);
    assert!(stream.is_ok());

    let stream = stream.unwrap();
    let playback = stream.play();
    assert!(playback.is_ok());

    std::thread::sleep(Duration::from_secs(3));
    let pause = stream.pause();
    assert!(pause.is_ok());
}
