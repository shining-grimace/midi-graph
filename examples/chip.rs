extern crate midi_graph;

use midi_graph::{
    font::SoundFontBuilder,
    generator::{LfsrNoiseSource, SawtoothWaveSource, SquareWaveSource, TriangleWaveSource},
    group::{MixerSource, Polyphony},
    util::midi_builder_from_file,
    BaseMixer, NoteRange,
};
use std::time::Duration;

const MIDI_FILE: &'static str = "resources/sample-in-c.mid";

const TRIANGLE_CHANNEL: usize = 0;
const SQUARE_CHANNEL: usize = 1;
const NOISE_CHANNEL: usize = 2;

fn main() {
    let triangle_unison = Polyphony::new(
        None,
        4,
        Box::new(MixerSource::new(
            None,
            0.5,
            Box::new(TriangleWaveSource::new(None, 1.0)),
            Box::new(SawtoothWaveSource::new(None, 0.25)),
        )),
    )
    .unwrap();
    let triangle_font = SoundFontBuilder::new(None)
        .add_range(NoteRange::new_full_range(), Box::new(triangle_unison))
        .unwrap()
        .build();
    let square_font = SoundFontBuilder::new(None)
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
    let noise_font = SoundFontBuilder::new(None)
        .add_range(
            NoteRange::new_full_range(),
            Box::new(LfsrNoiseSource::new(None, 0.25, false, 50)),
        )
        .unwrap()
        .build();
    let midi = midi_builder_from_file(None, MIDI_FILE)
        .unwrap()
        .add_channel_source(TRIANGLE_CHANNEL, Box::new(triangle_font))
        .add_channel_source(SQUARE_CHANNEL, Box::new(square_font))
        .add_channel_source(NOISE_CHANNEL, Box::new(noise_font))
        .build()
        .unwrap();
    let _mixer = BaseMixer::start_single_program(Box::new(midi)).expect("Could not open stream");
    std::thread::sleep(Duration::from_secs(16));
}
