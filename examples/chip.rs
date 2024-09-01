extern crate midi_graph;

use cpal::traits::StreamTrait;
use midi_graph::{
    util::smf_from_file, BaseMixer, MidiSourceBuilder, NoteRange, SoundFontBuilder,
    SquareWaveSource, TriangleWaveSource,
};
use std::time::Duration;

const MIDI_FILE: &'static str = "resources/sample-in-c.mid";

const SQUARE_CHANNEL: usize = 1;
const TRIANGLE_CHANNEL: usize = 0;

fn main() {
    let smf = smf_from_file(MIDI_FILE).unwrap();
    let square_font = SoundFontBuilder::new()
        .add_range(NoteRange::new_inclusive_range(0, 50), || {
            Box::new(SquareWaveSource::new(0.125, 0.5))
        })
        .add_range(NoteRange::new_inclusive_range(51, 255), || {
            Box::new(SquareWaveSource::new(0.125, 0.875))
        })
        .build();
    let triangle_font = SoundFontBuilder::new()
        .add_range(NoteRange::new_inclusive_range(0, 255), || {
            Box::new(TriangleWaveSource::new(1.0))
        })
        .build();
    let midi = MidiSourceBuilder::new(smf)
        .add_channel_font(SQUARE_CHANNEL, square_font)
        .add_channel_font(TRIANGLE_CHANNEL, triangle_font)
        .build()
        .unwrap();
    let mixer = BaseMixer::from_source(Box::new(midi));
    let stream = mixer.open_stream().expect("Could not open stream");
    stream.play().expect("Could not play the stream");
    std::thread::sleep(Duration::from_secs(16));
    stream.pause().expect("Could not pause the stream");
}
