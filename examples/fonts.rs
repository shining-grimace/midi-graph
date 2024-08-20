extern crate midi_graph;

use cpal::traits::StreamTrait;
use midi_graph::{
    util::smf_from_file, util::wav_from_file, BaseMixer, BufferConsumer, MidiSource, NoteRange,
    SoundFontBuilder, SquareWaveSource,
};
use std::time::Duration;

const MIDI_FILE: &'static str = "resources/dansenapolitaine.mid";
const WAV_FILE: &'static str = "resources/guitar-a2-48k-stereo.wav";

fn main() {
    let smf = smf_from_file(MIDI_FILE).unwrap();
    let fonts = (0..smf.tracks.len())
        .map(|index| {
            let spawner: fn() -> Box<dyn BufferConsumer + Send + 'static> = match index != 1 {
                true => || Box::new(SquareWaveSource::default()),
                false => || Box::new(wav_from_file(WAV_FILE, 45).unwrap()),
            };
            SoundFontBuilder::new()
                .add_range(NoteRange::new(0, 255), spawner)
                .build()
        })
        .collect();
    let midi = MidiSource::new(smf, fonts).unwrap();
    let mixer = BaseMixer::from_source(Box::new(midi));
    let stream = mixer.open_stream().expect("Could not open stream");
    stream.play().expect("Could not play the stream");
    std::thread::sleep(Duration::from_secs(5));
    stream.pause().expect("Could not pause the stream");
}
