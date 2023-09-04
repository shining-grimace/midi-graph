
### Test Non-WebAssembly

`cargo test`

### Test WebAssembly

- Run `cargo install wasm-pack` if needed
- `wasm-pack test [--node | --chrome | --firefox | --safari] wasm`

### Run WebAssembly

- (If needed) `cargo install wasm-pack`
- (If needed) `npm install -g parcel`
- `wasm-pack build wasm --target web`
- `parcel serve index.html`
