extern crate wasm_bindgen_test;

use crate::{
    AssetLoadPayload, AssetLoader, Balance, Error, GraphNode, SampleBuffer, SerializedFileMetadata,
    abstraction::NodeConfig,
    generator::SampleLoop,
    midi::{Midi, MidiDataSource},
};
use std::collections::HashMap;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

const MIDI_FILE_PATH: &str = "resources/sample-in-c.mid";
const WAV_FILE_PATH: &str = "resources/guitar-a2-48k-stereo.wav";

const MIDI_FILE: &[u8] = include_bytes!("../resources/sample-in-c.mid");
const WAV_FILE: &[u8] = include_bytes!("../resources/guitar-a2-48k-stereo.wav");

struct WasmAssetLoader;

impl AssetLoader for WasmAssetLoader {
    fn load_asset_data(&mut self, path: &str) -> Result<AssetLoadPayload, Error> {
        match path {
            MIDI_FILE_PATH => Ok(AssetLoadPayload::RawAssetData(MIDI_FILE.to_vec())),
            WAV_FILE_PATH => Ok(AssetLoadPayload::RawAssetData(WAV_FILE.to_vec())),
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

fn midi_node_from_file() -> Result<GraphNode, Error> {
    let midi_config = Midi {
        node_id: None,
        source: MidiDataSource::FilePath {
            path: MIDI_FILE_PATH.to_string(),
            track_index: 0,
        },
        channels: HashMap::new(),
    };
    let mut file_loader = WasmAssetLoader;
    midi_config.to_node(&mut file_loader)
}

fn wav_node_from_file() -> Result<GraphNode, Error> {
    let wav_config = SampleLoop {
        node_id: None,
        balance: Balance::Both,
        path: MIDI_FILE_PATH.to_owned(),
        base_note: 69,
        looping: None,
    };
    let mut file_loader = WasmAssetLoader;
    wav_config.to_node(&mut file_loader)
}

#[wasm_bindgen_test]
fn can_decode_files() {
    // Test MIDI file
    let midi_builder = midi_node_from_file();
    assert!(midi_builder.is_ok());

    // Test wav file
    let node_result = wav_node_from_file();
    assert!(node_result.is_ok());
}
