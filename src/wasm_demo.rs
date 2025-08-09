use crate::{
    AssetLoadPayload, AssetLoader, Balance, BaseMixer, Error, SampleBuffer, SerializedFileMetadata,
    abstraction::{ChildConfig, NodeConfig},
    generator::SquareWave,
    midi::{Midi, MidiDataSource},
};
use std::{collections::HashMap, time::Duration};
use wasm_bindgen::prelude::*;

const MIDI_FILE_PATH: &str = "resources/LoopingMidi.mid";
const MIDI_FILE: &'static [u8] = include_bytes!("../resources/LoopingMidi.mid");

struct WasmAssetLoader;

impl AssetLoader for WasmAssetLoader {
    fn load_asset_data(&mut self, path: &str) -> Result<AssetLoadPayload, Error> {
        match path {
            MIDI_FILE_PATH => Ok(AssetLoadPayload::RawAssetData(MIDI_FILE.to_vec())),
            _ => Err(Error::User(format!("Cannot find asset: {}", path))),
        }
    }

    fn store_prepared_data(
        &mut self,
        _path: &str,
        _metadata: SerializedFileMetadata,
        _sample_buffer: SampleBuffer,
    ) {
        panic!("Cannot store assets in WASM tests");
    }
}

#[wasm_bindgen]
pub fn play_stream() {
    let config = Midi {
        node_id: None,
        source: MidiDataSource::FilePath {
            path: MIDI_FILE_PATH.to_string(),
            track_index: 0,
        },
        channels: HashMap::from([(
            0,
            ChildConfig(Box::new(SquareWave {
                node_id: None,
                balance: Balance::Both,
                amplitude: 0.25,
                duty_cycle: 0.125,
            })),
        )]),
    };
    let mut asset_loader = WasmAssetLoader;
    let midi_source = config.to_node(&mut asset_loader).unwrap();
    let _mixer = BaseMixer::builder_with_default_registry()
        .unwrap()
        .set_initial_program(1, midi_source)
        .start(Some(1))
        .unwrap();
    std::thread::sleep(Duration::from_secs(5));
}
