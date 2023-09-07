
use crate::{MidiProcessor, SquareAudio};
use std::time::Duration;
use cpal::traits::StreamTrait;

const MIDI_FILE: &'static str = "resources/MIDI_sample.mid";

#[test]
fn can_decode_midi_file() {
    let smf = MidiProcessor::from_file(MIDI_FILE);
    assert!(smf.is_ok());
}

#[test]
fn can_play_stream() {

    let smf = MidiProcessor::from_file(MIDI_FILE).unwrap();
    let streamer = SquareAudio::default();
    let stream = smf.open_stream(streamer);
    assert!(stream.is_ok());

    let stream = stream.unwrap();
    let playback = stream.play();
    assert!(playback.is_ok());

    std::thread::sleep(Duration::from_secs(3));
    let pause = stream.pause();
    assert!(pause.is_ok());
}
