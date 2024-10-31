extern crate midi_graph;

use cpal::traits::StreamTrait;
use midi_graph::{
    util::midi_builder_from_file, BaseMixer, LfsrNoiseSource, MixerSource, NoteRange,
    SawtoothWaveSource, SoundFontBuilder, SquareWaveSource, TriangleWaveSource,
};
use std::time::Duration;

const MIDI_FILE: &'static str = "resources/sample-in-c.mid";

const TRIANGLE_CHANNEL: usize = 0;
const SQUARE_CHANNEL: usize = 1;
const NOISE_CHANNEL: usize = 2;

fn main() {
    let triangle_unison = MixerSource::new(
        None,
        0.5,
        Box::new(TriangleWaveSource::new(None, 1.0)),
        Box::new(SawtoothWaveSource::new(None, 0.25)),
    );
    let triangle_font = SoundFontBuilder::new()
        .add_range(NoteRange::new_full_range(), Box::new(triangle_unison))
        .unwrap()
        .build();
    let square_font = SoundFontBuilder::new()
        .add_range(
            NoteRange::new_inclusive_range(0, 50),
            Box::new(SquareWaveSource::new(None, 0.125, 0.5)),
        )
        .unwrap()
        .add_range(
            NoteRange::new_inclusive_range(51, 255),
            Box::new(SquareWaveSource::new(None, 0.125, 0.875)),
        )
        .unwrap()
        .build();
    let noise_font = SoundFontBuilder::new()
        .add_range(
            NoteRange::new_full_range(),
            Box::new(LfsrNoiseSource::new(None, 0.25, false, 50)),
        )
        .unwrap()
        .build();
    let midi = midi_builder_from_file(None, MIDI_FILE)
        .unwrap()
        .add_channel_font(TRIANGLE_CHANNEL, triangle_font)
        .add_channel_font(SQUARE_CHANNEL, square_font)
        .add_channel_font(NOISE_CHANNEL, noise_font)
        .build()
        .unwrap();
    let mixer = BaseMixer::from_consumer(Box::new(midi));
    let stream = mixer.open_stream().expect("Could not open stream");
    stream.play().expect("Could not play the stream");
    std::thread::sleep(Duration::from_secs(16));
    stream.pause().expect("Could not pause the stream");
}
