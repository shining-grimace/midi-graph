
use crate::MidiProcessor;

#[test]
fn it_works() {
    let smf = MidiProcessor::new("resources/MIDI_sample.mid");
    assert!(smf.is_ok());
}
