use crate::{
    Balance, BaseMixer, FileAssetLoader,
    config::{ChildConfig, NodeConfig},
    generator::{SampleLoop, SquareWave},
    group::{Font, FontSource, RangeSource},
    midi::{Midi, MidiDataSource},
};
use std::{collections::HashMap, time::Duration};

const MIDI_FILE: &'static str = "resources/sample-in-c.mid";
const WAV_FILE: &'static str = "resources/guitar-a2-48k-stereo.wav";

fn wav_config_from_file() -> ChildConfig {
    ChildConfig(Box::new(SampleLoop {
        node_id: None,
        balance: Balance::Both,
        path: WAV_FILE.to_owned(),
        base_note: 69,
        looping: None,
    }))
}

#[test]
fn can_decode_midi_file() {
    let mut asset_loader = FileAssetLoader::default();
    let midi = Midi {
        node_id: None,
        source: MidiDataSource::FilePath {
            path: MIDI_FILE.to_owned(),
            track_index: 0,
        },
        channels: HashMap::new(),
    };
    let midi_node_result = midi.to_node(&mut asset_loader);
    assert!(midi_node_result.is_ok());
}

#[test]
fn can_decode_wav_file() {
    let mut asset_loader = FileAssetLoader::default();
    let node_result = wav_config_from_file().0.to_node(&mut asset_loader);
    assert!(node_result.is_ok());
}

#[test]
fn can_play_square_stream() {
    let mut asset_loader = FileAssetLoader::default();
    let midi = Midi {
        node_id: None,
        source: MidiDataSource::FilePath {
            path: MIDI_FILE.to_owned(),
            track_index: 0,
        },
        channels: HashMap::from([(
            0,
            ChildConfig(Box::new(Font {
                node_id: None,
                config: FontSource::Ranges(vec![RangeSource {
                    source: ChildConfig(Box::new(SquareWave {
                        node_id: None,
                        balance: Balance::Both,
                        amplitude: 0.25,
                        duty_cycle: 0.125,
                    })),
                    lower: 0,
                    upper: 127,
                }]),
            })),
        )]),
    };
    let midi_node = midi.to_node(&mut asset_loader).unwrap();
    let mixer = BaseMixer::builder_with_default_registry()
        .unwrap()
        .set_initial_program(1, midi_node)
        .start(Some(1));
    assert!(mixer.is_ok());

    std::thread::sleep(Duration::from_secs(3));
}

#[test]
fn can_play_wav_stream() {
    let mut asset_loader = FileAssetLoader::default();
    let midi = Midi {
        node_id: None,
        source: MidiDataSource::FilePath {
            path: MIDI_FILE.to_owned(),
            track_index: 0,
        },
        channels: HashMap::from([(
            0,
            ChildConfig(Box::new(Font {
                node_id: None,
                config: FontSource::Ranges(vec![RangeSource {
                    source: wav_config_from_file(),
                    lower: 0,
                    upper: 127,
                }]),
            })),
        )]),
    };
    let midi_node = midi.to_node(&mut asset_loader).unwrap();
    let mixer = BaseMixer::builder_with_existing_registry()
        .set_initial_program(1, midi_node)
        .start(Some(1));
    assert!(mixer.is_ok());

    std::thread::sleep(Duration::from_secs(3));
}
