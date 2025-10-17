.PHONY: help build build-release test clean cli worker install

help:
	@echo "htmlens - Workspace Makefile"
	@echo ""
	@echo "Available targets:"
	@echo "  build           - Build all crates (debug)"
	@echo "  build-release   - Build all crates (release)"
	@echo "  cli             - Build CLI only (release)"
	@echo "  worker          - Build worker only (release)"
	@echo "  test            - Run tests"
	@echo "  clean           - Clean build artifacts"
	@echo "  install         - Install CLI globally"
	@echo "  check           - Check code without building"
	@echo "  run             - Run CLI with help"

build:
	cargo build --workspace

build-release:
	cargo build --release --workspace

cli:
	cargo build --release -p htmlens-cli

worker:
	cargo build --release -p htmlens-worker

test:
	cargo test --workspace

clean:
	cargo clean

install:
	cargo install --path crates/htmlens-cli

check:
	cargo check --workspace

run:
	cargo run -p htmlens-cli -- --help
