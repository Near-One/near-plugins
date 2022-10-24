#!/usr/bin/env bash

RUSTFLAGS='-C link-arg=-s' cargo build --features "counter2" --target wasm32-unknown-unknown --release
mv  ../../target/wasm32-unknown-unknown/release/upgradable_base.wasm ../../target/wasm32-unknown-unknown/release/upgradable_base_second.wasm
RUSTFLAGS='-C link-arg=-s' cargo build --features "counter1" --target wasm32-unknown-unknown --release
