
# MIDI Graph

A standalone audio engine written in Rust, based on a node graph.

Features include:
- Cross-platform, including Web, thanks to the `cpal` crate
- Play MIDI events either from `.mid` files or manually send them through an async channel
- A custom syntax for writing playback events inside of MIDI files, for manipulating playback position in various ways
- Shape sound using a node graph, assembled from included tools such as chiptune generators and sample file loaders (`.wav` and `.sf2` files are supported)
- A `.json` format for loading abstract node graph representations from files
- Some basic effects included, such as ADSR volume envelope and frequency filtering
- An event system for injecting mutations into the node graph
- An [integration with the Bevy engine](https://github.com/shining-grimace/bevy-midi-graph)

## Platform Compatibility Notes

Support is confirmed for Linux (ALSA backend) and WebAssembly. Any other platform supported by the CPAL crate should also work.

NOTE: This crate currently requires `std` support.

## Built-in Nodes

### Generators

- LfsrNoise: a pseudo-random noise generator based on hardware of the Nintendo Gameboy
- OneShot: playback of a sample from a file, without looping or adjusting pitch to MIDI notes
- SampleLoop: playback of a sample from a file, supporting both looping and adjusting pitch to MIDI notes
- SquareWave: a square wave with adjustable duty cycle
- SawtoothWave: a sawtooth wave with basic customisations
- TriangleWave: a triangle wave with basic customisations

### Effects

- AdsrEnvelope: applies an attack-decay-sustain-release envelope
- Fader: applies a volume transition over time
- Filter: applies a frequency filter, such as high-pass or notch
- Lfo: applies an oscillating modulation of volume, pan, pitch, mix balance, MIDI playback time dilation, or frequency filter cutoff
- Transition: applies a modulation over a set duration of volume, pan, pitch, mix baance, MIDI playback time dilation, or filter frequency cutoff

### Grouping

- CombinerNode: group together any number of child nodes which add together
- MixerNode: group exacty two children and customise the mix balance
- PolyphonyNode: group a number of clones of a single node, activating them only when a new note is played to achieve

## Events

A subset of standard MIDI events are currently supported. Events from a `mid` file will be coerced into a crate-specific format, and hese events can be generated in code as well.

These tables are non-exhaustive lists of MIDI event types, indicating which are used by MIDI
Graph and which are planned for an implementation.

### Crate Events

| Message | Related MIDI Event | Notes |
| --- | --- | --- |
| CueData | None | Custom feature; see description below |
| LoopCue | None | Custom feature; see description below |
| NoteOn | NoteOn |  |
| NoteOff | NoteOff | Velocity is unused |
| MixerBalance | None | No mapping yet; adjusts MixerNode balance between two child nodes |
| SourceBalance | None | No mapping yet; adjusts left-right balance of various generator nodes |
| Volume | None | No mapping yet; adjusts volume of various generator nodes |
| PitchMultiplier | None | No mapping yet; pitch bend for various generator nodes |
| TimeDilation | None | Modulates playback rate of a MIDI sequence |
| FilterFrequencyShift | None | No mapping yet; adjusts the changeover frequency of frequency filters |
| Fade | None | Begins a volume transition over time |
| Transition | None | Begins a transition over time of volume, pan, or more |
| Lfo | None | Begins an oscillating effect of volume, pan, or more |
| Filter | None | Sets the filter type and frequency type for a FilterNode |
| EndModulation | None | Stops an effect transition |

### MIDI Messages

| Message | Status | Notes |
| --- | --- | --- |
| NoteOff | Implemented | Velocity is unused |
| NoteOn | Implemented |  |
| Aftertouch | Planned |  |
| Controller | Not planned |  |
| ProgramChange | Planned |  |
| ChannelAftertouch | Not planned |  |
| PitchBend | Planned |  |

### MIDI Meta Messages

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

