extern crate midi_graph;

use cpal::traits::StreamTrait;
use midi_graph::{
    util::smf_from_file, util::wav_from_file, BaseMixer, MidiSourceBuilder, NoteRange,
    SoundFontBuilder, SquareWaveSource,
};
use std::time::Duration;

const MIDI_FILE: &'static str = "resources/sample-in-c.mid";
const WAV_FILE: &'static str = "resources/guitar-a2-48k-stereo.wav";

const SQUARE_CHANNEL: usize = 0;
const WAV_CHANNEL: usize = 1;

fn main() {
    let smf = smf_from_file(MIDI_FILE).unwrap();
    let square_wave_font = SoundFontBuilder::new()
        .add_range(NoteRange::new_inclusive_range(0, 255), || {
            Box::new(SquareWaveSource::default())
        })
        .build();
    let wav_font = SoundFontBuilder::new()
        .add_range(NoteRange::new_inclusive_range(0, 255), || {
            Box::new(wav_from_file(WAV_FILE, 45).unwrap())
        })
        .build();
    let midi = MidiSourceBuilder::new(smf)
        .add_channel_font(SQUARE_CHANNEL, square_wave_font)
        .add_channel_font(WAV_CHANNEL, wav_font)
        .build()
        .unwrap();
    let mixer = BaseMixer::from_source(Box::new(midi));
    let stream = mixer.open_stream().expect("Could not open stream");
    stream.play().expect("Could not play the stream");
    std::thread::sleep(Duration::from_secs(16));
    stream.pause().expect("Could not pause the stream");
}
