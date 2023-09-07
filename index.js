
import init from './pkg/midi_graph.js';

const run = async () => {
    const wasm = await init();
    const button = document.getElementById('play-button');
    button.disabled = false;
    button.addEventListener('click', () => {
        wasm.play_stream();
    });
};

run()
    .catch(e => {
        console.log('Error booting WebAssembly module');
        console.log(e);
    });
