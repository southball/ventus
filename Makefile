.PHONY: ventus-example-function
ventus-example-function:
	cd crates/ventus-example-function && cargo build --target wasm32-wasi --release
