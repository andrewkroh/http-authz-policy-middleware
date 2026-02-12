.PHONY: build test clean release check

build:
	cargo build --target wasm32-wasip1

release:
	cargo build --target wasm32-wasip1 --release
	cp target/wasm32-wasip1/release/traefik_authz_wasm.wasm plugin.wasm

test:
	cargo test

clean:
	cargo clean
	rm -f plugin.wasm

check:
	cargo clippy --target wasm32-wasip1
	cargo fmt --check
