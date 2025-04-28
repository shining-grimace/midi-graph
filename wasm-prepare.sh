#!/bin/bash

rustup target add wasm32-unknown-unknown
cargo install wasm-pack
npm install -g parcel

