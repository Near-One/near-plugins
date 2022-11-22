#!/usr/bin/env bash

mkdir -p ../../res

RUSTFLAGS='-C link-arg=-s' cargo build --target wasm32-unknown-unknown --release

cp ../../target/wasm32-unknown-unknown/release/pausable_base.wasm ../../res/