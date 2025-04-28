
# MIDI Graph

A standalone audio engine written in Rust.

Features include:
- Cross-platform, including Web, thanks to the `cpal` crate
- Play MIDI events either from `.mid` files or manually send them through an async channel
- Shape sound using a node graph, assembled from included tools such as chiptune generators
  and sample file loaders (`.wav` and `.sf2` files are supported)
- Load node configurations from `.ron` files or write them in Rust code
- Some basic effects included, such as volume (ADSR) envelope

### Examples

`cargo run --example <example-name>`

Various examples are included, testing various features:
- `async` to test manually-generated MIDI events without loading a `.mid` file
- `chip` to test some of the basic audio generators and effects on a melody from a `.mid` file
- `looping` to test a `.mid` file containing cue points, as well as controlling using manual async events
- `programs` to test storing multiple programs in the `BaseMixer`'s and changing during playback
- `ron` to test loading and using a node graph from a RON file (which also specifies a `.mid` file to play)
- `sf2` to test loading a soundfont from a `.sf2` file and applying it to a melody from a `.mid` file

### Testing

For the host desktop platform:

`cargo test`

For WebAssembly:

`./wasm-prepare.sh` if never run before, then
`./wasm-test.sh`

### Running WebAssembly Demo

`./wasm-prepare.sh` if never run before, then
`./wasm-run.sh`

