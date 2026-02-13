.PHONY: build test clean release check check-license release-notes

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

release-notes:
	git cliff --unreleased --strip all
