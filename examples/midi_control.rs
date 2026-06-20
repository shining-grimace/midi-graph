extern crate midi_graph;

use midi_graph::{
    Balance, BaseMixer, Event, EventTarget, EventTiming, FileAssetLoader, Message,
    MidiPlaybackState,
    abstraction::{ChildConfig, Loop, NodeConfig},
    generator::{SampleBufferSource, SampleLoop},
    midi::{Midi, MidiDataSource},
};
use std::{collections::HashMap, time::Duration};

const MIDI_0_FILE: &'static str = "resources/sample-in-c.mid";

const MIDI_NODE_ID: u64 = 100;
const SAMPLER_NODE_ID: u64 = 101;

const PROGRAM_0: usize = 0;

fn main() {
    fn wavetable_source_0() -> [f32; 16] {
        [
            0.0, 0.25, 1.0, -0.5, -0.125, -0.5, 0.0, 0.125, 0.125, 0.25, 0.875, 1.0, 0.25, -0.25,
            -0.5, -0.25,
        ]
    }

    fn wavetable_source_1() -> [f32; 16] {
        [
            0.0, 0.125, 0.25, 0.125, -0.75, -0.125, -0.25, -0.75, 0.0, 0.125, 0.25, 0.125, -0.75,
            -0.125, -0.25, -0.75,
        ]
    }

    fn wavetable() -> Box<dyn NodeConfig + Send + Sync + 'static> {
        let source_wavetable = wavetable_source_0();
        let source_size = source_wavetable.len();
        Box::new(SampleLoop {
            node_id: Some(SAMPLER_NODE_ID),
            balance: Balance::Both,
            source: SampleBufferSource::WavetableWithSampleRate((4096, source_wavetable)),
            base_note: 127,
            looping: Some(Loop {
                start: 0,
                end: source_size,
            }),
        })
    }

    let mut asset_loader = FileAssetLoader::default();
    let program_0 = Midi {
        node_id: Some(MIDI_NODE_ID),
        source: MidiDataSource::FilePath {
            path: MIDI_0_FILE.to_owned(),
            track_index: 0,
        },
        channels: HashMap::from([(0, ChildConfig(wavetable()))]),
    }
    .to_node(&mut asset_loader)
    .unwrap();

    let mixer = BaseMixer::builder_with_default_registry()
        .unwrap()
        .store_program(PROGRAM_0, program_0)
        .start(Some(PROGRAM_0))
        .unwrap();
    let sender = mixer.get_event_sender();
    let absolute_frame = sender.current_rendering_absolute_frame();
    sender
        .send(Message {
            target: EventTarget::SpecificNode(MIDI_NODE_ID),
            data: Event::MidiPlayback(MidiPlaybackState::Paused),
            timing: EventTiming::after_seconds(absolute_frame, 2.0),
        })
        .unwrap();
    sender
        .send(Message {
            target: EventTarget::SpecificNode(MIDI_NODE_ID),
            data: Event::MidiPlayback(MidiPlaybackState::Playing),
            timing: EventTiming::after_seconds(absolute_frame, 4.0),
        })
        .unwrap();
    sender
        .send(Message {
            target: EventTarget::SpecificNode(SAMPLER_NODE_ID),
            data: Event::Wavetable(Vec::from(wavetable_source_1())),
            timing: EventTiming::after_seconds(absolute_frame, 6.0),
        })
        .unwrap();
    std::thread::sleep(Duration::from_secs(8));
}
