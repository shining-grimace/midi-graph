extern crate midi_graph;

use cpal::traits::StreamTrait;
use midi_graph::{
    util::smf_from_file, BaseMixer, MidiSourceBuilder, NoteRange, SoundFontBuilder,
    SquareWaveSource,
};
use std::time::Duration;

const MIDI_FILE: &'static str = "resources/sample-in-c.mid";

const SQUARE_500_CHANNEL: usize = 0;
const SQUARE_875_CHANNEL: usize = 1;

fn main() {
    let smf = smf_from_file(MIDI_FILE).unwrap();
    let square_500_font = SoundFontBuilder::new()
        .add_range(NoteRange::new_inclusive_range(0, 255), || {
            Box::new(SquareWaveSource::new(0.25, 0.5))
        })
        .build();
    let square_875_font = SoundFontBuilder::new()
        .add_range(NoteRange::new_inclusive_range(0, 255), || {
            Box::new(SquareWaveSource::new(0.25, 0.875))
        })
        .build();
    let midi = MidiSourceBuilder::new(smf)
        .add_channel_font(SQUARE_500_CHANNEL, square_500_font)
        .add_channel_font(SQUARE_875_CHANNEL, square_875_font)
        .build()
        .unwrap();
    let mixer = BaseMixer::from_source(Box::new(midi));
    let stream = mixer.open_stream().expect("Could not open stream");
    stream.play().expect("Could not play the stream");
    std::thread::sleep(Duration::from_secs(16));
    stream.pause().expect("Could not pause the stream");
}
