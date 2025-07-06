
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

#### MIDI Graph Cues

Add project Markers ("Insert" > "Marker (prompt for name)") at a desired playback position and
use the name to specify MIDI Graph custom cue components.

#### Managing Instruments Per Channel

MIDI Graph loads a single MIDI file track into a MIDI node, with a separate child node assigned
to each channel used.

To simulate this in REAPER, compose all notes in one track, but use a separate channel for each
desired instrument. Then, add an additional track for each instrument. Send the original track
where the notes are to each additional track, and add the MIDI Channel Filter LV2 plugin to the
main track.

To make channels easier to work with, in the piano roll for your composition track, set the
"Color" option to "Channel", and it will be easier to tell which notes are for the same instrument
given that they're all mixed together in one view.

#### Project Configuration

The above notes on Managing Instruments Per Channel describes the setup for a single track to be
exported. This can be duplicated to implement multiple composition tracks in one project, though
each composition track (track with MIDI notes) will need to be loaded into a separate MIDI node
on the MIDI Graph side.

It is not necessary to set up per-channel instruments on blank tracks (with no MIDI items on the
track). There will be other ways to simulate in REAPER what is desired to be heard in MIDI Graph,
though the setup will depend on what kinds of instruments are used (samplers, synths, etc.) and
what configuration options your specific plugins support.

#### Export Settings

When exporting ("File" > "Export Project MIDI..."):
- Check the "Embed project tempo/time signature changes" option
- Check the "Export project markers as MIDI", choose "cues" instead of "markers", and uncheck
  the "Only export project markers that begin with '#'" setting to export cue components
  correctly
- Both "Merge to single MIDI track (type 0 MIDI file)" and "Multitrack MIDI file (type 1 MIDI
  file)" are supported, though in the multitrack case only tracks with note data should be
  loaded into a MIDI node in MIDI Graph, and only one track can be loaded be node

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

