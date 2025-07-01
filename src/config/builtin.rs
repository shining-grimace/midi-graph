use crate::{
    config::NodeRegistry,
    effect::{AdsrEnvelope, Fader, Filter, Lfo, Transition},
    generator::{LfsrNoise, Null, OneShot, SampleLoop, SawtoothWave, SquareWave, TriangleWave},
    group::{Combiner, Font, Mixer, Polyphony, Subtree},
    midi::Midi,
};

pub(crate) fn register_builtin_types(registry: &mut NodeRegistry) {
    registry.register_node_type::<AdsrEnvelope>("AdsrEnvelope");
    registry.register_node_type::<Fader>("Fader");
    registry.register_node_type::<Filter>("Filter");
    registry.register_node_type::<Lfo>("Lfo");
    registry.register_node_type::<Transition>("Transition");
    registry.register_node_type::<LfsrNoise>("LfsrNoise");
    registry.register_node_type::<Null>("Null");
    registry.register_node_type::<OneShot>("OneShot");
    registry.register_node_type::<SampleLoop>("SampleLoop");
    registry.register_node_type::<SquareWave>("SquareWave");
    registry.register_node_type::<SawtoothWave>("SawtoothWave");
    registry.register_node_type::<TriangleWave>("TriangleWave");
    registry.register_node_type::<Font>("Font");
    registry.register_node_type::<Mixer>("Mixer");
    registry.register_node_type::<Combiner>("Combiner");
    registry.register_node_type::<Polyphony>("Polyphony");
    registry.register_node_type::<Midi>("Midi");
    registry.register_node_type::<Subtree>("Subtree");
}
