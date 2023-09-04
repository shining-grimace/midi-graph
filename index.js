
import init from './pkg/midi_graph.js';
import { initAudio } from "./audio";

const run = async () => {
    const wasm = await init();
    initAudio(wasm);
};

run();
