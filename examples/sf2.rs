extern crate midi_graph;

use midi_graph::{
    Balance, BaseMixer, NoteRange,
    generator::LfsrNoiseNode,
    group::FontNodeBuilder,
    util::{midi_builder_from_file, soundfont_from_file},
};
use std::time::Duration;

const MIDI_FILE: &'static str = "resources/sample-in-c.mid";
const SF2_FILE: &'static str = "resources/demo-font.sf2";

const SOUNDFONT_0_CHANNEL: usize = 0;
const SOUNDFONT_1_CHANNEL: usize = 1;
const NOISE_CHANNEL: usize = 2;

fn main() {
    let font_0 = soundfont_from_file(None, SF2_FILE, 0, 4).unwrap();
    let font_1 = soundfont_from_file(None, SF2_FILE, 0, 4).unwrap();
    let noise_font = FontNodeBuilder::new(None)
        .add_range(
            NoteRange::new_full_range(),
            Box::new(LfsrNoiseNode::new(None, Balance::Both, 0.25, false, 50)),
        )
        .unwrap()
        .build();
    let midi = midi_builder_from_file(None, MIDI_FILE, 0)
        .unwrap()
        .add_channel_source(SOUNDFONT_0_CHANNEL, Box::new(font_0))
        .add_channel_source(SOUNDFONT_1_CHANNEL, Box::new(font_1))
        .add_channel_source(NOISE_CHANNEL, Box::new(noise_font))
        .build()
        .unwrap();

    let _mixer = BaseMixer::builder_with_default_registry()
        .unwrap()
        .set_initial_program(1, Box::new(midi))
        .start(Some(1))
        .unwrap();
    std::thread::sleep(Duration::from_secs(16));
}
