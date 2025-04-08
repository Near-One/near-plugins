build:
	cargo build --target wasm32-unknown-unknown --release

# Helpful for debugging. Requires `cargo-expand`.
expand:
	cargo expand > expanded.rs

.PHONY: build expand
