extern crate midi_graph;

use cpal::traits::StreamTrait;
use midi_graph::{
    util::{smf_from_file, soundfont_from_file, wav_from_file},
    BaseMixer, MidiSourceBuilder, NoteRange, SoundFontBuilder,
};
use std::time::Duration;

const MIDI_FILE: &'static str = "resources/sample-in-c.mid";
const SF2_FILE: &'static str = "resources/german8-harpsichord.sf2";
const WAV_FILE: &'static str = "resources/guitar-a2-48k-stereo.wav";

const SOUNDFONT_CHANNEL: usize = 0;
const WAV_CHANNEL: usize = 1;

fn main() {
    let smf = smf_from_file(MIDI_FILE).unwrap();
    let sf2_font = soundfont_from_file(SF2_FILE, 0).unwrap();
    let wav_font = SoundFontBuilder::new()
        .add_range(
            NoteRange::new_full_range(),
            Box::new(wav_from_file(WAV_FILE, 45).unwrap()),
        )?
        .build();
    let midi = MidiSourceBuilder::new(smf)
        .add_channel_font(SOUNDFONT_CHANNEL, sf2_font)
        .add_channel_font(WAV_CHANNEL, wav_font)
        .build()
        .unwrap();

    let mixer = BaseMixer::from_source(Box::new(midi));
    let stream = mixer.open_stream().expect("Could not open stream");
    stream.play().expect("Could not play the stream");
    std::thread::sleep(Duration::from_secs(16));
    stream.pause().expect("Could not pause the stream");
}
