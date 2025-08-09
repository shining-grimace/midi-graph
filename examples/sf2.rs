extern crate midi_graph;

use midi_graph::{
    Balance, BaseMixer, FileAssetLoader,
    abstraction::ChildConfig,
    generator::LfsrNoise,
    group::{Font, FontSource, RangeSource},
    midi::{Midi, MidiDataSource},
};
use std::{collections::HashMap, time::Duration};

const MIDI_FILE: &'static str = "resources/sample-in-c.mid";
const SF2_FILE: &'static str = "resources/demo-font.sf2";

const SOUNDFONT_0_CHANNEL: usize = 0;
const SOUNDFONT_1_CHANNEL: usize = 1;
const NOISE_CHANNEL: usize = 2;

fn main() {
    let mut asset_loader = FileAssetLoader::default();
    let font_0 = Font {
        node_id: None,
        config: FontSource::Sf2FilePath {
            path: SF2_FILE.to_owned(),
            instrument_index: 0,
            polyphony_voices: 4,
        },
    };
    let font_1 = Font {
        node_id: None,
        config: FontSource::Sf2FilePath {
            path: SF2_FILE.to_owned(),
            instrument_index: 0,
            polyphony_voices: 4,
        },
    };
    let noise_font = Font {
        node_id: None,
        config: FontSource::Ranges(vec![RangeSource {
            source: ChildConfig(Box::new(LfsrNoise {
                node_id: None,
                balance: Balance::Both,
                amplitude: 0.25,
                inside_feedback: false,
                note_for_16_shifts: 50,
            })),
            lower: 0,
            upper: 127,
        }]),
    };
    let midi = Midi {
        node_id: None,
        source: MidiDataSource::FilePath {
            path: MIDI_FILE.to_owned(),
            track_index: 0,
        },
        channels: HashMap::from([
            (SOUNDFONT_0_CHANNEL, ChildConfig(Box::new(font_0))),
            (SOUNDFONT_1_CHANNEL, ChildConfig(Box::new(font_1))),
            (NOISE_CHANNEL, ChildConfig(Box::new(noise_font))),
        ]),
    };

    let _mixer = BaseMixer::builder_with_default_registry()
        .unwrap()
        .set_initial_program_from_config(1, ChildConfig(Box::new(midi)), &mut asset_loader)
        .unwrap()
        .start(Some(1))
        .unwrap();
    std::thread::sleep(Duration::from_secs(16));
}
