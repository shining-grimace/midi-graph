#!/bin/bash

wasm-pack build --target web
parcel serve index.html

