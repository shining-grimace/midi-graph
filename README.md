
# MIDI Graph

A standalone audio engine written in Rust.

Features include:
- Cross-platform, including Web, thanks to the `cpal` crate
- Play MIDI events either from `.mid` files or manually send them through an async channel
- Shape sound using a node graph, assembled from included tools such as chiptune generators
  and sample file loaders (`.wav` and `.sf2` files are supported)
- Load node configurations from `.ron` files or write them in Rust code
- Some basic effects included, such as volume (ADSR) envelope

### Test Non-WebAssembly

`cargo test`

### Test WebAssembly

- Run `cargo install wasm-pack` if needed
- `wasm-pack test [--node | --chrome | --firefox | --safari]`

### Run WebAssembly

- (If needed) `cargo install wasm-pack`
- (If needed) `npm install -g parcel`
- `wasm-pack build --target web`
- `parcel serve index.html`
