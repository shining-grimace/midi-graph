extern crate midi_graph;

use cpal::traits::StreamTrait;
use midi_graph::{
    util::{midi_builder_from_file, soundfont_from_file},
    BaseMixer,
};
use std::time::Duration;

const MIDI_FILE: &'static str = "resources/sample-in-c.mid";
const SF2_FILE: &'static str = "resources/demo-font.sf2";

const SOUNDFONT_0_CHANNEL: usize = 0;
const SOUNDFONT_1_CHANNEL: usize = 1;

fn main() {
    let font_0 = soundfont_from_file(SF2_FILE, 0).unwrap();
    let font_1 = soundfont_from_file(SF2_FILE, 0).unwrap();
    let midi = midi_builder_from_file(MIDI_FILE)
        .unwrap()
        .add_channel_font(SOUNDFONT_0_CHANNEL, font_0)
        .add_channel_font(SOUNDFONT_1_CHANNEL, font_1)
        .build()
        .unwrap();

    let mixer = BaseMixer::from_consumer(Box::new(midi));
    let stream = mixer.open_stream().expect("Could not open stream");
    stream.play().expect("Could not play the stream");
    std::thread::sleep(Duration::from_secs(16));
    stream.pause().expect("Could not pause the stream");
}
