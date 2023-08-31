
use crate::MidiProcessor;

#[test]
fn it_works() {
    let smf = MidiProcessor::from_file("resources/MIDI_sample.mid");
    assert!(smf.is_ok());
}
