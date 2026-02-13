.PHONY: build test clean release check check-license release-notes playground

build:
	cargo build --target wasm32-wasip1

release:
	cargo build --target wasm32-wasip1 --release
	@if command -v wasm-opt > /dev/null 2>&1; then \
		echo "Optimizing with wasm-opt..."; \
		wasm-opt -Oz --enable-bulk-memory target/wasm32-wasip1/release/traefik_authz_wasm.wasm -o plugin.wasm; \
	else \
		echo "wasm-opt not found, skipping optimization (install with: cargo install wasm-opt)"; \
		cp target/wasm32-wasip1/release/traefik_authz_wasm.wasm plugin.wasm; \
	fi
	@ls -lh plugin.wasm

test:
	cargo test

clean:
	cargo clean
	rm -f plugin.wasm

check:
	cargo clippy --target wasm32-wasip1
	cargo fmt --check
	@$(MAKE) check-license

check-license:
	@./scripts/check-license-headers.sh

playground:
	cargo build --target wasm32-unknown-unknown --release --no-default-features --features playground
	wasm-bindgen target/wasm32-unknown-unknown/release/traefik_authz_wasm.wasm --out-dir playground/pkg --target web --no-typescript
	cp docs/authz-ferris.png playground/authz-ferris.png
	@if command -v wasm-opt > /dev/null 2>&1; then \
		echo "Optimizing playground WASM with wasm-opt..."; \
		wasm-opt -Oz --enable-bulk-memory playground/pkg/traefik_authz_wasm_bg.wasm -o playground/pkg/traefik_authz_wasm_bg.wasm; \
	else \
		echo "wasm-opt not found, skipping optimization"; \
	fi
	@ls -lh playground/pkg/traefik_authz_wasm_bg.wasm

release-notes:
	git cliff --unreleased --strip all
