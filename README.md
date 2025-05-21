
# MIDI Graph

A standalone audio engine written in Rust.

Features include:
- Cross-platform, including Web, thanks to the `cpal` crate
- Play MIDI events either from `.mid` files or manually send them through an async channel
- Shape sound using a node graph, assembled from included tools such as chiptune generators
  and sample file loaders (`.wav` and `.sf2` files are supported)
- Load node configurations from `.ron` files or write them in Rust code
- Some basic effects included, such as volume (ADSR) envelope

## Custom Cue Components

MIDI meta events of the "cue point" type can be used to encode signals recognised by MIDI Graph
for manipulating the playback position within a MIDI file's events.

The supported signals, encoded as strings, are:

| Signal | Example | Description |
| --- | --- | --- |
| #n | #3 | Mark a numbered time position, or "anchor"; does nothing when reached |
| >n | >n | Seek immediately to the anchor of the same number |
| ? | ? | Seek to the requested anchor if one has been requested |

### Notes on Cue Components

- Multiple signals can be grouped together, such as `?>3#1` such that one cue point event in the
  MIDI data (possibly from one marker within the DAW that exported the file) can encode multiple
  things at one point in time.
- The ordering of cue components within the marker label is important, since they're decoded as
  multiple separate events. For example, `#1>3` will cause seeking to anchor point 1 to then
  immediately seek again to anchor point 3
- Requesting (or clearing) the anchor to seek to at the next point marked with a "?" cue can be
  done by sending a custom event into the graph

## MIDI Event Compatibility

These tables are non-exhaustive lists of MIDI event types, indicating which are used by MIDI
Graph and which are planned for an implementation.

### Messages

| Message | Status | Notes |
| --- | --- | --- |
| NoteOff | Implemented | Velocity is unused |
| NoteOn | Implemented |  |
| Aftertouch | Planned |  |
| Controller | Not planned |  |
| ProgramChange | Planned |  |
| ChannelAftertouch | Not planned |  |
| PitchBend | Planned |  |

### Meta Messages

| Meta Message | Status | Description |
| --- | --- | --- |
| TrackNumber | Not planned |  |
| Text | Not planned |  |
| Copyright | Not planned |  |
| TrackName | Not planned |  |
| InstrumentName | Not planned |  |
| Lyric | Not planned |  |
| Marker | Not planned |  |
| CuePoint | Implemented | Used for custom cue signals |
| ProgramName | Not planned |  |
| DeviceName | Not planned |  |
| MidiChannel | Not planned |  |
| MidiPort | Not planned |  |
| EndOfTrack | Not planned |  |
| Tempo | Implemented |  |
| SmpteOffset | Implemented |  |
| TimeSignature | Not planned |  |
| KeySignature | Not planned |  |
| SequencerSpecific | Not planned |  |
| Unknown | Not planned |  |

## DAW Workflow

Suggested workflows for composing music in a DAW and exporting MIDI files for best compatibility
with MIDI Graph.

### REAPER 7

#### Workflow Tips

Add project Markers ("Insert" > "Marker (prompt for name)") at a desired playback position and
use the name to specify MIDI Graph custom cue components.

For a workflow to compose all notes in a single Track, add one Track for composing notes, and
then one additional Track per sound that will be used. Send the composition Track to every other
track. Add the MIDI Channel Filter LV2 plugin to the receiving Tracks, setting each one to a
different channel, and add plugins for the desired sound for each channel Track. Now in the piano
roll for the composition Track, set the "Color" option to "Channel" so notes are easier to
distinguish by channel.

#### Export Settings

When exporting ("File" > "Export Project MIDI..."):
- Check the "Export project markers as MIDI", choose "cues" instead of "markers", and uncheck
  the "Only export project markers that begin with '#'" setting to export cue components
  correctly
- If notes were composed in a single track using multiple channels, select the main composition
  track before exporting, and then in the export dialog choose "Selected tracks only" under the
  "Consolidate MIDI items" setting, and "Merge to single MIDI track (type 0 MIDI file)" under
  "Output"

## Examples

`cargo run --example <example-name>`

Various examples are included, testing various features:
- `async` to test manually-generated MIDI events without loading a `.mid` file
- `chip` to test some of the basic audio generators and effects on a melody from a `.mid` file
- `looping` to test a `.mid` file containing cue points, as well as controlling using manual async events
- `programs` to test storing multiple programs in the `BaseMixer`'s and changing during playback
- `ron` to test loading and using a node graph from a RON file (which also specifies a `.mid` file to play)
- `sf2` to test loading a soundfont from a `.sf2` file and applying it to a melody from a `.mid` file

## Testing

For the host desktop platform:

`cargo test`

For WebAssembly:

`./wasm-prepare.sh` if never run before, then
`./wasm-test.sh`

## Running WebAssembly Demo

`./wasm-prepare.sh` if never run before, then
`./wasm-run.sh`

